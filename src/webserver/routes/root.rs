use std::sync::Arc;

use axum::{
	Extension,
	extract::State,
	http::HeaderMap,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::{
		WebServerState, common,
		render::{
			Objects, Page, Theme,
			builders::{CodeBlockBuilder, TextBlobBuilder},
		},
	},
};

pub async fn root(
	State(state): State<WebServerState>,
	headers: HeaderMap,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
) -> impl IntoResponse {
	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let mut trace = trace.lock().await;
	let theme = Theme::default();

	let mut playing = None;
	if let Some(lastfm) = state.lastfm {
		playing = lastfm.get_playing().await;
	}

	let mut page: Vec<Objects> = vec![
		theme.title("Hello Stranger\n").into(),
		theme
			.subtitle("This site is made to also look good on curl\n\n")
			.into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("$ ").copyable(false).into(),
			TextBlobBuilder::new("curl https://kybe.xyz").into(),
		])
		.into(),
	];

	if let Some(playing) = playing {
		page.append(&mut vec![
			theme.text("\n").into(),
			theme
				.label(
					"Currently Listening",
					vec![
						theme
							.link_colored(
								format!("{} - {}", playing.artist, playing.name).as_str(),
								&playing.url,
							)
							.into(),
					],
				)
				.into(),
			theme.text("\n").into(),
		])
	};

	page.append(&mut vec![
		theme.title("\nProjects:\n\n").into(),
		theme
			.label(
				"cheat-sh",
				vec![
					theme
						.link_colored(
							"https://github.com/2kybe3/cheat-sh",
							"https://github.com/2kybe3/cheat-sh",
						)
						.into(),
					theme.comment(" (a tiny wrapper for cheat.sh)\n").into(),
				],
			)
			.into(),
		theme
			.label(
				"nix-dotfiles",
				vec![
					theme
						.link_colored(
							"https://codeberg.org/kybe/nix-dotfiles",
							"https://codeberg.org/kybe/nix-dotfiles",
						)
						.into(),
					theme.comment(" (i use nix btw)\n").into(),
				],
			)
			.into(),
		theme
			.label(
				"kybe-backend",
				vec![
					theme
						.link_colored(
							"https://github.com/2kybe3/kybe-backend",
							"https://github.com/2kybe3/kybe-backend",
						)
						.into(),
					theme.comment(" (this site)\n").into(),
				],
			)
			.into(),
		theme.title("\nContact:\n\n").into(),
		theme
			.label(
				"PGP",
				vec![
					theme
						.link_colored("https://kybe.xyz/pgp\n", "https://kybe.xyz/pgp")
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Matrix",
				vec![
					theme
						.link_colored("@kybe:kybe.xyz\n", "https://matrix.to/#/@kybe:kybe.xyz")
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Email",
				vec![
					theme
						.link_colored("kybe@kybe.xyz\n", "mailto:kybe@kybe.xyz")
						.into(),
				],
			)
			.into(),
		theme.title("\nOther Platforms:\n\n").into(),
		theme
			.label(
				"Github",
				vec![
					theme
						.link_colored("https://github.com/kybe236", "https://github.com/kybe236")
						.into(),
					theme.text(" ").into(),
					theme
						.link_colored("https://github.com/2kybe3\n", "https://github.com/2kybe3")
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Codeberg",
				vec![
					theme
						.link_colored("https://codeberg.org/kybe\n", "https://codeberg.org/kybe")
						.into(),
				],
			)
			.into(),
		theme.title("\nOther Endpoints:\n\n").into(),
		theme
			.label(
				"Identity",
				vec![
					theme
						.link_colored("https://kybe.xyz/ident.txt\n", "https://kybe.xyz/ident.txt")
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Canvas",
				vec![
					theme
						.link_colored("https://kybe.xyz/canvas\n", "https://kybe.xyz/canvas")
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"IP",
				vec![
					theme
						.link_colored("https://kybe.xyz/ip\n", "https://kybe.xyz/ip")
						.into(),
				],
			)
			.into(),
	]);
	page.append(&mut common::footer::footer(trace.trace_id));

	let page = Page::from_iter(page);

	let user_agent = user_agent.unwrap_or_default().to_lowercase();
	let is_cli = user_agent.contains("curl") || user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - root", &state.config.webserver.umami)
	};

	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_cli {
		(StatusCode::OK, result).into_response()
	} else {
		(StatusCode::OK, Html(result)).into_response()
	}
}
