use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::external::github::{GhRepo, GhUser, fetch_repo, fetch_user};
use clouder_core::utils::{format_count, get_embed_color};
use poise::serenity_prelude as serenity;
use serenity::all::{CreateEmbed, CreateEmbedFooter};
use tracing::warn;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// look up a GitHub user or repository
#[poise::command(slash_command)]
pub async fn github(
    ctx: Context<'_>,
    #[description = "GitHub username"] user: String,
    #[description = "repository name"] repo: Option<String>,
) -> Result<(), Error> {
    let token = ctx.data().config.github_token.as_deref();
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;

    if let Some(repo_name) = repo {
        match fetch_repo(&user, &repo_name, token).await {
            Ok(r) => {
                ctx.send(poise::CreateReply::default().embed(repo_embed(&r, color)))
                    .await?;
            }
            Err(e) => {
                let msg = if e.to_string() == "not found" {
                    format!("repository `{}/{}` not found", user, repo_name)
                } else {
                    warn!("github repo fetch failed: {}", e);
                    "failed to fetch repository, try again later".to_string()
                };
                ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                    .await?;
            }
        }
    } else {
        match fetch_user(&user, token).await {
            Ok(u) => {
                ctx.send(poise::CreateReply::default().embed(user_embed(&u, color)))
                    .await?;
            }
            Err(e) => {
                let msg = if e.to_string() == "not found" {
                    format!("user `{}` not found", user)
                } else {
                    warn!("github user fetch failed: {}", e);
                    "failed to fetch user, try again later".to_string()
                };
                ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                    .await?;
            }
        }
    }

    Ok(())
}

fn user_embed(u: &GhUser, color: serenity::Color) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(u.display_name())
        .url(&u.html_url)
        .thumbnail(&u.avatar_url)
        .color(color)
        .field("repos", u.public_repos.to_string(), true)
        .field("followers", format_count(u.followers as u64), true)
        .field("following", format_count(u.following as u64), true);

    if let Some(bio) = &u.bio
        && !bio.is_empty()
    {
        embed = embed.description(bio);
    }
    if let Some(loc) = &u.location {
        embed = embed.field("location", loc, true);
    }
    if let Some(company) = &u.company {
        embed = embed.field("company", company, true);
    }
    if let Some(blog) = &u.blog
        && !blog.is_empty()
    {
        embed = embed.field("website", blog, true);
    }

    embed.footer(CreateEmbedFooter::new(format!("github • {}", u.login)))
}

fn repo_embed(r: &GhRepo, color: serenity::Color) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(&r.full_name)
        .url(&r.html_url)
        .thumbnail(&r.owner.avatar_url)
        .color(color)
        .field("stars", format_count(r.stargazers_count as u64), true)
        .field("forks", format_count(r.forks_count as u64), true)
        .field("issues", r.open_issues_count.to_string(), true);

    if let Some(desc) = &r.description
        && !desc.is_empty()
    {
        embed = embed.description(desc);
    }
    if let Some(lang) = &r.language {
        embed = embed.field("language", lang, true);
    }
    if let Some(date) = r.pushed_date() {
        embed = embed.field("last push", date, true);
    }
    if let Some(license) = &r.license {
        embed = embed.field("license", &license.name, true);
    }
    if !r.topics.is_empty() {
        embed = embed.field("topics", r.topics.join(", "), false);
    }

    embed.footer(CreateEmbedFooter::new("github • cached 5 min"))
}
