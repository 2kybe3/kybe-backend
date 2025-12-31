use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::db::website_traces::WebsiteTrace;
use crate::notifications::{Notification, Notifications};
use axum::extract::{ConnectInfo, RawQuery, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
struct WebServerState {
    config: Arc<Config>,
    auth: Arc<AuthService>,
    database: Database,
}

#[derive(Deserialize, Serialize)]
struct RegisterPayload {
    username: String,
    email: String,
    password: String,
}

async fn root() -> &'static str {
    "Hello, Stranger!"
}

macro_rules! website_trace_from_headers {
    ($method:expr, $path:expr, $query:expr, $headers:expr, $remote_addr:expr, $state:expr) => {{
        let user_agent = $headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok());

        let ip = if $state.config.webserver.behind_proxy {
            $headers
                .get($state.config.webserver.trust_proxy_header.clone())
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        } else {
            Some($remote_addr.to_string())
        };

        WebsiteTrace::start($method, $path, $query, user_agent, ip)
    }};
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    success: bool,
    error: Option<String>,
    trace_id: Option<Uuid>,
}

#[axum::debug_handler]
async fn register(
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    State(state): State<WebServerState>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let mut trace: WebsiteTrace =
        website_trace_from_headers!("POST", "/register", query, headers, remote_addr, state);

    trace.request_body = Some(serde_json::json!({
        "username": payload.username,
        "email": payload.email
    }));

    let duration = chrono::Utc::now().signed_duration_since(trace.started_at).num_milliseconds();

    match state.auth.register(payload.username, payload.email, payload.password).await {
        Ok(_) => {
            trace.complete(duration, 201, None);

            if let Err(e) = state.database.save_website_trace(&trace).await {
                error!("error saving trace {:?}", e)
            }

            (StatusCode::CREATED, "Registration successful".into())
        }
        Err(e) => {
            trace.complete(duration, 400, None);

            let response = serde_json::json!(RegistrationResponse {
                success: false,
                error: Some(format!("{:?}", e)),
                trace_id: Some(trace.trace_id),
            });

            let response_str = match serde_json::to_string_pretty(&response) {
                Ok(v) => v,
                Err(e) => {
                    error!("error pretty printing {:?}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".into());
                }
            };

            trace.response_body = Some(response);

            if let Err(e) = state.database.save_website_trace(&trace).await {
                error!("error saving trace {:?}", e)
            }

            (StatusCode::BAD_REQUEST, response_str)
        }
    }
}

pub async fn init_webserver(
    notifications_clone: Arc<Notifications>,
    config: Arc<Config>,
    auth: Arc<AuthService>,
    database: Database,
) {
    if let Err(e) = init_webserver_inner(config, auth, database).await {
        notifications_clone
            .notify(Notification::new("Webserver", &format!("Webserver failed: {:?}", e)))
            .await;
    }
}

async fn init_webserver_inner(
    config: Arc<Config>,
    auth: Arc<AuthService>,
    database: Database,
) -> Result<(), anyhow::Error> {
    let webserver_state = WebServerState { auth, config, database };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .with_state(webserver_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
