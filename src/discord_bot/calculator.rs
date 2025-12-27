use crate::discord_bot::{CommandLog, Context, Error};
use crate::reply_or_attach;
use std::time::Instant;
use tracing::{debug, error};

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
    let start = Instant::now();
    let mut log = CommandLog::start(&ctx, "calculate");

    log.input = serde_json::json!({
        "expression": expression,
    });

    ctx.defer().await?;

    match meval::eval_str(&expression) {
        Ok(result) => {
            let response = format!("**{}** = **{}**", expression, result);
            log.output = Some(response.clone());
            reply_or_attach!(ctx, response, "result.txt");
        }
        Err(e) => {
            log.status = "error";
            log.error = Some(format!("{:?}", e));
            log.output = Some("Error evaluating expression".into());
            ctx.reply("Error evaluating expression").await?;
        }
    }

    log.duration_ms = start.elapsed().as_millis();

    if log.status == "error" {
        error!(log = ?log, "command finished");
    } else {
        debug!(log = ?log, "command finished");
    }

    Ok(())
}
