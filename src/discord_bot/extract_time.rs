use poise::serenity_prelude::Timestamp;

use crate::discord_bot::{Context, reply_or_attach};

const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn extract_time(ctx: Context<'_>, id: String) -> anyhow::Result<()> {
    let id = match id.parse::<u64>() {
        Ok(ip) => ip,
        Err(e) => {
            reply_or_attach(&ctx, format!("Invalid ID: {:?}", e), "error", "txt").await;
            return Ok(());
        }
    };

    let time = Timestamp::from_millis(((id >> 22) + DISCORD_EPOCH) as i64)?;

    reply_or_attach(&ctx, time.to_rfc2822(), "time", "txt").await;
    Ok(())
}
