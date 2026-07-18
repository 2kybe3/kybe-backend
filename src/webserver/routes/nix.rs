use axum::{Extension, extract::State, http::header, response::IntoResponse};
use reqwest::StatusCode;

use crate::webserver::{
    RequestContext, WebServerState,
    render::{Page, Theme, builders::ImageBuilder, object::Objects},
};

pub async fn nix(
    State(state): State<WebServerState>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let theme = Theme::default();

    let page: Vec<Objects> = vec![
        theme.title("/dev/urandom rice\n").into(),
        theme.subtitle("My NixOS config\n\n").into(),
        theme
            .label(
                "Download",
                vec![
                    theme
                        .link_colored("git", "https://git.kybe.xyz/2kybe3/hexix")
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored(
                            "tar.gz",
                            "https://git.kybe.xyz/2kybe3/hexix/archive/main.tar.gz",
                        )
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored("zip", "https://git.kybe.xyz/2kybe3/hexix/archive/main.zip")
                        .into(),
                ],
            )
            .into(),
        theme.text("\n\n").into(),
        ImageBuilder::new(ctx.url("/static/nix.png"), "My Config Demo", 800, 450).into(),
    ];

    let page = Page::from_iter("/dev/nix", &state.config, page);

    let mut result = page.render(&ctx.user_agent);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, result.take_content_type())],
        result.take_data(),
    )
        .into_response()
}
