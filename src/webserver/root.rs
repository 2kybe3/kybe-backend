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
			.style(Style::new_fg(Color::Red))
			.build(),
		TextBlobBuilder::new("This site supports curl\n").build(),
		CodeBlockBuilder::new("curl https://kybe.xyz")
			.title("curl")
			.language("bash")
			.build(),
		TextBlobBuilder::new("Projects:\n\n")
			.style(Style::new_fg(Color::Red))
			.build(),
		TextBlobBuilder::new("kybe-backend: ")
			.style(Style::new_fg(Color::Yellow))
			.build(),
		TextBlobBuilder::new("https://github.com/2kybe3/kybe-backend")
			.style(Style::new_fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://github.com/2kybe3/kybe-backend")
					.seperator_style(Style::new_fg(Color::White))
					.build(),
			)
			.build(),
		TextBlobBuilder::new(" (this site)\n")
			.style(Style::new_fg(Color::White).bold(true).dim(true))
			.build(),
		TextBlobBuilder::new("nix-dotfiles: ")
			.style(Style::new_fg(Color::Yellow))
			.build(),
		TextBlobBuilder::new("https://codeberg.org/kybe/nix-dotfiles")
			.style(Style::new_fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://codeberg.org/kybe/nix-dotfiles")
					.seperator_style(Style::new_fg(Color::White))
					.build(),
			)
			.build(),
		TextBlobBuilder::new(" (i use nix btw)\n")
			.style(Style::new_fg(Color::White).bold(true).dim(true))
			.build(),
		TextBlobBuilder::new("\nContact:\n\n")
			.style(Style::new_fg(Color::Red))
			.build(),
		TextBlobBuilder::new("PGP: ")
			.style(Style::new_fg(Color::Yellow))
			.build(),
		TextBlobBuilder::new("https://kybe.xyz/pgp\n")
			.style(Style::new_fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://kybe.xyz/pgp")
					.seperator_style(Style::new_fg(Color::White))
					.build(),
			)
			.build(),
		TextBlobBuilder::new("Email: ")
			.style(Style::new_fg(Color::Yellow))
			.build(),
		TextBlobBuilder::new("kybe@kybe.xyz\n")
			.style(Style::new_fg(Color::Green))
			.link_to(
				LinkToBuilder::new("mailto:kybe@kybe.xyz")
					.seperator_style(Style::new_fg(Color::White))
					.build(),
			)
			.build(),
		TextBlobBuilder::new("Matrix: ")
			.style(Style::new_fg(Color::Yellow))
			.build(),
		TextBlobBuilder::new("@kybe:kybe.xyz\n")
			.style(Style::new_fg(Color::Green))
			.link_to(
				LinkToBuilder::new("https://matrix.to/#/@kybe:kybe.xyz")
					.seperator_style(Style::new_fg(Color::White))
					.build(),
			)
			.build(),
	]);

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let result = if user_agent.contains("curl") || user_agent.contains("lynx") {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - root")
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

	(StatusCode::OK, Html(result)).into_response()
}
