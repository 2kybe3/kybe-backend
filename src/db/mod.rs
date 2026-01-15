pub mod command_traces;
pub mod users;
pub mod website_traces;

use crate::config::types::Config;
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

/// PgPool is already wrapped in an Arc so this is fine to clone for now
#[derive(Clone, Debug)]
pub struct Database {
	pool: PgPool,
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl Database {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	pub fn pool(&self) -> &PgPool {
		&self.pool
	}

	pub async fn init(config: Arc<Config>) -> Result<Self, DbError> {
		let pool = PgPoolOptions::new()
			.max_connections(config.database.max_connections)
			.acquire_timeout(Duration::from_secs(30))
			.connect(&config.database.postgres_url)
			.await?;

		info!("starting migrations");

		MIGRATOR.run(&pool).await?;

		info!("Migrations applied successfully");

		let db = Self::new(pool);

		db.delete_old_unverified_users_loop().await;

		Ok(db)
	}
}
