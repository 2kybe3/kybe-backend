use crate::discord_bot::{Context, Error};
use crate::roa;

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
            roa!(ctx, response, "result.txt");
        }
        Err(e) => {
            let error_msg = format!("Error evaluating expression: {}", e);
            roa!(ctx, error_msg, "error.txt");
        }
    }

    Ok(())
}
