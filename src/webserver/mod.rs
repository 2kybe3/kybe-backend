pub mod common;
pub mod render;
mod routes;

use crate::config::types::{Config, WebserverConfig};
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use crate::external::lastfm::LastFM;
use crate::maxmind::MaxMind;
use crate::maxmind::asn::AsnMin;
use crate::maxmind::city::CityMin;
use crate::webserver::routes::{canvas, fallback_404, ip, pgp, root};
use crate::webserver::routes::{metrics, now_playing};
use anyhow::anyhow;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Router, middleware};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, warn};

#[derive(Clone)]
struct WebServerState {
	mm: Arc<MaxMind>,
	database: Database,
	config: Arc<Config>,
	lastfm: Option<Arc<LastFM>>,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub struct Ident {
	pub i2p: bool,
	pub data: String,
	pub ipaddr: Option<IpAddr>,
}

// TODO: this config mess could be cleaned up
fn client_ip(
	headers: &HeaderMap,
	remote_addr: SocketAddr,
	config: &ClientIpKeyExtractor,
) -> Option<Ident> {
	if config.behind_proxy && remote_addr.ip() == config.proxy_ip.expect("proxy_ip not set") {
		headers
			.get(
				config
					.proxy_header
					.clone()
					.expect("trust_proxy_header not set"),
			)
			.and_then(|v| {
				v.to_str().ok().map(String::from).map(|s| Ident {
					i2p: false,
					data: s.clone(),
					ipaddr: s.parse().ok(),
				})
			})
	} else if config.behind_i2p && remote_addr.ip() == config.i2p_ip.expect("i2p_ip not set") {
		headers
			.get(config.i2p_header.clone().expect("i2p_header not set"))
			.and_then(|v| {
				v.to_str().ok().map(String::from).map(|s| Ident {
					i2p: true,
					data: s,
					ipaddr: None,
				})
			})
	} else {
		Some(Ident {
			i2p: false,
			data: remote_addr.to_string(),
			ipaddr: Some(remote_addr.ip()),
		})
	}
}

#[derive(Clone)]
pub struct ClientIpKeyExtractor {
	pub behind_proxy: bool,
	pub proxy_ip: Option<IpAddr>,
	pub proxy_header: Option<String>,
	pub behind_i2p: bool,
	pub i2p_ip: Option<IpAddr>,
	pub i2p_header: Option<String>,
}

impl ClientIpKeyExtractor {
	pub fn new(config: &WebserverConfig) -> Self {
		Self {
			behind_proxy: config.behind_proxy,
			proxy_ip: config
				.proxy_ip
				.clone()
				.map(|p| p.parse().expect("invalid proxy IP")),
			proxy_header: config.proxy_header.clone(),
			behind_i2p: config.behind_i2p,
			i2p_ip: config
				.i2p_ip
				.clone()
				.map(|p| p.parse().expect("invalid I2P IP")),
			i2p_header: config.i2p_header.clone(),
		}
	}
}

impl KeyExtractor for ClientIpKeyExtractor {
	type Key = Ident;

	fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
		let headers = req.headers();
		let connect_info = req
			.extensions()
			.get::<ConnectInfo<SocketAddr>>()
			.map(|ci| ci.0)
			.ok_or_else(|| GovernorError::UnableToExtractKey)?;

		let ident = client_ip(headers, connect_info, self)
			.ok_or_else(|| GovernorError::UnableToExtractKey)?;

		Ok(ident)
	}
}

async fn api_auth_middleware(
	State(state): State<WebServerState>,

	headers: HeaderMap,

	request: Request,
	next: Next,
) -> Response {
	let token = headers
		.get("Authorization")
		.and_then(|value| value.to_str().ok());

	match token {
		Some(token_value)
			if token_value == format!("Bearer {}", state.config.webserver.api_token) =>
		{
			next.run(request).await
		}
		_ => Response::builder()
			.status(StatusCode::UNAUTHORIZED)
			.body("Unauthorized".into())
			.expect("Failed to build Response"),
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestContext {
	pub user_agent: String,
	pub ident: Ident,
	pub mm_asn: Option<AsnMin>,
	pub mm_city: Option<CityMin>,
}

async fn user_context_middleware(
	State(state): State<WebServerState>,

	headers: HeaderMap,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,

	mut request: Request,
	next: Next,
) -> Response {
	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string())
		.unwrap_or_default();

	let ident = match client_ip(
		&headers,
		remote_addr,
		&ClientIpKeyExtractor::new(&state.config.webserver),
	) {
		Some(ip) => ip,
		None => {
			error!("error getting client_ip in middleware (most likely header missing)");
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"something bad happened lol",
			)
				.into_response();
		}
	};

	let ctx = if !ident.i2p
		&& let Some(ip) = ident.ipaddr
	{
		let mm = state.mm.lookup(ip);

		if let Err(ref e) = mm {
			warn!("MaxMind lookup failed for {ip} {e:?}");
		}

		match mm {
			Ok(s) => RequestContext {
				user_agent,
				ident,
				mm_city: s.city,
				mm_asn: s.asn,
			},
			Err(_) => RequestContext {
				user_agent,
				ident,
				mm_city: None,
				mm_asn: None,
			},
		}
	} else {
		RequestContext {
			user_agent,
			ident,
			mm_city: None,
			mm_asn: None,
		}
	};

	request.extensions_mut().insert(ctx);

	next.run(request).await
}

