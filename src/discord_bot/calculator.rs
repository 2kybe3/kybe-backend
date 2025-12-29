use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::{finalize_command_trace, reply_or_attach};
use std::time::Instant;

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
    let mut log = CommandTrace::start(&ctx, "calculate");

    log.input = serde_json::json!({
        "expression": expression,
    });

    if let Err(e) = ctx.defer().await {
        log.status = CommandStatus::Error;
        log.error = Some(format!("Defer failed: {:?}", e));

        finalize_command_trace!(ctx, log, start);
        return Err(e.into());
    }

    match meval::eval_str(&expression) {
        Ok(result) => {
            let response = format!("**{}** = **{}**", expression, result);
            log.output = Some(response.clone());
            reply_or_attach!(ctx, response, "result.txt");
        }
        Err(e) => {
            log.status = CommandStatus::Error;
            log.error = Some(format!("{:?}", e));
            log.output = Some("Error evaluating expression".into());
            ctx.reply("Error evaluating expression").await?;
        }
    }

    finalize_command_trace!(ctx, log, start);

    Ok(())
}
