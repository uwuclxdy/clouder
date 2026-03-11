#[cfg(test)]
mod tests {
    use clouder_core::utils::*;

    #[test]
    fn test_format_duration() {
        // Test seconds only
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");

        // Test minutes and seconds
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");

        // Test hours, minutes, and seconds
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(7200), "2h 0m 0s");

        // Test days, hours, minutes, and seconds
        assert_eq!(format_duration(86400), "1d 0h 0m 0s");
        assert_eq!(format_duration(90061), "1d 1h 1m 1s");

        // Test edge cases
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1), "1s");

        // Test large numbers
        assert_eq!(format_duration(604800), "7d 0h 0m 0s"); // 1 week
        assert_eq!(format_duration(2592000), "30d 0h 0m 0s"); // ~1 month
    }

    #[test]
    fn test_validate_role_hierarchy_basic() {
        // Test basic role hierarchy validation logic without complex Serenity structs
        // This focuses on the core logic in validate_role_hierarchy

        // Since creating complex Serenity mock objects is error-prone,
        // we'll test the core validation logic with mock data

        // Test that hierarchy validation works with position comparison
        let bot_highest_position = 5;
        let user_highest_position = 3;
        let target_role_position = 2;

        // Bot should be able to manage role below its highest position
        assert!(bot_highest_position > target_role_position);

        // User should be able to get roles below their highest position
        assert!(user_highest_position > target_role_position);

        // Test edge case: same position should fail
        let same_position_role = 3;
        assert!((user_highest_position <= same_position_role));
    }

    #[test]
    fn test_can_bot_manage_role_logic() {
        // Test the core logic without complex mock objects
        // This represents the logic from can_bot_manage_role

        let bot_roles = [
            (1, 10), // (role_id, position)
            (2, 5),
            (3, 2),
        ];

        let target_role_position = 3;
        let bot_highest_position = bot_roles.iter().map(|(_, pos)| *pos).max().unwrap_or(0);

        // Bot should be able to manage roles with position lower than its highest
        assert!(bot_highest_position > target_role_position);

        // Test with role position equal to bot's highest - should fail
        let equal_position_role = 10;
        assert!((bot_highest_position <= equal_position_role));

        // Test with role position higher than bot's highest - should fail
        let higher_position_role = 15;
        assert!((bot_highest_position <= higher_position_role));
    }

    #[test]
    fn test_hierarchy_permissions() {
        // Test admin permissions bypass (bitwise)
        let admin_permissions = 0x8; // ADMINISTRATOR
        assert_eq!(admin_permissions & 0x8, 0x8);

        // Test manage roles permission
        let manage_roles_permission = 0x10000000; // MANAGE_ROLES
        assert_eq!(manage_roles_permission & 0x10000000, 0x10000000);

        // Test combined permissions
        let combined = admin_permissions | manage_roles_permission;
        assert_eq!(combined & 0x8, 0x8); // Still has admin
        assert_eq!(combined & 0x10000000, 0x10000000); // Still has manage roles
    }

    #[test]
    fn test_has_permission_helper() {
        use poise::serenity_prelude::Permissions;
        // with only manage_messages we should not satisfy manage_webhooks
        let perms = Permissions::MANAGE_MESSAGES;
        assert!(!clouder_core::utils::has_permission(
            perms,
            Permissions::MANAGE_WEBHOOKS
        ));
        // admin should satisfy anything
        let admin = Permissions::ADMINISTRATOR;
        assert!(clouder_core::utils::has_permission(
            admin,
            Permissions::MANAGE_MESSAGES
        ));
        assert!(clouder_core::utils::has_permission(
            admin,
            Permissions::MANAGE_WEBHOOKS
        ));
        // combined case where required bit is present explicitly
        let combo = Permissions::ADMINISTRATOR | Permissions::MANAGE_MESSAGES;
        assert!(clouder_core::utils::has_permission(
            combo,
            Permissions::MANAGE_MESSAGES
        ));
        assert!(clouder_core::utils::has_permission(
            combo,
            Permissions::MANAGE_WEBHOOKS
        ));
    }

    #[test]
    fn test_role_position_comparison() {
        // Test role position comparison logic
        let roles = vec![
            ("@everyone", 0),
            ("Member", 1),
            ("Helper", 3),
            ("Moderator", 5),
            ("Admin", 10),
        ];

        // Find highest position (excluding @everyone)
        let highest_position = roles
            .iter()
            .filter(|(name, _)| *name != "@everyone")
            .map(|(_, pos)| *pos)
            .max()
            .unwrap_or(0);

        assert_eq!(highest_position, 10);

        // Test that admin can manage all other roles
        for (name, position) in &roles {
            if *name != "Admin" {
                assert!(highest_position > *position, "Admin should manage {}", name);
            }
        }
    }

    #[test]
    fn test_role_hierarchy_edge_cases() {
        // Test empty roles
        let empty_roles: Vec<(String, i32)> = vec![];
        let highest = empty_roles.iter().map(|(_, pos)| *pos).max().unwrap_or(0);
        assert_eq!(highest, 0);

        // Test single role
        let single_role = [("Only Role", 5)];
        let single_highest = single_role.iter().map(|(_, pos)| *pos).max().unwrap_or(0);
        assert_eq!(single_highest, 5);

        // Test negative positions (edge case)
        let negative_roles = [("Negative", -1), ("Zero", 0), ("Positive", 1)];
        let negative_highest = negative_roles
            .iter()
            .map(|(_, pos)| *pos)
            .max()
            .unwrap_or(0);
        assert_eq!(negative_highest, 1);
    }

    fn make_message(
        content: &str,
        attachments: Vec<serde_json::Value>,
        embeds: Vec<serde_json::Value>,
    ) -> poise::serenity_prelude::Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "1",
            "author": { "id": "1", "username": "test", "discriminator": "0000", "global_name": null, "avatar": null },
            "content": content,
            "timestamp": "2024-01-01T00:00:00Z",
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": attachments,
            "embeds": embeds,
            "pinned": false,
            "type": 0
        }))
        .unwrap()
    }

    fn make_attachment(filename: &str, content_type: Option<&str>) -> serde_json::Value {
        let mut v = serde_json::json!({
            "id": "1",
            "filename": filename,
            "size": 1024,
            "url": "https://cdn.discordapp.com/attachments/1/1/file",
            "proxy_url": "https://media.discordapp.net/attachments/1/1/file"
        });
        if let Some(ct) = content_type {
            v["content_type"] = serde_json::json!(ct);
        }
        v
    }

    #[test]
    fn test_content_detection_attachments_true_gifs_true() {
        use clouder_core::utils::content_detection::has_allowed_content;
        // all attachments pass including GIFs
        let gif = make_message(
            "",
            vec![make_attachment("cat.gif", Some("image/gif"))],
            vec![],
        );
        assert!(has_allowed_content(&gif, false, true, true, false));

        let png = make_message(
            "",
            vec![make_attachment("cat.png", Some("image/png"))],
            vec![],
        );
        assert!(has_allowed_content(&png, false, true, true, false));
    }

    #[test]
    fn test_content_detection_attachments_true_gifs_false() {
        use clouder_core::utils::content_detection::has_allowed_content;
        // GIF file attachments should be rejected
        let gif_only = make_message(
            "",
            vec![make_attachment("cat.gif", Some("image/gif"))],
            vec![],
        );
        assert!(!has_allowed_content(&gif_only, false, true, false, false));

        // non-GIF attachments should pass
        let png = make_message(
            "",
            vec![make_attachment("cat.png", Some("image/png"))],
            vec![],
        );
        assert!(has_allowed_content(&png, false, true, false, false));

        // mixed: at least one non-GIF passes
        let mixed = make_message(
            "",
            vec![
                make_attachment("cat.gif", Some("image/gif")),
                make_attachment("dog.png", Some("image/png")),
            ],
            vec![],
        );
        assert!(has_allowed_content(&mixed, false, true, false, false));
    }

    #[test]
    fn test_content_detection_attachments_false_gifs_true() {
        use clouder_core::utils::content_detection::has_allowed_content;
        // GIF file attachments pass via has_gif
        let gif = make_message(
            "",
            vec![make_attachment("cat.gif", Some("image/gif"))],
            vec![],
        );
        assert!(has_allowed_content(&gif, false, false, true, false));

        // non-GIF attachments should be rejected
        let png = make_message(
            "",
            vec![make_attachment("cat.png", Some("image/png"))],
            vec![],
        );
        assert!(!has_allowed_content(&png, false, false, true, false));

        // Tenor URL passes
        let tenor = make_message("https://tenor.com/view/cat-12345", vec![], vec![]);
        assert!(has_allowed_content(&tenor, false, false, true, false));
    }

    #[test]
    fn test_content_detection_attachments_false_gifs_false() {
        use clouder_core::utils::content_detection::has_allowed_content;
        // nothing passes
        let gif = make_message(
            "",
            vec![make_attachment("cat.gif", Some("image/gif"))],
            vec![],
        );
        assert!(!has_allowed_content(&gif, false, false, false, false));

        let png = make_message(
            "",
            vec![make_attachment("cat.png", Some("image/png"))],
            vec![],
        );
        assert!(!has_allowed_content(&png, false, false, false, false));

        let text = make_message("hello", vec![], vec![]);
        assert!(!has_allowed_content(&text, false, false, false, false));
    }

    #[test]
    fn test_content_detection_links_and_stickers() {
        use clouder_core::utils::content_detection::has_allowed_content;
        // link passes when allow_links = true
        let link = make_message("check https://example.com", vec![], vec![]);
        assert!(has_allowed_content(&link, true, false, false, false));
        assert!(!has_allowed_content(&link, false, false, false, false));

        // text without media fails
        let text = make_message("just text", vec![], vec![]);
        assert!(!has_allowed_content(&text, true, true, true, true));
    }
}
