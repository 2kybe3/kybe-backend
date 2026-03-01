use axum::{
	Extension,
	extract::State,
	http::HeaderMap,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		WebServerState, common,
		render::{
			Page, Theme,
			builders::{CodeBlockBuilder, TextBlobBuilder},
		},
	},
};

pub async fn pgp(
	State(state): State<WebServerState>,
	headers: HeaderMap,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
) -> impl IntoResponse {
	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let theme = Theme::default();

	let mut page = vec![
		theme
			.title("Hello Stranger, and maybe PGP user :-)\n\n")
			.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new(include_str!("../../../static/pgp.txt").trim()).into(),
		])
		.title("kybe <kybe@kybe.xyz> | 4B2067C3BD6D410F13E536A343CE43938A3C7A8F")
		.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("curl https://kybe.xyz/pgp.txt | gpg --import").into(),
		])
		.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("ssh kybe@ssh.kybe.xyz pgp | gpg --import").into(),
		])
		.into(),
	];
	page.append(&mut common::footer::footer(trace.lock().await.trace_id));

	let page = Page::from_iter(page);

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let is_cli = user_agent.contains("curl") || user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - pgp", &state.config.webserver.umami)
	};

	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_cli {
		(StatusCode::OK, result).into_response()
	} else {
		(StatusCode::OK, Html(result)).into_response()
	}
}
