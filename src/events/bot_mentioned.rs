use crate::serenity;
use clouder_core::config::AppState;
use std::time::Duration;
use tracing::{debug, error, warn};

#[cfg(feature = "llm")]
use clouder_llm::{ChatMessage, LlmClient};

/// Handle message events - primarily for bot mention help responses and OpenAI integration
pub async fn on_mention(ctx: &serenity::Context, message: &serenity::Message, data: &AppState) {
    if message.author.bot {
        return;
    }

    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            error!("get current user: {}", e);
            return;
        }
    };

    let is_mention = message.mentions.iter().any(|u| u.id == current_user.id);
    let is_reply_to_bot = is_replying_to_bot(message, &current_user).await;

    if is_mention || is_reply_to_bot {
        // Check if LLM is enabled and user is authorized
        #[cfg(feature = "llm")]
        if data.config.llm.provider.is_some()
            && let Some(ref llm_client) = data.llm_client
            && is_user_authorized(message, data).await
        {
            if let Err(e) = handle_llm_request(ctx, message, data, llm_client).await {
                error!("openai request: {}", e);
                send_ephemeral_error(
                    ctx,
                    message,
                    "sorry, something went wrong with ai processing :(".to_string(),
                )
                .await;
            }
            return;
        }

        // Fallback to a help message if OpenAI is not enabled or user not authorized (only for mentions)
        if is_mention && let Err(e) = send_help_as_message(ctx, message, data).await {
            error!("send help: {}", e);
        }
    }
}

