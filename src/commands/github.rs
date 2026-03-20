use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::external::github::{GhRepo, GhUser, fetch_repo, fetch_repos, fetch_user};
use clouder_core::utils::{format_count, get_embed_color};
use poise::serenity_prelude as serenity;
use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage,
};
use std::time::Duration;
use tracing::warn;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

const REPOS_PER_PAGE: usize = 5;

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
        return Ok(());
    }

    let (user_result, repos_result) =
        tokio::join!(fetch_user(&user, token), fetch_repos(&user, token));

    match user_result {
        Ok(u) => {
            let repos = match repos_result {
                Ok(r) => r,
                Err(e) => {
                    warn!("github repos fetch failed: {}", e);
                    Vec::new()
                }
            };
            let total_stars: u64 = repos.iter().map(|r| r.stargazers_count as u64).sum();

            let repos_id = format!("gh_repos_{}", ctx.id());
            let prev_id = format!("gh_prev_{}", ctx.id());
            let next_id = format!("gh_next_{}", ctx.id());

            let components = if !repos.is_empty() {
                repos_button(&repos_id)
            } else {
                vec![]
            };

            let mut msg = ctx
                .send(
                    poise::CreateReply::default()
                        .embed(user_embed(&u, total_stars, color))
                        .components(components),
                )
                .await?
                .into_message()
                .await?;

            if repos.is_empty() {
                return Ok(());
            }

            let mut page = 0usize;
            let mut viewing_repos = false;

            loop {
                let Some(interaction) = ComponentInteractionCollector::new(ctx.serenity_context())
                    .message_id(msg.id)
                    .author_id(ctx.author().id)
                    .timeout(Duration::from_secs(60))
                    .await
                else {
                    msg.edit(ctx.http(), EditMessage::new().components(vec![]))
                        .await
                        .ok();
                    break;
                };

                if interaction.data.custom_id == repos_id {
                    viewing_repos = true;
                    page = 0;
                } else if interaction.data.custom_id == prev_id {
                    page = page.saturating_sub(1);
                } else if interaction.data.custom_id == next_id {
                    let total = repos.len().div_ceil(REPOS_PER_PAGE);
                    page = (page + 1).min(total - 1);
                }

                let total_pages = repos.len().div_ceil(REPOS_PER_PAGE);
                let (embed, components) = if viewing_repos {
                    (
                        repos_page_embed(&repos, page, total_pages, &u, color),
                        nav_buttons(&prev_id, &next_id, page, total_pages),
                    )
                } else {
                    (user_embed(&u, total_stars, color), repos_button(&repos_id))
                };

                interaction
                    .create_response(
                        ctx.http(),
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(embed)
                                .components(components),
                        ),
                    )
                    .await?;
            }
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

    Ok(())
}

fn repos_button(repos_id: &str) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(repos_id)
            .label("repos")
            .style(ButtonStyle::Secondary),
    ])]
}

fn nav_buttons(prev_id: &str, next_id: &str, page: usize, total: usize) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(prev_id)
            .label("◀")
            .style(ButtonStyle::Secondary)
            .disabled(page == 0),
        CreateButton::new(next_id)
            .label("▶")
            .style(ButtonStyle::Secondary)
            .disabled(page + 1 >= total),
    ])]
}

fn repos_page_embed(
    repos: &[GhRepo],
    page: usize,
    total_pages: usize,
    u: &GhUser,
    color: serenity::Color,
) -> CreateEmbed {
    let start = page * REPOS_PER_PAGE;
    let page_repos = &repos[start..(start + REPOS_PER_PAGE).min(repos.len())];
    let login = &u.login;

    let mut embed = CreateEmbed::new()
        .title(format!("{}'s repositories", u.display_name()))
        .url(&u.html_url)
        .thumbnail(&u.avatar_url)
        .color(color);

    for r in page_repos {
        let name = r
            .full_name
            .strip_prefix(&format!("{login}/"))
            .unwrap_or(&r.full_name);
        let mut value = format!(
            "⭐ {} • [open]({})",
            format_count(r.stargazers_count as u64),
            r.html_url
        );
        if let Some(lang) = &r.language {
            value = format!("{} • {}", lang, value);
        }
        if let Some(desc) = &r.description
            && !desc.is_empty()
        {
            value.push('\n');
            value.push_str(desc);
        }
        embed = embed.field(name, value, false);
    }

    embed.footer(CreateEmbedFooter::new(format!(
        "github • page {}/{}",
        page + 1,
        total_pages
    )))
}

fn user_embed(u: &GhUser, total_stars: u64, color: serenity::Color) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(u.display_name())
        .url(&u.html_url)
        .thumbnail(&u.avatar_url)
        .color(color)
        .field("repos", u.public_repos.to_string(), true)
        .field("stars", format_count(total_stars), true)
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
