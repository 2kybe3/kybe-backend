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
	webserver::{
		RequestContext, WebServerState,
		render::{Objects, Page, Theme, builders::ImageBuilder},
	},
};

pub async fn nix(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let theme = Theme::default();

	let page: Vec<Objects> = vec![
		theme.title("/dev/urandom rice\n").into(),
		theme.subtitle("My nixos config\n\n").into(),
		theme
			.label(
				"Download",
				vec![
					theme
						.link_colored(
							"https://git.kybe.xyz/kybe/nix-dotfiles\n",
							"https://git.kybe.xyz/kybe/nix-dotfiles",
						)
						.into(),
				],
			)
			.into(),
		theme.text("\n").into(),
		ImageBuilder::new(ctx.url("/static/nix.png"), "My Config Demo", 800, 450).into(),
	];

	let page = Page::from_iter(page);

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
