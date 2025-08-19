#[cfg(test)]
mod tests {
    use crate::utils::*;
    use serenity::all::{Member, Permissions, Role, RoleId, UserId, GuildId};
    use chrono::{DateTime, FixedOffset};

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
        assert_eq!(format_duration(7323), "2h 2m 3s");
    }

    #[test]
    fn test_validate_role_hierarchy() {
        // Bot role is higher than target role - should be true
        assert!(validate_role_hierarchy(10, 5));
        assert!(validate_role_hierarchy(100, 1));
        
        // Bot role is equal to target role - should be false
        assert!(!validate_role_hierarchy(5, 5));
        
        // Bot role is lower than target role - should be false
        assert!(!validate_role_hierarchy(5, 10));
        assert!(!validate_role_hierarchy(1, 100));
    }

    #[test]
    fn test_can_bot_manage_role() {
        // Bot has role higher than target
        let bot_positions = vec![10, 5, 3];
        assert!(can_bot_manage_role(&bot_positions, 2));
        assert!(can_bot_manage_role(&bot_positions, 4));
        assert!(can_bot_manage_role(&bot_positions, 9));
        
        // Bot has no roles higher than target
        assert!(!can_bot_manage_role(&bot_positions, 10));
        assert!(!can_bot_manage_role(&bot_positions, 15));
        
        // Empty bot positions
        let empty_positions = vec![];
        assert!(!can_bot_manage_role(&empty_positions, 1));
        
        // Edge case: exact match positions
        let bot_positions = vec![5];
        assert!(!can_bot_manage_role(&bot_positions, 5));
    }

    #[test]
    fn test_can_bot_manage_roles_in_guild_admin() {
        // Create mock data for admin bot
        let admin_permissions = Permissions::ADMINISTRATOR;
        let mock_member = Member {
            user: serenity::all::User {
                id: UserId::new(12345),
                name: "TestBot".to_string(),
                discriminator: 0,
                avatar: None,
                bot: true,
                system: false,
                mfa_enabled: false,
                banner: None,
                accent_colour: None,
                locale: None,
                verified: None,
                email: None,
                flags: None,
                premium_type: None,
                public_flags: None,
                avatar_decoration: None,
                global_name: None,
            },
            nick: None,
            roles: vec![RoleId::new(67890)],
            joined_at: None,
            premium_since: None,
            deaf: false,
            mute: false,
            flags: serenity::all::GuildMemberFlags::empty(),
            pending: false,
            permissions: Some(admin_permissions),
            communication_disabled_until: None,
            avatar: None,
            banner: None,
        };

        let guild_roles = vec![
            Role {
                id: RoleId::new(67890),
                name: "Admin".to_string(),
                colour: serenity::all::Colour::new(0),
                hoist: false,
                icon: None,
                managed: false,
                mentionable: false,
                permissions: admin_permissions,
                position: 10,
                flags: serenity::all::RoleFlags::empty(),
                tags: None,
                unicode_emoji: None,
            }
        ];

        let (is_admin, positions) = can_bot_manage_roles_in_guild(&mock_member, &guild_roles);
        assert!(is_admin);
        assert!(positions.is_empty()); // Admin returns empty positions
    }

    #[test]
    fn test_can_bot_manage_roles_in_guild_non_admin() {
        // Create mock data for non-admin bot
        let non_admin_permissions = Permissions::SEND_MESSAGES | Permissions::READ_MESSAGE_HISTORY;
        let mock_member = Member {
            user: serenity::all::User {
                id: UserId::new(12345),
                name: "TestBot".to_string(),
                discriminator: 0,
                avatar: None,
                bot: true,
                system: false,
                mfa_enabled: false,
                banner: None,
                accent_colour: None,
                locale: None,
                verified: None,
                email: None,
                flags: None,
                premium_type: None,
                public_flags: None,
                avatar_decoration: None,
                global_name: None,
            },
            nick: None,
            roles: vec![RoleId::new(11111), RoleId::new(22222)],
            joined_at: None,
            premium_since: None,
            deaf: false,
            mute: false,
            flags: serenity::all::GuildMemberFlags::empty(),
            pending: false,
            permissions: Some(non_admin_permissions),
            communication_disabled_until: None,
            avatar: None,
            banner: None,
        };

        let guild_roles = vec![
            Role {
                id: RoleId::new(11111),
                name: "Moderator".to_string(),
                colour: serenity::all::Colour::new(0),
                hoist: false,
                icon: None,
                managed: false,
                mentionable: false,
                permissions: non_admin_permissions,
                position: 8,
                flags: serenity::all::RoleFlags::empty(),
                tags: None,
                unicode_emoji: None,
            },
            Role {
                id: RoleId::new(22222),
                name: "Helper".to_string(),
                colour: serenity::all::Colour::new(0),
                hoist: false,
                icon: None,
                managed: false,
                mentionable: false,
                permissions: non_admin_permissions,
                position: 5,
                flags: serenity::all::RoleFlags::empty(),
                tags: None,
                unicode_emoji: None,
            }
        ];

        let (is_admin, positions) = can_bot_manage_roles_in_guild(&mock_member, &guild_roles);
        assert!(!is_admin);
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&8));
        assert!(positions.contains(&5));
    }

    #[test]
    fn test_format_discord_timestamp() {
        let test_time = "2025-08-19T16:05:00Z";
        
        // Test all timestamp formats
        assert!(format_discord_timestamp(test_time, 'F').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'F').ends_with(":F>"));
        
        assert!(format_discord_timestamp(test_time, 'f').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'f').ends_with(":f>"));
        
        assert!(format_discord_timestamp(test_time, 'D').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'D').ends_with(":D>"));
        
        assert!(format_discord_timestamp(test_time, 'd').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'd').ends_with(":d>"));
        
        assert!(format_discord_timestamp(test_time, 't').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 't').ends_with(":t>"));
        
        assert!(format_discord_timestamp(test_time, 'T').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'T').ends_with(":T>"));
        
        assert!(format_discord_timestamp(test_time, 'R').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'R').ends_with(":R>"));
        
        // Test default format
        assert!(format_discord_timestamp(test_time, 'X').starts_with("<t:"));
        assert!(format_discord_timestamp(test_time, 'X').ends_with(":f>"));
        
        // Test invalid timestamp
        assert_eq!(format_discord_timestamp("invalid", 'F'), "Invalid timestamp");
        assert_eq!(format_discord_timestamp("", 'F'), "Invalid timestamp");
        assert_eq!(format_discord_timestamp("not-a-date", 'F'), "Invalid timestamp");
    }

    #[test]
    fn test_format_discord_timestamp_specific_values() {
        // Test with a specific known timestamp
        let test_time = "2025-08-19T16:05:00+00:00";
        let expected_timestamp = 1755398700; // Known timestamp for this date
        
        assert_eq!(
            format_discord_timestamp(test_time, 'F'),
            format!("<t:{}:F>", expected_timestamp)
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test empty bot role positions
        let empty_positions: Vec<u16> = vec![];
        assert!(!can_bot_manage_role(&empty_positions, 0));
        assert!(!can_bot_manage_role(&empty_positions, 1));
        
        // Test zero position roles
        let zero_positions = vec![0];
        assert!(!can_bot_manage_role(&zero_positions, 0));
        assert!(!can_bot_manage_role(&zero_positions, 1));
        
        // Test high position numbers
        let high_positions = vec![65535];
        assert!(can_bot_manage_role(&high_positions, 65534));
        assert!(!can_bot_manage_role(&high_positions, 65535));
        
        // Test format_duration with large numbers
        assert_eq!(format_duration(86400), "24h 0m 0s"); // 1 day
        assert_eq!(format_duration(90061), "25h 1m 1s"); // > 1 day
    }
}