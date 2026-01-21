mod canvas;
mod fallback_404;
mod ip;
mod pgp;
mod register;
mod render;
mod root;

use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use crate::maxmind::MaxMind;
use crate::maxmind::asn::AsnMin;
use crate::maxmind::city::CityMin;
use anyhow::anyhow;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Router, middleware};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
struct WebServerState {
	mm: Arc<MaxMind>,
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
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
			.unwrap();

		let ip = client_ip(headers, connect_info, &self).unwrap();

		Ok(ip)
	}
}

#[derive(Clone)]
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

	let ip = client_ip(&headers, remote_addr, &*state.config).unwrap();

	let mm = state.mm.lookup(ip).unwrap();

	let ctx = RequestContext {
		user_agent,
		ip,
		mm_city: mm.city,
		mm_asn: mm.asn,
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
) -> anyhow::Result<()> {
	let register_limiter = make_limiter(&config, 60, 10)?;
	let root_limiter = make_limiter(&config, 5, 10)?;
	let asset_limiter = make_limiter(&config, 5, 20)?;

	let webserver_state = WebServerState {
		mm,
		auth,
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

	// TODO: figure out how i can make it also use fallback_404::fallback_404 for ServeDir#fallback
	let unlogged_route = Router::new()
		.route("/health", get(|| async { "OK" }))
		.nest_service("/favicon.ico", ServeFile::new("static/de.png"))
		.nest_service("/static", ServeDir::new("static"))
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

	let listener = TcpListener::bind("0.0.0.0:3001").await?;
	axum::serve(
		listener,
		app.into_make_service_with_connect_info::<SocketAddr>(),
	)
	.await?;
	Ok(())
}
