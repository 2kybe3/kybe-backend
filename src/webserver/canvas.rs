use std::net::SocketAddr;

use axum::{
	extract::{ConnectInfo, Query, RawQuery},
	http::HeaderMap,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		WebServerState, client_ip, finish_trace,
		render::{COLOR_MAPPING, CanvasBuilder, Color, Page, Style, TextBlobBuilder},
	},
};

#[derive(Deserialize)]
pub struct CanvasParamters {
	pub q: Option<String>,
}

pub async fn canvas(
	headers: HeaderMap,
	RawQuery(query): RawQuery,
	Query(parsed_query): Query<CanvasParamters>,
	axum::extract::State(state): axum::extract::State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
	const METHOD: &str = "GET";
	const PATH: &str = "/canvas";

	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = WebsiteTrace::start(METHOD, PATH.to_string(), query, user_agent.clone(), ip);

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
					"\n\nExample: https://kybe.xyz/canvas?q=BBBBBBBBBBNLRRRRRRRRRRNLYYYYYYYYYY\n",
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

	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	tokio::spawn(async move {
		finish_trace(
			&mut trace,
			StatusCode::CREATED.as_u16(),
			None,
			&state.database,
		)
		.await
	});

	if is_cli {
		(StatusCode::OK, result).into_response()
	} else {
		(StatusCode::OK, Html(result)).into_response()
	}
}
