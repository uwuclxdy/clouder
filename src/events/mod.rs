use crate::config::AppState;
use crate::events::bot_mentioned::{handle_ai_retry_interaction, on_mention};
use crate::events::mediaonly_handler::handle_media_only_message;
use crate::events::selfroles::{handle_selfrole_interaction, selfrole_message_delete};
use crate::logging::info;
use crate::{serenity, Data, Error};

mod bot_mentioned;
mod mediaonly_handler;
pub mod member_events;
mod selfroles;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> anyhow::Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => {
            info!("bot ready");
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            handle_interaction_create(ctx, interaction, data).await;
        }
        serenity::FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            selfrole_message_delete(ctx, channel_id, deleted_message_id, guild_id, data).await;
        }
        serenity::FullEvent::Message { new_message } => {
            on_mention(ctx, new_message, data).await;
            handle_media_only_message(ctx, new_message, data).await;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            member_events::member_addition(ctx, &new_member.guild_id, new_member).await;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            member_events::member_removal(ctx, guild_id, user, member_data_if_available).await;
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_interaction_create(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    data: &AppState,
) {
    if let serenity::Interaction::Component(component_interaction) = interaction {
        handle_component_interaction(ctx, component_interaction, data).await;
    }
}

pub async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    if interaction.data.custom_id.starts_with("selfrole_") {
        handle_selfrole_interaction(ctx, interaction, data).await;
    } else if interaction.data.custom_id.starts_with("ai_retry_") {
        handle_ai_retry_interaction(ctx, interaction, data).await;
    }
}
