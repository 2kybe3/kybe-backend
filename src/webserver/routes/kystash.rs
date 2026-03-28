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
		render::{Page, Theme},
	},
};

pub async fn kystash(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let theme = Theme::default();

	let mut page = vec![theme.title("Kystash is under construction :-)\n\n").into()];
	page.append(&mut common::footer::footer(trace.lock().await.trace_id));

	let page = Page::from_iter(page);

	let (is_html, result) = page.render(&ctx.user_agent, "/kystash", &state.config.webserver.umami);

	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_html {
		(StatusCode::OK, Html(result)).into_response()
	} else {
		(StatusCode::OK, result).into_response()
	}
}
