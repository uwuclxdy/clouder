#[cfg(test)]
mod tests {
    use crate::config::{Config, AppState, DiscordConfig, WebConfig, DatabaseConfig, OAuthConfig, EmbedConfig};
    use std::sync::Arc;
    use std::env;

    #[test]
    fn test_config_creation() {
        let config = Config::test_config();
        
        assert_eq!(config.discord.token, "test_token");
        assert_eq!(config.discord.application_id, 12345);
        assert_eq!(config.web.host, "127.0.0.1");
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.base_url, "http://localhost:3000");
        assert_eq!(config.web.oauth.client_id, "test_client_id");
        assert_eq!(config.web.oauth.client_secret, "test_client_secret");
        assert_eq!(config.web.oauth.redirect_uri, "http://localhost:3000/auth/callback");
        assert_eq!(config.database.url, ":memory:");
    }

    #[test]
    fn test_config_from_env_missing_token() {
        // Clear any existing environment variables
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
        
        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DISCORD_TOKEN"));
    }

    #[test]
    fn test_config_from_env_with_all_vars() {
        // Set up environment variables
        env::set_var("DISCORD_TOKEN", "test_discord_token");
        env::set_var("DISCORD_APPLICATION_ID", "123456789");
        env::set_var("DISCORD_CLIENT_ID", "test_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "test_client_secret");
        env::set_var("BASE_URL", "https://example.com");
        env::set_var("HOST", "0.0.0.0");
        env::set_var("PORT", "8080");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.discord.token, "test_discord_token");
        assert_eq!(config.discord.application_id, 123456789);
        assert_eq!(config.web.oauth.client_id, "test_client_id");
        assert_eq!(config.web.oauth.client_secret, "test_client_secret");
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
    fn test_config_from_env_with_defaults() {
        // Set required variables only
        env::set_var("DISCORD_TOKEN", "test_discord_token");
        env::set_var("DISCORD_APPLICATION_ID", "123456789");
        env::set_var("DISCORD_CLIENT_ID", "test_client_id");
        env::set_var("DISCORD_CLIENT_SECRET", "test_client_secret");
        
        // Remove optional variables
        env::remove_var("BASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.web.base_url, "http://localhost:3000");
        assert_eq!(config.web.host, "127.0.0.1");
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.oauth.redirect_uri, "http://localhost:3000/auth/callback");
        
        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_APPLICATION_ID");
        env::remove_var("DISCORD_CLIENT_ID");
        env::remove_var("DISCORD_CLIENT_SECRET");
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
}