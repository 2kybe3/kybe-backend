use crate::config::types::Config;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

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

    pub async fn init(config: Arc<Config>) -> Result<Self, DbError> {
        info!("initializing db using: {:?}", config.database);

        let pool =
            PgPoolOptions::new().max_connections(5).connect(&config.database.postgres_url).await?;

        info!("DB connected");

        Ok(Self::new(pool, config))
    }
}
