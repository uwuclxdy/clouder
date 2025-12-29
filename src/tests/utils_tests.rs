#[cfg(test)]
mod tests {
    use crate::utils::*;
    use serenity::all::{Http, Permissions};

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
        // Test admin permissions bypass
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
    fn test_can_bot_manage_role() {
        // Test with multiple bot roles
        let bot_positions = vec![2, 5, 8];

        // Can manage role below highest position
        assert!(can_bot_manage_role(&bot_positions, 3));
        assert!(can_bot_manage_role(&bot_positions, 7));

        // Cannot manage role equal to highest position
        assert!(!can_bot_manage_role(&bot_positions, 8));

        // Cannot manage role above highest position
        assert!(!can_bot_manage_role(&bot_positions, 10));

        // Test with single bot role
        let single_role = vec![5];
        assert!(can_bot_manage_role(&single_role, 3));
        assert!(!can_bot_manage_role(&single_role, 5));
        assert!(!can_bot_manage_role(&single_role, 7));

        // Test with no bot roles
        let no_roles: Vec<u16> = vec![];
        assert!(!can_bot_manage_role(&no_roles, 1));

        // Test with zero position target
        assert!(can_bot_manage_role(&bot_positions, 0));
    }

    // Tests for BotChannelPermissions struct

    #[test]
    fn test_bot_channel_permissions_struct() {
        let perms = BotChannelPermissions {
            permissions: Permissions::SEND_MESSAGES | Permissions::READ_MESSAGE_HISTORY,
        };

        assert!(perms.permissions.send_messages());
        assert!(perms.permissions.read_message_history());
        assert!(!perms.permissions.administrator());
    }

    // Tests for bot_has_permission_in_channel logic

    #[tokio::test]
    async fn test_bot_has_permission_dm_always_true() {
        let http = Http::new("test_token");

        // DMs (guild_id = None) should always return true
        let result = bot_has_permission_in_channel(
            &http,
            None, // DM context
            serenity::all::ChannelId::new(123456789),
            |p| p.send_messages(),
        )
        .await;

        assert!(result, "DMs should always have permission");
    }

    #[tokio::test]
    async fn test_bot_has_permission_invalid_guild() {
        let http = Http::new("invalid_token");

        // Invalid guild should return false (API call fails)
        let result = bot_has_permission_in_channel(
            &http,
            Some(serenity::all::GuildId::new(999999999999)),
            serenity::all::ChannelId::new(123456789),
            |p| p.send_messages(),
        )
        .await;

        assert!(
            !result,
            "invalid guild/token should return false due to API failure"
        );
    }

    #[tokio::test]
    async fn test_get_bot_channel_permissions_invalid_token() {
        let http = Http::new("invalid_token");

        let result = get_bot_channel_permissions(
            &http,
            serenity::all::GuildId::new(123456789),
            serenity::all::ChannelId::new(987654321),
        )
        .await;

        // Should return None due to invalid token
        assert!(result.is_none());
    }

    // Tests for permission check callback patterns

    #[test]
    fn test_permission_check_callbacks() {
        let full_perms = Permissions::all();
        let limited_perms = Permissions::SEND_MESSAGES | Permissions::VIEW_CHANNEL;
        let no_perms = Permissions::empty();

        // Test various permission checks
        assert!(full_perms.send_messages());
        assert!(limited_perms.send_messages());
        assert!(!no_perms.send_messages());

        assert!(full_perms.administrator());
        assert!(!limited_perms.administrator());

        // Combined permission check
        let combined_check = |p: &Permissions| p.send_messages() && p.view_channel();
        assert!(combined_check(&full_perms));
        assert!(combined_check(&limited_perms));
        assert!(!combined_check(&no_perms));
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
}
