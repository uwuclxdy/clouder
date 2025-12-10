use crate::config::AppState;
use crate::database::mediaonly::MediaOnlyConfig;
use crate::utils::get_default_embed_color;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, Mentionable};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
    ephemeral
)]
pub async fn mediaonly(
    ctx: Context<'_>,
    #[description = "Channel to configure (defaults to current channel)"] channel: Option<
        serenity::GuildChannel,
    >,
    #[description = "Enable or disable media-only mode (toggles if not specified)"] enabled: Option<
        bool,
    >,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let target_channel = if let Some(ref channel) = channel {
        channel
    } else {
        &ctx.guild_channel().await.unwrap()
    };
    let channel_id = target_channel.id.to_string();

    let final_enabled = if let Some(enabled) = enabled {
        MediaOnlyConfig::upsert(&ctx.data().db, &guild_id, &channel_id, enabled).await?;
        enabled
    } else {
        MediaOnlyConfig::toggle(&ctx.data().db, &guild_id, &channel_id).await?
    };

    let status_text = if final_enabled { "enabled" } else { "disabled" };

    let embed = CreateEmbed::new()
        .title(format!("media-only mode {}", status_text))
        .description(format!(
            "media-only mode is now **{}** for {}\n\nMessages without media will be deleted.\nConfigure allowed content types in the [dashboard]({}/dashboard/{}/mediaonly)",
            status_text,
            target_channel.mention(),
            ctx.data().config.web.base_url,
            guild_id
        ))
        .color(get_default_embed_color(ctx.data()));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
