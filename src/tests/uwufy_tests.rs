#[cfg(test)]
mod tests {
    use crate::tests::create_test_db;
    use clouder_core::database::uwufy::UwufyToggle;

    #[tokio::test]
    async fn test_toggle_enables_new_user() {
        let db = create_test_db().await;
        let enabled = UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        assert!(enabled);
    }

    #[tokio::test]
    async fn test_toggle_disables_enabled_user() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        let enabled = UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        assert!(!enabled);
    }

    #[tokio::test]
    async fn test_is_enabled_false_for_unknown() {
        let db = create_test_db().await;
        let enabled = UwufyToggle::is_enabled(&db, "guild1", "unknown")
            .await
            .unwrap();
        assert!(!enabled);
    }

    #[tokio::test]
    async fn test_is_enabled_true_after_toggle() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        let enabled = UwufyToggle::is_enabled(&db, "guild1", "user1")
            .await
            .unwrap();
        assert!(enabled);
    }

    #[tokio::test]
    async fn test_get_enabled_in_guild() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        UwufyToggle::toggle(&db, "guild1", "user2").await.unwrap();
        UwufyToggle::toggle(&db, "guild2", "user3").await.unwrap();

        let enabled = UwufyToggle::get_enabled_in_guild(&db, "guild1")
            .await
            .unwrap();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"user1".to_string()));
        assert!(enabled.contains(&"user2".to_string()));
    }

    #[tokio::test]
    async fn test_get_enabled_excludes_disabled() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        UwufyToggle::toggle(&db, "guild1", "user2").await.unwrap();
        // toggle user2 off
        UwufyToggle::toggle(&db, "guild1", "user2").await.unwrap();

        let enabled = UwufyToggle::get_enabled_in_guild(&db, "guild1")
            .await
            .unwrap();
        assert_eq!(enabled.len(), 1);
        assert!(enabled.contains(&"user1".to_string()));
    }

    #[tokio::test]
    async fn test_disable_all_in_guild() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        UwufyToggle::toggle(&db, "guild1", "user2").await.unwrap();
        UwufyToggle::toggle(&db, "guild1", "user3").await.unwrap();
        // different guild should not be affected
        UwufyToggle::toggle(&db, "guild2", "user4").await.unwrap();

        let count = UwufyToggle::disable_all_in_guild(&db, "guild1")
            .await
            .unwrap();
        assert_eq!(count, 3);

        let enabled = UwufyToggle::get_enabled_in_guild(&db, "guild1")
            .await
            .unwrap();
        assert!(enabled.is_empty());

        // guild2 unaffected
        let enabled2 = UwufyToggle::get_enabled_in_guild(&db, "guild2")
            .await
            .unwrap();
        assert_eq!(enabled2.len(), 1);
    }

    #[tokio::test]
    async fn test_disable_all_returns_zero_when_none_enabled() {
        let db = create_test_db().await;
        let count = UwufyToggle::disable_all_in_guild(&db, "guild1")
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_set_enabled_explicit() {
        let db = create_test_db().await;

        UwufyToggle::set_enabled(&db, "guild1", "user1", true)
            .await
            .unwrap();
        assert!(
            UwufyToggle::is_enabled(&db, "guild1", "user1")
                .await
                .unwrap()
        );

        UwufyToggle::set_enabled(&db, "guild1", "user1", false)
            .await
            .unwrap();
        assert!(
            !UwufyToggle::is_enabled(&db, "guild1", "user1")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_set_enabled_overwrites_toggle() {
        let db = create_test_db().await;
        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        assert!(
            UwufyToggle::is_enabled(&db, "guild1", "user1")
                .await
                .unwrap()
        );

        UwufyToggle::set_enabled(&db, "guild1", "user1", false)
            .await
            .unwrap();
        assert!(
            !UwufyToggle::is_enabled(&db, "guild1", "user1")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_returns_toggle_record() {
        let db = create_test_db().await;
        assert!(
            UwufyToggle::get(&db, "guild1", "user1")
                .await
                .unwrap()
                .is_none()
        );

        UwufyToggle::toggle(&db, "guild1", "user1").await.unwrap();
        let record = UwufyToggle::get(&db, "guild1", "user1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.guild_id, "guild1");
        assert_eq!(record.user_id, "user1");
        assert!(record.enabled);
    }
}
