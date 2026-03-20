use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::external::github_trending::{Period, TrendingRepo, fetch_trending};
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

struct ButtonIds {
    daily: String,
    weekly: String,
    monthly: String,
    prev: String,
    next: String,
}

impl ButtonIds {
    fn new(ctx_id: u64) -> Self {
        Self {
            daily: format!("ght_daily_{ctx_id}"),
            weekly: format!("ght_weekly_{ctx_id}"),
            monthly: format!("ght_monthly_{ctx_id}"),
            prev: format!("ght_prev_{ctx_id}"),
            next: format!("ght_next_{ctx_id}"),
        }
    }

    fn period_for(&self, id: &str) -> Option<Period> {
        if id == self.daily {
            Some(Period::Daily)
        } else if id == self.weekly {
            Some(Period::Weekly)
        } else if id == self.monthly {
            Some(Period::Monthly)
        } else {
            None
        }
    }
}

/// browse github trending repositories
#[poise::command(slash_command)]
pub async fn gh_trending(ctx: Context<'_>) -> Result<(), Error> {
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;
    let ids = ButtonIds::new(ctx.id());

    let mut repos = match fetch_trending(Period::Daily).await {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("no trending repos found")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        Err(e) => {
            warn!("gh-trending fetch failed: {}", e);
            ctx.send(
                poise::CreateReply::default()
                    .content("failed to fetch trending repos, try again later")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let mut period = Period::Daily;
    let mut page = 0usize;

    let mut msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(page_embed(&repos, page, period, color))
                .components(all_buttons(&ids, period, page, total_pages(repos.len()))),
        )
        .await?
        .into_message()
        .await?;

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

        let id = &interaction.data.custom_id;

        if let Some(selected) = ids.period_for(id) {
            if selected != period {
                match fetch_trending(selected).await {
                    Ok(r) if !r.is_empty() => {
                        repos = r;
                        period = selected;
                        page = 0;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("gh-trending fetch failed: {}", e);
                    }
                }
            }
        } else if id == &ids.prev {
            page = page.saturating_sub(1);
        } else if id == &ids.next {
            page = (page + 1).min(total_pages(repos.len()).saturating_sub(1));
        }

        interaction
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(page_embed(&repos, page, period, color))
                        .components(all_buttons(&ids, period, page, total_pages(repos.len()))),
                ),
            )
            .await?;
    }

    Ok(())
}

fn total_pages(repo_count: usize) -> usize {
    repo_count.div_ceil(REPOS_PER_PAGE).max(1)
}

fn all_buttons(
    ids: &ButtonIds,
    period: Period,
    page: usize,
    total_pages: usize,
) -> Vec<CreateActionRow> {
    vec![
        period_row(ids, period),
        nav_row(&ids.prev, &ids.next, page, total_pages),
    ]
}

fn period_row(ids: &ButtonIds, active: Period) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new(&ids.daily)
            .label("daily")
            .style(ButtonStyle::Secondary)
            .disabled(active == Period::Daily),
        CreateButton::new(&ids.weekly)
            .label("weekly")
            .style(ButtonStyle::Secondary)
            .disabled(active == Period::Weekly),
        CreateButton::new(&ids.monthly)
            .label("monthly")
            .style(ButtonStyle::Secondary)
            .disabled(active == Period::Monthly),
    ])
}

fn nav_row(prev_id: &str, next_id: &str, page: usize, total_pages: usize) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new(prev_id)
            .label("◀")
            .style(ButtonStyle::Secondary)
            .disabled(page == 0),
        CreateButton::new(next_id)
            .label("▶")
            .style(ButtonStyle::Secondary)
            .disabled(page + 1 >= total_pages),
    ])
}

fn page_embed(
    repos: &[TrendingRepo],
    page: usize,
    period: Period,
    color: serenity::Color,
) -> CreateEmbed {
    let pages = total_pages(repos.len());
    let start = page * REPOS_PER_PAGE;
    let page_repos = &repos[start..(start + REPOS_PER_PAGE).min(repos.len())];

    let mut embed = CreateEmbed::new()
        .title(format!("github trending — {}", period.label()))
        .url(format!(
            "https://github.com/trending?since={}",
            period.as_query()
        ))
        .color(color);

    for (i, repo) in page_repos.iter().enumerate() {
        let rank = start + i + 1;
        let field_name = format!("{}. {}", rank, repo.full_name());

        let mut value = format!(
            "⭐ {} • 🍴 {} • {} • [open]({})",
            format_count(repo.stars),
            format_count(repo.forks),
            repo.stars_period,
            repo.url()
        );
        if let Some(lang) = &repo.language {
            value = format!("{} • {}", lang, value);
        }
        if let Some(desc) = &repo.description {
            value.push('\n');
            value.push_str(&truncate(desc, 80));
        }

        embed = embed.field(field_name, value, false);
    }

    embed.footer(CreateEmbedFooter::new(format!(
        "github trending • page {}/{}",
        page + 1,
        pages
    )))
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}…", &s[..end])
    }
}
