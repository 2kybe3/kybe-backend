use axum::{Extension, Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
	db::website_traces::{RequestStatus, WebsiteTrace},
	webserver::WebServerState,
};

pub async fn now_playing(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
) -> impl IntoResponse {
	let mut trace = trace.lock().await;
	trace.request_status = RequestStatus::Success;
	trace.status_code = StatusCode::OK.into();

	let playing = if let Some(lastfm) = state.lastfm {
		Some(lastfm.get_playing(Some(&mut trace.data)).await)
	} else {
		None
	};

	(StatusCode::OK, Json(playing)).into_response()
}
