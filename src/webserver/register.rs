use crate::db::website_traces::WebsiteTrace;
use crate::webserver::{WebServerState, client_ip, finish_trace};
use axum::Json;
use axum::extract::{ConnectInfo, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
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
	headers: HeaderMap,
	RawQuery(query): RawQuery,
	State(state): State<WebServerState>,
	ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
	Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
	const METHOD: &str = "POST";
	const PATH: &str = "/register";

	let user_agent = headers
		.get(axum::http::header::USER_AGENT)
		.and_then(|v| v.to_str().ok())
		.map(|s| s.to_string());

	let ip = client_ip(&headers, remote_addr, &*state.config);

	let mut trace = WebsiteTrace::start(METHOD, PATH.to_string(), query, user_agent, ip);

	trace.request_body = Some(serde_json::json!({
		"username": payload.username,
		"email": payload.email
	}));

	if let Err(e) = payload.validate() {
		let response = RegistrationResponse::error("Invalid input".into(), trace.trace_id);

		trace.error = Some(format!("{:?}", e));
		let response_json: Value = serde_json::to_value(&response).unwrap_or_default();
		trace.response_body = Some(response_json.clone());
		finish_trace(
			&mut trace,
			StatusCode::BAD_REQUEST.as_u16(),
			None,
			&state.database,
		)
		.await;

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
			finish_trace(
				&mut trace,
				StatusCode::CREATED.as_u16(),
				Some(user_id),
				&state.database,
			)
			.await;

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
			finish_trace(&mut trace, status.as_u16(), None, &state.database).await;

			(status, Json(response_json)).into_response()
		}
	}
}
