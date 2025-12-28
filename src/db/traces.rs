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
