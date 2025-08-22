use crate::config::AppState;
use crate::utils::video::{generate_preview_html, sanitize_html_content, save_video_preview};
use rand::distr::Alphanumeric;
use rand::Rng;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// Create a video embed to bypass Discord's 10MB limit
#[poise::command(
    slash_command,
    guild_only = true
)]
pub async fn video(
    ctx: Context<'_>,
    #[description = "Direct URL to the video file (e.g., https://example.com/video.mp4)"] url: String,
    #[description = "Custom title for the video (optional)"] title: Option<String>
) -> Result<(), Error> {
    ctx.defer().await?;

    let embed_id = rand::rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect::<String>();
    let html_content = generate_preview_html(&url);

    let embed_dir = &ctx.data().config.web.embed.directory;
    match save_video_preview(embed_dir, &embed_id, &html_content).await {
        Ok(_) => {
            let base_url = &ctx.data().config.web.base_url;
            let embed_url = format!("{}/video/{}.html", base_url, embed_id);

            let video_title = title.clone().unwrap_or_else(|| url.to_string());
            let sanitized_title = sanitize_html_content(&video_title);

            let mut url_message = embed_url.clone();
            if title.is_some() {
                url_message = format!("[{sanitized_title}]({embed_url})");
            }

            ctx.say(url_message).await?;

            info!("Created video embed {} for URL: {}", embed_id, url);
        }
        Err(e) => {
            error!("Failed to save embed file: {}", e);
            ctx.send(poise::CreateReply::default()
                .content("❌ failed to create embed file. please try again later.")
                .ephemeral(true)).await?;
        }
    }

    Ok(())
}
