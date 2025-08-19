#[cfg(test)]
mod tests {
    use crate::config::{Config, AppState};
    use std::env;
    use std::sync::Arc;
    use serenity;

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

    #[test]
    fn test_config_from_env_missing_required() {
        // Clear ALL environment variables to test error handling
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");

        // Test missing DISCORD_TOKEN
        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DISCORD_TOKEN"));
    }

    #[test]
    fn test_config_from_env_with_custom_values() {
        // Clean up any existing vars first
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");
        
        // Set custom environment variables
        env::set_var("DISCORD_TOKEN", "custom_token");
        env::set_var("DISCORD_APPLICATION_ID", "98765");
        env::set_var("DISCORD_CLIENT_ID", "custom_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "custom_client_secret");
        env::set_var("BASE_URL", "https://example.com");
        env::set_var("HOST", "0.0.0.0");
        env::set_var("PORT", "8080");

        let config = Config::from_env().expect("Config should be created successfully");

        assert_eq!(config.discord.token, "custom_token");
        assert_eq!(config.discord.application_id, 98765);
        assert_eq!(config.web.oauth.client_id, "custom_client_id");
        assert_eq!(config.web.oauth.client_secret, "custom_client_secret");
        assert_eq!(config.web.base_url, "https://example.com");
        assert_eq!(config.web.host, "0.0.0.0");
        assert_eq!(config.web.port, 8080);
        assert_eq!(config.web.oauth.redirect_uri, "https://example.com/auth/callback");

        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");
    }

    #[test]
    fn test_config_from_env_defaults() {
        // Clean up any existing vars first
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");
        
        // Set required variables only
        env::set_var("DISCORD_TOKEN", "test_token");
        env::set_var("DISCORD_APPLICATION_ID", "12345");
        env::set_var("DISCORD_CLIENT_ID", "test_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "test_client_secret");
        
        // Remove optional variables to test defaults
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");

        let config = Config::from_env().expect("Config should be created with defaults");

        assert_eq!(config.web.base_url, "http://localhost:3000");
        assert_eq!(config.web.host, "127.0.0.1");
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.oauth.redirect_uri, "http://localhost:3000/auth/callback");
        assert_eq!(config.database.url, "data/db.sqlite");

        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
    }

    #[test]
    fn test_config_invalid_application_id() {
        env::set_var("DISCORD_TOKEN", "test_token");
        env::set_var("DISCORD_APPLICATION_ID", "not_a_number");
        env::set_var("DISCORD_CLIENT_ID", "test_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "test_client_secret");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid DISCORD_APPLICATION_ID format"));

        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
    }

    #[test]
    fn test_config_invalid_port() {
        env::set_var("DISCORD_TOKEN", "test_token");
        env::set_var("DISCORD_APPLICATION_ID", "12345");
        env::set_var("DISCORD_CLIENT_ID", "test_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "test_client_secret");
        env::set_var("PORT", "not_a_number");

        let config = Config::from_env().expect("Config should use default port for invalid PORT");
        assert_eq!(config.web.port, 3000); // Should default to 3000

        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        env::remove_var("PORT");
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
        
        // Test that config can be serialized to JSON
        let json = serde_json::to_string(&config).expect("Config should serialize to JSON");
        assert!(json.contains("test_token"));
        assert!(json.contains("test_client_id"));
        
        // Test that config can be deserialized from JSON
        let deserialized: Config = serde_json::from_str(&json).expect("Config should deserialize from JSON");
        assert_eq!(deserialized.discord.token, config.discord.token);
        assert_eq!(deserialized.web.oauth.client_id, config.web.oauth.client_id);
    }

    #[test]
    fn test_embed_config_values() {
        let config = Config::test_config();
        
        // Test that embed config has reasonable values
        assert!(!config.web.embed.directory.is_empty());
        assert!(config.web.embed.max_age_hours > 0);
        assert!(config.web.embed.cleanup_interval_hours > 0);
        assert!(config.web.embed.cleanup_interval_hours <= config.web.embed.max_age_hours);
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
}