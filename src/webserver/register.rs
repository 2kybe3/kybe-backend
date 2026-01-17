use crate::db::website_traces::WebsiteTrace;
use crate::webserver::WebServerState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct RegisterPayload {
	#[validate(length(min = 3, max = 20))]
	username: String,
	#[validate(email)]
	email: String,
	#[validate(length(min = 5, max = 25))]
	password: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
	success: bool,
	error: Option<String>,
	trace_id: Option<Uuid>,
	user_id: Option<Uuid>,
}

impl RegistrationResponse {
	fn success(user_id: Uuid) -> Self {
		Self {
			success: true,
			error: None,
			trace_id: None,
			user_id: Some(user_id),
		}
	}

	fn error(msg: String, trace_id: Uuid) -> Self {
		Self {
			success: false,
			error: Some(msg),
			trace_id: Some(trace_id),
			user_id: None,
		}
	}
}

#[axum::debug_handler]
pub async fn register(
	State(state): State<WebServerState>,
	Extension(trace): Extension<Arc<Mutex<WebsiteTrace>>>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	let mut trace = trace.lock().await;
	trace.request_body = Some(serde_json::json!({
		"username": payload.username,
		"email": payload.email
	}));

	if let Err(e) = payload.validate() {
		let response = RegistrationResponse::error("Invalid input".into(), trace.trace_id);

		trace.error = Some(format!("{:?}", e));
		let response_json: Value = serde_json::to_value(&response).unwrap_or_default();
		trace.response_body = Some(response_json.clone());

		return (StatusCode::BAD_REQUEST, Json(response_json)).into_response();
	};

	let result = state
		.auth
		.register(payload.username, payload.email, payload.password)
		.await;

	match result {
		Ok(user_id) => {
			let response = RegistrationResponse::success(user_id);
			let response_json = serde_json::to_value(&response).unwrap_or_default();
			trace.response_body = Some(response_json.clone());

			(StatusCode::CREATED, Json(response_json)).into_response()
		}
		Err(auth_err) => {
			let (status, error_msg) = match auth_err {
				crate::auth::AuthError::UsernameTaken | crate::auth::AuthError::EmailTaken => {
					(StatusCode::CONFLICT, "Username or email already in use")
				}
				crate::auth::AuthError::PasswordHashing(_) => {
					error!("Password hashing error during registration: {auth_err:?}");
					(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
				}
				crate::auth::AuthError::DatabaseError(_) => {
					error!("Database error during registration: {auth_err:?}");
					(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
				}
			};

			let response = RegistrationResponse::error(error_msg.to_string(), trace.trace_id);
			let response_json = serde_json::to_value(&response).unwrap_or_default();

			trace.error = Some(format!("{:?}", auth_err));
			trace.response_body = Some(response_json.clone());

			(status, Json(response_json)).into_response()
		}
	}
}
