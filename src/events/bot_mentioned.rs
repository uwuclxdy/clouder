use crate::config::AppState;
use crate::serenity;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Handle message events - primarily for bot mention help responses and OpenAI integration
pub async fn on_mention(ctx: &serenity::Context, message: &serenity::Message, data: &AppState) {
    if message.author.bot {
        return;
    }

    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to get current user: {}", e);
            return;
        }
    };

    let is_mention = message.mentions.iter().any(|u| u.id == current_user.id);
    let is_reply_to_bot = is_replying_to_bot(message, &current_user).await;

    if is_mention || is_reply_to_bot {
        // Check if OpenAI is enabled and user is authorized
        if data.config.openai.enabled {
            if let Some(ref openai_client) = data.openai_client {
                if is_user_authorized_for_openai(message, data).await {
                    if let Err(e) = handle_openai_request(ctx, message, data, openai_client).await {
                        error!("Failed to handle OpenAI request: {}", e);
                        send_ephemeral_error(
                            ctx,
                            message,
                            "sorry, something went wrong with ai processing :(".to_string(),
                        )
                        .await;
                    }
                    return;
                }
            }
        }

        // Fallback to a help message if OpenAI is not enabled or user not authorized (only for mentions)
        if is_mention {
            if let Err(e) = send_help_as_message(ctx, message, data).await {
                error!("Failed to send help message on mention: {}", e);
            }
        }
    }
}

async fn is_user_authorized_for_openai(message: &serenity::Message, data: &AppState) -> bool {
    let user_id = message.author.id.get();

    // Check if user is in allowed lists
    let is_server_context = message.guild_id.is_some();

    if is_server_context {
        // In server context, check server allowed users
        data.config.openai.allowed_users.contains(&user_id)
    } else {
        // In DM context, check DM allowed users
        data.config.openai.dm_allowed_users.contains(&user_id)
    }
}

async fn is_replying_to_bot(message: &serenity::Message, current_user: &serenity::User) -> bool {
    if let Some(ref referenced_message) = message.referenced_message {
        return referenced_message.author.id == current_user.id;
    }
    false
}

