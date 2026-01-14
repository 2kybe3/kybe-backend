use std::net::SocketAddr;

use axum::{
	extract::{ConnectInfo, RawQuery},
	http::HeaderMap,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		WebServerState, client_ip, finish_trace,
		render::{Color, Object, Page, Style},
	},
};

pub async fn pgp(
	headers: HeaderMap,
	RawQuery(query): RawQuery,
	axum::extract::State(state): axum::extract::State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
	const METHOD: &str = "GET";
	const PATH: &str = "/pgp";

	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = WebsiteTrace::start(METHOD, PATH.to_string(), query, user_agent.clone(), ip);

	let page = Page::from_iter([
		Object::text("Hello Stranger, and maybe PGP user :-)\n\n")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		Object::code(include_str!("../../assets/key.pgp"))
			.title("kybe <kybe@kybe.xyz>")
			.into(),
	]);

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let is_cli = user_agent.contains("curl") || user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - pgp")
	};

	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	finish_trace(
		&mut trace,
		StatusCode::CREATED.as_u16(),
		None,
		&state.database,
	)
	.await;

	if is_cli {
		(StatusCode::OK, result).into_response()
	} else {
		(StatusCode::OK, Html(result)).into_response()
	}
}
