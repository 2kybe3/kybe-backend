mod notifications;

use tracing::info;
use crate::notifications::{Notification, Notifications};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("initializing backend");

    let notifications = Notifications::new();
    let notification = Notification::new("Started".to_string(), "Backend started successfully".to_string(), 0);

    notifications.notify(notification).await;
}

