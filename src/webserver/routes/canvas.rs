use std::sync::Arc;

use axum::{
	Extension,
	extract::{Query, State},
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		RequestContext, WebServerState,
		render::{
			Color, Page, Style,
			builders::{COLOR_MAPPING, CanvasBuilder, TextBlobBuilder},
		},
	},
};

#[derive(Deserialize)]
pub struct CanvasParameters {
	pub q: Option<String>,
}

pub async fn canvas(
	State(state): State<WebServerState>,
	Query(parsed_query): Query<CanvasParameters>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let q = parsed_query.q.clone();
	let page = match q {
		Some(q) => Page::from_iter([
			CanvasBuilder::new(q).into(),
			TextBlobBuilder::new("\n").style(Style::default()).into(),
		]),
		None => {
			let mut list = COLOR_MAPPING
				.iter()
				.map(|(key, value)| format!("{key}: {value:?}"))
				.collect::<Vec<_>>();
			list.push("NL: NewLine".into());
			Page::from_iter([
				TextBlobBuilder::new("Canvas\n\n")
					.style(Style::new().fg(Color::Red))
					.into(),
				TextBlobBuilder::new("Use the q query parameter to use this canvas api\n\n").into(),
				TextBlobBuilder::new(list.join("\n"))
					.style(Style::new().fg(Color::Yellow))
					.into(),
				TextBlobBuilder::new(
					"\n\nExample: https://kybe.xyz/canvas?q=BLBLBLBLBLBLBLBLBLBLNLRRRRRRRRRRNLYYYYYYYYYY\n",
				)
				.into(),
			])
		}
	};

	let (is_html, result) = page.render(
		&ctx.user_agent,
		"/dev/canvas",
		&state.config.webserver.umami,
	);

	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_html {
		(StatusCode::OK, Html(result)).into_response()
	} else {
		(StatusCode::OK, result).into_response()
	}
}
