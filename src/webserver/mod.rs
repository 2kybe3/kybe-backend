pub mod common;
pub mod render;
mod routes;

use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use crate::external::lastfm::LastFM;
use crate::maxmind::MaxMind;
use crate::maxmind::asn::AsnMin;
use crate::maxmind::city::CityMin;
use crate::webserver::routes::{canvas, fallback_404, ip, pgp, root, user::register};
use anyhow::anyhow;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
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
	auth: Arc<AuthService>,
	lastfm: Option<Arc<LastFM>>,
}

pub trait IpExtractionConfig {
	fn behind_proxy(&self) -> bool;
	fn trust_proxy_header(&self) -> String;
}

impl IpExtractionConfig for Config {
	fn behind_proxy(&self) -> bool {
		self.webserver.behind_proxy
	}

	fn trust_proxy_header(&self) -> String {
		self.webserver
			.trust_proxy_header
			.clone()
			.unwrap_or("X-Forwarded-For".into())
	}
}

impl IpExtractionConfig for &ClientIpKeyExtractor {
	fn behind_proxy(&self) -> bool {
		self.behind_proxy
	}

	fn trust_proxy_header(&self) -> String {
		self.trust_proxy_header.clone()
	}
}

fn client_ip<T: IpExtractionConfig>(
	headers: &HeaderMap,
	remote_addr: SocketAddr,
	config: &T,
) -> Option<IpAddr> {
	if config.behind_proxy() {
		headers
			.get(config.trust_proxy_header())
			.and_then(|v| v.to_str().ok())
			.and_then(|s| {
				s.split(',')
					.next()
					.map(str::trim)
					.and_then(|ip| ip.parse::<IpAddr>().ok())
			})
	} else {
		Some(remote_addr.ip())
	}
}

#[derive(Clone)]
pub struct ClientIpKeyExtractor {
	behind_proxy: bool,
	trust_proxy_header: String,
}

impl ClientIpKeyExtractor {
	pub fn new(behind_proxy: bool, trust_proxy_header: String) -> Self {
		Self {
			behind_proxy,
			trust_proxy_header,
		}
	}
}

impl KeyExtractor for ClientIpKeyExtractor {
	type Key = IpAddr;

	fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
		let headers = req.headers();
		let connect_info = req
			.extensions()
			.get::<ConnectInfo<SocketAddr>>()
			.map(|ci| ci.0)
			.ok_or_else(|| GovernorError::UnableToExtractKey)?;

		let ip = client_ip(headers, connect_info, &self)
			.ok_or_else(|| GovernorError::UnableToExtractKey)?;

		Ok(ip)
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestContext {
	pub user_agent: String,
	pub ip: IpAddr,
	pub mm_asn: Option<AsnMin>,
	pub mm_city: Option<CityMin>,
}

async fn user_contex_middleware(
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

	let ip = match client_ip(&headers, remote_addr, &*state.config) {
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

	let mm = state.mm.lookup(ip);

	if let Err(ref e) = mm {
		warn!("MaxMind lookup failed for {ip} {e:?}");
	}

	let ctx = match mm {
		Ok(s) => RequestContext {
			user_agent,
			ip,
			mm_city: s.city,
			mm_asn: s.asn,
		},
		Err(_) => RequestContext {
			user_agent,
			ip,
			mm_city: None,
			mm_asn: None,
		},
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
		ctx.ip.to_string(),
		serde_json::json!(ctx.mm_asn),
		serde_json::json!(ctx.mm_city),
	)));

	request.extensions_mut().insert(trace.clone());

	let response = next.run(request).await;

	let status = response.status().into();

	let db = state.database.clone();
	tokio::spawn(async move {
		let mut t = trace.lock().await;
		t.finish(status, None, &db).await
	});

	response
}

pub fn make_limiter(
	config: &Config,
	per_second: u64,
	burst_size: u32,
) -> anyhow::Result<Arc<GovernorConfig<ClientIpKeyExtractor, NoOpMiddleware<QuantaInstant>>>> {
	Ok(Arc::new(
		GovernorConfigBuilder::default()
			.per_second(per_second)
			.burst_size(burst_size)
			.key_extractor(ClientIpKeyExtractor::new(
				config.webserver.behind_proxy,
				config
					.webserver
					.trust_proxy_header
					.clone()
					.unwrap_or("X-Forwarded-For".into()),
			))
			.finish()
			.ok_or(anyhow!("governor init failed"))?,
	))
}

pub async fn init_webserver(
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
	mm: Arc<MaxMind>,
	lastfm: Option<Arc<LastFM>>,
) -> anyhow::Result<()> {
	let register_limiter = make_limiter(&config, 60, 10)?;
	let root_limiter = make_limiter(&config, 5, 10)?;
	let asset_limiter = make_limiter(&config, 5, 20)?;

	let webserver_state = WebServerState {
		mm,
		auth,
		lastfm,
		config,
		database,
	};

	let ctx_layer = middleware::from_fn_with_state(webserver_state.clone(), user_contex_middleware);
	let trace_layer = middleware::from_fn_with_state(webserver_state.clone(), trace_middleware);

	let register_limiter_layer = GovernorLayer::new(register_limiter);
	let root_limiter_layer = GovernorLayer::new(root_limiter);
	let asset_limiter_layer = GovernorLayer::new(asset_limiter);

	let register_route_service = ServiceBuilder::new()
		.layer(register_limiter_layer)
		.layer(trace_layer.clone());
	let root_route_service = ServiceBuilder::new()
		.layer(root_limiter_layer)
		.layer(trace_layer.clone());

	let fallback_router = Router::new()
		.fallback(get(fallback_404::fallback_404))
		.layer(ctx_layer.clone())
		.with_state(webserver_state.clone());

	let unlogged_route = Router::new()
		.route("/health", get(|| async { "OK" }))
		.nest_service("/favicon.ico", ServeFile::new("static/de.png"))
		.nest_service(
			"/static",
			ServeDir::new("static").fallback(fallback_router.into_service()),
		)
		.layer(asset_limiter_layer.clone());

	let api_routes = Router::new()
		.route("/", get(root::root))
		.route("/ip", get(ip::ip))
		.route("/pgp", get(pgp::pgp))
		.route("/canvas", get(canvas::canvas))
		.layer(root_route_service)
		.route(
			"/register",
			post(register::register).layer(register_route_service),
		);

	let app = unlogged_route
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