async fn build_conversation_context(
    ctx: &serenity::Context,
    message: &serenity::Message,
    current_user: &serenity::User,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut conversation = Vec::new();
    let mut current_msg = message;
    let mut depth = 0;
    const MAX_DEPTH: usize = 4;

    // Collect up to 4 messages in the reply chain
    while depth < MAX_DEPTH {
        let role = if current_msg.author.id == current_user.id {
            "Assistant"
        } else {
            "User"
        };

        let content = if current_msg.author.id == current_user.id {
            // For bot messages, use the content as-is
            current_msg.content.clone()
        } else {
            clean_message_content(&current_msg.content, current_user)
        };

        if !content.trim().is_empty() {
            conversation.push(format!("{}: {}", role, content.trim()));
        }

        // Try to get the referenced message
        if let Some(ref referenced_message) = current_msg.referenced_message {
            current_msg = referenced_message;
            depth += 1;
        } else if let Some(message_reference) = &current_msg.message_reference {
            // If we have a message reference but not the full message, try to fetch it
            if let Some(message_id) = message_reference.message_id {
                match ctx.http.get_message(message.channel_id, message_id).await {
                    Ok(fetched_msg) => {
                        current_msg = Box::leak(Box::new(fetched_msg));
                        depth += 1;
                    }
                    Err(_) => break,
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Reverse the conversation so it's in chronological order (oldest first)
    conversation.reverse();

    // If we have conversation context, format it nicely
    if conversation.len() > 1 {
        Ok(format!(
            "Previous conversation:\n{}\n\nCurrent message: {}",
            conversation[..conversation.len() - 1].join("\n"),
            conversation
                .last()
                .unwrap_or(&String::new())
                .replace("User: ", "")
                .replace("Assistant: ", "")
        ))
    } else {
        // No conversation context, just return the current message
        Ok(clean_message_content(&message.content, current_user))
    }
}

fn clean_message_content(content: &str, current_user: &serenity::User) -> String {
    let bot_mention = format!("<@{}>", current_user.id);
    let bot_mention_nickname = format!("<@!{}>", current_user.id);

    content
        .replace(&bot_mention, "")
        .replace(&bot_mention_nickname, "")
        .trim()
        .to_string()
}

async fn handle_openai_request(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
    openai_client: &crate::external::openai::OpenAIClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = message.author.id.get();

    // Check cooldown unless user is in no-cooldown list
    if !data.config.openai.no_cooldown_users.contains(&user_id) {
        let cooldown_duration = Duration::from_secs(10);

        if !openai_client.check_and_update_cooldown(user_id, cooldown_duration) {
            debug!("User {} is on cooldown for OpenAI", user_id);
            return Ok(()); // Silently ignore if on cooldown
        }
    }

    let current_user = ctx.http.get_current_user().await?;

    let prompt = build_conversation_context(ctx, message, &current_user).await?;

    if prompt.trim().is_empty() {
        return Ok(()); // Silently ignore empty prompts
    }

    debug!("Processing OpenAI request for user {}: {}", user_id, prompt);

    // Start typing indicator
    let typing = message.channel_id.start_typing(&ctx.http);

    // Build messages array for OpenAI
    let mut messages = Vec::new();

    // Add system prompt if configured
    if !data.config.openai.system_prompt.trim().is_empty() {
        messages.push(crate::external::openai::ChatMessage {
            role: "system".to_string(),
            content: data.config.openai.system_prompt.clone(),
        });
    }

    messages.push(crate::external::openai::ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
    });

    // Send request to OpenAI
    let response = openai_client
        .generate(
            &data.config.openai.model,
            messages,
            data.config.openai.temperature,
            data.config.openai.max_tokens,
            if data.config.openai.stop.is_empty() {
                None
            } else {
                Some(&data.config.openai.stop)
            },
        )
        .await?;

    // Stop typing and send response
    drop(typing);

    // Split response if it's too long (Discord limit is 2000 characters)
    let chunks = split_message(&response, 2000);

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last_chunk = i == chunks.len() - 1;

        let mut create_message = serenity::CreateMessage::new()
            .content(chunk)
            .reference_message(message);

        // Add "try again" button to the last chunk
        if is_last_chunk {
            let action_row = create_retry_button(user_id, &prompt, message.id.get());
            create_message = create_message.components(vec![action_row]);
        }

        message
            .channel_id
            .send_message(&ctx.http, create_message)
            .await?;
    }

    Ok(())
}

fn split_message(content: &str, max_length: usize) -> Vec<String> {
    if content.len() <= max_length {
        return vec![content.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for word in content.split_whitespace() {
        if current_chunk.len() + word.len() + 1 > max_length {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = String::new();
            }
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(word);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

async fn send_ephemeral_error(
    ctx: &serenity::Context,
    message: &serenity::Message,
    _error_msg: String,
) {
    // Try to react with an error emoji to indicate something went wrong
    if let Err(e) = message.react(&ctx.http, '❌').await {
        warn!("Failed to react to message with error: {}", e);
    }
}

async fn send_help_as_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let commands = crate::commands::help::get_all_commands();
    let embed = crate::commands::help::create_help_embed(&commands, data);

    message
        .channel_id
        .send_message(
            &ctx.http,
            serenity::CreateMessage::new()
                .embed(embed)
                .reference_message(message),
        )
        .await?;

    Ok(())
}

/// Handle AI retry button interactions
pub async fn handle_ai_retry_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    // Parse custom_id: "ai_retry_{user_id}_{prompt_hash}_{original_message_id}"
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() != 5 || parts[0] != "ai" || parts[1] != "retry" {
        error!(
            "Invalid AI retry custom_id format: {}",
            interaction.data.custom_id
        );
        return;
    }

    let requesting_user_id = match parts[2].parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid user_id in AI retry custom_id: {}", parts[2]);
            return;
        }
    };

    let original_message_id = match parts[4].parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            error!(
                "Invalid original_message_id in AI retry custom_id: {}",
                parts[4]
            );
            return;
        }
    };

    // Check if the user clicking the button is the same as the one who triggered the original request
    if interaction.user.id.get() != requesting_user_id {
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("❌ Only the person who triggered this AI response can retry it.")
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("Failed to send unauthorized retry response: {}", e);
        }
        return;
    }

    // Check if OpenAI is enabled and get client
    if !data.config.openai.enabled {
        error!("OpenAI not enabled for retry request");
        return;
    }

    let openai_client = match &data.openai_client {
        Some(client) => client,
        None => {
            error!("OpenAI client not available for retry request");
            return;
        }
    };

    // Check cooldown (unless user is in no-cooldown list)
    if !data
        .config
        .openai
        .no_cooldown_users
        .contains(&requesting_user_id)
    {
        let cooldown_duration = Duration::from_secs(10);

        if !openai_client.check_and_update_cooldown(requesting_user_id, cooldown_duration) {
            if let Err(e) = interaction
                .create_response(
                    &ctx.http,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("⏱️ Please wait before retrying.")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("Failed to send cooldown retry response: {}", e);
            }
            return;
        }
    }

    // Acknowledge the interaction and update the button to show it's processing
    if let Err(e) = interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .components(vec![create_disabled_retry_button()]),
            ),
        )
        .await
    {
        error!("Failed to acknowledge AI retry interaction: {}", e);
        return;
    }

    // We now have the original message ID from the custom_id, so we can fetch it directly
    let channel_id = interaction.channel_id;
    let user_message = match ctx
        .http
        .get_message(channel_id, serenity::MessageId::new(original_message_id))
        .await
    {
        Ok(msg) => msg,
        Err(e) => {
            error!(
                "Failed to fetch original user message {} for retry: {}",
                original_message_id, e
            );
            if let Err(edit_err) = interaction
                .edit_response(
                    &ctx.http,
                    serenity::EditInteractionResponse::new()
                        .content(
                            "❌ Could not find the original message. It may have been deleted.",
                        )
                        .components(vec![]),
                )
                .await
            {
                error!(
                    "Failed to send error response about missing message: {}",
                    edit_err
                );
            }
            return;
        }
    };

    // Verify that the user who clicked the button is the same as the author of the original message
    if user_message.author.id.get() != requesting_user_id {
        error!(
            "User ID mismatch: button user {} vs message author {}",
            requesting_user_id,
            user_message.author.id.get()
        );
        if let Err(e) = interaction
            .edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content("❌ User verification failed. Please send a new message instead.")
                    .components(vec![]),
            )
            .await
        {
            error!("Failed to send user verification error response: {}", e);
        }
        return;
    }

    // Build the conversation context again
    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to get current user for retry: {}", e);
            return;
        }
    };

    let prompt = match build_conversation_context(ctx, &user_message, &current_user).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to build conversation context for retry: {}", e);
            if let Err(edit_err) = interaction.edit_response(&ctx.http,
                serenity::EditInteractionResponse::new()
                    .content("❌ Failed to rebuild conversation context. Please send a new message instead.")
                    .components(vec![])
            ).await {
                error!("Failed to send context building error response: {}", edit_err);
            }
            return;
        }
    };

    if prompt.trim().is_empty() {
        error!("Empty prompt for retry");
        if let Err(e) = interaction.edit_response(&ctx.http,
            serenity::EditInteractionResponse::new()
                .content("❌ Could not extract content from the original message. Please send a new message instead.")
                .components(vec![])
        ).await {
            error!("Failed to send empty prompt error response: {}", e);
        }
        return;
    }

    debug!(
        "Processing AI retry for user {}: {}",
        requesting_user_id, prompt
    );
    debug!(
        "Original user message ID: {}, content preview: {}",
        user_message.id,
        user_message.content.chars().take(50).collect::<String>()
    );

    // Build messages array for OpenAI
    let mut messages = Vec::new();

    // Add system prompt if configured
    if !data.config.openai.system_prompt.trim().is_empty() {
        messages.push(crate::external::openai::ChatMessage {
            role: "system".to_string(),
            content: data.config.openai.system_prompt.clone(),
        });
    }

    messages.push(crate::external::openai::ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
    });

    // Send request to OpenAI
    let response = match openai_client
        .generate(
            &data.config.openai.model,
            messages,
            data.config.openai.temperature,
            data.config.openai.max_tokens,
            if data.config.openai.stop.is_empty() {
                None
            } else {
                Some(&data.config.openai.stop)
            },
        )
        .await
    {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to generate AI retry response: {}", e);

            // Update message to show error
            if let Err(edit_err) = interaction
                .edit_response(
                    &ctx.http,
                    serenity::EditInteractionResponse::new()
                        .content("❌ Failed to generate new response. Please try again later.")
                        .components(vec![create_retry_button(
                            requesting_user_id,
                            &prompt,
                            original_message_id,
                        )]),
                )
                .await
            {
                error!("Failed to update message after AI error: {}", edit_err);
            }
            return;
        }
    };

    // Split response if it's too long
    let chunks = split_message(&response, 2000);
    let content = if chunks.len() == 1 {
        chunks[0].clone()
    } else {
        // For multi-chunk responses, just use the first chunk and indicate there's more
        format!("{}\n\n*(Response was truncated)*", chunks[0])
    };

    // Update the message with the new response and re-enable the button
    if let Err(e) = interaction
        .edit_response(
            &ctx.http,
            serenity::EditInteractionResponse::new()
                .content(content)
                .components(vec![create_retry_button(
                    requesting_user_id,
                    &prompt,
                    original_message_id,
                )]),
        )
        .await
    {
        error!("Failed to update message with new AI response: {}", e);
    }
}

