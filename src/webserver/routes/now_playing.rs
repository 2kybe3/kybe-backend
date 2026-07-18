use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;

use crate::webserver::WebServerState;

pub async fn now_playing(State(state): State<WebServerState>) -> impl IntoResponse {
    let playing = if let Some(lastfm) = state.lastfm {
        Some(lastfm.get_playing().await)
    } else {
        None
    };

    (StatusCode::OK, Json(playing)).into_response()
}
