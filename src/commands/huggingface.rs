use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::external::huggingface::{HfModel, fetch_latest, fetch_trending};
use clouder_core::utils::{format_count, get_embed_color};
use poise::serenity_prelude as serenity;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage,
};
use serenity::collector::ComponentInteractionCollector;
use std::time::Duration;
use tracing::warn;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// browse huggingface models
#[poise::command(slash_command, subcommands("latest", "trending"))]
pub async fn hf(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// browse recently updated models
#[poise::command(slash_command)]
async fn latest(ctx: Context<'_>) -> Result<(), Error> {
    paginate(ctx, fetch_latest().await, "latest").await
}

/// browse top trending models
#[poise::command(slash_command)]
async fn trending(ctx: Context<'_>) -> Result<(), Error> {
    paginate(ctx, fetch_trending().await, "trending").await
}

async fn paginate(
    ctx: Context<'_>,
    result: anyhow::Result<Vec<HfModel>>,
    label: &'static str,
) -> Result<(), Error> {
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;

    let models = match result {
        Ok(m) if !m.is_empty() => m,
        Ok(_) => {
            ctx.send(
                poise::CreateReply::default()
                    .content("no models returned")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        Err(e) => {
            warn!("hf fetch failed: {}", e);
            ctx.send(
                poise::CreateReply::default()
                    .content("failed to fetch models, try again later")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let total = models.len();
    let prev_id = format!("hf_prev_{}", ctx.id());
    let next_id = format!("hf_next_{}", ctx.id());
    let mut page = 0usize;

    let mut msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(build_embed(&models[page], page, total, label, color))
                .components(nav_buttons(&prev_id, &next_id, page, total)),
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

        if interaction.data.custom_id == prev_id {
            page = page.saturating_sub(1);
        } else if interaction.data.custom_id == next_id {
            page = (page + 1).min(total - 1);
        }

        interaction
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(build_embed(&models[page], page, total, label, color))
                        .components(nav_buttons(&prev_id, &next_id, page, total)),
                ),
            )
            .await?;
    }

    Ok(())
}

fn build_embed(
    model: &HfModel,
    page: usize,
    total: usize,
    label: &str,
    color: serenity::Color,
) -> CreateEmbed {
    let url = format!("https://huggingface.co/{}", model.id);
    let mut embed = CreateEmbed::new()
        .title(model.short_name())
        .url(&url)
        .color(color);

    if let Some(author) = model.resolved_author() {
        embed = embed.author(CreateEmbedAuthor::new(author));
    }

    if let Some(desc) = model.description() {
        embed = embed.description(truncate(desc, 100));
    }

    if let Some(task) = &model.pipeline_tag {
        embed = embed.field("task", task, true);
    }

    embed = embed
        .field("downloads", format_count(model.downloads), true)
        .field("likes", format_count(model.likes), true);

    let tags = model.relevant_tags(4);
    if !tags.is_empty() {
        embed = embed.field("tags", tags.join(", "), false);
    }

    if let Some(date) = &model.last_modified {
        embed = embed.field("updated", &date[..date.len().min(10)], true);
    }

    embed
        .field("id", format!("[{}]({})", model.id, url), false)
        .footer(CreateEmbedFooter::new(format!(
            "{}/{} • huggingface.co {} • cached 5 min",
            page + 1,
            total,
            label
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
