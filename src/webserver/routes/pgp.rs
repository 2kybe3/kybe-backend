use axum::{
	Extension,
	extract::State,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		RequestContext, WebServerState, common,
		render::{
			Page, Theme,
			builders::{CodeBlockBuilder, TextBlobBuilder},
		},
	},
};

pub async fn pgp(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
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
			TextBlobBuilder::new("$ ").copyable(false).into(),
			TextBlobBuilder::new("curl https://kybe.xyz/pgp.txt | gpg --import").into(),
		])
		.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("$ ").copyable(false).into(),
			TextBlobBuilder::new("ssh ssh.kybe.xyz pgp | gpg --import").into(),
		])
		.title("IPv6 required")
		.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("$ ").copyable(false).into(),
			TextBlobBuilder::new("resolvectl openpgp kybe@kybe.xyz --raw=payload | gpg --import")
				.into(),
		])
		.title("RFC7929")
		.into(),
	];
	page.append(&mut common::footer::footer(trace.lock().await.trace_id));

	let page = Page::from_iter(page);

	let (is_html, result) = page.render(&ctx.user_agent, "/pgp", &state.config.webserver.umami);

	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_html {
		(StatusCode::OK, Html(result)).into_response()
	} else {
		(StatusCode::OK, result).into_response()
	}
}
