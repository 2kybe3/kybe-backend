use crate::discord_bot::{Context, Error};
use crate::reply_or_attach;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn calculate(
    ctx: Context<'_>,
    #[description = "Mathematical expression (e.g. 1+2*(3^2) or sqrt(16) or sin(pi/2)"]
    expression: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    match meval::eval_str(&expression) {
        Ok(result) => {
            let response = format!("**{}** = **{}**", expression, result);
            reply_or_attach!(ctx, response, "result.txt");
        }
        Err(e) => {
            let error_msg = format!("Error evaluating expression: {}", e);
            reply_or_attach!(ctx, error_msg, "error.txt");
        }
    }

    Ok(())
}
