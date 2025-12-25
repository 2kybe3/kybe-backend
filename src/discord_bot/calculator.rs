use crate::discord_bot::{Context, Error};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn calculate(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("calculating smth ig")
        .await
        .expect("TODO: panic message");
    Ok(())
}
