use crate::logging::error;
use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateChannel, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage, Mentionable,
};
use serenity::collector::ComponentInteractionCollector;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(
    slash_command,
    subcommands("delete", "clone_channel", "nuke"),
    guild_only
)]
pub async fn channel(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
    ephemeral
)]
async fn delete(
    ctx: Context<'_>,
    #[description = "channel to delete (defaults to current)"] channel: Option<
        serenity::GuildChannel,
    >,
) -> Result<(), Error> {
    let target = resolve_channel(&ctx, channel).await?;
    let name = target.name.clone();

    if let Some(interaction) =
        confirm_action(&ctx, &target, "delete", "this cannot be undone.").await?
    {
        let result = match target.delete(ctx.http()).await {
            Ok(_) => format!("deleted #{}.", name),
            Err(e) => {
                error!("delete channel {}: {}", target.id, e);
                "failed to delete the channel.".to_string()
            }
        };
        respond_update(&interaction, ctx.http(), result).await?;
    }

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
    ephemeral,
    rename = "clone"
)]
async fn clone_channel(
    ctx: Context<'_>,
    #[description = "channel to clone (defaults to current)"] channel: Option<
        serenity::GuildChannel,
    >,
) -> Result<(), Error> {
    let source = resolve_channel(&ctx, channel).await?;
    let guild_id = ctx.guild_id().expect("guild_only");

    match guild_id
        .create_channel(ctx.serenity_context(), build_create_channel(&source, true))
        .await
    {
        Ok(new_ch) => {
            let embed = CreateEmbed::new()
                .description(format!(
                    "cloned {} → {}",
                    source.mention(),
                    new_ch.mention()
                ))
                .color(get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            error!("clone channel {}: {}", source.id, e);
            ctx.say("failed to clone the channel.").await?;
        }
    }

    Ok(())
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
    ephemeral
)]
async fn nuke(
    ctx: Context<'_>,
    #[description = "channel to nuke (defaults to current)"] channel: Option<
        serenity::GuildChannel,
    >,
) -> Result<(), Error> {
    let target = resolve_channel(&ctx, channel).await?;
    let guild_id = ctx.guild_id().expect("guild_only");
    let name = target.name.clone();
    let create_builder = build_create_channel(&target, false);

    if let Some(interaction) = confirm_action(
        &ctx,
        &target,
        "nuke",
        "all message history will be permanently lost.",
    )
    .await?
    {
        let result = match target.delete(ctx.http()).await {
            Err(e) => {
                error!("nuke delete {}: {}", target.id, e);
                "failed to delete the channel.".to_string()
            }
            Ok(_) => {
                match guild_id
                    .create_channel(ctx.serenity_context(), create_builder)
                    .await
                {
                    Ok(new_ch) => format!("nuked #{} → {}", name, new_ch.mention()),
                    Err(e) => {
                        error!("nuke recreate #{}: {}", name, e);
                        format!("deleted #{} but failed to recreate it.", name)
                    }
                }
            }
        };
        respond_update(&interaction, ctx.http(), result).await?;
    }

    Ok(())
}

async fn resolve_channel(
    ctx: &Context<'_>,
    channel: Option<serenity::GuildChannel>,
) -> Result<serenity::GuildChannel, Error> {
    match channel {
        Some(ch) => Ok(ch),
        None => match ctx.channel_id().to_channel(ctx.http()).await? {
            serenity::Channel::Guild(ch) => Ok(ch),
            _ => Err("must be used in a guild text channel".into()),
        },
    }
}

fn build_create_channel(source: &serenity::GuildChannel, copy_suffix: bool) -> CreateChannel<'_> {
    let name = if copy_suffix {
        format!("{}-copy", source.name)
    } else {
        source.name.clone()
    };

    let mut builder = CreateChannel::new(name)
        .kind(source.kind)
        .nsfw(source.nsfw)
        .position(source.position)
        .permissions(source.permission_overwrites.clone());

    if let Some(topic) = &source.topic {
        builder = builder.topic(topic);
    }
    if let Some(slow) = source.rate_limit_per_user {
        builder = builder.rate_limit_per_user(slow);
    }
    if let Some(parent) = source.parent_id {
        builder = builder.category(parent);
    }

    builder
}

async fn confirm_action(
    ctx: &Context<'_>,
    target: &serenity::GuildChannel,
    action: &str,
    warning: &str,
) -> Result<Option<serenity::ComponentInteraction>, Error> {
    let confirm_id = format!("confirm_{}", ctx.id());
    let cancel_id = format!("cancel_{}", ctx.id());

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content(format!("{} {}?\n-# {}", action, target.mention(), warning))
        .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new(&confirm_id)
                .label(action)
                .style(ButtonStyle::Danger),
            CreateButton::new(&cancel_id)
                .label("cancel")
                .style(ButtonStyle::Secondary),
        ])]);

    let mut msg = ctx.send(reply).await?.into_message().await?;

    let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .message_id(msg.id)
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(30))
        .await;

    match interaction {
        None => {
            msg.edit(
                ctx.http(),
                EditMessage::new().content("timed out.").components(vec![]),
            )
            .await
            .ok();
            Ok(None)
        }
        Some(i) if i.data.custom_id == cancel_id => {
            i.create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content("cancelled.")
                        .components(vec![]),
                ),
            )
            .await?;
            Ok(None)
        }
        Some(i) => Ok(Some(i)),
    }
}

async fn respond_update(
    interaction: &serenity::ComponentInteraction,
    http: &serenity::Http,
    content: String,
) -> Result<(), Error> {
    interaction
        .create_response(
            http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}
