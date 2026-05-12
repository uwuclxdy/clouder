use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::external::tinyfox::{fetch_animal_image, progress_url};
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command, subcommands("animal", "progress"))]
pub async fn tinyfox(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn animal(
    ctx: Context<'_>,
    #[description = "animal type"]
    #[choices(
        "fox", "yeen", "dog", "manul", "snek", "poss", "leo", "serval", "bleat", "shiba", "racc",
        "dook", "ott", "snep", "woof", "chi", "capy", "bear", "bun", "caracal", "puma", "mane",
        "marten", "tig", "wah", "skunk", "jaguar", "yote"
    )]
    animal: &str,
) -> Result<(), Error> {
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;

    match fetch_animal_image(animal).await {
        Ok(url) => {
            let embed = CreateEmbed::new()
                .title(animal)
                .image(url)
                .color(color)
                .footer(CreateEmbedFooter::new("tinyfox.dev"));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(_) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("could not fetch image")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn progress(
    ctx: Context<'_>,
    #[description = "time period"]
    #[choices("day", "month", "year")]
    period: &str,
    #[description = "timezone (e.g. America/New_York)"] timezone: Option<String>,
) -> Result<(), Error> {
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;
    let url = progress_url(period, timezone.as_deref());

    let embed = CreateEmbed::new()
        .title(format!("{period} progress"))
        .image(url)
        .color(color)
        .footer(CreateEmbedFooter::new("tinyfox.dev"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
