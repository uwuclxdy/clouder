#[cfg(test)]
mod tests {
    use crate::database::welcome_goodbye::{replace_placeholders, WelcomeGoodbyeConfig};
    use crate::utils::welcome_goodbye::{validate_message_config, validate_url};
    use serenity::{model::id::UserId, model::user::User};
    use sqlx::SqlitePool;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_placeholder_replacement() {
        let mut placeholders = HashMap::new();
        placeholders.insert("user".to_string(), "<@12345>".to_string());
        placeholders.insert("username".to_string(), "TestUser".to_string());
        placeholders.insert("server".to_string(), "Test Server".to_string());

        let content = "Welcome {user} to {server}! Your username is {username}.";
        let result = replace_placeholders(content, &placeholders);

        assert_eq!(
            result,
            "Welcome <@12345> to Test Server! Your username is TestUser."
        );
    }

    #[test]
    fn test_validate_message_config() {
        // Test embed with title
        let result =
            validate_message_config("embed", &None, &Some("Test Title".to_string()), &None);
        assert!(result.is_ok());

        // Test embed with description
        let result =
            validate_message_config("embed", &None, &None, &Some("Test Description".to_string()));
        assert!(result.is_ok());

        // Test embed with both
        let result = validate_message_config(
            "embed",
            &None,
            &Some("Test Title".to_string()),
            &Some("Test Description".to_string()),
        );
        assert!(result.is_ok());

        // Test embed with neither
        let result = validate_message_config("embed", &None, &None, &None);
        assert!(result.is_err());

        // Test text with content
        let result =
            validate_message_config("text", &Some("Test Content".to_string()), &None, &None);
        assert!(result.is_ok());

        // Test text without content
        let result = validate_message_config("text", &None, &None, &None);
        assert!(result.is_err());

        // Test invalid message type
        let result =
            validate_message_config("invalid", &Some("Test Content".to_string()), &None, &None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://example.com"));
        assert!(validate_url("https://example.com/path/to/image.png"));
        assert!(!validate_url("not-a-url"));
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url(""));
    }

    #[tokio::test]
    async fn test_welcome_goodbye_config_crud() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Run migrations
        let migrations = [
            include_str!("../../migrations/001_initial.sql"),
            include_str!("../../migrations/002_reminders.sql"),
            include_str!("../../migrations/003_welcome_goodbye.sql"),
        ];

        for migration_content in migrations.iter() {
            for statement in migration_content.split(';') {
                let statement = statement.trim();
                if !statement.is_empty() {
                    sqlx::query(statement).execute(&pool).await.unwrap();
                }
            }
        }

        let guild_id = "123456789";

        // Test getting non-existent config
        let config = WelcomeGoodbyeConfig::get_config(&pool, guild_id)
            .await
            .unwrap();
        assert!(config.is_none());

        // Test creating config
        let mut config = WelcomeGoodbyeConfig::default();
        config.guild_id = guild_id.to_string();
        config.welcome_enabled = true;
        config.welcome_channel_id = Some("987654321".to_string());
        config.welcome_message_type = "embed".to_string();
        config.welcome_embed_title = Some("Welcome!".to_string());

        WelcomeGoodbyeConfig::upsert_config(&pool, &config)
            .await
            .unwrap();

        // Test getting config
        let retrieved_config = WelcomeGoodbyeConfig::get_config(&pool, guild_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_config.guild_id, guild_id);
        assert!(retrieved_config.welcome_enabled);
        assert_eq!(
            retrieved_config.welcome_channel_id,
            Some("987654321".to_string())
        );
        assert_eq!(
            retrieved_config.welcome_embed_title,
            Some("Welcome!".to_string())
        );

        // Test updating config
        let mut updated_config = retrieved_config.clone();
        updated_config.goodbye_enabled = true;
        updated_config.goodbye_channel_id = Some("555555555".to_string());

        WelcomeGoodbyeConfig::upsert_config(&pool, &updated_config)
            .await
            .unwrap();

