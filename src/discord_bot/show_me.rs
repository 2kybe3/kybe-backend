use poise::serenity_prelude::Message;

use crate::discord_bot::{Context, Error, reply_or_attach};

#[poise::command(
    context_menu_command = "Show Me",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn show_me(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
    reply_or_attach(&ctx, serde_json::to_string_pretty(&msg)?, "msg", "json").await;
    Ok(())
}
