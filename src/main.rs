mod notifications;
mod config;

use tracing::{error, info};
use crate::config::types::Config;
use crate::notifications::{Notification, Notifications, NotificationConfig};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("initializing backend");

    let config = match Config::init().await {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let notifications = Notifications::new(
        NotificationConfig {
            gotify_enabled: config.notification.gotify.enabled,
            gotify_url: config.notification.gotify.url,
            discord_enabled: config.notification.discord.enabled,
            discord_url: config.notification.discord.url,
            log_enabled: config.notification.log.enabled,
        }
    );
    let notification = Notification::new("Started".to_string(), "Backend started successfully".to_string(), 0);

    notifications.notify(notification).await;
}

