use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage,
};
use serenity::collector::ComponentInteractionCollector;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: Option<String>,
    pub category: CommandCategory,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Core,
    Info,
    ApiIntegration,
    Management,
    Utility,
}

impl CommandCategory {
    pub fn as_str(&self) -> &str {
        match self {
            CommandCategory::Core => "core",
            CommandCategory::Info => "info",
            CommandCategory::ApiIntegration => "api stuff",
            CommandCategory::Management => "management",
            CommandCategory::Utility => "utility",
        }
    }

    fn short_str(&self) -> &str {
        match self {
            CommandCategory::Core => "core",
            CommandCategory::Info => "info",
            CommandCategory::ApiIntegration => "api",
            CommandCategory::Management => "management",
            CommandCategory::Utility => "utility",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Core,
            Self::Info,
            Self::Management,
            Self::ApiIntegration,
            Self::Utility,
        ]
    }
}

pub fn get_all_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "/selfroles".to_string(),
            description: "manage selfroles".to_string(),
            usage: Some("/selfroles".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage roles".to_string()),
        },
        CommandInfo {
            name: "/about bot".to_string(),
            description: "some info about me :3".to_string(),
            usage: Some("/about bot".to_string()),
            category: CommandCategory::Info,
            permissions: None,
        },
        CommandInfo {
            name: "/about server".to_string(),
            description: "show info about this server".to_string(),
            usage: Some("/about server".to_string()),
            category: CommandCategory::Info,
            permissions: None,
        },
        CommandInfo {
            name: "/about user".to_string(),
            description: "show info about a user".to_string(),
            usage: Some("/about user [@user]".to_string()),
            category: CommandCategory::Info,
            permissions: None,
        },
        CommandInfo {
            name: "/purge".to_string(),
            description: "purges messages from channel".to_string(),
            usage: Some("/purge [number / message_id]".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage messages".to_string()),
        },
        CommandInfo {
            name: "/mediaonly".to_string(),
            description: "configure media-only channels".to_string(),
            usage: Some("/mediaonly [channel] [enable/disable]".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage channels".to_string()),
        },
        CommandInfo {
            name: "/channel delete".to_string(),
            description: "delete a channel".to_string(),
            usage: Some("/channel delete [channel]".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage channels".to_string()),
        },
        CommandInfo {
            name: "/channel clone".to_string(),
            description: "clone a channel with all its settings".to_string(),
            usage: Some("/channel clone [channel]".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage channels".to_string()),
        },
        CommandInfo {
            name: "/channel nuke".to_string(),
            description: "delete and recreate a channel, wiping all history".to_string(),
            usage: Some("/channel nuke [channel]".to_string()),
            category: CommandCategory::Management,
            permissions: Some("manage channels".to_string()),
        },
        CommandInfo {
            name: "/random".to_string(),
            description: "freaky link generator".to_string(),
            usage: Some("/random".to_string()),
            category: CommandCategory::Core,
            permissions: None,
        },
        CommandInfo {
            name: "/uwufy".to_string(),
            description: "toggle uwufy mode for a user".to_string(),
            usage: Some("/uwufy [@user]".to_string()),
            category: CommandCategory::Core,
            permissions: Some("manage server".to_string()),
        },
        CommandInfo {
            name: "/hf trending".to_string(),
            description: "browse top trending models on huggingface".to_string(),
            usage: Some("/hf trending".to_string()),
            category: CommandCategory::ApiIntegration,
            permissions: None,
        },
        CommandInfo {
            name: "/hf latest".to_string(),
            description: "browse recently updated models on huggingface".to_string(),
            usage: Some("/hf latest".to_string()),
            category: CommandCategory::ApiIntegration,
            permissions: None,
        },
    ]
}

#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "show help message"]
    #[autocomplete = "category_autocomplete"]
    category: Option<String>,
) -> Result<(), Error> {
    let commands = get_all_commands();

    match category {
        Some(cat) => show_category_help(ctx, &commands, &cat).await?,
        None => show_general_help(ctx, &commands).await?,
    }

    Ok(())
}

async fn show_general_help(ctx: Context<'_>, commands: &[CommandInfo]) -> Result<(), Error> {
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;
    let embed = create_help_embed(commands, color);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub fn create_help_embed(commands: &[CommandInfo], color: serenity::Color) -> CreateEmbed {
    let mut categories = std::collections::HashMap::new();
    for cmd in commands {
        categories
            .entry(cmd.category.clone())
            .or_insert_with(Vec::new)
            .push(cmd);
    }

    let mut embed = CreateEmbed::new()
        .title("command list")
        .description("`/help [category]` for more details")
        .color(color);

    for category in CommandCategory::all() {
        if let Some(category_commands) = categories.get(category) {
            let command_list = category_commands
                .iter()
                .map(|cmd| {
                    format!(
                        "**{}** - {}",
                        cmd.name,
                        truncate_description(&cmd.description, 50)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            embed = embed.field(
                format!("> ***{}***", category.as_str()),
                command_list,
                false,
            );
        }
    }

    let footer_text = &format!("version {}", env!("CARGO_PKG_VERSION")).to_string();
    embed = embed.footer(CreateEmbedFooter::new(footer_text));

    embed
}

const COMMANDS_PER_PAGE: usize = 5;

async fn show_category_help(
    ctx: Context<'_>,
    commands: &[CommandInfo],
    category_name: &str,
) -> Result<(), Error> {
    let category = match category_name.to_lowercase().as_str() {
        "core" => CommandCategory::Core,
        "info" | "information" => CommandCategory::Info,
        "management" | "manage" => CommandCategory::Management,
        "api" | "integration" => CommandCategory::ApiIntegration,
        "utility" | "util" => CommandCategory::Utility,
        _ => {
            ctx.send(
                poise::CreateReply::default()
                    .content("invalid category! available: `core`, `info`, `management`, `api`, `utility`")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let category_commands: Vec<&CommandInfo> = commands
        .iter()
        .filter(|cmd| cmd.category == category)
        .collect();

    if category_commands.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("no commands for '{}' yet", category.as_str()))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let total_pages = category_commands.len().div_ceil(COMMANDS_PER_PAGE);
    let color = get_embed_color(ctx.data(), ctx.guild_id().map(|g| g.get())).await;

    if total_pages <= 1 {
        let embed = build_page_embed(&category_commands, &category, 0, 1, color);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let prev_id = format!("help_prev_{}", ctx.id());
    let next_id = format!("help_next_{}", ctx.id());
    let mut page = 0usize;

    let mut msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(build_page_embed(
                    &category_commands,
                    &category,
                    page,
                    total_pages,
                    color,
                ))
                .components(nav_buttons(&prev_id, &next_id, page, total_pages)),
        )
        .await?
        .into_message()
        .await?;

    loop {
        let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
            .message_id(msg.id)
            .author_id(ctx.author().id)
            .timeout(Duration::from_secs(60))
            .await;

        let Some(interaction) = interaction else {
            msg.edit(ctx.http(), EditMessage::new().components(vec![]))
                .await
                .ok();
            break;
        };

        if interaction.data.custom_id == prev_id {
            page = page.saturating_sub(1);
        } else if interaction.data.custom_id == next_id {
            page = (page + 1).min(total_pages - 1);
        }

        interaction
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(build_page_embed(
                            &category_commands,
                            &category,
                            page,
                            total_pages,
                            color,
                        ))
                        .components(nav_buttons(&prev_id, &next_id, page, total_pages)),
                ),
            )
            .await?;
    }

    Ok(())
}

fn build_page_embed(
    commands: &[&CommandInfo],
    category: &CommandCategory,
    page: usize,
    total_pages: usize,
    color: serenity::Color,
) -> CreateEmbed {
    let start = page * COMMANDS_PER_PAGE;
    let page_commands = &commands[start..(start + COMMANDS_PER_PAGE).min(commands.len())];

    let title = if total_pages > 1 {
        format!("{} — page {}/{}", category.as_str(), page + 1, total_pages)
    } else {
        format!("{} — details", category.as_str())
    };

    let mut embed = CreateEmbed::new().title(title).color(color);

    for cmd in page_commands {
        let mut field = format!("**desc:** {}\n", cmd.description);
        if let Some(usage) = &cmd.usage {
            field.push_str(&format!("**usage:** `{}`\n", usage));
        }
        if let Some(perms) = &cmd.permissions {
            field.push_str(&format!("**permissions:** {}\n", perms));
        }
        embed = embed.field(&cmd.name, field, false);
    }

    embed.footer(CreateEmbedFooter::new(format!(
        "{} commands in {} • use /help for all categories",
        commands.len(),
        category.as_str()
    )))
}

fn nav_buttons(
    prev_id: &str,
    next_id: &str,
    page: usize,
    total_pages: usize,
) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(prev_id)
            .label("◀")
            .style(ButtonStyle::Secondary)
            .disabled(page == 0),
        CreateButton::new(next_id)
            .label("▶")
            .style(ButtonStyle::Secondary)
            .disabled(page + 1 >= total_pages),
    ])]
}

async fn category_autocomplete(_ctx: Context<'_>, partial: &str) -> impl Iterator<Item = String> {
    let partial = partial.to_lowercase();
    CommandCategory::all()
        .iter()
        .map(|c| c.short_str())
        .filter(move |name| name.starts_with(&partial))
        .map(|s| s.to_string())
}

pub fn truncate_description(desc: &str, max_len: usize) -> String {
    if desc.len() <= max_len {
        desc.to_string()
    } else {
        format!("{}...", &desc[..max_len.saturating_sub(3)])
    }
}
