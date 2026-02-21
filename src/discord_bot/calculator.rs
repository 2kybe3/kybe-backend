use anyhow::anyhow;
use rsc::Interpreter;

use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error, defer, finalize_command_trace};

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
	let mut trace = CommandTrace::start(&ctx, "calculate");

	trace.input = serde_json::json!({
		"expression": expression,
	});

	defer(&ctx, &mut trace).await?;

	match evaluate(&expression, &mut rsc::Interpreter::default()) {
		Ok(result) => {
			let response = format!("**{}** = **{}**", expression, result);
			trace.output = Some(response.clone());
			ctx.reply(response).await?;
		}
		Err(e) => {
			trace.status = CommandStatus::Error;
			trace.error = Some(format!("{:?}", e));
			trace.output = Some(format!("Error evaluating expression: {:?}", e));
			ctx.reply(format!("Error evaluating expression: {:?}", e))
				.await?;
		}
	}

	finalize_command_trace(&ctx, &mut trace).await?;

	Ok(())
}

fn evaluate(input: &str, interpreter: &mut Interpreter<f64>) -> anyhow::Result<f64> {
	let tokens = rsc::tokenize(input).map_err(|e| anyhow!("error tokenizing: {e:?}"))?;
	let expr = rsc::parse(&tokens).map_err(|e| anyhow!("error parsing: {e:?}"))?;
	interpreter
		.eval(&expr)
		.map_err(|e| anyhow!("error evaluating: {e:?}"))
}
