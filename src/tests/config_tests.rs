#[cfg(test)]
mod tests {
    use crate::config::Config;

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
}