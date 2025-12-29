use crate::db::Database;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "command_status", rename_all = "lowercase")]
pub enum CommandStatus {
    Success,
    Error,
    Disabled,
    Unauthorized,
}

impl CommandStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Disabled => "disabled",
            Self::Unauthorized => "unauthorized",
        }
    }
}

impl TryFrom<&str> for CommandStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, <CommandStatus as TryFrom<&str>>::Error> {
        match s.to_lowercase().as_str() {
            "success" => Ok(Self::Success),
            "error" => Ok(Self::Error),
            "disabled" => Ok(Self::Disabled),
            "unauthorized" => Ok(Self::Unauthorized),
            other => Err(format!("Invalid command status: {}", other)),
        }
    }
}

#[derive(Debug)]
pub struct CommandTrace {
    pub trace_id: Uuid,
    pub command: String,
    pub user_id: i64,
    pub username: String,
    pub guild_id: Option<i64>,
    pub channel_id: i64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: i64,
    pub status: CommandStatus,
    pub input: serde_json::Value,
    pub data: serde_json::Value,
    pub output: Option<String>,
    pub error: Option<String>,
}

impl CommandTrace {
    pub fn start<S: Into<String>>(ctx: &crate::discord_bot::Context<'_>, command_name: S) -> Self {
        Self {
            trace_id: Uuid::now_v7(),
            command: command_name.into(),
            user_id: ctx.author().id.get() as i64,
            username: ctx.author().name.clone(),
            guild_id: ctx.guild_id().map(|id| id.get() as i64),
            channel_id: ctx.channel_id().get() as i64,
            started_at: chrono::Utc::now(),
            duration_ms: 0,
            status: CommandStatus::Success,
            input: serde_json::json!({}),
            data: serde_json::json!({}),
            output: None,
            error: None,
        }
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

impl Database {
    pub async fn save_command_trace(&self, log: &CommandTrace) -> Result<Uuid, sqlx::Error> {
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
}
