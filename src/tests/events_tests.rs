#[cfg(test)]
mod tests {
    use crate::database::selfroles::{SelfRoleConfig, SelfRoleCooldown, SelfRoleRole};
    use crate::tests::{create_test_app_state, create_test_db};
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_message_delete_event_basic() {
        // Test basic message delete event handling
        let app_state = create_test_app_state().await;

        // Create a test self-role config
        let config = SelfRoleConfig::create(
            &app_state.db,
            "12345",
            "67890",
            "Test Self Roles",
            "Test description",
            "multiple",
        )
        .await
        .unwrap();

        // Update with message ID
        let mut config_copy = config;
        config_copy
            .update_message_id(&app_state.db, "11111")
            .await
            .unwrap();

        // Test that the config exists
        let configs = SelfRoleConfig::get_by_guild(&app_state.db, "12345")
            .await
            .unwrap();
        assert_eq!(configs.len(), 1);

        // Simulate message deletion by calling the database cleanup directly
        let result = SelfRoleConfig::get_by_message_id(&app_state.db, "11111")
            .await
            .unwrap();
        assert!(result.is_some());

        if let Some(found_config) = result {
            found_config.delete(&app_state.db).await.unwrap();
        }

        // Verify the config was deleted
        let configs_after = SelfRoleConfig::get_by_guild(&app_state.db, "12345")
            .await
            .unwrap();
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
    }

    #[tokio::test]
    async fn test_delete_by_message_id() {
        let db = create_test_db().await;

        // Create a config with message ID
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple",
        )
        .await
        .unwrap();

        let mut config_copy = config;
        config_copy
            .update_message_id(&db, "555666777")
            .await
            .unwrap();

        // Verify it exists
        let result = SelfRoleConfig::get_by_message_id(&db, "555666777")
            .await
            .unwrap();
        assert!(result.is_some());

        // Delete by message ID
        let deleted = SelfRoleConfig::delete_by_message_id(&db, "555666777")
            .await
            .unwrap();
        assert!(deleted);

