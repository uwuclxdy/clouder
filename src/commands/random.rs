use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::utils::get_default_embed_color;
use poise::serenity_prelude as serenity;
use serenity::CreateEmbed;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command)]
pub async fn random(ctx: Context<'_>) -> Result<(), Error> {
    let number = rand::random_range(100_000..=9_999_999);
    let url = format!("https://nhentai.to/g/{}", number);

    let embed = CreateEmbed::new()
        .title("freaky link :3")
        .description(format!("[{}]({})", number, url))
        .color(get_default_embed_color(ctx.data()));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
