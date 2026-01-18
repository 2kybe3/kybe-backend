use std::sync::Arc;

use axum::{Extension, response::IntoResponse};
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::RequestContext,
};

pub async fn ip(
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	(StatusCode::OK, ctx.ip.to_string()).into_response()
}
