mod canvas;
mod ip;
mod pgp;
mod register;
mod render;
mod root;

use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use anyhow::anyhow;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Router, middleware};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};

#[derive(Clone)]
struct WebServerState {
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
}

pub trait IpExtractionConfig {
	fn behind_proxy(&self) -> bool;
	fn trust_proxy_header(&self) -> &str;
}

impl IpExtractionConfig for Config {
	fn behind_proxy(&self) -> bool {
		self.webserver.behind_proxy
	}

	fn trust_proxy_header(&self) -> &str {
		&self.webserver.trust_proxy_header
	}
}

impl IpExtractionConfig for &ClientIpKeyExtractor {
	fn behind_proxy(&self) -> bool {
		self.behind_proxy
	}

	fn trust_proxy_header(&self) -> &str {
		&self.trust_proxy_header
	}
}

fn client_ip<T: IpExtractionConfig>(
	headers: &HeaderMap,
	remote_addr: SocketAddr,
	config: &T,
) -> Option<String> {
	if config.behind_proxy() {
		headers
			.get(config.trust_proxy_header())
			.and_then(|v| v.to_str().ok())
			.map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
	} else {
		Some(remote_addr.ip().to_string())
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
	type Key = String;

	fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
		let headers = req.headers();
		let connect_info = req
			.extensions()
			.get::<ConnectInfo<SocketAddr>>()
			.map(|ci| ci.0)
			.unwrap_or(SocketAddr::from(([127, 0, 0, 1], 0)));

		let ip = client_ip(headers, connect_info, &self)
			.unwrap_or_else(|| connect_info.ip().to_string());

		Ok(ip)
	}
}

async fn trace_middleware(
	State(state): State<WebServerState>,

	headers: HeaderMap,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,

	mut request: Request,
	next: Next,
) -> Response {
	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let trace = Arc::new(Mutex::new(WebsiteTrace::start(
		request.method(),
		request.uri().path(),
		request.uri().query(),
		user_agent.clone(),
		ip.clone(),
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
				config.webserver.trust_proxy_header.clone(),
			))
			.finish()
			.ok_or(anyhow!("governor init failed"))?,
	))
}

pub async fn init_webserver(
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
) -> anyhow::Result<()> {
	let register_limiter = make_limiter(&config, 60, 10)?;
	let root_limiter = make_limiter(&config, 1, 10)?;

	let webserver_state = WebServerState {
		auth,
		config,
		database,
	};

	let trace_layer = middleware::from_fn_with_state(webserver_state.clone(), trace_middleware);
	let root_limiter_layer = GovernorLayer::new(root_limiter);

	let app = Router::new()
		.route(
			"/",
			get(root::root)
				.layer(root_limiter_layer.clone())
				.route_layer(trace_layer.clone()),
		)
		.route(
			"/ip",
			get(ip::ip)
				.layer(root_limiter_layer.clone())
				.route_layer(trace_layer.clone()),
		)
		.route(
			"/pgp",
			get(pgp::pgp)
				.layer(root_limiter_layer.clone())
				.route_layer(trace_layer.clone()),
		)
		.route(
			"/canvas",
			get(canvas::canvas)
				.layer(root_limiter_layer.clone())
				.route_layer(trace_layer.clone()),
		)
		.route("/health", get(|| async { "OK" }))
		.route(
			"/register",
			post(register::register)
				.layer(GovernorLayer::new(register_limiter))
				.route_layer(trace_layer.clone()),
		)
		.with_state(webserver_state);

	let listener = TcpListener::bind("0.0.0.0:3000").await?;
	axum::serve(
		listener,
		app.into_make_service_with_connect_info::<SocketAddr>(),
	)
	.await?;
	Ok(())
}
