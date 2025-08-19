#[cfg(test)]
mod tests {

    use crate::database::selfroles::{SelfRoleConfig};


    use crate::tests::{create_test_db, create_test_app_state};

    #[tokio::test]
    async fn test_message_delete_event_basic() {
        // Test basic message delete event handling
        let app_state = create_test_app_state().await;

        // Create a test self-role config
        let config = SelfRoleConfig::create(
            &*app_state.db,
            "12345",
            "67890",
            "Test Self Roles",
            "Test description",
            "multiple"
        ).await.unwrap();

        // Update with message ID
        let mut config_copy = config;
        config_copy.update_message_id(&*app_state.db, "11111").await.unwrap();

        // Test that the config exists
        let configs = SelfRoleConfig::get_by_guild(&*app_state.db, "12345").await.unwrap();
        assert_eq!(configs.len(), 1);

        // Simulate message deletion by calling the database cleanup directly
        let result = SelfRoleConfig::get_by_message_id(&*app_state.db, "11111").await.unwrap();
        assert!(result.is_some());

        if let Some(found_config) = result {
            found_config.delete(&*app_state.db).await.unwrap();
        }

        // Verify the config was deleted
        let configs_after = SelfRoleConfig::get_by_guild(&*app_state.db, "12345").await.unwrap();
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
            "multiple"
        ).await.unwrap();

        let mut config_copy = config;
        config_copy.update_message_id(&db, "555666777").await.unwrap();

        // Verify it exists
        let result = SelfRoleConfig::get_by_message_id(&db, "555666777").await.unwrap();
        assert!(result.is_some());

        // Delete by message ID
        let deleted = SelfRoleConfig::delete_by_message_id(&db, "555666777").await.unwrap();
        assert!(deleted);

        // Verify it's gone
        let result_after = SelfRoleConfig::get_by_message_id(&db, "555666777").await.unwrap();
        assert!(result_after.is_none());
    }
}
