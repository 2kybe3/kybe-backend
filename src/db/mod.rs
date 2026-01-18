pub mod command_traces;
pub mod users;
pub mod website_traces;

use crate::config::types::Config;
use crate::maxmind::MaxMind;
use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use std::net::IpAddr;
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

	pub async fn sync_maxmind(&self, mm: Arc<MaxMind>) -> anyhow::Result<()> {
		let traces = sqlx::query!(
			r#"
            SELECT trace_id, ip_address
            FROM website_traces
            WHERE mm_city is NULL OR mm_asn is NULL
            "#
		)
		.fetch_all(self.pool())
		.await?;

		for trace in traces {
			if let Some(ip_str) = trace.ip_address
				&& let Ok(ip) = ip_str.parse::<IpAddr>()
				&& let Ok(lookup) = mm.lookup(ip)
				&& let Some(mm_city) = lookup.0
				&& let Some(mm_asn) = lookup.1
			{
				info!(
					"updating {} to {:?} and {:?}",
					trace.trace_id, mm_city, mm_asn
				);
				sqlx::query!(
					r#"
                    UPDATE website_traces
                    SET mm_city = $1, mm_asn = $2
                    WHERE trace_id = $3
                    "#,
					serde_json::to_value(mm_city)?,
					serde_json::to_value(mm_asn)?,
					trace.trace_id,
				)
				.execute(self.pool())
				.await?;
			}
		}

		Ok(())
	}
}
