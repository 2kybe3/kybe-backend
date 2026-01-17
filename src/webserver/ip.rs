use std::{net::SocketAddr, sync::Arc};

use axum::{Extension, extract::ConnectInfo, http::HeaderMap, response::IntoResponse};
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{WebServerState, client_ip},
};

pub async fn ip(
	headers: HeaderMap,
	axum::extract::State(state): axum::extract::State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
) -> impl IntoResponse {
	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	(StatusCode::OK, ip.unwrap_or_default()).into_response()
}
