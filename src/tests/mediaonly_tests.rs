#[cfg(test)]
mod tests {
    use crate::tests::create_test_db;
    use clouder_core::database::mediaonly::MediaOnlyConfig;

    #[tokio::test]
    async fn test_get_by_channel_not_found() {
        let db = create_test_db().await;
        let result = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_upsert_and_get_by_channel() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(config.guild_id, "guild1");
        assert_eq!(config.channel_id, "channel1");
        assert!(config.enabled);
        assert!(config.allow_links);
        assert!(config.allow_attachments);
        assert!(config.allow_gifs);
        assert!(config.allow_stickers);
    }

    #[tokio::test]
    async fn test_upsert_updates_enabled_state() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();
        MediaOnlyConfig::upsert(&db, "guild1", "channel1", false)
            .await
            .unwrap();

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_get_by_guild_isolation() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();
        MediaOnlyConfig::upsert(&db, "guild1", "channel2", false)
            .await
            .unwrap();
        MediaOnlyConfig::upsert(&db, "guild2", "channel3", true)
            .await
            .unwrap();

        let configs = MediaOnlyConfig::get_by_guild(&db, "guild1").await.unwrap();
        assert_eq!(configs.len(), 2);

        let channels: Vec<&str> = configs.iter().map(|c| c.channel_id.as_str()).collect();
        assert!(channels.contains(&"channel1"));
        assert!(channels.contains(&"channel2"));
    }

    #[tokio::test]
    async fn test_get_by_guild_empty() {
        let db = create_test_db().await;
        let configs = MediaOnlyConfig::get_by_guild(&db, "nonexistent")
            .await
            .unwrap();
        assert!(configs.is_empty());
    }

    #[tokio::test]
    async fn test_toggle_flips_existing() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();

        let new_state = MediaOnlyConfig::toggle(&db, "guild1", "channel1")
            .await
            .unwrap();
        assert!(!new_state);

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_toggle_creates_enabled_when_missing() {
        let db = create_test_db().await;

        let new_state = MediaOnlyConfig::toggle(&db, "guild1", "channel1")
            .await
            .unwrap();
        assert!(new_state);

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_upsert_with_config_stores_permissions() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert_with_config(&db, "guild1", "channel1", false, true, false, true)
            .await
            .unwrap();

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();

        assert!(!config.allow_links);
        assert!(config.allow_attachments);
        assert!(!config.allow_gifs);
        assert!(config.allow_stickers);
    }

    #[tokio::test]
    async fn test_upsert_with_config_updates_permissions() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert_with_config(&db, "guild1", "channel1", true, true, true, true)
            .await
            .unwrap();
        MediaOnlyConfig::upsert_with_config(&db, "guild1", "channel1", false, false, false, false)
            .await
            .unwrap();

        let config = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap()
            .unwrap();

        assert!(!config.allow_links);
        assert!(!config.allow_attachments);
        assert!(!config.allow_gifs);
        assert!(!config.allow_stickers);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();
        MediaOnlyConfig::delete(&db, "guild1", "channel1")
            .await
            .unwrap();

        let result = MediaOnlyConfig::get_by_channel(&db, "guild1", "channel1")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_only_removes_target_channel() {
        let db = create_test_db().await;

        MediaOnlyConfig::upsert(&db, "guild1", "channel1", true)
            .await
            .unwrap();
        MediaOnlyConfig::upsert(&db, "guild1", "channel2", true)
            .await
            .unwrap();

        MediaOnlyConfig::delete(&db, "guild1", "channel1")
            .await
            .unwrap();

        let configs = MediaOnlyConfig::get_by_guild(&db, "guild1").await.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].channel_id, "channel2");
    }
}
