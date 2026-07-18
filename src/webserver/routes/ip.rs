use axum::{Extension, Json, response::IntoResponse};
use reqwest::StatusCode;

use crate::webserver::RequestContext;

pub async fn ip(Extension(ctx): Extension<RequestContext>) -> impl IntoResponse {
    (StatusCode::OK, Json(ctx)).into_response()
}