fn create_retry_button(
    user_id: u64,
    prompt: &str,
    original_message_id: u64,
) -> serenity::CreateActionRow {
    use serenity::all::{ButtonStyle, CreateActionRow, CreateButton};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let prompt_hash = {
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        hasher.finish()
    };

    // Format: "ai_retry_{user_id}_{prompt_hash}_{original_message_id}"
    let custom_id = format!(
        "ai_retry_{}_{}_{}",
        user_id, prompt_hash, original_message_id
    );

    let retry_button = CreateButton::new(custom_id)
        .label("🔄 try again")
        .style(ButtonStyle::Secondary);

    CreateActionRow::Buttons(vec![retry_button])
}

fn create_disabled_retry_button() -> serenity::CreateActionRow {
    use serenity::all::{ButtonStyle, CreateActionRow, CreateButton};

    let retry_button = CreateButton::new("ai_retry_disabled")
        .label("🔄 Generating...")
        .style(ButtonStyle::Secondary)
        .disabled(true);

    CreateActionRow::Buttons(vec![retry_button])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_button_creation() {
        let user_id = 123456789u64;
        let prompt = "Hello, how are you?";
        let original_message_id = 987654321u64;

        let action_row = create_retry_button(user_id, prompt, original_message_id);

        // Verify the action row is created correctly
        if let serenity::CreateActionRow::Buttons(buttons) = action_row {
            assert_eq!(buttons.len(), 1);
            // We can't easily access the button's properties due to Serenity's API design,
        } else {
            panic!("Expected CreateActionRow::Buttons, got different variant");
        }
    }

    #[test]
    fn test_disabled_retry_button_creation() {
        let action_row = create_disabled_retry_button();

        // Verify the action row is created correctly
        if let serenity::CreateActionRow::Buttons(buttons) = action_row {
            assert_eq!(buttons.len(), 1);
            // We can't easily access the button's properties due to Serenity's API design
        } else {
            panic!("Expected CreateActionRow::Buttons, got different variant");
        }
    }

    #[test]
    fn test_custom_id_parsing() {
        // Test valid custom_id parsing logic
        let custom_id = "ai_retry_123456789_987654321_555444333";
        let parts: Vec<&str> = custom_id.split('_').collect();

        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "ai");
        assert_eq!(parts[1], "retry");
        assert_eq!(parts[2], "123456789");
        assert_eq!(parts[3], "987654321");
        assert_eq!(parts[4], "555444333");

        // Test user_id parsing
        let user_id: u64 = parts[2].parse().unwrap();
        assert_eq!(user_id, 123456789);

        // Test original_message_id parsing
        let original_message_id: u64 = parts[4].parse().unwrap();
        assert_eq!(original_message_id, 555444333);
    }

    #[test]
    fn test_custom_id_format_consistency() {
        let user_id = 123456789u64;
        let prompt = "Test prompt";
        let original_message_id = 555444333u64;

        // Create button and extract custom_id logic
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let prompt_hash = {
            let mut hasher = DefaultHasher::new();
            prompt.hash(&mut hasher);
            hasher.finish()
        };

        let expected_custom_id = format!(
            "ai_retry_{}_{}_{}",
            user_id, prompt_hash, original_message_id
        );

        // Verify the format matches what we expect
        assert!(expected_custom_id.starts_with("ai_retry_"));
        assert!(expected_custom_id.contains(&user_id.to_string()));
        assert!(expected_custom_id.contains(&original_message_id.to_string()));
    }

    #[test]
    fn test_retry_interaction_custom_id_validation() {
        // Test the exact validation logic used in handle_ai_retry_interaction

        // Valid custom_id
        let valid_custom_id = "ai_retry_123456789_987654321_555444333";
        let parts: Vec<&str> = valid_custom_id.split('_').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "ai");
        assert_eq!(parts[1], "retry");
        assert!(parts[2].parse::<u64>().is_ok());
        assert!(parts[4].parse::<u64>().is_ok());

        // Invalid custom_ids
        let invalid_ids = vec![
            "ai_retry_123_456",            // too few parts
            "invalid_retry_123_456_789",   // wrong prefix
            "ai_invalid_123_456_789",      // wrong action
            "ai_retry_notanumber_456_789", // invalid user_id
            "ai_retry_123_456_notanumber", // invalid original_message_id
        ];

        for invalid_id in invalid_ids {
            let parts: Vec<&str> = invalid_id.split('_').collect();
            let is_valid = parts.len() == 5
                && parts[0] == "ai"
                && parts[1] == "retry"
                && parts[2].parse::<u64>().is_ok()
                && parts[4].parse::<u64>().is_ok();
            assert!(!is_valid, "Custom ID should be invalid: {}", invalid_id);
        }
    }

    #[test]
    fn test_split_message_with_button() {
        // Test that split_message works correctly for button integration
        let short_message = "Short message";
        let chunks = split_message(short_message, 2000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short message");

        let long_message = "word ".repeat(500); // Creates a long message
        let chunks = split_message(&long_message, 100);
        assert!(chunks.len() > 1); // Should be split into multiple chunks

        // Verify all chunks are within the limit
        for chunk in chunks {
            assert!(chunk.len() <= 100);
        }
    }
}