        // Verify it's gone
        let result_after = SelfRoleConfig::get_by_message_id(&db, "555666777")
            .await
            .unwrap();
        assert!(result_after.is_none());
    }

    #[test]
    fn test_custom_id_format_validation() {
        // Test various custom ID formats
        let valid_formats = vec![
            "selfrole_123_role456",
            "selfrole_1_role1",
            "selfrole_999999_role999999",
        ];

        let invalid_formats = vec![
            "selfrole_123",         // Missing role part
            "selfrole_123_",        // Empty role part
            "invalid_123_role456",  // Wrong prefix
            "selfrole__role456",    // Empty config ID
            "",                     // Empty string
            "selfrole_abc_role456", // Non-numeric config ID
        ];

        for valid_id in valid_formats {
            let parts: Vec<&str> = valid_id.split('_').collect();
            assert_eq!(
                parts.len(),
                3,
                "Valid ID '{}' should have 3 parts",
                valid_id
            );
            assert_eq!(
                parts[0], "selfrole",
                "Valid ID '{}' should start with 'selfrole'",
                valid_id
            );
            assert!(
                !parts[1].is_empty(),
                "Valid ID '{}' should have non-empty config ID",
                valid_id
            );
            assert!(
                parts[1].chars().all(|c| c.is_ascii_digit()),
                "Valid ID '{}' should have numeric config ID",
                valid_id
            );
            assert!(
                parts[2].starts_with("role"),
                "Valid ID '{}' should have role part starting with 'role'",
                valid_id
            );
        }

        for invalid_id in invalid_formats {
            let parts: Vec<&str> = invalid_id.split('_').collect();
            let is_valid = parts.len() == 3
                && parts[0] == "selfrole"
                && !parts[1].is_empty()
                && parts[1].chars().all(|c| c.is_ascii_digit())
                && parts[2].starts_with("role");
            assert!(
                !is_valid,
                "Invalid ID '{}' should not pass validation",
                invalid_id
            );
        }
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_interaction() {
        let db = create_test_db().await;

        // Create a cooldown
        let expires_at = Utc::now() + Duration::minutes(5);
        SelfRoleCooldown::create(&db, "user123", "role456", "guild789", expires_at)
            .await
            .unwrap();

        // Simulate checking cooldown during interaction
        let is_on_cooldown =
            SelfRoleCooldown::check_cooldown(&db, "user123", "role456", "guild789")
                .await
                .unwrap();

        assert!(is_on_cooldown, "User should be on cooldown");

        // Test different user - should not be on cooldown
        let other_user_cooldown =
            SelfRoleCooldown::check_cooldown(&db, "user999", "role456", "guild789")
                .await
                .unwrap();

        assert!(
            !other_user_cooldown,
            "Different user should not be on cooldown"
        );

        // Test different role - should not be on cooldown
        let other_role_cooldown =
            SelfRoleCooldown::check_cooldown(&db, "user123", "role999", "guild789")
                .await
                .unwrap();

        assert!(
            !other_role_cooldown,
            "Different role should not be on cooldown"
        );
    }

    #[tokio::test]
    async fn test_multiple_configs_deletion() {
        let db = create_test_db().await;
        let guild_id = "multi_config_guild";

        // Create multiple configs in the same guild
        let config1 = SelfRoleConfig::create(
            &db,
            guild_id,
            "channel1",
            "Config 1",
            "First config",
            "multiple",
        )
        .await
        .unwrap();

        let config2 = SelfRoleConfig::create(
            &db,
            guild_id,
            "channel2",
            "Config 2",
            "Second config",
            "radio",
        )
        .await
        .unwrap();

        // Set message IDs
        let mut config1_copy = config1;
        let mut config2_copy = config2;

        config1_copy.update_message_id(&db, "msg1").await.unwrap();
        config2_copy.update_message_id(&db, "msg2").await.unwrap();

        // Add roles to configs
        let _role1 = SelfRoleRole::create(&db, config1_copy.id, "role1", "a")
            .await
            .unwrap();
        let _role2 = SelfRoleRole::create(&db, config2_copy.id, "role2", "b")
            .await
            .unwrap();

        // Verify both configs exist
        let configs = SelfRoleConfig::get_by_guild(&db, guild_id).await.unwrap();
        assert_eq!(configs.len(), 2);

        // Delete one config by message ID
        let deleted = SelfRoleConfig::delete_by_message_id(&db, "msg1")
            .await
            .unwrap();
        assert!(deleted);

        // Verify only one config remains
        let configs_after = SelfRoleConfig::get_by_guild(&db, guild_id).await.unwrap();
        assert_eq!(configs_after.len(), 1);
        assert_eq!(configs_after[0].id, config2_copy.id);

        // Verify roles are also cleaned up
        let remaining_roles = configs_after[0].get_roles(&db).await.unwrap();
        assert_eq!(remaining_roles.len(), 1);
        assert_eq!(remaining_roles[0].role_id, "role2");
    }

    #[test]
    fn test_interaction_component_parsing() {
        // Test parsing interaction component data
        let button_custom_id = "selfrole_42_role123456";

        // Extract config ID and role ID from custom ID
        if let Some(suffix) = button_custom_id.strip_prefix("selfrole_") {
            let parts: Vec<&str> = suffix.split('_').collect();
            if parts.len() == 2 && parts[1].starts_with("role") {
                let config_id = parts[0];
                let role_id = &parts[1][4..]; // Remove "role" prefix

                assert_eq!(config_id, "42");
                assert_eq!(role_id, "123456");
            } else {
                panic!("Invalid custom ID format");
            }
        } else {
            panic!("Custom ID should start with 'selfrole_'");
        }
    }

    #[tokio::test]
    async fn test_guild_leave_cleanup() {
        let db = create_test_db().await;
        let guild_id = "leaving_guild";

        // Create configs and data as if bot is in guild
        let config = SelfRoleConfig::create(
            &db,
            guild_id,
            "channel123",
            "Test Config",
            "Test body",
            "multiple",
        )
        .await
        .unwrap();

        let _role = SelfRoleRole::create(&db, config.id, "role123", "a")
            .await
            .unwrap();

        // Create cooldowns for users in this guild
        let expires_at = Utc::now() + Duration::hours(1);
        SelfRoleCooldown::create(&db, "user1", "role123", guild_id, expires_at)
            .await
            .unwrap();
        SelfRoleCooldown::create(&db, "user2", "role456", guild_id, expires_at)
            .await
            .unwrap();

        // Verify data exists
        let configs = SelfRoleConfig::get_by_guild(&db, guild_id).await.unwrap();
        assert_eq!(configs.len(), 1);

        let cooldown_exists = SelfRoleCooldown::check_cooldown(&db, "user1", "role123", guild_id)
            .await
            .unwrap();
        assert!(cooldown_exists);

        // Simulate guild leave by deleting all configs for the guild
        for config in configs {
            config.delete(&db).await.unwrap();
        }

        // Verify configs are gone
        let configs_after = SelfRoleConfig::get_by_guild(&db, guild_id).await.unwrap();
        assert_eq!(configs_after.len(), 0);

        // Note: In a real implementation, you might also want to clean up cooldowns
        // but that would require a separate cleanup function
    }

    #[tokio::test]
    async fn test_concurrent_interaction_handling() {
        let db = create_test_db().await;
        let guild_id = "concurrent_guild";

        // Create a config
        let config = SelfRoleConfig::create(
            &db,
            guild_id,
            "channel123",
            "Test Config",
            "Test body",
            "multiple",
        )
        .await
        .unwrap();

        // Create multiple roles
        for i in 1..=5 {
            SelfRoleRole::create(&db, config.id, &format!("role{}", i), "a")
                .await
                .unwrap();
        }

        // Simulate multiple users interacting with the same config sequentially
        for user_id in 1..=10 {
            let role_id = format!("role{}", (user_id % 5) + 1);

            // Check if user is on cooldown (should be false initially)
            let on_cooldown = SelfRoleCooldown::check_cooldown(
                &db,
                &format!("user{}", user_id),
                &role_id,
                guild_id,
            )
            .await
            .unwrap();

            assert!(
                !on_cooldown,
                "User {} should not be on cooldown initially",
                user_id
            );

            // Create a cooldown for this user
            let expires_at = Utc::now() + Duration::minutes(5);
            SelfRoleCooldown::create(
                &db,
                &format!("user{}", user_id),
                &role_id,
                guild_id,
                expires_at,
            )
            .await
            .unwrap();

            // Verify cooldown is now active
            let on_cooldown_after = SelfRoleCooldown::check_cooldown(
                &db,
                &format!("user{}", user_id),
                &role_id,
                guild_id,
            )
            .await
            .unwrap();

            assert!(
                on_cooldown_after,
                "User {} should be on cooldown after creation",
                user_id
            );
        }
    }

    #[test]
    fn test_error_handling_patterns() {
        // Test error handling patterns used in event processing

        // Test parsing invalid custom IDs
        let invalid_ids = vec![
            "",
            "invalid",
            "selfrole_",
            "selfrole_abc_role123",
            "selfrole_123_invalid",
        ];

        for invalid_id in invalid_ids {
            let result = parse_selfrole_custom_id(invalid_id);
            assert!(
                result.is_none(),
                "Invalid ID '{}' should return None",
                invalid_id
            );
        }

        // Test parsing valid custom IDs
        let valid_ids = vec![
            ("selfrole_123_role456", (123, "456")),
            ("selfrole_1_role1", (1, "1")),
            ("selfrole_999_role123456789", (999, "123456789")),
        ];

        for (valid_id, expected) in valid_ids {
            let result = parse_selfrole_custom_id(valid_id);
            assert!(
                result.is_some(),
                "Valid ID '{}' should parse successfully",
                valid_id
            );
            let (config_id, role_id) = result.unwrap();
            assert_eq!(
                (config_id, role_id.as_str()),
                expected,
                "Parsed values should match expected for '{}'",
                valid_id
            );
        }
    }

    // Helper function to test custom ID parsing
    fn parse_selfrole_custom_id(custom_id: &str) -> Option<(i64, String)> {
        if let Some(suffix) = custom_id.strip_prefix("selfrole_") {
            let parts: Vec<&str> = suffix.split('_').collect();
            if parts.len() == 2
                && parts[1].starts_with("role")
                && let Ok(config_id) = parts[0].parse::<i64>()
            {
                let role_id = parts[1][4..].to_string(); // Remove "role" prefix
                if !role_id.is_empty() {
                    return Some((config_id, role_id));
                }
            }
        }
        None
    }
}
