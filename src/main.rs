mod notifications;
mod config;

use tracing::{error, info};
use crate::config::types::Config;
use crate::notifications::{Notification, Notifications};

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
    let notification = Notification::new("Started".to_string(), "Backend started successfully".to_string(), 0);

    notifications.notify(notification).await;
    Ok(())
}

