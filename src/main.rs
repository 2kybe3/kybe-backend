mod config;
mod db;
mod discord_bot;
mod notifications;
pub mod translator;

use crate::config::types::Config;
use crate::db::init_db;
use crate::discord_bot::init_bot;
use crate::notifications::notification_types::startup::StartupNotification;
use crate::notifications::{Notification, Notifications};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    tracing_subscriber::fmt::init();
    info!("initializing...");

    if args.iter().any(|arg| arg == "--generate-example") {
        Config::create_local_default().await?;
        println!("Generated config.toml.example");
        return Ok(());
    }

    let config = match Config::init().await {
        Ok(cfg) => Arc::new(cfg),
        Err(e) => {
            error!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let notifications = Arc::new(Notifications::new(&config.notification));

    let notification = StartupNotification::new(false);
    notifications.notify(notification).await;

    if !config.database.force_disable {
        init_db(Arc::clone(&config)).await.unwrap();
    }

    let notifications_clone = notifications.clone();
    let bot_handle = tokio::spawn(async move {
        if let Err(e) = init_bot(notifications_clone.clone(), Arc::clone(&config)).await {
            notifications_clone
                .notify(Notification::new("Discord Bot Failure", &e.to_string()))
                .await;
        }
    });

    let notification = StartupNotification::new(true);
    notifications.notify(notification).await;

    bot_handle.await?;
    Ok(())
}
