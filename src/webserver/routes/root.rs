use std::sync::Arc;

use axum::{
	Extension,
	extract::State,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	external::lastfm,
	webserver::{
		RequestContext, WebServerState, common,
		render::{
			Objects, Page, Theme,
			builders::{CodeBlockBuilder, TextBlobBuilder},
		},
	},
};

pub async fn root(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let mut trace = trace.lock().await;
	let theme = Theme::default();

	let playing = if let Some(lastfm) = state.lastfm {
		lastfm.get_playing(Some(&mut trace.data)).await.result
	} else {
		None
	};

	let mut page: Vec<Objects> = vec![
		theme.title("Hello Stranger (also on ").into(),
		theme
			.title(theme.link_colored("i2p", "http://kybe.i2p"))
			.into(),
		theme.title(")\n").into(),
		theme.subtitle("kybe – /dev/urandom stuff\n\n").into(),
		CodeBlockBuilder::new(vec![
			TextBlobBuilder::new("$ ").copyable(false).into(),
			TextBlobBuilder::new("curl https://kybe.xyz").into(),
		])
		.into(),
	];

	if let Some(playing) = playing {
		page.append(&mut vec![
			theme.title("\nCurrently Listening:\n\n").into(),
			theme
				.label(
					"Artist",
					vec![
						theme
							.link_colored(
								format!("{}\n", playing.artist.as_str()).as_str(),
								&lastfm::artist_url(&playing.artist),
							)
							.into(),
					],
				)
				.into(),
			theme
				.label(
					"Name",
					vec![
						theme
							.link_colored(
								format!("{}\n", playing.name.as_str()).as_str(),
								&playing.url,
							)
							.into(),
					],
				)
				.into(),
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
							"https://git.kybe.xyz/2kybe3/cheat-sh",
							"https://git.kybe.xyz/2kybe3/cheat-sh",
						)
						.into(),
					theme.comment(" (a tiny wrapper for cheat.sh)\n").into(),
				],
			)
			.into(),
		theme
			.label(
				"kybe-backend",
				vec![
					theme
						.link_colored(
							"https://git.kybe.xyz/2kybe3/kybe-backend",
							"https://git.kybe.xyz/2kybe3/kybe-backend",
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
						.link_colored(ctx.url("/pgp\n"), &ctx.url("/pgp"))
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
				"Git",
				vec![
					theme
						.link_colored("https://git.kybe.xyz/\n", "https://git.kybe.xyz/")
						.into(),
				],
			)
			.into(),
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
				"Nix Config",
				vec![
					theme
						.link_colored(ctx.url("/nix\n"), &ctx.url("/nix"))
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Identity",
				vec![
					theme
						.link_colored(ctx.url("/ident.txt\n"), &ctx.url("/ident.txt"))
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Now Listening",
				vec![
					theme
						.link_colored(ctx.url("/now\n"), &ctx.url("/now"))
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"Canvas",
				vec![
					theme
						.link_colored(ctx.url("/canvas\n"), &ctx.url("/canvas"))
						.into(),
				],
			)
			.into(),
		theme
			.label(
				"IP",
				vec![theme.link_colored(ctx.url("/ip\n"), &ctx.url("/ip")).into()],
			)
			.into(),
	]);
	page.append(&mut common::footer::footer(trace.trace_id));

	let page = Page::from_iter(page);

	let (is_html, result) = page.render(&ctx.user_agent, "/", &state.config.webserver.umami);

	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	if is_html {
		(StatusCode::OK, Html(result)).into_response()
	} else {
		(StatusCode::OK, result).into_response()
	}
}
