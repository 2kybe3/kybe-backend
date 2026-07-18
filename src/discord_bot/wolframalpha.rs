use crate::discord_bot::{Context, Error};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn wolframalpha(ctx: Context<'_>, expression: String) -> Result<(), Error> {
    ctx.defer().await?;

    let res = ctx.data().wolframalpha.query(expression).await?;

    let mut response = String::new();
    for pod in res {
        let mut description = String::new();
        for subpod in pod.subpods {
            description.push_str(&subpod.plaintext);
        }

        if description.is_empty() {
            continue;
        }

        response.push_str(&format!("# {}\n", pod.title));
        response.push_str(&format!("```\n{description}\n```\n"));
    }

    ctx.reply(response).await?;

    Ok(())
}
