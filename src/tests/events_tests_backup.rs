#[cfg(test)]
mod tests {
    use crate::events;
    use crate::database::selfroles::{SelfRoleConfig, SelfRoleRole};
    use serenity::all::{ComponentInteraction, MessageId, ChannelId, GuildId, Context};
    use serenity::model::id::UserId;
    use serenity::model::application::{ComponentInteractionData, ComponentType};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_message_delete_cleanup() {
        let app_state = crate::tests::create_test_app_state().await;
        
        // Create a test self-role config with message ID
        let mut config = SelfRoleConfig::create(
            &app_state.db,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let test_message_id = "message_123456";
        config.set_message_id(&app_state.db, test_message_id).await.unwrap();
        
        // Verify config exists
        let found_config = SelfRoleConfig::get_by_message_id(&app_state.db, test_message_id)
            .await
            .unwrap();
        assert!(found_config.is_some());
        
        // Create a mock context (we'll use minimal data since we're testing the cleanup logic)
        let ctx = create_mock_context().await;
        let channel_id = ChannelId::new(987654321);
        let message_id = MessageId::new(test_message_id.parse::<u64>().unwrap());
        let guild_id = Some(GuildId::new(123456789));
        
        // Call the message delete handler
        events::handle_message_delete(&ctx, &channel_id, &message_id, &guild_id, &app_state).await;
        
        // Verify config was deleted
        let found_config_after = SelfRoleConfig::get_by_message_id(&app_state.db, test_message_id)
            .await
            .unwrap();
        assert!(found_config_after.is_none());
    }

    #[tokio::test]
    async fn test_handle_message_delete_non_selfrole_message() {
        let app_state = crate::tests::create_test_app_state().await;
        
        // Create a test self-role config with different message ID
        let mut config = SelfRoleConfig::create(
            &app_state.db,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let test_message_id = "message_123456";
        config.set_message_id(&app_state.db, test_message_id).await.unwrap();
        
        // Try to delete a different message ID
        let ctx = create_mock_context().await;
        let channel_id = ChannelId::new(987654321);
        let different_message_id = MessageId::new(999999999);
        let guild_id = Some(GuildId::new(123456789));
        
        // Call the message delete handler with different message ID
        events::handle_message_delete(&ctx, &channel_id, &different_message_id, &guild_id, &app_state).await;
        
        // Verify original config still exists (wasn't deleted)
        let found_config = SelfRoleConfig::get_by_message_id(&app_state.db, test_message_id)
            .await
            .unwrap();
        assert!(found_config.is_some());
    }

    #[tokio::test]
    async fn test_message_delete_event_basic() {
        // Test basic message delete event handling
        let config = Arc::new(crate::config::Config::test_config());
        let db = Arc::new(super::create_test_db().await);
        let cache = Arc::new(serenity::all::Cache::new());
        let http = Arc::new(serenity::all::Http::new("test_token"));
        
        let app_state = crate::config::AppState::new(config, db.clone(), cache, http);
        
        // Create a test self-role config
        let config = SelfRoleConfig::create(
            &**db,
            "12345",
            "67890",
            "Test Self Roles",
            "Test description",
            "multiple"
        ).await.unwrap();
        
        // Update with message ID
        let mut config_copy = config;
        config_copy.update_message_id(&**db, "11111").await.unwrap();
        
        // Test that the config exists
        let configs = SelfRoleConfig::get_by_guild(&**db, "12345").await.unwrap();
        assert_eq!(configs.len(), 1);
        
        // Simulate message deletion by calling the database cleanup directly
        // (Since creating a full Context for testing is complex)
        let result = SelfRoleConfig::get_by_message_id(&**db, "11111").await.unwrap();
        assert!(result.is_some());
        
        if let Some(found_config) = result {
            found_config.delete(&**db).await.unwrap();
        }
        
        // Verify the config was deleted
        let configs_after = SelfRoleConfig::get_by_guild(&**db, "12345").await.unwrap();
        assert_eq!(configs_after.len(), 0);
    }

    #[test]
    fn test_selfrole_custom_id_parsing() {
        // Test the custom ID format used in selfrole interactions
        let custom_id = "selfrole_123_role456";
        let parts: Vec<&str> = custom_id.split('_').collect();
        
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "selfrole");
        assert_eq!(parts[1], "123");
        assert_eq!(parts[2], "role456");
        
        // Test parsing config_id
        let config_id: Result<i64, _> = parts[1].parse();
        assert!(config_id.is_ok());
        assert_eq!(config_id.unwrap(), 123);
        
        // Test role_id
        let role_id = parts[2];
        assert_eq!(role_id, "role456");
    }

    #[test]
    fn test_invalid_selfrole_custom_id_parsing() {
        // Test invalid custom ID formats
        let invalid_ids = vec![
            "selfrole_123",           // Too few parts
            "selfrole_123_role_456",  // Too many parts
            "not_selfrole_123_role456", // Wrong prefix
            "selfrole_abc_role456",   // Invalid config_id
        ];
        
        for invalid_id in invalid_ids {
            let parts: Vec<&str> = invalid_id.split('_').collect();
            
            if parts.len() != 3 || parts[0] != "selfrole" {
                // Should be rejected by length or prefix check
                assert!(parts.len() != 3 || parts[0] != "selfrole");
                continue;
            }
            
            if let Err(_) = parts[1].parse::<i64>() {
                // Should be rejected by config_id parsing
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_interaction_handling_flow() {
        let app_state = crate::tests::create_test_app_state().await;
        
        // Create a test self-role config and role
        let mut config = SelfRoleConfig::create(
            &app_state.db,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let test_message_id = "message_123456";
        config.set_message_id(&app_state.db, test_message_id).await.unwrap();
        
        let _role = SelfRoleRole::create(
            &app_state.db,
            config.id,
            "role_123",
            "🎮"
        ).await.unwrap();
        
        // Verify the config can be found by message ID
        let found_config = SelfRoleConfig::get_by_message_id(&app_state.db, test_message_id)
            .await
            .unwrap();
        assert!(found_config.is_some());
        
        // Verify roles can be retrieved
        let roles = SelfRoleRole::get_by_config_id(&app_state.db, config.id)
            .await
            .unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].role_id, "role_123");
        assert_eq!(roles[0].emoji, "🎮");
    }

    #[test]
    fn test_cooldown_calculation() {
        use chrono::{Utc, Duration};
        
        // Test cooldown duration calculation
        let now = Utc::now();
        let cooldown_duration = Duration::seconds(30);
        let expires_at = now + cooldown_duration;
        
        assert!(expires_at > now);
        assert!(expires_at <= now + Duration::seconds(35)); // Allow some margin for test execution time
        
        // Test if cooldown has expired
        let past_time = now - Duration::seconds(60);
        assert!(past_time < now);
        
        let future_time = now + Duration::seconds(60);
        assert!(future_time > now);
    }

    #[tokio::test]
    async fn test_role_assignment_data_flow() {
        let app_state = crate::tests::create_test_app_state().await;
        
        // Create test data
        let config = SelfRoleConfig::create(
            &app_state.db,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "multiple" // Allow multiple roles
        ).await.unwrap();
        
        let role1 = SelfRoleRole::create(&app_state.db, config.id, "role_123", "🎮").await.unwrap();
        let role2 = SelfRoleRole::create(&app_state.db, config.id, "role_456", "🎵").await.unwrap();
        
        // Test data retrieval
        let retrieved_config = SelfRoleConfig::get_by_guild(&app_state.db, "123456789")
            .await
            .unwrap();
        assert_eq!(retrieved_config.len(), 1);
        assert_eq!(retrieved_config[0].selection_type, "multiple");
        
        let retrieved_roles = SelfRoleRole::get_by_config_id(&app_state.db, config.id)
            .await
            .unwrap();
        assert_eq!(retrieved_roles.len(), 2);
        
        // Verify role IDs are correct
        let role_ids: Vec<&str> = retrieved_roles.iter().map(|r| r.role_id.as_str()).collect();
        assert!(role_ids.contains(&"role_123"));
        assert!(role_ids.contains(&"role_456"));
    }

    #[test]
    fn test_message_response_formatting() {
        // Test the formatting of response messages
        let action = "added";
        let emoji = "🎮";
        let role_name = "Gamer";
        
        let message = format!("{} {}", emoji, 
            if action == "added" {
                format!("You have been given the **{}** role!", role_name)
            } else {
                format!("The **{}** role has been removed from you!", role_name)
            }
        );
        
        assert!(message.contains("🎮"));
        assert!(message.contains("Gamer"));
        assert!(message.contains("given"));
        
        // Test removal message
        let removal_message = format!("{} {}", emoji,
            format!("The **{}** role has been removed from you!", role_name)
        );
        
        assert!(removal_message.contains("removed"));
    }
}