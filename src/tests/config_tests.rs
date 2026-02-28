#[cfg(test)]
mod tests {
    use clouder_core::config::{AppState, Config};
    use serenity;
    use std::sync::Arc;

    #[test]
    fn test_config_creation() {
        let config = Config::test_config();

        assert_eq!(config.discord.token, "test_token");
        assert_eq!(config.discord.application_id, 12345);
        assert_eq!(config.web.api_url, "http://127.0.0.1:8080");
        assert_eq!(config.web.bind_addr, "127.0.0.1:8080");
        assert_eq!(config.database.url, ":memory:");
    }

    #[test]
    fn test_oauth_config() {
        let config = Config::test_config();

        assert_eq!(config.web.oauth.client_id, "12345");
        assert_eq!(config.web.oauth.client_secret, "test_client_secret");
        assert_eq!(
            config.web.oauth.redirect_uri,
            "http://127.0.0.1:8080/auth/callback"
        );
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = Arc::new(Config::test_config());
        let db = Arc::new(crate::tests::create_test_db().await);
        let http = Arc::new(serenity::all::Http::new("test_token"));

        let app_state = AppState::new(config.clone(), db.clone(), http.clone());

        assert_eq!(app_state.config.discord.token, "test_token");
        assert!(Arc::ptr_eq(&app_state.config, &config));
        assert!(Arc::ptr_eq(&app_state.db, &db));
        assert!(Arc::ptr_eq(&app_state.http, &http));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::test_config();

        let json = serde_json::to_string(&config).expect("Config should serialize to JSON");
        assert!(json.contains("test_token"));
        assert!(json.contains("12345"));

        let deserialized: Config =
            serde_json::from_str(&json).expect("Config should deserialize from JSON");
        assert_eq!(deserialized.discord.token, config.discord.token);
        assert_eq!(deserialized.web.oauth.client_id, config.web.oauth.client_id);
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::test_config();
        let config2 = config1.clone();

        assert_eq!(config1.discord.token, config2.discord.token);
        assert_eq!(
            config1.discord.application_id,
            config2.discord.application_id
        );
        assert_eq!(config1.web.api_url, config2.web.api_url);
    }

    #[test]
    fn test_discord_config_structure() {
        let config = Config::test_config();

        assert!(!config.discord.token.is_empty());
        assert!(config.discord.application_id > 0);
        // application_id is parsed from client_id
        assert_eq!(
            config.discord.application_id,
            config.web.oauth.client_id.parse::<u64>().unwrap()
        );
    }

    #[test]
    fn test_web_config_structure() {
        let config = Config::test_config();

        assert!(!config.web.api_url.is_empty());
        assert!(config.web.api_url.starts_with("http"));
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
}
