use crate::config::AppState;
use crate::database::mediaonly::MediaOnlyConfig;
use crate::logging::{error, warn};
use crate::utils::content_detection::has_allowed_content;
use crate::utils::get_bot_channel_permissions;
use poise::serenity_prelude as serenity;

/// Handle media-only channel message processing
pub async fn handle_media_only_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) {
    // Ignore bot messages
    if message.author.bot {
        return;
    }

    // Only process guild messages
    let Some(guild_id) = message.guild_id else {
        return;
    };

    let channel_id = message.channel_id;

    // Check if this channel has media-only enabled
    let config = match MediaOnlyConfig::get_by_channel(
        &data.db,
        &guild_id.to_string(),
        &channel_id.to_string(),
    )
    .await
    {
        Ok(Some(config)) if config.enabled => config,
        Ok(_) => return, // No config or disabled
        Err(e) => {
            error!("fetch media-only config: {}", e);
            return;
        }
    };

    // Check if bot has MANAGE_MESSAGES permission in this channel
    let perms = match get_bot_channel_permissions(&ctx.http, guild_id, channel_id).await {
        Some(p) => p,
        None => {
            warn!("get channel permissions for {}", channel_id);
            return;
        }
    };

    if !perms.permissions.manage_messages() {
        warn!("no MANAGE_MESSAGES in channel {}", channel_id);
        return;
    }

    // Check if a message contains allowed content
    if has_allowed_content(
        message,
        config.allow_links,
        config.allow_attachments,
        config.allow_gifs,
        config.allow_stickers,
    ) {
        return; // Message has allowed content, don't delete
    }

    // Delete message that doesn't have allowed content
    let message_id = message.id;
    let channel_id = message.channel_id;
    let http = ctx.http.clone();

    tokio::spawn(async move {
        match http.delete_message(channel_id, message_id, None).await {
            Ok(_) => {
                // Successfully deleted - no logging to avoid spam
            }
            Err(serenity::Error::Http(http_error)) => {
                // Check if it's a "message not found" error (already deleted)
                if let serenity::HttpError::UnsuccessfulRequest(error_response) = &http_error
                    && error_response.status_code == 404
                {
                    // Message was already deleted, this is fine
                    return;
                }
                warn!("delete non-media message: {}", http_error);
            }
            Err(e) => {
                warn!("delete non-media message: {}", e);
            }
        }
    });
}