async fn trace_middleware(
	State(state): State<WebServerState>,

	mut request: Request,
	next: Next,
) -> Response {
	let ctx = request
		.extensions()
		.get::<RequestContext>()
		.expect("user_context_middleware must run first");

	let trace = Arc::new(Mutex::new(WebsiteTrace::start(
		request.method(),
		request.uri().path(),
		request.uri().query(),
		ctx.user_agent.clone(),
		ctx.ident
			.ipaddr
			.map(|ip| ip.to_string())
			.unwrap_or_default(),
		serde_json::json!(ctx.mm_asn),
		serde_json::json!(ctx.mm_city),
	)));

	request.extensions_mut().insert(trace.clone());

	let response = next.run(request).await;

	let status = response.status().into();

	let db = state.database.clone();
	tokio::spawn(async move {
		let mut t = trace.lock().await;
		t.finish(status, &db).await
	});

	response
}

pub fn make_limiter(
	config: &Config,
	replentish_after_ms: u64,
	burst_size: u32,
) -> anyhow::Result<Arc<GovernorConfig<ClientIpKeyExtractor, NoOpMiddleware<QuantaInstant>>>> {
	Ok(Arc::new(
		GovernorConfigBuilder::default()
			.per_millisecond(replentish_after_ms)
			.burst_size(burst_size)
			.key_extractor(ClientIpKeyExtractor::new(&config.webserver))
			.finish()
			.ok_or(anyhow!("governor init failed"))?,
	))
}

pub async fn init_webserver(
	config: Arc<Config>,
	database: Database,
	mm: Arc<MaxMind>,
	lastfm: Option<Arc<LastFM>>,
) -> anyhow::Result<()> {
	let root_limiter = make_limiter(&config, 500, 10)?;
	let asset_limiter = make_limiter(&config, 500, 20)?;

	let webserver_state = WebServerState {
		mm,
		lastfm,
		config,
		database,
	};

	let api_auth_layer =
		middleware::from_fn_with_state(webserver_state.clone(), api_auth_middleware);
	let ctx_layer =
		middleware::from_fn_with_state(webserver_state.clone(), user_context_middleware);
	let trace_layer = middleware::from_fn_with_state(webserver_state.clone(), trace_middleware);

	let root_limiter_layer = GovernorLayer::new(root_limiter);
	let asset_limiter_layer = GovernorLayer::new(asset_limiter);

	let root_route_service = ServiceBuilder::new()
		.layer(root_limiter_layer)
		.layer(trace_layer.clone());

	let fallback_router = Router::new()
		.fallback(get(fallback_404::fallback_404))
		.layer(ctx_layer.clone())
		.with_state(webserver_state.clone());

	let unlogged_route2 = Router::new()
		.route("/metrics", get(metrics::metrics))
		.layer(api_auth_layer)
		.with_state(webserver_state.clone());

	let unlogged_route = Router::new()
		.route("/health", get(|| async { "OK" }))
		.nest_service("/ident.txt", ServeFile::new("static/ident.txt"))
		.nest_service("/favicon.ico", ServeFile::new("static/de.png"))
		.nest_service("/pgp.txt", ServeFile::new("static/pgp.txt"))
		.nest_service(
			"/static",
			ServeDir::new("static").fallback(fallback_router.into_service()),
		)
		.layer(asset_limiter_layer.clone());

	let api_routes = Router::new()
		.route("/", get(root::root))
		.route("/ip", get(ip::ip))
		.route("/now", get(now_playing::now_playing))
		.route("/pgp", get(pgp::pgp))
		.route("/canvas", get(canvas::canvas))
		.layer(root_route_service);

	let app = unlogged_route
		.merge(unlogged_route2)
		.merge(api_routes)
		.fallback(fallback_404::fallback_404)
		.with_state(webserver_state)
		.layer(ctx_layer);

	let listener = TcpListener::bind("0.0.0.0:3000").await?;
	axum::serve(
		listener,
		app.into_make_service_with_connect_info::<SocketAddr>(),
	)
	.await?;
	Ok(())
}
