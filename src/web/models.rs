use serde::{Deserialize, Serialize, Deserializer};

fn deserialize_permissions<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("Expected string or number for permissions"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub discriminator: String,
    pub avatar: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub global_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    #[serde(deserialize_with = "deserialize_permissions")]
    pub permissions: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_optional_permissions")]
    pub permissions_new: Option<String>,
    // Additional optional fields that might be present
    #[serde(default)]
    pub banner: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub splash: Option<String>,
    #[serde(default)]
    pub discovery_splash: Option<String>,
    #[serde(default)]
    pub preferred_locale: Option<String>,
    #[serde(default)]
    pub approximate_member_count: Option<u64>,
    #[serde(default)]
    pub approximate_presence_count: Option<u64>,
}

fn deserialize_optional_permissions<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s)),
        Value::Number(n) => Ok(Some(n.to_string())),
        _ => Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user: DiscordUser,
    pub guilds: Vec<Guild>,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCallback {
    #[allow(dead_code)]
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}

impl SessionUser {
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
    
    pub fn get_manageable_guilds(&self) -> Vec<&Guild> {
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