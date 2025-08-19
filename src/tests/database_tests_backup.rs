#[cfg(test)]
mod tests {
    #[cfg(test)]
mod tests {
    use crate::database::selfroles::{SelfRoleConfig, SelfRoleRole, SelfRoleCooldown};
    use chrono::{Utc, Duration};

    #[tokio::test]
    async fn test_selfrole_config_creation() {
        let db = super::create_test_db().await;
        
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
        let db = super::create_test_db().await;
        
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
        let pool = super::create_test_db().await;
        
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
        
        // Should be ordered by created_at DESC (newest first)
        assert_eq!(configs[0].id, config2.id);
        assert_eq!(configs[1].id, config1.id);
    }

    #[tokio::test]
    async fn test_selfrole_config_update() {
        let db = super::create_test_db().await;
        
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
        let db = super::create_test_db().await;
        
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
        let db = super::create_test_db().await;
        
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
        let db = super::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles",
            "Select your roles below:",
            "multiple"
        ).await.unwrap();
        
        let role1 = SelfRoleRole::create(
            &db,
            config.id,
            "111222333",
            "🎮"
        ).await.unwrap();
        
        let role2 = SelfRoleRole::create(
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
    async fn test_selfrole_role_delete_by_config_id() {
        let db = super::create_test_db().await;
        
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
        
        // Verify roles exist
        let roles_before = config.get_roles(&db).await.unwrap();
        assert_eq!(roles_before.len(), 2);
        
        // Delete all roles for this config
        SelfRoleRole::delete_by_config_id(&db, config.id).await.unwrap();
        
        // Verify roles are gone
        let roles_after = config.get_roles(&db).await.unwrap();
        assert_eq!(roles_after.len(), 0);
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_creation() {
        let db = super::create_test_db().await;
        
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
        let db = super::create_test_db().await;
        
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
    async fn test_selfrole_cooldown_cleanup() {
        let db = super::create_test_db().await;
        
        // Create expired cooldown
        let expired_time = Utc::now() - Duration::minutes(5);
        SelfRoleCooldown::create(
            &db,
            "user1",
            "role1",
            "guild1",
            expired_time
        ).await.unwrap();
        
        // Create active cooldown
        let future_time = Utc::now() + Duration::minutes(5);
        SelfRoleCooldown::create(
            &db,
            "user2",
            "role2",
            "guild2",
            future_time
        ).await.unwrap();
        
        // Clean up expired cooldowns
        SelfRoleCooldown::cleanup_expired(&db).await.unwrap();
        
        // Check that expired cooldown is gone
        let is_expired_on_cooldown = SelfRoleCooldown::check_cooldown(
            &db,
            "user1",
            "role1",
            "guild1"
        ).await.unwrap();
        assert!(!is_expired_on_cooldown);
        
        // Check that active cooldown is still there
        let is_active_on_cooldown = SelfRoleCooldown::check_cooldown(
            &db,
            "user2",
            "role2",
            "guild2"
        ).await.unwrap();
        assert!(is_active_on_cooldown);
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_replace() {
        let db = super::create_test_db().await;
        
        let expires_at1 = Utc::now() + Duration::minutes(5);
        let expires_at2 = Utc::now() + Duration::minutes(10);
        
        // Create initial cooldown
        SelfRoleCooldown::create(
            &db,
            "user123",
            "role456",
            "guild789",
            expires_at1
        ).await.unwrap();
        
        // Replace with new cooldown (should update expires_at)
        SelfRoleCooldown::create(
            &db,
            "user123",
            "role456",
            "guild789",
            expires_at2
        ).await.unwrap();
        
        // Should still be on cooldown with the new time
        let is_on_cooldown = SelfRoleCooldown::check_cooldown(
            &db,
            "user123",
            "role456",
            "guild789"
        ).await.unwrap();
        
        assert!(is_on_cooldown);
    }

    #[tokio::test]
    async fn test_database_constraints() {
        let db = super::create_test_db().await;
        
        // Test unique message_id constraint
        let config1 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles 1",
            "First message",
            "multiple"
        ).await.unwrap();
        
        let mut config1_copy = config1;
        config1_copy.update_message_id(&db, "unique_message_123").await.unwrap();
        
        let config2 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654322",
            "Test Roles 2",
            "Second message",
            "multiple"
        ).await.unwrap();
        
        let mut config2_copy = config2;
        // This should fail due to unique constraint on message_id
        let result = config2_copy.update_message_id(&db, "unique_message_123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_foreign_key_cascade() {
        let db = super::create_test_db().await;
        
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
        
        // Verify roles exist
        let roles_before = config.get_roles(&db).await.unwrap();
        assert_eq!(roles_before.len(), 2);
        
        // Delete the config (should cascade delete roles)
        config.delete(&db).await.unwrap();
        
        // Verify roles are also deleted due to foreign key cascade
        let result = sqlx::query("SELECT COUNT(*) as count FROM selfrole_roles WHERE config_id = ?")
            .bind(config.id)
            .fetch_one(&db)
            .await
            .unwrap();
        let count: i64 = result.get("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_selection_type_constraint() {
        let db = super::create_test_db().await;
        
        // Valid selection types should work
        let config1 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654321",
            "Test Roles 1",
            "Radio selection",
            "radio"
        ).await.unwrap();
        
        let config2 = SelfRoleConfig::create(
            &db,
            "123456789",
            "987654322",
            "Test Roles 2",
            "Multiple selection",
            "multiple"
        ).await.unwrap();
        
        assert_eq!(config1.selection_type, "radio");
        assert_eq!(config2.selection_type, "multiple");
        
        // Invalid selection type should fail
        let result = sqlx::query(
            "INSERT INTO selfrole_configs (guild_id, channel_id, title, body, selection_type) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("123456789")
        .bind("987654323")
        .bind("Test Invalid")
        .bind("Invalid selection type")
        .bind("invalid_type")
        .execute(&db)
        .await;
        
        assert!(result.is_err());
    }
}
    use chrono::{Utc, Duration};
    use sqlx::Row;

    #[tokio::test]
    async fn test_selfrole_config_creation() {
        let db = super::create_test_db().await;
        
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
    async fn test_selfrole_config_get_by_guild() {
        let pool = super::create_test_db().await;
        
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
        
        // Should be ordered by created_at DESC (newest first)
        assert_eq!(configs[0].id, config2.id);
        assert_eq!(configs[1].id, config1.id);
    }
        let _config3 = SelfRoleConfig::create(
            &pool,
            "999999999",
            "987654323",
            "Different Guild",
            "Different guild message",
            "single"
        ).await.unwrap();
        
        let configs = SelfRoleConfig::get_by_guild(&pool, "123456789").await.unwrap();
        assert_eq!(configs.len(), 2);
        
        let config_ids: Vec<i64> = configs.iter().map(|c| c.id).collect();
        assert!(config_ids.contains(&config1.id));
        assert!(config_ids.contains(&config2.id));
    }

    #[tokio::test]
    async fn test_selfrole_config_set_message_id() {
        let pool = crate::tests::create_test_db().await;
        
        let mut config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        assert!(config.message_id.is_none());
        
        config.set_message_id(&pool, "message_123").await.unwrap();
        assert_eq!(config.message_id, Some("message_123".to_string()));
        
        // Verify in database
        let retrieved = SelfRoleConfig::get_by_message_id(&pool, "message_123").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, config.id);
    }

    #[tokio::test]
    async fn test_selfrole_config_get_by_message_id() {
        let pool = crate::tests::create_test_db().await;
        
        let mut config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        config.set_message_id(&pool, "message_456").await.unwrap();
        
        let retrieved = SelfRoleConfig::get_by_message_id(&pool, "message_456").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, config.id);
        assert_eq!(retrieved.message_id, Some("message_456".to_string()));
        
        // Test non-existent message ID
        let not_found = SelfRoleConfig::get_by_message_id(&pool, "nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_selfrole_config_update() {
        let pool = crate::tests::create_test_db().await;
        
        let mut config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        config.update(
            &pool,
            "Updated Title",
            "Updated body text",
            "multiple"
        ).await.unwrap();
        
        assert_eq!(config.title, "Updated Title");
        assert_eq!(config.body, "Updated body text");
        assert_eq!(config.selection_type, "multiple");
        
        // Verify in database
        let row = sqlx::query("SELECT title, body, selection_type FROM selfrole_configs WHERE id = ?")
            .bind(config.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(row.get::<String, _>("title"), "Updated Title");
        assert_eq!(row.get::<String, _>("body"), "Updated body text");
        assert_eq!(row.get::<String, _>("selection_type"), "multiple");
    }

    #[tokio::test]
    async fn test_selfrole_config_delete() {
        let pool = crate::tests::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let config_id = config.id;
        
        config.delete(&pool).await.unwrap();
        
        // Verify deletion
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM selfrole_configs WHERE id = ?")
            .bind(config_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_selfrole_role_create() {
        let pool = crate::tests::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let role = SelfRoleRole::create(
            &pool,
            config.id,
            "role_123",
            "🎮"
        ).await.unwrap();
        
        assert_eq!(role.config_id, config.id);
        assert_eq!(role.role_id, "role_123");
        assert_eq!(role.emoji, "🎮");
        assert!(role.id > 0);
    }

    #[tokio::test]
    async fn test_selfrole_role_get_by_config_id() {
        let pool = crate::tests::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let role1 = SelfRoleRole::create(&pool, config.id, "role_123", "🎮").await.unwrap();
        let role2 = SelfRoleRole::create(&pool, config.id, "role_456", "🎵").await.unwrap();
        
        let roles = SelfRoleRole::get_by_config_id(&pool, config.id).await.unwrap();
        assert_eq!(roles.len(), 2);
        
        let role_ids: Vec<String> = roles.iter().map(|r| r.role_id.clone()).collect();
        assert!(role_ids.contains(&role1.role_id));
        assert!(role_ids.contains(&role2.role_id));
    }

    #[tokio::test]
    async fn test_selfrole_role_delete_by_config_id() {
        let pool = crate::tests::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let _role1 = SelfRoleRole::create(&pool, config.id, "role_123", "🎮").await.unwrap();
        let _role2 = SelfRoleRole::create(&pool, config.id, "role_456", "🎵").await.unwrap();
        
        // Create role for different config
        let config2 = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654322",
            "Different Config",
            "Different message",
            "single"
        ).await.unwrap();
        let _role3 = SelfRoleRole::create(&pool, config2.id, "role_789", "🎭").await.unwrap();
        
        SelfRoleRole::delete_by_config_id(&pool, config.id).await.unwrap();
        
        // Verify roles for config.id are deleted
        let roles = SelfRoleRole::get_by_config_id(&pool, config.id).await.unwrap();
        assert_eq!(roles.len(), 0);
        
        // Verify role for config2.id still exists
        let roles2 = SelfRoleRole::get_by_config_id(&pool, config2.id).await.unwrap();
        assert_eq!(roles2.len(), 1);
    }

    #[tokio::test]
    async fn test_selfrole_cooldown_cleanup_expired() {
        let pool = crate::tests::create_test_db().await;
        
        // Insert expired cooldown
        let expired_time = Utc::now() - Duration::hours(1);
        sqlx::query(
            "INSERT INTO selfrole_cooldowns (user_id, role_id, guild_id, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind("user_123")
        .bind("role_123")
        .bind("guild_123")
        .bind(expired_time)
        .execute(&pool)
        .await
        .unwrap();
        
        // Insert future cooldown
        let future_time = Utc::now() + Duration::hours(1);
        sqlx::query(
            "INSERT INTO selfrole_cooldowns (user_id, role_id, guild_id, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind("user_456")
        .bind("role_456")
        .bind("guild_123")
        .bind(future_time)
        .execute(&pool)
        .await
        .unwrap();
        
        // Clean up expired cooldowns
        SelfRoleCooldown::cleanup_expired(&pool).await.unwrap();
        
        // Verify only future cooldown remains
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM selfrole_cooldowns")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(count, 1);
        
        // Verify the remaining cooldown is the future one
        let remaining_user: String = sqlx::query_scalar("SELECT user_id FROM selfrole_cooldowns")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(remaining_user, "user_456");
    }

    #[tokio::test]
    async fn test_selfrole_config_cascade_delete_roles() {
        let pool = crate::tests::create_test_db().await;
        
        let config = SelfRoleConfig::create(
            &pool,
            "123456789",
            "987654321",
            "Test Role Message",
            "Click buttons to get roles",
            "single"
        ).await.unwrap();
        
        let _role1 = SelfRoleRole::create(&pool, config.id, "role_123", "🎮").await.unwrap();
        let _role2 = SelfRoleRole::create(&pool, config.id, "role_456", "🎵").await.unwrap();
        
        // Verify roles exist
        let roles_before = SelfRoleRole::get_by_config_id(&pool, config.id).await.unwrap();
        assert_eq!(roles_before.len(), 2);
        
        // Delete config
        config.delete(&pool).await.unwrap();
        
        // Verify roles are also deleted (cascade)
        let roles_after = SelfRoleRole::get_by_config_id(&pool, config.id).await.unwrap();
        assert_eq!(roles_after.len(), 0);
    }
}