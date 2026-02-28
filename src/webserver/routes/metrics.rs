use axum::response::IntoResponse;
use reqwest::StatusCode;

pub async fn metrics() -> impl IntoResponse {
	(
		StatusCode::OK,
		crate::prometheus::export_metrics().unwrap_or("ERROR".to_string()),
	)
		.into_response()
}
