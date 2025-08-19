use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub access_token: String,
    pub user_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub guilds: Vec<GuildInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: String,
    pub owner: bool,
}

#[derive(Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscordGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: String,
    pub owner: bool,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

impl UserSession {
    pub fn has_manage_roles_in_guild(&self, guild_id: &str) -> bool {
        if let Some(guild) = self.guilds.iter().find(|g| g.id == guild_id) {
            if guild.owner {
                return true;
            }
            
            let permissions = guild.permissions.parse::<u64>().unwrap_or(0);
            const MANAGE_ROLES: u64 = 0x10000000; // 268435456
            (permissions & MANAGE_ROLES) != 0
        } else {
            false
        }
    }
    
    pub fn get_manageable_guilds(&self) -> Vec<&GuildInfo> {
        self.guilds.iter()
            .filter(|guild| {
                if guild.owner {
                    return true;
                }
                let permissions = guild.permissions.parse::<u64>().unwrap_or(0);
                const MANAGE_ROLES: u64 = 0x10000000;
                (permissions & MANAGE_ROLES) != 0
            })
            .collect()
    }
}