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
