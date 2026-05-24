use axum::{Extension, extract::State, http::header, response::IntoResponse};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    db::website_traces::{RequestStatus, WebsiteTrace},
    webserver::{
        RequestContext, TERMINAL_PROMPT, WebServerState, common,
        render::{
            Page, Theme,
            builders::{CodeBlockBuilder, TextBlobBuilder},
        },
    },
};

pub async fn pgp(
    State(state): State<WebServerState>,
    Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let theme = Theme::default();

    let mut page = vec![
        theme
            .title("Hello Stranger, and maybe PGP user :-)\n\n")
            .into(),
        CodeBlockBuilder::new(vec![
            theme.terminal_prompt(TERMINAL_PROMPT).into(),
            TextBlobBuilder::new("curl https://kybe.xyz/pgp.txt | gpg --import").into(),
        ])
        .title("Curl")
        .into(),
        CodeBlockBuilder::new(vec![
            theme.terminal_prompt(TERMINAL_PROMPT).into(),
            TextBlobBuilder::new("resolvectl openpgp kybe@kybe.xyz --raw=payload | gpg --import")
                .into(),
        ])
        .title("RFC7929")
        .into(),
        CodeBlockBuilder::new(vec![
            theme.terminal_prompt(TERMINAL_PROMPT).into(),
            TextBlobBuilder::new("ssh ssh.kybe.xyz pgp | gpg --import").into(),
        ])
        .title("SSH (IPv6 required)")
        .into(),
        CodeBlockBuilder::new(vec![
            TextBlobBuilder::new(include_str!("../../../static/pgp.txt").trim()).into(),
        ])
        .title("kybe <kybe@kybe.xyz> | 4B2067C3BD6D410F13E536A343CE43938A3C7A8F")
        .into(),
    ];
    page.append(&mut common::footer::footer(trace.lock().await.trace_id));

    let page = Page::from_iter("/pgp", &state.config, page);

    let mut result = page.render(&ctx.user_agent);

    let mut trace = trace.lock().await;

    trace.request_status = RequestStatus::Success;
    trace.status_code = StatusCode::OK.into();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, result.take_content_type())],
        result.take_data(),
    )
        .into_response()
}
