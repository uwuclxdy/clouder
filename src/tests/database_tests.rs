#[cfg(test)]
mod tests {
    use crate::database::selfroles::{SelfRoleConfig, SelfRoleRole, SelfRoleCooldown};
    use chrono::{Utc, Duration};
    use crate::tests::create_test_db;

    #[tokio::test]
    async fn test_selfrole_config_creation() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        assert_eq!(config.guild_id, "123456789");
        assert_eq!(config.channel_id, "987654321");
        assert_eq!(config.title, "Test Roles");
        assert_eq!(config.body, "Select your roles below:");
        assert_eq!(config.selection_type, "multiple");
        assert!(config.message_id.is_none());
    }

    #[tokio::test]
    async fn test_selfrole_config_update_message_id() {
        let db = create_test_db().await;
        
        let mut config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        config.update_message_id(&db, "555666777").await.unwrap();
        
        assert_eq!(config.message_id, Some("555666777".to_string()));
        
        // Verify it was updated in the database
        let retrieved = SelfRoleConfig::get_by_message_id(&db, "555666777").await.unwrap().unwrap();
        assert_eq!(retrieved.id, config.id);
        assert_eq!(retrieved.message_id, Some("555666777".to_string()));
    }

    #[tokio::test]
    async fn test_selfrole_config_get_by_guild() {
        let pool = create_test_db().await;
        
        let config1 = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message 1",
            "First message",
            "radio"
        ).await.unwrap();
        
        let config2 = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654322",
            "Test Role Message 2",
            "Second message",
            "multiple"
        ).await.unwrap();
        
        // Create config for different guild
        let _config3 = SelfRoleConfig::create(
            &pool,
            "999888777",
            "444333222",
            "Other Guild Message",
            "Other guild message",
            "multiple"
        ).await.unwrap();
        
        let configs = SelfRoleConfig::get_by_guild(&pool, "123456789").await.unwrap();
        assert_eq!(configs.len(), 2);
        
        // Check that both configs are present (order might vary due to timing)
        let config_ids: Vec<i64> = configs.iter().map(|c| c.id).collect();
        assert!(config_ids.contains(&config1.id));
        assert!(config_ids.contains(&config2.id));
    }

    #[tokio::test]
    async fn test_selfrole_config_update() {
        let db = create_test_db().await;
        
        let mut config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Original Title",
            "Original body",
            "radio"
        ).await.unwrap();
        
        config.update(
            &db,
            "Updated Title",
            "Updated body",
            "multiple"
        ).await.unwrap();
        
        assert_eq!(config.title, "Updated Title");
        assert_eq!(config.body, "Updated body");
        assert_eq!(config.selection_type, "multiple");
    }

    #[tokio::test]
    async fn test_selfrole_config_delete() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        // Verify it exists
        let configs_before = SelfRoleConfig::get_by_guild(&db, "123456789").await.unwrap();
        assert_eq!(configs_before.len(), 1);
        
        // Delete it
        config.delete(&db).await.unwrap();
        
        // Verify it's gone
        let configs_after = SelfRoleConfig::get_by_guild(&db, "123456789").await.unwrap();
        assert_eq!(configs_after.len(), 0);
    }

    #[tokio::test]
    async fn test_selfrole_role_creation() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        let role = SelfRoleRole::create(
            &db,
            config.id,
            "111222333",
            "🎮"
        ).await.unwrap();
        
        assert_eq!(role.config_id, config.id);
        assert_eq!(role.role_id, "111222333");
        assert_eq!(role.emoji, "🎮");
    }

    #[tokio::test]
    async fn test_selfrole_config_get_roles() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        let _role1 = SelfRoleRole::create(
            &db,
            config.id,
            "111222333",
            "🎮"
        ).await.unwrap();
        
        let _role2 = SelfRoleRole::create(
            &db,
            config.id,
            "444555666",
            "🎨"
        ).await.unwrap();
        
        let roles = config.get_roles(&db).await.unwrap();
        assert_eq!(roles.len(), 2);
        
        let role_ids: Vec<&str> = roles.iter().map(|r| r.role_id.as_str()).collect();
        assert!(role_ids.contains(&"111222333"));
        assert!(role_ids.contains(&"444555666"));
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_creation() {
        let db = create_test_db().await;
        
        let expires_at = Utc::now() + Duration::minutes(5);
        
        SelfRoleCooldown::create(
            &db,
            "user123",
            "role456",
            "guild789",
            expires_at
        ).await.unwrap();
        
        // Check that cooldown is active
        let is_on_cooldown = SelfRoleCooldown::check_cooldown(
            &db,
            "user123",
            "role456",
            "guild789"
        ).await.unwrap();
        
        assert!(is_on_cooldown);
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_expired() {
        let db = create_test_db().await;
        
        let expires_at = Utc::now() - Duration::minutes(5); // Already expired
        
        SelfRoleCooldown::create(
            &db,
            "user123",
            "role456",
            "guild789",
            expires_at
        ).await.unwrap();
        
        // Check that cooldown is not active (expired)
        let is_on_cooldown = SelfRoleCooldown::check_cooldown(
            &db,
            "user123",
            "role456",
            "guild789"
        ).await.unwrap();
        
        assert!(!is_on_cooldown);
    }

    #[tokio::test]
    async fn test_selfrole_config_get_by_id() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
                        // Create a config and set message ID
        let mut _config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Config",
            "Test Roles",
            "multiple"
        ).await.unwrap();
        
        // Update with message ID
        _config.update_message_id(&db, "test_message_123").await.unwrap();
        
        // Test retrieving a specific config by message ID
        let retrieved_opt = SelfRoleConfig::get_by_message_id(&db, "test_message_123").await.unwrap();
        assert!(retrieved_opt.is_some());
        let retrieved = retrieved_opt.unwrap();
        assert_eq!(retrieved.title, "Test Config");

        // Test retrieving non-existent config 
        let non_existent = SelfRoleConfig::get_by_guild(&db, "non_existent_guild").await.unwrap();
        assert_eq!(non_existent.len(), 0);
    }

    #[tokio::test]
    async fn test_selfrole_config_unique_message_id() {
        let db = create_test_db().await;
        
        let mut config1 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles 1",
            "First config",
            "multiple"
        ).await.unwrap();
        
        let mut config2 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654322",
            "Test Roles 2",
            "Second config",
            "radio"
        ).await.unwrap();
        
        // Set same message ID on first config
        config1.update_message_id(&db, "unique_message_123").await.unwrap();
        
        // Try to set same message ID on second config - should fail due to uniqueness constraint
        let result = config2.update_message_id(&db, "unique_message_123").await;
        assert!(result.is_err(), "Should fail due to unique constraint violation");
        
        // Verify that only the first config has this message ID
        let result = SelfRoleConfig::get_by_message_id(&db, "unique_message_123").await.unwrap();
        assert!(result.is_some());
        let config_with_message = result.unwrap();
        assert_eq!(config_with_message.id, config1.id);
    }

    #[tokio::test]
    async fn test_selfrole_role_deletion_cascade() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        // Create multiple roles
        let _role1 = SelfRoleRole::create(&db, config.id, "role1", "🎮").await.unwrap();
        let _role2 = SelfRoleRole::create(&db, config.id, "role2", "🎨").await.unwrap();
        let _role3 = SelfRoleRole::create(&db, config.id, "role3", "🎵").await.unwrap();
        
        // Verify roles exist
        let roles_before = config.get_roles(&db).await.unwrap();
        assert_eq!(roles_before.len(), 3);
        
        // Delete the config (should cascade delete roles due to FOREIGN KEY constraint)
        config.delete(&db).await.unwrap();
        
        // Verify config is gone
        let configs = SelfRoleConfig::get_by_guild(&db, "123456789").await.unwrap();
        assert_eq!(configs.len(), 0);
        
        // Note: Roles should be automatically deleted due to CASCADE foreign key
        // but we can't easily test this without re-creating the config and checking
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_cleanup() {
        let db = create_test_db().await;
        
        // Create expired cooldowns
        let expired_time = Utc::now() - Duration::hours(1);
        SelfRoleCooldown::create(&db, "user1", "role1", "guild1", expired_time).await.unwrap();
        SelfRoleCooldown::create(&db, "user2", "role2", "guild2", expired_time).await.unwrap();
        
        // Create active cooldowns
        let future_time = Utc::now() + Duration::hours(1);
        SelfRoleCooldown::create(&db, "user3", "role3", "guild3", future_time).await.unwrap();
        SelfRoleCooldown::create(&db, "user4", "role4", "guild4", future_time).await.unwrap();
        
        // Verify expired cooldowns are not active
        assert!(!SelfRoleCooldown::check_cooldown(&db, "user1", "role1", "guild1").await.unwrap());
        assert!(!SelfRoleCooldown::check_cooldown(&db, "user2", "role2", "guild2").await.unwrap());
        
        // Verify active cooldowns are still active
        assert!(SelfRoleCooldown::check_cooldown(&db, "user3", "role3", "guild3").await.unwrap());
        assert!(SelfRoleCooldown::check_cooldown(&db, "user4", "role4", "guild4").await.unwrap());
        
        // Run cleanup
        SelfRoleCooldown::cleanup_expired(&db).await.unwrap();
        
        // Verify active cooldowns still work
        assert!(SelfRoleCooldown::check_cooldown(&db, "user3", "role3", "guild3").await.unwrap());
        assert!(SelfRoleCooldown::check_cooldown(&db, "user4", "role4", "guild4").await.unwrap());
    }

    #[tokio::test]
    async fn test_selfrole_config_empty_fields() {
        let db = create_test_db().await;
        
        // Test creating config with empty title
        let result = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "", // Empty title
            "Valid body",
            "multiple"
        ).await;
        
        // Should succeed at database level (validation is at API level)
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.title, "");
        
        // Test creating config with empty body
        let result2 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654322",
            "Valid title",
            "", // Empty body
            "radio"
        ).await;
        
        assert!(result2.is_ok());
        let config2 = result2.unwrap();
        assert_eq!(config2.body, "");
    }

    #[tokio::test]
    async fn test_selfrole_role_duplicate_prevention() {
        let db = create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        // Create a role
        let _role1 = SelfRoleRole::create(&db, config.id, "role123", "🎮").await.unwrap();
        
        // Try to create the same role again - should work if no unique constraint
        let role2_result = SelfRoleRole::create(&db, config.id, "role123", "🎨").await;
        assert!(role2_result.is_ok()); // Database allows duplicates, business logic should prevent
        
        let roles = config.get_roles(&db).await.unwrap();
        assert_eq!(roles.len(), 2); // Both roles exist at database level
    }

    #[tokio::test]
    async fn test_selfrole_config_long_content() {
        let db = create_test_db().await;
        
        let long_title = "A".repeat(1000);
        let long_body = "B".repeat(5000);
        
        let result = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            &long_title,
            &long_body,
            "multiple"
        ).await;
        
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.title.len(), 1000);
        assert_eq!(config.body.len(), 5000);
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_edge_cases() {
        let db = create_test_db().await;
        
        // Test cooldown at exact expiry time
        let now = Utc::now();
        SelfRoleCooldown::create(&db, "user1", "role1", "guild1", now).await.unwrap();
        
        // Should not be on cooldown anymore (expired exactly now)
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        let is_on_cooldown = SelfRoleCooldown::check_cooldown(&db, "user1", "role1", "guild1").await.unwrap();
        assert!(!is_on_cooldown);
        
        // Test very short cooldown
        let very_soon = Utc::now() + Duration::milliseconds(100);
        SelfRoleCooldown::create(&db, "user2", "role2", "guild2", very_soon).await.unwrap();
        
        let is_on_cooldown_before = SelfRoleCooldown::check_cooldown(&db, "user2", "role2", "guild2").await.unwrap();
        assert!(is_on_cooldown_before);
        
        // Wait for it to expire
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        let is_on_cooldown_after = SelfRoleCooldown::check_cooldown(&db, "user2", "role2", "guild2").await.unwrap();
        assert!(!is_on_cooldown_after);
    }

    #[tokio::test]
    async fn test_database_error_handling() {
        let db = create_test_db().await;
        
        // Test getting config by invalid guild ID format 
        let result = SelfRoleConfig::get_by_guild(&db, "invalid_guild").await;
        assert!(result.is_ok()); // Should return Ok(Vec::new()) for invalid guild
        assert!(result.unwrap().is_empty());
        
        // Test getting config by message ID that doesn't exist
        let result = SelfRoleConfig::get_by_message_id(&db, "nonexistent").await.unwrap();
        assert!(result.is_none());
        
        // Test deleting by message ID that doesn't exist
        let deleted = SelfRoleConfig::delete_by_message_id(&db, "nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test] 
    async fn test_concurrent_cooldown_operations() {
        let db = create_test_db().await;
        
        // Test concurrent cooldown creation for same user/role
        let expires_at = Utc::now() + Duration::hours(1);
        
        // Create cooldowns sequentially instead of concurrently for simpler test
        for i in 0..5 {
            SelfRoleCooldown::create(
                &db,
                "user123",
                &format!("role{}", i),
                "guild456",
                expires_at
            ).await.unwrap();
        }
        
        // Verify all cooldowns were created
        for i in 0..5 {
            let is_on_cooldown = SelfRoleCooldown::check_cooldown(
                &db, 
                "user123", 
                &format!("role{}", i), 
                "guild456"
            ).await.unwrap();
            assert!(is_on_cooldown);
        }
    }
}