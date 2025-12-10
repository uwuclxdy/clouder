use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;

/// Configuration for building an embed message
pub struct EmbedConfig<'a> {
    pub title: &'a Option<String>,
    pub description: &'a Option<String>,
    pub color: Option<i32>,
    pub footer: &'a Option<String>,
    pub thumbnail: &'a Option<String>,
    pub image: &'a Option<String>,
    pub timestamp: bool,
    pub default_color: u64,
}

/// Build an embed from config and placeholders
pub fn build_embed(config: &EmbedConfig, placeholders: &HashMap<String, String>) -> CreateEmbed {
    let mut embed = CreateEmbed::new();

    if let Some(title) = config.title
        && !title.trim().is_empty()
    {
        embed = embed.title(replace_placeholders(title, placeholders));
    }

    if let Some(description) = config.description
        && !description.trim().is_empty()
    {
        embed = embed.description(replace_placeholders(description, placeholders));
    }

    let color = config
        .color
        .map(|c| c as u64)
        .unwrap_or(config.default_color);
    embed = embed.color(color);

    if let Some(footer) = config.footer
        && !footer.trim().is_empty()
    {
        embed = embed.footer(CreateEmbedFooter::new(replace_placeholders(
            footer,
            placeholders,
        )));
    }

    if let Some(thumbnail) = config.thumbnail
        && !thumbnail.trim().is_empty()
    {
        embed = embed.thumbnail(replace_placeholders(thumbnail, placeholders));
    }

    if let Some(image) = config.image
        && !image.trim().is_empty()
    {
        embed = embed.image(replace_placeholders(image, placeholders));
    }

    if config.timestamp {
        embed = embed.timestamp(serenity::model::timestamp::Timestamp::now());
    }

    embed
}

/// Replace placeholders in content with their values
pub fn replace_placeholders(content: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (key, value) in placeholders {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

pub fn validate_message_config(
    message_type: &str,
    message_content: &Option<String>,
    embed_title: &Option<String>,
    embed_description: &Option<String>,
) -> Result<(), String> {
    match message_type {
        "embed" => {
            let has_title = embed_title.as_ref().is_some_and(|t| !t.trim().is_empty());
            let has_description = embed_description
                .as_ref()
                .is_some_and(|d| !d.trim().is_empty());

            if !has_title && !has_description {
                return Err("Embed messages require either a title or description".to_string());
            }
        }
        "text" => {
            let has_content = message_content
                .as_ref()
                .is_some_and(|c| !c.trim().is_empty());

            if !has_content {
                return Err("Text messages require content".to_string());
            }
        }
        _ => {
            return Err("Invalid message type. Must be 'embed' or 'text'".to_string());
        }
    }

    Ok(())
}

pub fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}
