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
		render::{CodeBlockBuilder, Color, LinkToBuilder, Page, Style, TextBlobBuilder},
	},
};

pub async fn root(
	headers: HeaderMap,
	RawQuery(query): RawQuery,
	axum::extract::State(state): axum::extract::State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
	const METHOD: &str = "GET";
	const PATH: &str = "/";

	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = WebsiteTrace::start(METHOD, PATH.to_string(), query, user_agent.clone(), ip);

	let page = Page::from_iter([
		TextBlobBuilder::new("Hello Stranger\n\n")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new("This site is made to also look good on curl\n").into(),
		CodeBlockBuilder::new("curl https://kybe.xyz")
			.title("curl")
			.language("bash")
			.into(),
		TextBlobBuilder::new("Projects:\n\n")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new("kybe-backend: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("https://github.com/2kybe3/kybe-backend")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://github.com/2kybe3/kybe-backend")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new(" (this site)\n")
			.style(Style::new().fg(Color::White).bold(true).dim(true))
			.into(),
		TextBlobBuilder::new("nix-dotfiles: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("https://codeberg.org/kybe/nix-dotfiles")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://codeberg.org/kybe/nix-dotfiles")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new(" (i use nix btw)\n")
			.style(Style::new().fg(Color::White).bold(true).dim(true))
			.into(),
		TextBlobBuilder::new("\nContact:\n\n")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new("PGP: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("https://kybe.xyz/pgp\n")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://kybe.xyz/pgp")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new("Email: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("kybe@kybe.xyz\n")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("mailto:kybe@kybe.xyz")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new("Matrix: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("@kybe:kybe.xyz\n")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://matrix.to/#/@kybe:kybe.xyz")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new("\nOther Endpoints:\n")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new("IP ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("https://kybe.xyz/ip\n")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://kybe.xyz/ip")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		TextBlobBuilder::new("Canvas ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("https://kybe.xyz/canvas\n")
			.style(Style::new().fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://kybe.xyz/canvas")
					.seperator_style(Style::new().fg(Color::White))
					.into(),
			)
			.into(),
		// DE flag
		TextBlobBuilder::new("\n\n").into(),
		TextBlobBuilder::new("           \n")
			.style(Style::new().fg(Color::Black).bg(Color::Black))
			.into(),
		TextBlobBuilder::new("           \n")
			.style(Style::new().fg(Color::BrightRed).bg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new("           \n")
			.style(Style::new().fg(Color::Yellow).bg(Color::Yellow))
			.into(),
	]);

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let is_cli = user_agent.contains("curl") || user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - root")
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