        let final_config = WelcomeGoodbyeConfig::get_config(&pool, guild_id)
            .await
            .unwrap()
            .unwrap();
        assert!(final_config.goodbye_enabled);
        assert_eq!(
            final_config.goodbye_channel_id,
            Some("555555555".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_member_placeholders() {
        // Create a mock user
        let mut user = User::default();
        user.id = UserId::new(123456789);
        user.name = "TestUser".to_string();

        // Create a mock guild
        let mut guild = serenity::model::guild::Guild::default();
        guild.name = "Test Server".to_string();
        guild.member_count = 42;
    }

    #[tokio::test]
    async fn test_database_config_with_embed_fields() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Run migrations
        let migrations = [
            include_str!("../../migrations/001_initial.sql"),
            include_str!("../../migrations/002_reminders.sql"),
            include_str!("../../migrations/003_welcome_goodbye.sql"),
        ];

        for migration_content in migrations.iter() {
            for statement in migration_content.split(';') {
                let statement = statement.trim();
                if !statement.is_empty() {
                    sqlx::query(statement).execute(&pool).await.unwrap();
                }
            }
        }

        let guild_id = "987654321";

        let mut config = WelcomeGoodbyeConfig::default();
        config.guild_id = guild_id.to_string();
        config.welcome_enabled = true;
        config.welcome_embed_title = Some("Welcome Title".to_string());
        config.welcome_embed_description = Some("Welcome Description".to_string());
        config.welcome_embed_color = Some(0x5865F2);
        config.welcome_embed_footer = Some("Footer Text".to_string());
        config.welcome_embed_thumbnail = Some("https://example.com/thumb.png".to_string());
        config.welcome_embed_image = Some("https://example.com/image.png".to_string());
        config.welcome_embed_timestamp = true;

        WelcomeGoodbyeConfig::upsert_config(&pool, &config)
            .await
            .unwrap();

        let retrieved = WelcomeGoodbyeConfig::get_config(&pool, guild_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            retrieved.welcome_embed_title,
            Some("Welcome Title".to_string())
        );
        assert_eq!(
            retrieved.welcome_embed_description,
            Some("Welcome Description".to_string())
        );
        assert_eq!(retrieved.welcome_embed_color, Some(0x5865F2));
        assert_eq!(
            retrieved.welcome_embed_footer,
            Some("Footer Text".to_string())
        );
        assert_eq!(
            retrieved.welcome_embed_thumbnail,
            Some("https://example.com/thumb.png".to_string())
        );
        assert_eq!(
            retrieved.welcome_embed_image,
            Some("https://example.com/image.png".to_string())
        );
        assert!(retrieved.welcome_embed_timestamp);
    }

    #[test]
    fn test_config_default_values() {
        let config = WelcomeGoodbyeConfig::default();

        assert!(!config.welcome_enabled);
        assert!(!config.goodbye_enabled);
        assert_eq!(config.welcome_message_type, "embed");
        assert_eq!(config.goodbye_message_type, "embed");
        assert!(!config.welcome_embed_timestamp);
        assert!(!config.goodbye_embed_timestamp);
    }

    #[test]
    fn test_edge_case_placeholder_replacement() {
        let mut placeholders = HashMap::new();
        placeholders.insert("user".to_string(), "<@12345>".to_string());

        // Test empty content
        let result = replace_placeholders("", &placeholders);
        assert_eq!(result, "");

        // Test content with no placeholders
        let result = replace_placeholders("No placeholders here", &placeholders);
        assert_eq!(result, "No placeholders here");

        // Test content with unknown placeholder
        let result = replace_placeholders("Hello {unknown}", &placeholders);
        assert_eq!(result, "Hello {unknown}");

        // Test repeated placeholders
        let result = replace_placeholders("Hello {user}, welcome {user}!", &placeholders);
        assert_eq!(result, "Hello <@12345>, welcome <@12345>!");
    }
}
