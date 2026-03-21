#[cfg(test)]
mod tests {
    use crate::tests::create_test_app_state;
    use clouder_core::shared::{
        create_custom_reminder, update_custom_reminder, upsert_reminder_config,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_upsert_reminder_config_validates_content_lengths() {
        let app_state = create_test_app_state().await;
        let payload = json!({
            "reminder_type": "wysi",
            "message_content": "x".repeat(2001),
            "embed_title": "ok",
            "embed_description": "ok",
        });

        let error = upsert_reminder_config(&app_state, 123, &payload)
            .await
            .unwrap_err();

        assert_eq!(error, "message_content exceeds 2000 characters");
    }

    #[tokio::test]
    async fn test_create_custom_reminder_validates_content_lengths() {
        let app_state = create_test_app_state().await;
        let payload = json!({
            "name": "test reminder",
            "schedule_time": "12:30",
            "message_content": "ok",
            "embed_title": "x".repeat(257),
            "embed_description": "ok",
        });

        let error = create_custom_reminder(&app_state, 123, &payload)
            .await
            .unwrap_err();

        assert_eq!(error, "embed_title exceeds 256 characters");
    }

    #[tokio::test]
    async fn test_update_custom_reminder_validates_content_lengths() {
        let app_state = create_test_app_state().await;
        let create_payload = json!({
            "name": "test reminder",
            "schedule_time": "12:30",
            "message_content": "ok",
            "embed_title": "ok",
            "embed_description": "ok",
        });

        let created = create_custom_reminder(&app_state, 123, &create_payload)
            .await
            .unwrap();
        let reminder_id = created["id"].as_i64().unwrap();

        let update_payload = json!({
            "embed_description": "x".repeat(4097),
        });

        let error = update_custom_reminder(&app_state, 123, reminder_id, &update_payload)
            .await
            .unwrap_err();

        assert_eq!(error, "embed_description exceeds 4096 characters");
    }
}
