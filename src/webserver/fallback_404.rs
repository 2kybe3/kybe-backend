use axum::{
	Extension,
	extract::State,
	response::{Html, IntoResponse},
};
use reqwest::StatusCode;

use crate::webserver::{
	RequestContext, WebServerState,
	render::{Page, Theme, builders::TextBlobBuilder},
};

pub async fn fallback_404(
	State(state): State<WebServerState>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let theme = Theme::default();

	let page = Page::from_iter([
		theme
			.title("Look's like we couldn't server your request ")
			.into(),
		TextBlobBuilder::new(":-(")
			.style(theme.title.bold(true))
			.into(),
	]);

	let is_cli = ctx.user_agent.contains("curl") || ctx.user_agent.contains("lynx");
	let result = if is_cli {
		page.render_ansi()
	} else {
		page.render_html_page("kybe - 404", &state.config.webserver.umami)
	};

	if is_cli {
		(StatusCode::NOT_FOUND, result).into_response()
	} else {
		(StatusCode::NOT_FOUND, Html(result)).into_response()
	}
}
