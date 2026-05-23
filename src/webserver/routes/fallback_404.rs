use axum::{Extension, extract::State, http::header, response::IntoResponse};
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

	let page = vec![
		theme
			.title("Look's like we couldn't serve your request ")
			.into(),
		TextBlobBuilder::new(":-(")
			.style(theme.title.bold(true))
			.into(),
	];

	let page = Page::from_iter("/dev/null", &state.config, page);

	let mut result = page.render(&ctx.user_agent);

	(
		StatusCode::OK,
		[(header::CONTENT_TYPE, result.take_content_type())],
		result.take_data(),
	)
		.into_response()
}
