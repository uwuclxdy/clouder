#[cfg(test)]
mod tests {
    use crate::tests::create_test_db;
    use clouder_core::database::reminders::{
        CustomReminder, CustomReminderLog, CustomReminderPingRole, CustomReminderSubscription,
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
        use clouder::scheduler::next_727_timestamp;
        use clouder_core::utils::parse_hhmm;
        let t = parse_hhmm("07:27").unwrap();
        assert_eq!(t.hour(), 7);
        assert_eq!(t.minute(), 27);

        let tz: chrono_tz::Tz = "UTC".parse().unwrap();
        let ts = next_727_timestamp(&tz);
        assert!(ts.is_some());
        assert!(ts.unwrap().starts_with("<t:"));
    }

    #[tokio::test]
    async fn test_custom_reminder_crud() {
        let db = create_test_db().await;
        sqlx::query("INSERT INTO guild_configs (guild_id) VALUES ('g1')")
            .execute(&db)
            .await
            .unwrap();

        // create
        let id = CustomReminder::create(
            &db,
            "g1",
            "standup",
            Some("chan1"),
            "09:00",
            "1,2,3,4,5",
            "UTC",
            "embed",
            None,
            Some("standup"),
            None,
            None,
        )
        .await
        .unwrap();
        assert!(id > 0);

        // get_by_guild
        let all = CustomReminder::get_by_guild(&db, "g1").await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "standup");
        assert_eq!(all[0].schedule_days, "1,2,3,4,5");

        // get_by_id
        let got = CustomReminder::get_by_id(&db, id).await.unwrap().unwrap();
        assert_eq!(got.name, "standup");

        // count
        let count = CustomReminder::count_by_guild(&db, "g1").await.unwrap();
        assert_eq!(count, 1);

        // update
        CustomReminder::update(
            &db,
            id,
            "daily standup",
            Some("chan2"),
            "10:00",
            "",
            "UTC",
            "text",
            Some("hello"),
            None,
            None,
            None,
        )
        .await
        .unwrap();
        let updated = CustomReminder::get_by_id(&db, id).await.unwrap().unwrap();
        assert_eq!(updated.name, "daily standup");
        assert_eq!(updated.schedule_days, "");
        assert_eq!(updated.message_type, "text");

        // set_enabled
        CustomReminder::set_enabled(&db, id, true).await.unwrap();
        let enabled = CustomReminder::get_by_id(&db, id).await.unwrap().unwrap();
        assert!(enabled.enabled);

        // delete
        CustomReminder::delete(&db, id).await.unwrap();
        let gone = CustomReminder::get_by_id(&db, id).await.unwrap();
        assert!(gone.is_none());
    }

    #[tokio::test]
    async fn test_custom_reminder_ping_roles() {
        let db = create_test_db().await;
        sqlx::query("INSERT INTO guild_configs (guild_id) VALUES ('g1')")
            .execute(&db)
            .await
            .unwrap();
        let id = CustomReminder::create(
            &db,
            "g1",
            "test",
            Some("c"),
            "12:00",
            "",
            "UTC",
            "embed",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // set roles
        CustomReminderPingRole::set_roles(&db, id, &["r1".to_string(), "r2".to_string()])
            .await
            .unwrap();
        let roles = CustomReminderPingRole::get_by_reminder(&db, id)
            .await
            .unwrap();
        assert_eq!(roles.len(), 2);

        // replace roles
        CustomReminderPingRole::set_roles(&db, id, &["r3".to_string()])
            .await
            .unwrap();
        let roles = CustomReminderPingRole::get_by_reminder(&db, id)
            .await
            .unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].role_id, "r3");

        // delete
        CustomReminderPingRole::delete_by_reminder(&db, id)
            .await
            .unwrap();
        let roles = CustomReminderPingRole::get_by_reminder(&db, id)
            .await
            .unwrap();
        assert!(roles.is_empty());
    }

    #[tokio::test]
    async fn test_custom_reminder_subscriptions() {
        let db = create_test_db().await;
        sqlx::query("INSERT INTO guild_configs (guild_id) VALUES ('g1')")
            .execute(&db)
            .await
            .unwrap();
        let rid = CustomReminder::create(
            &db,
            "g1",
            "test",
            Some("c"),
            "12:00",
            "",
            "UTC",
            "embed",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // subscribe
        CustomReminderSubscription::subscribe(&db, "u1", rid)
            .await
            .unwrap();
        let subs = CustomReminderSubscription::get_by_user(&db, "u1")
            .await
            .unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].reminder_id, rid);

        // duplicate subscribe is ignored
        CustomReminderSubscription::subscribe(&db, "u1", rid)
            .await
            .unwrap();
        let subs = CustomReminderSubscription::get_by_user(&db, "u1")
            .await
            .unwrap();
        assert_eq!(subs.len(), 1);

        // unsubscribe
        CustomReminderSubscription::unsubscribe(&db, "u1", rid)
            .await
            .unwrap();
        let subs = CustomReminderSubscription::get_by_user(&db, "u1")
            .await
            .unwrap();
        assert!(subs.is_empty());
    }

    #[tokio::test]
    async fn test_custom_reminder_log() {
        let db = create_test_db().await;
        sqlx::query("INSERT INTO guild_configs (guild_id) VALUES ('g1')")
            .execute(&db)
            .await
            .unwrap();
        let rid = CustomReminder::create(
            &db,
            "g1",
            "test",
            Some("c"),
            "12:00",
            "",
            "UTC",
            "embed",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        CustomReminderLog::create(&db, rid, "success", None, true, 5, 1)
            .await
            .unwrap();
        let logs = CustomReminderLog::get_recent_by_reminder(&db, rid, 10)
            .await
            .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, "success");
        assert!(logs[0].channel_sent);
        assert_eq!(logs[0].dm_count, 5);
    }

    #[tokio::test]
    async fn test_schedule_days_match() {
        use clouder::scheduler::schedule_days_match;

        // empty = every day
        assert!(schedule_days_match("", 0));
        assert!(schedule_days_match("", 3));
        assert!(schedule_days_match("", 6));

        // weekdays
        assert!(schedule_days_match("1,2,3,4,5", 1)); // mon
        assert!(schedule_days_match("1,2,3,4,5", 5)); // fri
        assert!(!schedule_days_match("1,2,3,4,5", 0)); // sun
        assert!(!schedule_days_match("1,2,3,4,5", 6)); // sat

        // weekend
        assert!(schedule_days_match("0,6", 0));
        assert!(schedule_days_match("0,6", 6));
        assert!(!schedule_days_match("0,6", 3));

        // single day
        assert!(schedule_days_match("3", 3));
        assert!(!schedule_days_match("3", 4));
    }
}
