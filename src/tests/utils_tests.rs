#[cfg(test)]
mod tests {
    use crate::utils::{format_duration, validate_role_hierarchy};

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1), "1s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(120), "2m 0s");
        assert_eq!(format_duration(150), "2m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(7200), "2h 0m 0s");
        assert_eq!(format_duration(7260), "2h 1m 0s");
        assert_eq!(format_duration(7290), "2h 1m 30s");
    }

    #[test]
    fn test_format_duration_edge_cases() {
        // Test edge cases
        assert_eq!(format_duration(61), "1m 1s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(86400), "24h 0m 0s"); // 1 day = 24 hours
    }

    #[test]
    fn test_format_duration_large_values() {
        // Test very large durations (converted to hours)
        assert_eq!(format_duration(90000), "25h 0m 0s"); // 25 hours
        assert_eq!(format_duration(604800), "168h 0m 0s"); // 1 week = 168 hours
    }

    #[test]
    fn test_validate_role_hierarchy_valid() {
        // Test valid role hierarchy scenarios
        
        // Bot can assign role below its position
        assert!(validate_role_hierarchy(10, 5)); // bot_pos=10, target_role=5
        
        // Bot with high position can assign lower roles
        assert!(validate_role_hierarchy(100, 50));
        
        // Bot with position 1 can assign role at position 0
        assert!(validate_role_hierarchy(1, 0));
    }

    #[test]
    fn test_validate_role_hierarchy_invalid() {
        // Test invalid role hierarchy scenarios
        
        // Bot cannot assign role above its own position
        assert!(!validate_role_hierarchy(5, 10)); // bot_pos=5, target_role=10
        
        // Bot cannot assign role at same position
        assert!(!validate_role_hierarchy(10, 10));
        
        // Bot at position 0 cannot assign any positive roles
        assert!(!validate_role_hierarchy(0, 1));
    }

    #[test]
    fn test_validate_role_hierarchy_edge_cases() {
        // Test edge cases
        
        // Very high positions
        assert!(validate_role_hierarchy(1000, 500));
        assert!(!validate_role_hierarchy(500, 1000));
        
        // Negative positions (Discord uses i16)
        assert!(validate_role_hierarchy(0, -1));
        assert!(!validate_role_hierarchy(-1, 0));
        
        // Same positions should fail
        assert!(!validate_role_hierarchy(5, 5));
    }

    #[test]
    fn test_validate_role_hierarchy_boundary_conditions() {
        // Test boundary conditions for role hierarchy
        
        // Minimum difference
        assert!(validate_role_hierarchy(1, 0));
        assert!(!validate_role_hierarchy(0, 1));
        
        // Maximum i16 values
        assert!(validate_role_hierarchy(i16::MAX, i16::MAX - 1));
        assert!(!validate_role_hierarchy(i16::MAX - 1, i16::MAX));
        
        // Minimum i16 values
        assert!(validate_role_hierarchy(0, i16::MIN));
        assert!(!validate_role_hierarchy(i16::MIN, 0));
    }

    #[test]
    fn test_duration_formatting_consistency() {
        // Test that duration formatting is consistent across ranges
        let test_cases = vec![
            (1, "1s"),
            (59, "59s"),
            (60, "1m 0s"),
            (61, "1m 1s"),
            (3600, "1h 0m 0s"),
            (3661, "1h 1m 1s"),
        ];
        
        for (seconds, expected) in test_cases {
            assert_eq!(format_duration(seconds), expected);
        }
    }

    #[test]
    fn test_role_hierarchy_realistic_scenarios() {
        // Test realistic Discord role hierarchy scenarios
        
        // Admin bot (position 50) assigning member roles (position 10)
        assert!(validate_role_hierarchy(50, 10));
        
        // Moderator bot (position 25) assigning basic roles (position 5)
        assert!(validate_role_hierarchy(25, 5));
        
        // Bot trying to assign admin role above its position (should fail)
        assert!(!validate_role_hierarchy(25, 45));
        
        // Bot trying to assign role at same level (should fail)
        assert!(!validate_role_hierarchy(30, 30));
        
        // Owner-level bot operations (very high positions)
        assert!(validate_role_hierarchy(100, 50));
    }

    #[test]
    fn test_utils_error_handling() {
        // Test that utility functions handle edge cases gracefully
        
        // Very large numbers (should not panic)
        let result = format_duration(u64::MAX);
        assert!(!result.is_empty());
        
        // Role hierarchy with extreme values
        assert!(validate_role_hierarchy(i16::MAX, i16::MIN));
        assert!(!validate_role_hierarchy(i16::MIN, i16::MAX));
    }

    #[test]
    fn test_format_duration_zero() {
        // Test zero duration specifically
        assert_eq!(format_duration(0), "0s");
    }

    #[test]
    fn test_validate_role_hierarchy_equals() {
        // Test that equal positions always return false
        for pos in [-100, -1, 0, 1, 50, 100] {
            assert!(!validate_role_hierarchy(pos, pos));
        }
    }

    #[test]
    fn test_format_duration_no_leading_zeros() {
        // Test that there are no unnecessary leading zeros
        let result = format_duration(3600);
        assert_eq!(result, "1h 0m 0s");
        
        let result = format_duration(60);
        assert_eq!(result, "1m 0s");
        
        // Verify no extra whitespace
        assert!(!result.starts_with(' '));
        assert!(!result.ends_with(' '));
    }

    #[test]
    fn test_validate_role_hierarchy_comprehensive() {
        // Comprehensive test matrix
        let test_cases = vec![
            (10, 5, true),   // bot higher than target
            (5, 10, false),  // bot lower than target
            (10, 10, false), // bot equal to target
            (0, 0, false),   // both at zero
            (-5, -10, true), // negative positions, bot higher
            (-10, -5, false), // negative positions, bot lower
            (5, -5, true),   // positive bot, negative target
            (-5, 5, false),  // negative bot, positive target
        ];
        
        for (bot_pos, target_pos, expected) in test_cases {
            assert_eq!(
                validate_role_hierarchy(bot_pos, target_pos), 
                expected,
                "Failed for bot_pos: {}, target_pos: {}", 
                bot_pos, 
                target_pos
            );
        }
    }
}