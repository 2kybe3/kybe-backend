use std::net::SocketAddr;

use axum::{
	extract::{ConnectInfo, RawQuery},
	http::HeaderMap,
	response::IntoResponse,
};
use reqwest::StatusCode;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{WebServerState, client_ip, finish_trace},
};

pub async fn ip(
	headers: HeaderMap,
	RawQuery(query): RawQuery,
	axum::extract::State(state): axum::extract::State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
	const METHOD: &str = "GET";
	const PATH: &str = "/ip";

	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = WebsiteTrace::start(
		METHOD,
		PATH.to_string(),
		query,
		user_agent.clone(),
		ip.clone(),
	);

	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	finish_trace(
		&mut trace,
		StatusCode::CREATED.as_u16(),
		None,
		&state.database,
	)
	.await;

	(StatusCode::OK, ip.unwrap_or_default()).into_response()
}
