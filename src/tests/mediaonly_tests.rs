#[cfg(test)]
mod tests {
    use crate::database::mediaonly::MediaOnlyConfig;
    use crate::tests::create_test_db;
    use crate::web::mediaonly::{
        MediaOnlyConfigDisplay, MediaOnlyConfigRequest, MediaOnlyConfigUpdateRequest,
    };
    use serde_json::json;

    #[test]
    fn test_mediaonly_config_request_serialization() {
        // Test MediaOnlyConfigRequest serialization/deserialization
        let request = MediaOnlyConfigRequest {
            channel_id: "123456789".to_string(),
            enabled: true,
            allow_links: true,
            allow_attachments: false,
            allow_gifs: true,
            allow_stickers: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MediaOnlyConfigRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.channel_id, "123456789");
        assert!(deserialized.enabled);
        assert!(deserialized.allow_links);
        assert!(!deserialized.allow_attachments);
        assert!(deserialized.allow_gifs);
        assert!(!deserialized.allow_stickers);
    }

    #[test]
    fn test_mediaonly_config_update_request_serialization() {
        // Test MediaOnlyConfigUpdateRequest serialization/deserialization
        let request = MediaOnlyConfigUpdateRequest {
            allow_links: false,
            allow_attachments: true,
            allow_gifs: false,
            allow_stickers: true,
            enabled: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MediaOnlyConfigUpdateRequest = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.allow_links);
        assert!(deserialized.allow_attachments);
        assert!(!deserialized.allow_gifs);
        assert!(deserialized.allow_stickers);
        assert_eq!(deserialized.enabled, Some(true));
    }

