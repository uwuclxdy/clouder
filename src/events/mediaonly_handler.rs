use crate::config::AppState;
use crate::database::mediaonly::MediaOnlyConfig;
use crate::utils::content_detection::has_allowed_content;
use poise::serenity_prelude as serenity;
use tracing::{error, warn};

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
            error!("Failed to fetch media-only config: {}", e);
            return;
        }
    };

    // Check if bot has MANAGE_MESSAGES permission in this channel
    let bot_member = match crate::web::get_bot_member_info(&ctx.http, guild_id).await {
        Ok(member) => member,
        Err(e) => {
            warn!("Failed to get bot member info: {}", e);
            return;
        }
    };

    let guild = match ctx.http.get_guild(guild_id).await {
        Ok(guild) => guild,
        Err(e) => {
            warn!("Failed to get guild info: {}", e);
            return;
        }
    };

    let channel = match ctx.http.get_channel(channel_id).await {
        Ok(serenity::Channel::Guild(channel)) => channel,
        Ok(_) => {
            warn!("Channel {} is not a guild channel", channel_id);
            return;
        }
        Err(e) => {
            warn!("Failed to get channel info: {}", e);
            return;
        }
    };

    let channel_permissions = guild.user_permissions_in(&channel, &bot_member);

    if !channel_permissions.manage_messages() {
        warn!("Bot lacks MANAGE_MESSAGES permission in channel {} for media-only enforcement", channel_id);
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
                if let serenity::HttpError::UnsuccessfulRequest(error_response) = &http_error {
                    if error_response.status_code == 404 {
                        // Message was already deleted, this is fine
                        return;
                    }
                }
                warn!("Failed to delete non-media message: {}", http_error);
            }
            Err(e) => {
                warn!("Failed to delete non-media message: {}", e);
            }
        }
    });
}
