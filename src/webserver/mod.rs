use crate::notifications::{Notification, Notifications};
use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn init_webserver(notifications_clone: Arc<Notifications>) {
    if let Err(e) = init_webserver_inner().await {
        notifications_clone
            .notify(Notification::new("Webserver", &format!("Webserver failed: {:?}", e)))
            .await;
    }
}

async fn init_webserver_inner() -> Result<(), anyhow::Error> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::serve(TcpListener::bind(addr).await.unwrap(), app).await?;
    Ok(())
}