    #[test]
    fn test_mediaonly_config_display_serialization() {
        // Test MediaOnlyConfigDisplay serialization (only Serialize, not Deserialize)
        use chrono::Utc;

        let display = MediaOnlyConfigDisplay {
            id: 1,
            channel_id: "123456789".to_string(),
            channel_name: "test-channel".to_string(),
            enabled: true,
            allow_links: true,
            allow_attachments: true,
            allow_gifs: false,
            allow_stickers: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&display).unwrap();
        // Just verify it serializes to valid JSON
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"channel_id\":\"123456789\""));
        assert!(json.contains("\"channel_name\":\"test-channel\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_json_response_formats() {
        // Test JSON response formats used in web handlers
        let success_response = json!({"success": true});
        assert_eq!(success_response["success"], true);

        let error_response = json!({"success": false, "error": "Test error"});
        assert_eq!(error_response["success"], false);
        assert_eq!(error_response["error"], "Test error");

        let configs_response = json!({"success": true, "configs": []});
        assert_eq!(configs_response["success"], true);
        assert!(configs_response["configs"].is_array());
    }

    #[tokio::test]
    async fn test_mediaonly_config_struct() {
        let db = create_test_db().await;

        // Test MediaOnlyConfig upsert (create)
        MediaOnlyConfig::upsert(
            &db, "12345", // guild_id
            "67890", // channel_id
            true,    // enabled
        )
        .await
        .unwrap();

        // Test getting config by channel
        let config = MediaOnlyConfig::get_by_channel(&db, "12345", "67890")
            .await
            .unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.guild_id, "12345");
        assert_eq!(config.channel_id, "67890");
        assert!(config.enabled);
        assert!(config.allow_links); // Default value
        assert!(config.allow_attachments); // Default value
        assert!(config.allow_gifs); // Default value
        assert!(config.allow_stickers); // Default value

        // Test getting config by guild
        let configs = MediaOnlyConfig::get_by_guild(&db, "12345").await.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].channel_id, "67890");

        // Test updating permissions
        MediaOnlyConfig::update_permissions(
            &db, "12345", "67890", false, // allow_links
            true,  // allow_attachments
            false, // allow_gifs
            true,  // allow_stickers
        )
        .await
        .unwrap();

        let updated_config = MediaOnlyConfig::get_by_channel(&db, "12345", "67890")
            .await
            .unwrap()
            .unwrap();
        assert!(!updated_config.allow_links);
        assert!(updated_config.allow_attachments);
        assert!(!updated_config.allow_gifs);
        assert!(updated_config.allow_stickers);

        // Test deletion
        MediaOnlyConfig::delete(&db, "12345", "67890")
            .await
            .unwrap();

        let configs_after = MediaOnlyConfig::get_by_guild(&db, "12345").await.unwrap();
        assert_eq!(configs_after.len(), 0);
    }

    #[tokio::test]
    async fn test_mediaonly_config_validation() {
        let db = create_test_db().await;

        // Test upsert with empty guild_id (this should work but create invalid data)
        // Note: The database constraints will handle validation, not the upsert method
        let result = MediaOnlyConfig::upsert(&db, "", "67890", true).await;
        assert!(result.is_ok()); // upsert doesn't validate, just inserts

        // Test upsert with empty channel_id
        let result = MediaOnlyConfig::upsert(&db, "12345", "", true).await;
        assert!(result.is_ok()); // upsert doesn't validate, just inserts

        // Test getting non-existent configs
        let result = MediaOnlyConfig::get_by_guild(&db, "nonexistent")
            .await
            .unwrap();
        assert_eq!(result.len(), 0);

        let result = MediaOnlyConfig::get_by_channel(&db, "nonexistent", "nonexistent")
            .await
            .unwrap();
        assert!(result.is_none());

        // Clean up test data
        MediaOnlyConfig::delete(&db, "", "67890").await.ok();
        MediaOnlyConfig::delete(&db, "12345", "").await.ok();
    }

    #[tokio::test]
    async fn test_mediaonly_config_multiple_channels() {
        let db = create_test_db().await;

        // Create multiple configs for the same guild
        MediaOnlyConfig::upsert(&db, "12345", "67890", true)
            .await
            .unwrap();
        MediaOnlyConfig::upsert(&db, "12345", "67891", false)
            .await
            .unwrap();

        // Test getting all configs by guild
        let configs = MediaOnlyConfig::get_by_guild(&db, "12345").await.unwrap();
        assert_eq!(configs.len(), 2);

        let channel_ids: Vec<String> = configs.iter().map(|c| c.channel_id.clone()).collect();
        assert!(channel_ids.contains(&"67890".to_string()));
        assert!(channel_ids.contains(&"67891".to_string()));

        // Test toggle functionality
        let enabled = MediaOnlyConfig::toggle(&db, "12345", "67891")
            .await
            .unwrap();
        assert!(enabled); // Was false, now true

        let config = MediaOnlyConfig::get_by_channel(&db, "12345", "67891")
            .await
            .unwrap()
            .unwrap();
        assert!(config.enabled);

        // Clean up
        MediaOnlyConfig::delete(&db, "12345", "67890")
            .await
            .unwrap();
        MediaOnlyConfig::delete(&db, "12345", "67891")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_mediaonly_command_logic() {
        let db = create_test_db().await;

        // Test the basic logic that the mediaonly command uses
        let guild_id = "12345";
        let channel_id = "67890";

        // Test upsert (enable)
        MediaOnlyConfig::upsert(&db, guild_id, channel_id, true)
            .await
            .unwrap();
        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(config.enabled);

        // Test toggle (should disable)
        let enabled = MediaOnlyConfig::toggle(&db, guild_id, channel_id)
            .await
            .unwrap();
        assert!(!enabled);

        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(!config.enabled);

        // Test toggle again (should enable)
        let enabled = MediaOnlyConfig::toggle(&db, guild_id, channel_id)
            .await
            .unwrap();
        assert!(enabled);

        // Clean up
        MediaOnlyConfig::delete(&db, guild_id, channel_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_mediaonly_event_handler_logic() {
        let db = create_test_db().await;

        // Test the core logic of the event handler
        let guild_id = "12345";
        let channel_id = "67890";

        // Test that handler ignores channels without config
        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap();
        assert!(config.is_none()); // No config, should be ignored

        // Create a config
        MediaOnlyConfig::upsert(&db, guild_id, channel_id, true)
            .await
            .unwrap();

        // Test that handler would process channels with enabled config
        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(config.enabled);

        // Test that handler would ignore disabled configs
        MediaOnlyConfig::upsert(&db, guild_id, channel_id, false)
            .await
            .unwrap();
        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(!config.enabled);

        // Clean up
        MediaOnlyConfig::delete(&db, guild_id, channel_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_mediaonly_event_handler_permissions() {
        let db = create_test_db().await;

        // Test different permission combinations
        let guild_id = "12345";
        let channel_id = "67890";

        // Create config with different permission settings
        MediaOnlyConfig::upsert(&db, guild_id, channel_id, true)
            .await
            .unwrap();

        // Test updating permissions (this is what the event handler uses)
        MediaOnlyConfig::update_permissions(
            &db, guild_id, channel_id, true,  // allow_links
            false, // allow_attachments
            true,  // allow_gifs
            false, // allow_stickers
        )
        .await
        .unwrap();

        let config = MediaOnlyConfig::get_by_channel(&db, guild_id, channel_id)
            .await
            .unwrap()
            .unwrap();
        assert!(config.allow_links);
        assert!(!config.allow_attachments);
        assert!(config.allow_gifs);
        assert!(!config.allow_stickers);

        // Clean up
        MediaOnlyConfig::delete(&db, guild_id, channel_id)
            .await
            .unwrap();
    }
}
