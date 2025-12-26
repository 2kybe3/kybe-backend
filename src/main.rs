mod config;
mod db;
mod discord_bot;
mod notifications;
pub mod translator;

use crate::config::types::Config;
use crate::db::Database;
use crate::discord_bot::init_bot;
use crate::notifications::{Notification, Notifications};
use std::sync::Arc;
use std::time::{Duration, Instant};
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

    let database = match Database::init(Arc::clone(&config), Arc::clone(&notifications)).await {
        Ok(db) => db,
        Err(e) => {
            error!("Database init failed: {e}");
            notifications
                .notify(Notification::new("Database", &format!("Database init failed: {e}")))
                .await;
            std::process::exit(1);
        }
    };

    let notifications_clone = Arc::clone(&notifications);
    let bot_handle = tokio::spawn(async move {
        let mut retries = 0;
        let mut last_failure: Option<Instant> = None;

        loop {
            match init_bot(notifications_clone.clone(), config.clone(), database.clone()).await {
                Ok(_) => break,
                Err(e) => {
                    let now = Instant::now();

                    if let Some(last) = last_failure
                        && now.duration_since(last) > Duration::from_mins(5)
                    {
                        retries = 0;
                    }

                    last_failure = Some(now);
                    retries += 1;

                    notifications_clone
                        .notify(Notification::new(
                            "Discord Bot Critical Failure",
                            &format!("attempt {retries}: {e}"),
                        ))
                        .await;

                    if retries >= 5 {
                        break;
                    }

                    tokio::time::sleep(Duration::from_secs(5 * retries)).await;
                }
            }
        }
    });

    bot_handle.await?;
    Ok(())
}
