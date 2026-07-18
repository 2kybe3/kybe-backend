use crate::discord_bot::{Context, Error};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!("version: `{}`", crate::GIT_SHA.to_owned()))
        .await?;
    Ok(())
}
