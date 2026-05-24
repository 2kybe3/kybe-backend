use std::sync::Arc;

use axum::{
    Extension,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    db::website_traces::{RequestStatus, WebsiteTrace},
    webserver::{
        RequestContext, WebServerState,
        render::{
            Page, Style,
            builders::{COLOR_MAPPING, CanvasBuilder, TextBlobBuilder},
            color::bit4::Bit4Color,
        },
    },
};

#[derive(Deserialize)]
pub struct CanvasParameters {
    pub q: Option<String>,
}

pub async fn canvas(
    State(state): State<WebServerState>,
    Query(parsed_query): Query<CanvasParameters>,
    Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let q = parsed_query.q.clone();
    let page = match q {
        Some(q) => vec![
            CanvasBuilder::new(q).into(),
            TextBlobBuilder::new("\n").style(Style::new()).into(),
        ],
        None => {
            let mut list = COLOR_MAPPING
                .iter()
                .map(|(key, value)| format!("{key}: {value:?}"))
                .collect::<Vec<_>>();
            list.push("NL: NewLine".into());
            vec![
				TextBlobBuilder::new("Canvas\n\n")
					.style(Style::new().fg(Bit4Color::RED))
					.into(),
				TextBlobBuilder::new("Use the q query parameter to use this canvas api\n\n").into(),
				TextBlobBuilder::new(list.join("\n"))
					.style(Style::new().fg(Bit4Color::YELLOW))
					.into(),
				TextBlobBuilder::new(
					"\n\nExample: https://kybe.xyz/canvas?q=BLBLBLBLBLBLBLBLBLBLNLRRRRRRRRRRNLYYYYYYYYYY\n",
				)
				.into(),
			]
        }
    };

    let page = Page::from_iter("/dev/canvas", &state.config, page);

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
