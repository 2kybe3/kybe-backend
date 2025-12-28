pub mod traces;

use crate::config::types::Config;
use crate::db::traces::{CommandStatus, CommandTrace};
use crate::notifications::{Notification, Notifications};
use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

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
    #[allow(unused)]
    config: Arc<Config>,
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl Database {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Self { pool, config }
    }

    #[allow(unused)]
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

    pub async fn get_command_trace(
        &self,
        trace_id: Uuid,
    ) -> Result<Option<CommandTrace>, sqlx::Error> {
        let row = sqlx::query_as!(
            CommandTraceRow,
            r#"
            SELECT
                trace_id, command, user_id, username, guild_id, channel_id,
                started_at, duration_ms, status, input, data, output, error
            FROM command_traces
            WHERE trace_id = $1
            "#,
            trace_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_latest_command_trace(&self) -> Result<Option<CommandTrace>, sqlx::Error> {
        let row = sqlx::query_as!(
            CommandTraceRow,
            r#"
            SELECT
                trace_id, command, user_id, username, guild_id, channel_id,
                started_at, duration_ms, status, input, data, output, error
            FROM command_traces
            ORDER BY started_at DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn save_command_trace(
        &self,
        log: &CommandTrace,
    ) -> Result<sqlx::types::Uuid, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO command_traces (
                trace_id, command, user_id, username, guild_id, channel_id,
                started_at, duration_ms, status, input, data, output, error
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING trace_id
            "#,
            log.trace_id,
            log.command,
            log.user_id,
            log.username.as_str(),
            log.guild_id,
            log.channel_id,
            log.started_at,
            log.duration_ms,
            log.status.as_str(),
            &log.input,
            &log.data,
            log.output.as_deref(),
            log.error.as_deref()
        )
        .fetch_one(self.pool())
        .await?;

        Ok(row.trace_id)
    }
}

#[derive(sqlx::FromRow)]
pub struct CommandTraceRow {
    trace_id: Uuid,
    command: String,
    user_id: i64,
    username: String,
    guild_id: Option<i64>,
    channel_id: i64,
    started_at: chrono::DateTime<chrono::Utc>,
    duration_ms: i64,
    status: String,
    input: serde_json::Value,
    data: serde_json::Value,
    output: Option<String>,
    error: Option<String>,
}

impl From<CommandTraceRow> for CommandTrace {
    fn from(row: CommandTraceRow) -> Self {
        Self {
            trace_id: row.trace_id,
            command: row.command,
            user_id: row.user_id,
            username: row.username,
            guild_id: row.guild_id,
            channel_id: row.channel_id,
            started_at: row.started_at,
            duration_ms: row.duration_ms,
            status: CommandStatus::try_from(row.status.as_str()).unwrap_or(CommandStatus::Error),
            input: row.input,
            data: row.data,
            output: row.output,
            error: row.error,
        }
    }
}
