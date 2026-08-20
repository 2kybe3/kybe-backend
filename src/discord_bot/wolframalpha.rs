use crate::discord_bot::{Context, Error, reply_or_attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn wolframalpha(ctx: Context<'_>, expression: String) -> Result<(), Error> {
    ctx.defer().await?;

    let res = match ctx.data().wolframalpha.query(expression).await {
        Ok(v) => v,
        Err(e) => {
            reply_or_attach(&ctx, e.to_string(), "error", "txt").await;
            return Ok(());
        }
    };

    let mut response = String::new();
    for pod in res {
        let mut description = String::new();
        for subpod in pod.subpods {
            description.push_str(&subpod.plaintext);
        }

        if description.is_empty() {
            continue;
        }

        let description = description.lines().collect::<Vec<_>>().join("\n> ");

        response.push_str(&format!("## {}\n", pod.title));
        response.push_str(&format!("> {description}\n"));
    }

    reply_or_attach(&ctx, response, "response", "md").await;

    Ok(())
}
