use crate::notifications::{Notification, Notifications};
use axum::{Json, Router};
use axum::routing::{get, post};
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::State;
use serde::Deserialize;
use tokio::net::TcpListener;
use crate::auth::Auth;
use crate::config::types::Config;
use crate::db::Database;

#[derive(Clone)]
struct WebServerState {
    config: Arc<Config>,
    auth: Arc<Auth>,
    database: Database,
}

#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
    email: String,
    password: String,
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[axum::debug_handler]
async fn register(
    State(state): State<WebServerState>,
    Json(payload): Json<RegisterPayload>
) -> Result<String, String> {
    state.auth.register(payload.username, payload.email, payload.password).await.unwrap();

    Ok("some".into())
}

pub async fn init_webserver(
    notifications_clone: Arc<Notifications>,
    config: Arc<Config>,
    auth: Arc<Auth>,
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
    auth: Arc<Auth>,
    database: Database,
) -> Result<(), anyhow::Error> {
    let webserver_state = WebServerState {
        config,
        auth,
        database,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .with_state(webserver_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::serve(TcpListener::bind(addr).await.unwrap(), app).await?;
    Ok(())
}