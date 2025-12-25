mod config;
mod notifications;

use crate::config::types::Config;
use crate::notifications::Notifications;
use crate::notifications::notification_types::startup::StartupNotification;
use tracing::{error, info};

#[tokio::main]
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
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let notifications = Notifications::new(&config.notification);
    let notification = StartupNotification::new(false);
    notifications.notify(notification).await;

    // TODO: startup logic

    let notification = StartupNotification::new(true);
    notifications.notify(notification).await;
    Ok(())
}
