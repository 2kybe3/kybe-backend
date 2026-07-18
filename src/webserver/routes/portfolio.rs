use axum::{Extension, extract::State, http::header, response::IntoResponse};
use reqwest::StatusCode;

use crate::webserver::{
    RequestContext, WebServerState, common,
    render::{Page, Theme, object::Objects},
};

macro_rules! include_page {
    ($name:literal) => {
        format!(
            "{}\n",
            include_str!(concat!("../../../assets/portfolio/", $name)).trim()
        )
    };
}

macro_rules! git_repo {
    ($theme:expr,$url:literal) => {
        $theme
            .label("git", vec![$theme.link_colored($url, $url).into()])
            .into()
    };
}

pub async fn portfolio(
    State(state): State<WebServerState>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let theme = Theme::default();

    let mut page: Vec<Objects> = vec![
        theme.title_underlined("kybe :: portfolio"),
        theme.section_underlined("About me"),
        theme
            .text(format!("{}\n\n", include_page!("About Me").trim()))
            .into(),
        theme.section_underlined("Projects"),
        theme.sub_section_underlined("hexix"),
        theme
            .text(format!("{}\n", include_page!("projects/hexix").trim()))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/hexix"),
        theme.text("\n\n").into(),
        theme.sub_section_underlined("kymenu"),
        theme
            .text(format!("{}\n", include_page!("projects/kymenu").trim()))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/kymenu"),
        theme.text("\n\n").into(),
        theme.sub_section_underlined("kymenu-extras"),
        theme
            .text(format!(
                "{}\n",
                include_page!("projects/kymenu-extras").trim()
            ))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/kymenu-extras"),
        theme.text("\n\n").into(),
        theme.sub_section_underlined("webhook-router"),
        theme
            .text(format!(
                "{}\n",
                include_page!("projects/webhook-router").trim()
            ))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/webhook-router"),
        theme.text("\n\n").into(),
        theme.sub_section_underlined("cheat-sh"),
        theme
            .text(format!("{}\n", include_page!("projects/cheat-sh").trim()))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/cheat-sh"),
        theme.text("\n\n").into(),
        theme.sub_section_underlined("kybe-backend"),
        theme
            .text(format!(
                "{}\n",
                include_page!("projects/kybe-backend").trim()
            ))
            .into(),
        git_repo!(theme, "https://git.kybe.xyz/2kybe3/kybe-backend"),
        theme.text("\n\n").into(),
        theme.raw("\n").into(),
        theme.title_underlined("Contact"),
        theme
            .label(
                "PGP",
                vec![
                    theme
                        .link_colored(
                            "4B2067C3BD6D410F13E5 36A343CE43938A3C7A8F\n",
                            &ctx.url("/pgp"),
                        )
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
        theme
            .label(
                "Matrix",
                vec![
                    theme
                        .link_colored("@kybe:kybe.xyz", "https://matrix.to/#/@kybe:kybe.xyz")
                        .into(),
                    theme.comment(" (preferred)\n").into(),
                ],
            )
            .into(),
        theme.raw("\n").into(),
        theme.title_underlined("Other Platforms"),
        theme
            .label(
                "Git",
                vec![
                    theme
                        .link_colored("2kybe3", "https://git.kybe.xyz/2kybe3")
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored("renovate", "https://git.kybe.xyz/renovate")
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored("archive", "https://git.kybe.xyz/archive")
                        .into(),
                    theme.comment(" (most used)\n").into(),
                ],
            )
            .into(),
        theme
            .label(
                "Github",
                vec![
                    theme
                        .link_colored("kybe236", "https://github.com/kybe236")
                        .into(),
                    theme.text(", ").into(),
                    theme
                        .link_colored("2kybe3\n", "https://github.com/2kybe3")
                        .into(),
                ],
            )
            .into(),
        theme.raw("\n").into(),
        theme.title_underlined("Other Endpoints"),
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
    ];

    page.append(&mut common::footer::footer());

    let page = Page::from_iter("/portfolio", &state.config, page);

    let mut result = page.render(&ctx.user_agent);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, result.take_content_type())],
        result.take_data(),
    )
        .into_response()
}