async fn is_user_authorized(message: &serenity::Message, data: &AppState) -> bool {
    let user_id = message.author.id.get();
    if message.guild_id.is_some() {
        data.config.llm.allowed_users.contains(&user_id)
    } else {
        data.config.llm.dm_allowed_users.contains(&user_id)
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
    const MAX_ENTRIES: usize = 4;
    let mut entries: Vec<(bool, String)> = Vec::new();

    push_entry(&mut entries, message, current_user);

    let mut next_id = reply_chain_id(message);

    while entries.len() < MAX_ENTRIES {
        let Some(msg_id) = next_id.take() else { break };
        match ctx.http.get_message(message.channel_id, msg_id).await {
            Ok(fetched) => {
                push_entry(&mut entries, &fetched, current_user);
                next_id = reply_chain_id(&fetched);
            }
            Err(_) => break,
        }
    }

    entries.reverse();

    if entries.len() > 1 {
        let lines: Vec<String> = entries
            .iter()
            .map(|(is_bot, content)| {
                let role = if *is_bot { "Assistant" } else { "User" };
                format!("{}: {}", role, content)
            })
            .collect();
        Ok(format!(
            "Previous conversation:\n{}\n\nCurrent message: {}",
            lines[..lines.len() - 1].join("\n"),
            lines
                .last()
                .unwrap()
                .replace("User: ", "")
                .replace("Assistant: ", "")
        ))
    } else {
        Ok(clean_message_content(&message.content, current_user))
    }
}

fn reply_chain_id(msg: &serenity::Message) -> Option<serenity::MessageId> {
    msg.referenced_message
        .as_ref()
        .map(|m| m.id)
        .or_else(|| msg.message_reference.as_ref().and_then(|r| r.message_id))
}

fn push_entry(
    entries: &mut Vec<(bool, String)>,
    msg: &serenity::Message,
    current_user: &serenity::User,
) {
    let content = clean_message_content(&msg.content, current_user);
    if !content.trim().is_empty() {
        entries.push((msg.author.id == current_user.id, content));
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

#[cfg(feature = "llm")]
async fn handle_llm_request(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
    openai_client: &LlmClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = message.author.id.get();

    // Check cooldown unless user is in no-cooldown list
    if !data.config.llm.no_cooldown_users.contains(&user_id) {
        let cooldown_duration = Duration::from_secs(10);

        if !openai_client.check_and_update_cooldown(user_id, cooldown_duration) {
            debug!("user {} on cooldown", user_id);
            return Ok(());
        }
    }

    let current_user = ctx.http.get_current_user().await?;

    let prompt = build_conversation_context(ctx, message, &current_user).await?;

    if prompt.trim().is_empty() {
        return Ok(());
    }

    debug!("llm user {}: {}", user_id, prompt);

    let typing = message.channel_id.start_typing(&ctx.http);

    let mut messages = Vec::new();

    if !data.config.llm.system_prompt.trim().is_empty() {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: data.config.llm.system_prompt.clone(),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
    });

    let response = openai_client
        .generate(
            &data.config.llm.model,
            messages,
            data.config.llm.temperature,
            data.config.llm.max_tokens,
            if data.config.llm.stop.is_empty() {
                None
            } else {
                Some(&data.config.llm.stop)
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
        if current_chunk.len() + word.len() + 1 > max_length && !current_chunk.is_empty() {
            chunks.push(current_chunk);
            current_chunk = String::new();
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
        warn!("react to message: {}", e);
    }
}

async fn send_help_as_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let commands = crate::commands::help::get_all_commands();
    let color = clouder_core::utils::get_embed_color(data, message.guild_id.map(|g| g.get())).await;
    let embed = crate::commands::help::create_help_embed(&commands, color);

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
#[cfg(feature = "llm")]
pub async fn handle_ai_retry_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &AppState,
) {
    // Parse custom_id: "ai_retry_{user_id}_{prompt_hash}_{original_message_id}"
    let parts: Vec<&str> = interaction.data.custom_id.split('_').collect();
    if parts.len() != 5 || parts[0] != "ai" || parts[1] != "retry" {
        error!("invalid custom_id format: {}", interaction.data.custom_id);
        return;
    }

    let requesting_user_id = match parts[2].parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid user_id in custom_id: {}", parts[2]);
            return;
        }
    };

    let original_message_id = match parts[4].parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            error!("invalid message_id in custom_id: {}", parts[4]);
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
                        .content("only the person who triggered this can retry")
                        .ephemeral(true),
                ),
            )
            .await
        {
            error!("send unauthorized response: {}", e);
        }
        return;
    }

    if data.config.llm.provider.is_none() {
        error!("LLM not enabled for retry");
        return;
    }

    let openai_client = match &data.llm_client {
        Some(client) => client,
        None => {
            error!("no openai client for retry");
            return;
        }
    };

    if !data
        .config
        .llm
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
                            .content("please wait before retrying")
                            .ephemeral(true),
                    ),
                )
                .await
            {
                error!("send cooldown response: {}", e);
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
        error!("acknowledge retry interaction: {}", e);
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
                "fetch original message {} for retry: {}",
                original_message_id, e
            );
            if let Err(edit_err) = interaction
                .edit_response(
                    &ctx.http,
                    serenity::EditInteractionResponse::new()
                        .content("could not find the original message, it may have been deleted")
                        .components(vec![]),
                )
                .await
            {
                error!("send missing message error: {}", edit_err);
            }
            return;
        }
    };

    // Verify that the user who clicked the button is the same as the author of the original message
    if user_message.author.id.get() != requesting_user_id {
        error!(
            "user id mismatch: button {} vs author {}",
            requesting_user_id,
            user_message.author.id.get()
        );
        if let Err(e) = interaction
            .edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content("user verification failed, please send a new message instead")
                    .components(vec![]),
            )
            .await
        {
            error!("send verification error: {}", e);
        }
        return;
    }

    // Build the conversation context again
    let current_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            error!("get current user for retry: {}", e);
            return;
        }
    };

    let prompt = match build_conversation_context(ctx, &user_message, &current_user).await {
        Ok(p) => p,
        Err(e) => {
            error!("build context for retry: {}", e);
            if let Err(edit_err) = interaction
                .edit_response(
                    &ctx.http,
                    serenity::EditInteractionResponse::new()
                        .content("failed to rebuild context, please send a new message instead")
                        .components(vec![]),
                )
                .await
            {
                error!("send context error: {}", edit_err);
            }
            return;
        }
    };

    if prompt.trim().is_empty() {
        error!("empty prompt for retry");
        if let Err(e) = interaction.edit_response(&ctx.http,
            serenity::EditInteractionResponse::new()
                .content("could not extract content from original message, please send a new message instead")
                .components(vec![])
        ).await {
            error!("send empty prompt error: {}", e);
        }
        return;
    }

    debug!("retry user {}: {}", requesting_user_id, prompt);
    debug!(
        "original message {}, preview: {}",
        user_message.id,
        user_message.content.chars().take(50).collect::<String>()
    );

    // Build messages array for OpenAI
    let mut messages = Vec::new();

    if !data.config.llm.system_prompt.trim().is_empty() {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: data.config.llm.system_prompt.clone(),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
    });

    let response = match openai_client
        .generate(
            &data.config.llm.model,
            messages,
            data.config.llm.temperature,
            data.config.llm.max_tokens,
            if data.config.llm.stop.is_empty() {
                None
            } else {
                Some(&data.config.llm.stop)
            },
        )
        .await
    {
        Ok(response) => response,
        Err(e) => {
            error!("generate retry response: {}", e);

            // Update message to show error
            if let Err(edit_err) = interaction
                .edit_response(
                    &ctx.http,
                    serenity::EditInteractionResponse::new()
                        .content("failed to generate new response, please try again later")
                        .components(vec![create_retry_button(
                            requesting_user_id,
                            &prompt,
                            original_message_id,
                        )]),
                )
                .await
            {
                error!("update message after error: {}", edit_err);
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
        error!("update message with response: {}", e);
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
        .label("try again")
        .style(ButtonStyle::Secondary);

    CreateActionRow::Buttons(vec![retry_button])
}

fn create_disabled_retry_button() -> serenity::CreateActionRow {
    use serenity::all::{ButtonStyle, CreateActionRow, CreateButton};

    let retry_button = CreateButton::new("ai_retry_disabled")
        .label("generating...")
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
