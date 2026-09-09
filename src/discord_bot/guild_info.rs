use poise::serenity_prelude::GuildId;

use crate::discord_bot::{Context, reply_or_attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn guild_info(ctx: Context<'_>, guild: String) -> anyhow::Result<()> {
    let id = match guild.parse::<u64>() {
        Ok(ip) => ip,
        Err(e) => {
            reply_or_attach(&ctx, format!("Invalid guild ID: {:?}", e), "error", "txt").await;
            return Ok(());
        }
    };

    reply_or_attach(
        &ctx,
        serde_json::to_string_pretty(&GuildId::new(id).get_preview(ctx).await?)?,
        "guild",
        "json",
    )
    .await;
    Ok(())
}
