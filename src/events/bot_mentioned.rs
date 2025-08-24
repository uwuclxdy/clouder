use crate::config::AppState;
use crate::serenity;

/// Handle message events - primarily for bot mention help responses
pub async fn on_mention(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) {
    if message.author.bot {
        return;
    }

    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to get current user: {}", e);
            return;
        }
    };

    if message.mentions.iter().any(|u| u.id == current_user.id) {
        if let Err(e) = send_help_as_message(ctx, message, data).await {
            tracing::error!("Failed to send help message on mention: {}", e);
        }
    }
}

async fn send_help_as_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let commands = crate::commands::help::get_all_commands();
    let embed = crate::commands::help::create_help_embed(&commands, data);

    message.channel_id.send_message(
        &ctx.http,
        serenity::CreateMessage::new()
            .embed(embed)
            .reference_message(message)
    ).await?;

    Ok(())
}
