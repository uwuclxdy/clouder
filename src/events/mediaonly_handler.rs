use crate::logging::{error, warn};
use clouder_core::config::AppState;
use clouder_core::database::mediaonly::MediaOnlyConfig;
use clouder_core::utils::content_detection::has_allowed_content;
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;
use std::time::Duration;

pub async fn handle_media_only_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) {
    if message.author.bot {
        return;
    }

    let Some(guild_id) = message.guild_id else {
        return;
    };

    let channel_id = message.channel_id;

    let config = match MediaOnlyConfig::get_by_channel(
        &data.db,
        &guild_id.to_string(),
        &channel_id.to_string(),
    )
    .await
    {
        Ok(Some(config)) if config.enabled => config,
        Ok(_) => return,
        Err(e) => {
            error!("fetch media-only config: {}", e);
            return;
        }
    };

    if has_allowed_content(
        message,
        config.allow_links,
        config.allow_attachments,
        config.allow_gifs,
        config.allow_stickers,
    ) {
        return;
    }

    let message_id = message.id;
    let author_id = message.author.id;
    let channel_id = message.channel_id;
    let http = ctx.http.clone();
    let embed_color = get_embed_color(data, Some(guild_id.get())).await;
    let allowed_types = build_allowed_types(&config);
    let footer = crate::serenity::CreateEmbedFooter::new(format!("allowed types: {allowed_types}"));

    const AUTO_DELETE_DELAY: Duration = Duration::from_secs(5);

    tokio::spawn(async move {
        match http.delete_message(channel_id, message_id, None).await {
            Ok(_) => {
                let embed = serenity::builder::CreateEmbed::new()
                    .description(format!("<@{author_id}> this channel is media-only"))
                    .footer(footer)
                    .color(embed_color);
                let message = serenity::builder::CreateMessage::new().embed(embed);
                if let Ok(notification) = channel_id.send_message(&http, message).await {
                    tokio::time::sleep(AUTO_DELETE_DELAY).await;
                    let _ = notification.delete(&http).await;
                }
            }
            Err(serenity::Error::Http(http_error)) => {
                if let serenity::HttpError::UnsuccessfulRequest(error_response) = &http_error
                    && error_response.status_code == 404
                {
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

fn build_allowed_types(config: &MediaOnlyConfig) -> String {
    let types: Vec<&str> = [
        (config.allow_attachments, "attachments"),
        (config.allow_links, "links"),
        (config.allow_gifs, "GIFs"),
        (config.allow_stickers, "stickers"),
    ]
    .into_iter()
    .filter_map(|(enabled, name)| enabled.then_some(name))
    .collect();

    if types.is_empty() {
        "none".to_string()
    } else {
        types.join(", ")
    }
}
