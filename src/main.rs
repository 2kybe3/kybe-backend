mod notifications;

use tracing::info;
use crate::notifications::{Notification, Notifications, NotificationsConfig};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("initializing backend");

    // TODO: load from a config file
    let notifications = Notifications::new(
        NotificationsConfig {
            gotify_enabled: true,
            gotify_url: "https://gotify.kybe.xyz".into(),
            gotify_token: "".into(),
            log_enabled: true,
        }
    );
    let notification = Notification::new("Started".to_string(), "Backend started successfully".to_string(), 0);

    notifications.notify(notification).await;
}

