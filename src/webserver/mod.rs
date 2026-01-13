mod pgp;
mod register;
mod render;
mod root;

use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use crate::notifications::{Notification, Notifications};
use anyhow::anyhow;
use axum::Router;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request};
use axum::routing::{get, post};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tracing::{debug, error};
use uuid::Uuid;

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

async fn finish_trace(
	trace: &mut WebsiteTrace,
	status_code: u16,
	user_id: Option<Uuid>,
	database: &Database,
) {
	let duration = chrono::Utc::now()
		.signed_duration_since(trace.started_at)
		.num_milliseconds();

	trace.complete(duration, status_code, user_id);

	if let Err(e) = database.save_website_trace(trace).await {
		error!("Failed to save website trace: {e:?}");
	}

	debug!(trace = ?trace, "API request finished");
}

pub async fn init_webserver(
	notifications_clone: Arc<Notifications>,
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
) {
	if let Err(e) = init_webserver_inner(config, auth, database).await {
		notifications_clone
			.notify(Notification::new(
				"Webserver",
				&format!("Webserver init failed: {:?}", e),
			))
			.await;
		std::process::exit(1);
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

async fn init_webserver_inner(
	config: Arc<Config>,
	auth: Arc<AuthService>,
	database: Database,
) -> anyhow::Result<()> {
	let register_limiter = Arc::new(
		GovernorConfigBuilder::default()
			.per_second(60)
			.burst_size(10)
			.key_extractor(ClientIpKeyExtractor::new(
				config.webserver.behind_proxy,
				config.webserver.trust_proxy_header.clone(),
			))
			.finish()
			.ok_or(anyhow!("governor init failed"))?,
	);
	let root_limiter = Arc::new(
		GovernorConfigBuilder::default()
			.per_second(1)
			.burst_size(10)
			.key_extractor(ClientIpKeyExtractor::new(
				config.webserver.behind_proxy,
				config.webserver.trust_proxy_header.clone(),
			))
			.finish()
			.ok_or(anyhow!("governor init failed"))?,
	);

	let webserver_state = WebServerState {
		auth,
		config,
		database,
	};

	let app = Router::new()
		.route(
			"/",
			get(root::root).layer(GovernorLayer::new(root_limiter.clone())),
		)
		.route(
			"/pgp",
			get(pgp::pgp).layer(GovernorLayer::new(root_limiter)),
		)
		.route("/health", get(|| async { "OK" }))
		.route(
			"/register",
			post(register::register).layer(GovernorLayer::new(register_limiter)),
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
