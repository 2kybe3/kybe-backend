use crate::config::types::Config;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;
use crate::notifications::{Notification, Notifications};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLx error: {0}")]
    Connection(#[from] sqlx::Error),
}

#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
    #[allow(unused)]
    config: Arc<Config>,
}

impl Database {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Self { pool, config }
    }

    #[allow(unused)]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn init(config: Arc<Config>, notifications: Arc<Notifications>) -> Result<Self, DbError> {
        info!("initializing db using: {:?}", config.database);

        for attempt in 1..=5 {
            match PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database.postgres_url)
                .await
            {
                Ok(pool) => {
                    info!("DB connected");
                    return Ok(Self::new(pool, Arc::clone(&config)));
                }
                Err(e) => {
                    let wait = 5 * attempt;
                    let text = format!("DB connection failed, retrying in {}s (attempt {})", wait, attempt);
                    info!(text);
                    notifications.notify(Notification::new("Database", &text)).await;
                    if attempt == 5 {
                        return Err(DbError::Connection(e));
                    }
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
            }
        }

        unreachable!();
    }
}
