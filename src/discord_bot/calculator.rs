use anyhow::anyhow;
use rsc::Interpreter;

use crate::discord_bot::{Context, Error};

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

    match evaluate(&expression, &mut rsc::Interpreter::default()) {
        Ok(result) => {
            ctx.reply(format!("**{}** = **{}**", expression, result))
                .await?;
        }
        Err(e) => {
            ctx.reply(format!("Error evaluating expression: {:?}", e))
                .await?;
        }
    }

    Ok(())
}

fn evaluate(input: &str, interpreter: &mut Interpreter<f64>) -> anyhow::Result<f64> {
    let tokens = rsc::tokenize(input).map_err(|e| anyhow!("error tokenizing: {e:?}"))?;
    let expr = rsc::parse(&tokens).map_err(|e| anyhow!("error parsing: {e:?}"))?;
    interpreter
        .eval(&expr)
        .map_err(|e| anyhow!("error evaluating: {e:?}"))
}
