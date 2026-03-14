use clouder_core::config::AppState;
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command, required_permissions = "MANAGE_ROLES", guild_only)]
pub async fn selfroles(ctx: Context<'_>) -> Result<(), Error> {
    let dashboard_url = &ctx.data().config.web.api_base;
    let guild_id = ctx.guild_id().expect("guild_only command").to_string();

    let embed = serenity::CreateEmbed::new()
        .title("self-roles config")
        .description("click the link below to configure self-roles for your server")
        .field(
            "web dashboard",
            format!("{}/dashboard/{}/selfroles", dashboard_url, guild_id),
            false,
        )
        .color(get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await)
        .footer(serenity::CreateEmbedFooter::new(
            "you need 'manage roles' permission to configure self-roles",
        ));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
