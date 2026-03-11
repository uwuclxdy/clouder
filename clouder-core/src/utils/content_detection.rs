use regex::Regex;
use serenity::all::Message;
use std::sync::LazyLock;

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+").unwrap());
static GIF_URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://\S*\.gif(\?\S*)?").unwrap());
static TENOR_GIPHY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https?://(tenor\.com|giphy\.com|media\.tenor\.com|media\.giphy\.com|c\.tenor\.com)\S*",
    )
    .unwrap()
});

/// Check if a message contains links/URLs
pub fn has_link(message: &Message) -> bool {
    URL_REGEX.is_match(&message.content)
}

/// Check if a message has Discord auto-embedded links
pub fn has_embedded_link(message: &Message) -> bool {
    !message.embeds.is_empty()
}

/// Check if a message has file attachments
pub fn has_attachment(message: &Message) -> bool {
    !message.attachments.is_empty()
}

/// Check if a message contains GIFs (files, URLs, or Tenor/Giphy)
pub fn has_gif(message: &Message) -> bool {
    // Check for GIF file attachments
    let has_gif_attachment = message.attachments.iter().any(|attachment| {
        attachment.filename.to_lowercase().ends_with(".gif")
            || attachment
                .content_type
                .as_ref()
                .is_some_and(|ct: &String| ct.starts_with("image/gif"))
    });

    if has_gif_attachment {
        return true;
    }

    // Check for GIF URLs in message content
    if GIF_URL_REGEX.is_match(&message.content) {
        return true;
    }

    // Check for Tenor/Giphy URLs
    if TENOR_GIPHY_REGEX.is_match(&message.content) {
        return true;
    }

    // Check for GIF embeds
    message.embeds.iter().any(|embed| {
        embed
            .image
            .as_ref()
            .is_some_and(|img| img.url.to_lowercase().contains(".gif"))
            || embed
                .video
                .as_ref()
                .is_some_and(|video| video.url.to_lowercase().contains(".gif"))
            || embed
                .thumbnail
                .as_ref()
                .is_some_and(|thumb| thumb.url.to_lowercase().contains(".gif"))
    })
}

/// Check if a message contains Discord stickers
pub fn has_sticker(message: &Message) -> bool {
    !message.sticker_items.is_empty()
}

fn has_non_gif_attachment(message: &Message) -> bool {
    message.attachments.iter().any(|a| {
        !a.filename.to_lowercase().ends_with(".gif")
            && !a
                .content_type
                .as_ref()
                .is_some_and(|ct| ct.starts_with("image/gif"))
    })
}

/// Check if a message contains any allowed content based on configuration
pub fn has_allowed_content(
    message: &Message,
    allow_links: bool,
    allow_attachments: bool,
    allow_gifs: bool,
    allow_stickers: bool,
) -> bool {
    if allow_links && (has_link(message) || has_embedded_link(message)) {
        return true;
    }

    if allow_attachments {
        let valid = if allow_gifs {
            has_attachment(message)
        } else {
            has_non_gif_attachment(message)
        };
        if valid {
            return true;
        }
    }

    if allow_gifs && has_gif(message) {
        return true;
    }

    if allow_stickers && has_sticker(message) {
        return true;
    }

    false
}
