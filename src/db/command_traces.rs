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

#[derive(Debug, sqlx::FromRow)]
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

    #[allow(unused)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
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
            created_at: None,
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
            log.status.clone() as CommandStatus,
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
        sqlx::query_as!(
            CommandTrace,
            r#"
            SELECT
                trace_id, command, user_id, username, guild_id, channel_id,
                started_at, duration_ms, status AS "status: CommandStatus", input, data, output, error, created_at
            FROM command_traces
            WHERE trace_id = $1
            "#,
            trace_id
        )
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_latest_command_trace(&self) -> Result<Option<CommandTrace>, sqlx::Error> {
        sqlx::query_as!(
            CommandTrace,
            r#"
            SELECT
                trace_id, command, user_id, username, guild_id, channel_id,
                started_at, duration_ms, status AS "status: CommandStatus", input, data, output, error, created_at
            FROM command_traces
            ORDER BY started_at DESC
            LIMIT 1
            "#
        )
            .fetch_optional(&self.pool)
            .await
    }
}
