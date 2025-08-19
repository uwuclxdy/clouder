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
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub username: String,
    #[allow(dead_code)]
    pub avatar: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscordGuild {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub icon: Option<String>,
    #[allow(dead_code)]
    pub permissions: String,
    #[allow(dead_code)]
    pub owner: bool,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    #[allow(dead_code)]
    pub access_token: String,
    #[allow(dead_code)]
    pub token_type: String,
    #[allow(dead_code)]
    pub expires_in: u64,
    #[allow(dead_code)]
    pub refresh_token: String,
    #[allow(dead_code)]
    pub scope: String,
}

impl UserSession {
    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
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