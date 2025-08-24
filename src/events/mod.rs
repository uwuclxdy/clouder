use crate::config::AppState;
use crate::events::bot_mentioned::on_mention;
use crate::events::selfroles::{handle_selfrole_interaction, selfrole_message_delete};
use crate::{serenity, Data, Error};
use tracing::info;

pub mod member_events;
mod bot_mentioned;
mod selfroles;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> anyhow::Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("Bot {} is ready!", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            handle_interaction_create(ctx, interaction, data).await;
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            selfrole_message_delete(ctx, channel_id, deleted_message_id, guild_id, data).await;
        }
        serenity::FullEvent::Message { new_message } => {
            on_mention(ctx, new_message, data).await;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            member_events::member_addition(ctx, &new_member.guild_id, new_member).await;
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available } => {
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
    match interaction {
        serenity::Interaction::Component(component_interaction) => {
            handle_component_interaction(ctx, component_interaction, data).await;
        }
        _ => {}
    }
}

pub async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    if interaction.data.custom_id.starts_with("selfrole_") {
        handle_selfrole_interaction(ctx, interaction, data).await;
    }
}
