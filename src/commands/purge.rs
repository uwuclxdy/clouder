use crate::config::AppState;
use crate::utils::get_default_embed_color;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, MessageId};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only,
    ephemeral
)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Number of messages to delete OR message ID to delete up to"]
    #[min = 1]
    #[max = 100]
    amount_or_id: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id();

    // Parse input as either number or message ID
    let messages_to_delete = if let Ok(count) = amount_or_id.parse::<u8>() {
        // Delete last N messages
        if count == 0 || count > 100 {
            ctx.send(poise::CreateReply::default()
                .content("❌ number must be between 1 and 100!")
                .ephemeral(true))
                .await?;
            return Ok(());
        }

        let messages = channel_id
            .messages(&ctx.http(), serenity::GetMessages::new().limit(count))
            .await?;

        messages
    } else if let Ok(message_id) = amount_or_id.parse::<u64>() {
        // Delete messages up to specific message ID
        let target_id = MessageId::new(message_id);

        // Get messages after the target message
        let messages = channel_id
            .messages(&ctx.http(), serenity::GetMessages::new().after(target_id).limit(100))
            .await?;

        messages
    } else {
        ctx.send(poise::CreateReply::default()
            .content("❌ invalid input! provide either a number (1-100) or a message ID!")
            .ephemeral(true))
            .await?;
        return Ok(());
    };

    if messages_to_delete.is_empty() {
        ctx.send(poise::CreateReply::default()
            .content("❌ no messages found to delete!")
            .ephemeral(true))
            .await?;
        return Ok(());
    }

    // Delete messages
    let deleted_count = if messages_to_delete.len() == 1 {
        // Single message deletion
        match messages_to_delete[0].delete(&ctx.http()).await {
            Ok(_) => 1,
            Err(e) => {
                ctx.send(poise::CreateReply::default()
                    .content(&format!("❌ failed to delete message: {}", e))
                    .ephemeral(true))
                    .await?;
                return Ok(());
            }
        }
    } else {
        // Bulk deletion (only works for messages < 14 days old)
        let message_ids: Vec<MessageId> = messages_to_delete.iter().map(|m| m.id).collect();

        match channel_id.delete_messages(&ctx.http(), &message_ids).await {
            Ok(_) => message_ids.len(),
            Err(e) => {
                // Fallback to individual deletion if bulk fails
                let mut success_count = 0;
                for message in &messages_to_delete {
                    if message.delete(&ctx.http()).await.is_ok() {
                        success_count += 1;
                    }
                }

                if success_count == 0 {
                    ctx.send(poise::CreateReply::default()
                        .content(&format!("❌ failed to delete messages: {}", e))
                        .ephemeral(true))
                        .await?;
                    return Ok(());
                }
                success_count
            }
        }
    };

    // Send success response
    let embed = CreateEmbed::new()
        .title("🗑️ messages purged")
        .description(&format!("successfully deleted **`{}`** message{}",
            deleted_count,
            if deleted_count == 1 { "" } else { "s" }
        ))
        .color(get_default_embed_color(ctx.data()))
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default()
        .embed(embed)
        .ephemeral(true))
        .await?;

    Ok(())
}
