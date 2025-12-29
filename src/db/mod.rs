pub mod command_traces;
pub mod users;
pub mod website_traces;

use crate::config::types::Config;
use crate::notifications::{Notification, Notifications};
use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLx error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] MigrateError),
}

#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
    _config: Arc<Config>,
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl Database {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Self { pool, _config: config }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn inner_init(config: Arc<Config>) -> Result<Self, DbError> {
        let pool =
            PgPoolOptions::new().max_connections(5).connect(&config.database.postgres_url).await?;

        Ok(Self::new(pool, config))
    }

    pub async fn init(
        config: Arc<Config>,
        notifications: Arc<Notifications>,
    ) -> Result<Self, DbError> {
        info!("initializing db using: {:?}", config.database);

        for attempt in 1..=5 {
            match Self::inner_init(Arc::clone(&config)).await {
                Ok(db) => {
                    MIGRATOR
                        .run(&db.pool)
                        .await
                        .map(|_| info!("Migrations applied successfully"))?;
                    return Ok(db);
                }
                Err(e) => {
                    let wait = 5 * attempt;
                    let text = format!(
                        "DB connection failed, retrying in {}s (attempt {})",
                        wait, attempt
                    );
                    info!(text);
                    notifications.notify(Notification::new("Database", &text)).await;
                    if attempt == 5 {
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
            }
        }

        unreachable!();
    }
}
