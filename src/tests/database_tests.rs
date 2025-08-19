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
}