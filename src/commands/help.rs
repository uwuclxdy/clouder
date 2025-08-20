//! # Help Command System
//!
//! ## Features
//! - **Automatic Command Registration**: Simply add commands to `get_all_commands()`
//! - **Category-based Organization**: Commands are grouped by logical categories
//! - **Detailed Command Information**: Shows usage, permissions, and descriptions
//! - **Autocomplete Support**: Category names have autocomplete in Discord
//! - **Bot Mention Trigger**: Help is also shown when the bot is mentioned (reuses same logic)
//! - **Future-proof Design**: Easy to add new commands and categories
//! - **No Code Duplication**: Shared embed creation for both slash commands and mentions
//!
//! ## Adding New Commands
//! To add a new command to the help system:
//! 1. Add a `CommandInfo` entry to the `get_all_commands()` function
//! 2. Choose the appropriate `CommandCategory`
//! 3. Fill in name, description, usage, and permissions
//! 4. The command will automatically appear in help listings
//!
//! ## Usage
//! - `/help` - Shows all commands organized by category
//! - `/help [category]` - Shows detailed help for a specific category
//! - `@BotName` - Mention the bot to show help (same as `/help`)
//!
//! Categories: core, info, management, api, utility

use crate::config::AppState;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, Color, CreateEmbedFooter};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

// Command documentation structure for easy maintenance
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
            CommandCategory::Core => "🔧 Core",
            CommandCategory::Info => "ℹ️ Information",
            CommandCategory::ApiIntegration => "🌐 API Integration",
            CommandCategory::Management => "⚙️ Management",
            CommandCategory::Utility => "🛠️ Utility",
        }
    }
}

// Centralized command registry - add new commands here for automatic help inclusion
//
// To add a new command to the help system:
// 1. Add a new CommandInfo entry to this function with appropriate details
// 2. Choose the correct CommandCategory for your command
// 3. The command will automatically appear in both general help and category-specific help
// 4. No other changes needed - the help system is fully automatic
pub fn get_all_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "/selfroles".to_string(),
            description: "manage selfroles".to_string(),
            usage: Some("/selfroles".to_string()),
            category: CommandCategory::Management,
            permissions: None,
        },

        // Info Commands
        CommandInfo {
            name: "/about bot".to_string(),
            description: "show info about me and my server :3".to_string(),
            usage: Some("/about bot".to_string()),
            category: CommandCategory::Info,
            permissions: None,
        },
        CommandInfo {
            name: "/about server".to_string(),
            description: "show info about this discord server".to_string(),
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

        // Management Commands - These will be added as they're implemented
        CommandInfo {
            name: "/video".to_string(),
            description: "make any video playable on discord (tested on nextcloud host)".to_string(),
            usage: Some("/video".to_string()),
            category: CommandCategory::Utility,
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
    let embed = create_help_embed(commands);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Create the help embed that can be reused in different contexts
pub fn create_help_embed(commands: &[CommandInfo]) -> CreateEmbed {
    // Group commands by category
    let mut categories = std::collections::HashMap::new();
    for cmd in commands {
        categories.entry(cmd.category.clone()).or_insert_with(Vec::new).push(cmd);
    }

    let mut embed = CreateEmbed::new()
        .title("✍️ command list")
        .description("`/help [category]` for more details")
        .color(Color::BLITZ_BLUE);

    // Add each category as a field
    for category in [
        CommandCategory::Core,
        CommandCategory::Info,
        CommandCategory::Management,
        CommandCategory::ApiIntegration,
        CommandCategory::Utility,
    ] {
        if let Some(category_commands) = categories.get(&category) {
            let command_list = category_commands
                .iter()
                .map(|cmd| format!("**{}** - {}", cmd.name, truncate_description(&cmd.description, 50)))
                .collect::<Vec<_>>()
                .join("\n");

            embed = embed.field(category.as_str(), command_list, false);
        }
    }

    let footer_text = &format!("version {}", env!("CARGO_PKG_VERSION")).to_string();
    embed = embed.footer(CreateEmbedFooter::new(footer_text));
    
    embed
}

async fn show_category_help(ctx: Context<'_>, commands: &[CommandInfo], category_name: &str) -> Result<(), Error> {
    // Find the matching category
    let category = match category_name.to_lowercase().as_str() {
        "core" => CommandCategory::Core,
        "info" | "information" => CommandCategory::Info,
        "management" | "manage" => CommandCategory::Management,
        "api" | "integration" => CommandCategory::ApiIntegration,
        "utility" | "util" => CommandCategory::Utility,
        _ => {
            ctx.send(poise::CreateReply::default()
                .content("❌ Invalid category! Available categories: `core`, `info`, `management`, `api`, `utility`")
                .ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let category_commands: Vec<&CommandInfo> = commands
        .iter()
        .filter(|cmd| cmd.category == category)
        .collect();

    if category_commands.is_empty() {
        ctx.send(poise::CreateReply::default()
            .content(&format!("❌ no commands for '{}' yet", category.as_str()))
            .ephemeral(true))
            .await?;
        return Ok(());
    }

    let mut embed = CreateEmbed::new()
        .title(&format!("{} - details", category.as_str()))
        .color(Color::DARK_BLUE);

    // Add each command as a detailed field
    for cmd in &category_commands {
        let mut field_value = format!("**desc:** {}\n", cmd.description);

        if let Some(usage) = &cmd.usage {
            field_value.push_str(&format!("**usage:** `{}`\n", usage));
        }

        if let Some(permissions) = &cmd.permissions {
            field_value.push_str(&format!("**permissions:** {}\n", permissions));
        }

        embed = embed.field(&cmd.name, field_value, false);
    }

    embed = embed.footer(CreateEmbedFooter::new(&format!(
        "{} commands in {} category • Use /help for all categories",
        category_commands.len(),
        category_name
    )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn category_autocomplete(
    _ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = String> {
    let categories = vec!["core", "info", "management", "api", "utility"];

    categories
        .into_iter()
        .filter(move |category| category.starts_with(&partial.to_lowercase()))
        .map(|s| s.to_string())
}

// Utility function to truncate descriptions for the overview
pub fn truncate_description(desc: &str, max_len: usize) -> String {
    if desc.len() <= max_len {
        desc.to_string()
    } else {
        format!("{}...", &desc[..max_len.saturating_sub(3)])
    }
}

// Helper function to add new commands to the registry (for future use)
#[allow(dead_code)]
pub fn register_command(command: CommandInfo) -> Vec<CommandInfo> {
    let mut commands = get_all_commands();
    commands.push(command);
    commands.sort_by(|a, b| {
        a.category.as_str().cmp(b.category.as_str())
            .then_with(|| a.name.cmp(&b.name))
    });
    commands
}
