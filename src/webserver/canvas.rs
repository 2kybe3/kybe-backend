use std::sync::Arc;

use axum::{
	Extension,
	extract::Query,
	http::HeaderMap,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::render::{
		Color, Page, Style,
		builders::{COLOR_MAPPING, CanvasBuilder, TextBlobBuilder},
	},
};

#[derive(Deserialize)]
pub struct CanvasParamters {
	pub q: Option<String>,
}

pub async fn canvas(
	headers: HeaderMap,
	Query(parsed_query): Query<CanvasParamters>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
) -> impl IntoResponse {
	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

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
				TextBlobBuilder::new("Use the q query paramter to use this canvas api\n\n").into(),
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

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let is_cli = user_agent.contains("curl") || user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - canvas")
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
