#[cfg(test)]
mod tests {
    use crate::tests::create_test_db;
    use clouder_core::database::reminders::{
        ReminderConfig, ReminderPingRole, ReminderSubscription, ReminderType, UserSettings,
    };

    #[tokio::test]
    async fn test_user_settings_crud() {
        let db = create_test_db().await;
        let uid = "user123";

        // initially none
        let maybe = UserSettings::get(&db, uid).await.unwrap();
        assert!(maybe.is_none());

        UserSettings::upsert(&db, uid, "America/Los_Angeles", false)
            .await
            .unwrap();

        let got = UserSettings::get(&db, uid).await.unwrap().unwrap();
        assert_eq!(got.timezone, "America/Los_Angeles");
        assert!(!got.dm_reminders_enabled);

        // update again
        UserSettings::upsert(&db, uid, "UTC", true).await.unwrap();
        let got = UserSettings::get(&db, uid).await.unwrap().unwrap();
        assert_eq!(got.timezone, "UTC");
        assert!(got.dm_reminders_enabled);
    }

    #[tokio::test]
    async fn test_reminder_config_and_roles() {
        let db = create_test_db().await;
        let guild = "guild1";

        // insert config using upsert
        let id = ReminderConfig::upsert(
            &db,
            guild,
            &ReminderType::Wysi,
            Some("123"),
            "embed",
            None,
            None,
            None,
            None,
            Some("08:00"),
            Some("20:00"),
            "UTC",
        )
        .await
        .unwrap();

        assert!(id > 0);
        let configs = ReminderConfig::get_by_guild(&db, guild).await.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].id, id);

        let by_type = ReminderConfig::get_by_type(&db, guild, &ReminderType::Wysi)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_type.id, id);

        let by_id = ReminderConfig::get_by_id(&db, id).await.unwrap().unwrap();
        assert_eq!(by_id.guild_id, guild);

        ReminderConfig::set_enabled(&db, id, false).await.unwrap();
        let updated = ReminderConfig::get_by_id(&db, id).await.unwrap().unwrap();
        assert!(!updated.enabled);

        // ping roles
        ReminderPingRole::set_roles(&db, id, &["111".to_string(), "222".to_string()])
            .await
            .unwrap();
        let roles = ReminderPingRole::get_by_config(&db, id).await.unwrap();
        let ids: Vec<String> = roles.iter().map(|r| r.role_id.clone()).collect();
        assert!(ids.contains(&"111".to_string()));
        assert!(ids.contains(&"222".to_string()));

        ReminderPingRole::delete_by_config(&db, id).await.unwrap();
        let roles = ReminderPingRole::get_by_config(&db, id).await.unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_subscriptions_lifecycle() {
        let db = create_test_db().await;
        let user = "u1";
        let guild = "g1";

        let config_id = ReminderConfig::upsert(
            &db,
            guild,
            &ReminderType::Wysi,
            Some("123"),
            "text",
            Some("hello"),
            None,
            None,
            None,
            Some("07:27"),
            Some("19:27"),
            "UTC",
        )
        .await
        .unwrap();

        // subscribe
        ReminderSubscription::subscribe(&db, user, config_id)
            .await
            .unwrap();
        let subs = ReminderSubscription::get_by_user(&db, user).await.unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].config_id, config_id);

        // unsubscribe by config
        ReminderSubscription::unsubscribe(&db, user, config_id)
            .await
            .unwrap();
        let subs = ReminderSubscription::get_by_user(&db, user).await.unwrap();
        assert!(subs.is_empty());

        // subscribe twice and then unsubscribe all
        ReminderSubscription::subscribe(&db, user, config_id)
            .await
            .unwrap();
        ReminderSubscription::unsubscribe_all_for_user(&db, user)
            .await
            .unwrap();
        let subs = ReminderSubscription::get_by_user(&db, user).await.unwrap();
        assert!(subs.is_empty());
    }

    #[tokio::test]
    async fn test_hhmm_parsing_and_next727() {
        use chrono::Timelike;
        use clouder::scheduler::{next_727_timestamp, parse_hhmm};
        let t = parse_hhmm("07:27").unwrap();
        assert_eq!(t.hour(), 7);
        assert_eq!(t.minute(), 27);

        let tz: chrono_tz::Tz = "UTC".parse().unwrap();
        let ts = next_727_timestamp(&tz);
        assert!(ts.is_some());
        assert!(ts.unwrap().starts_with("<t:"));
    }
}
