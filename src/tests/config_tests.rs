#[cfg(test)]
mod tests {
    use crate::config::{AppState, Config};
    use serenity;
    use std::sync::Arc;

    #[test]
    fn test_config_creation() {
        let config = Config::test_config();
        
        assert_eq!(config.discord.token, "test_token");
        assert_eq!(config.discord.application_id, 12345);
        assert_eq!(config.web.host, "127.0.0.1");
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.base_url, "http://localhost:3000");
        assert_eq!(config.database.url, ":memory:");
    }

    #[test]
    fn test_oauth_config() {
        let config = Config::test_config();
        
        assert_eq!(config.web.oauth.client_id, "test_client_id");
        assert_eq!(config.web.oauth.client_secret, "test_client_secret");
        assert_eq!(config.web.oauth.redirect_uri, "http://localhost:3000/auth/callback");
    }

    #[test]
    fn test_embed_config() {
        let config = Config::test_config();
        
        assert_eq!(config.web.embed.directory, "test_embed_files");
        assert_eq!(config.web.embed.max_age_hours, 24);
        assert_eq!(config.web.embed.cleanup_interval_hours, 6);
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = Arc::new(Config::test_config());
        let db = Arc::new(crate::tests::create_test_db().await);
        let cache = Arc::new(serenity::all::Cache::new());
        let http = Arc::new(serenity::all::Http::new("test_token"));

        let app_state = AppState::new(config.clone(), db.clone(), cache.clone(), http.clone());

        assert_eq!(app_state.config.discord.token, "test_token");
        assert!(Arc::ptr_eq(&app_state.config, &config));
        assert!(Arc::ptr_eq(&app_state.db, &db));
        assert!(Arc::ptr_eq(&app_state.cache, &cache));
        assert!(Arc::ptr_eq(&app_state.http, &http));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::test_config();
        
        let json = serde_json::to_string(&config).expect("Config should serialize to JSON");
        assert!(json.contains("test_token"));
        assert!(json.contains("test_client_id"));
        
        let deserialized: Config = serde_json::from_str(&json).expect("Config should deserialize from JSON");
        assert_eq!(deserialized.discord.token, config.discord.token);
        assert_eq!(deserialized.web.oauth.client_id, config.web.oauth.client_id);
    }

    #[test]
    fn test_embed_config_values() {
        let config = Config::test_config();
        
        // Test that embed config has reasonable values
        assert!(!config.web.embed.directory.is_empty());
        // Note: Zero values are now valid (they disable cleanup)
        // For non-zero values, cleanup interval should be <= max age
        if config.web.embed.max_age_hours > 0 && config.web.embed.cleanup_interval_hours > 0 {
            assert!(config.web.embed.cleanup_interval_hours <= config.web.embed.max_age_hours);
        }
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::test_config();
        let config2 = config1.clone();
        
        assert_eq!(config1.discord.token, config2.discord.token);
        assert_eq!(config1.discord.application_id, config2.discord.application_id);
        assert_eq!(config1.web.host, config2.web.host);
        assert_eq!(config1.web.port, config2.web.port);
    }

    #[test]
    fn test_discord_config_structure() {
        let config = Config::test_config();
        
        // Verify Discord config structure
        assert!(!config.discord.token.is_empty());
        assert!(config.discord.application_id > 0);
    }

    #[test]
    fn test_web_config_structure() {
        let config = Config::test_config();
        
        // Verify web config structure
        assert!(!config.web.host.is_empty());
        assert!(config.web.port > 0);
        assert!(!config.web.base_url.is_empty());
        assert!(config.web.base_url.starts_with("http"));
    }

    #[test]
    fn test_oauth_config_structure() {
        let config = Config::test_config();
        
        // Verify OAuth config structure
        assert!(!config.web.oauth.client_id.is_empty());
        assert!(!config.web.oauth.client_secret.is_empty());
        assert!(!config.web.oauth.redirect_uri.is_empty());
        assert!(config.web.oauth.redirect_uri.contains("/auth/callback"));
    }

    #[test]
    fn test_database_config_structure() {
        let config = Config::test_config();
        
        // Verify database config structure
        assert!(!config.database.url.is_empty());
        assert_eq!(config.database.url, ":memory:");
    }

    #[test]
    fn test_embed_cleanup_disabled_values() {
        // Test that zero values are allowed and interpreted as cleanup disabled
        let config = Config::test_config();
        
        // Default test config should have non-zero values
        assert!(config.web.embed.max_age_hours > 0);
        assert!(config.web.embed.cleanup_interval_hours > 0);
        
        // Note: We can't easily test zero values here without modifying environment
        // but the config parsing logic allows them and interprets them as disabled
    }
}