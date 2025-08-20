#[cfg(test)]
mod tests {
    use crate::commands::help::{get_all_commands, truncate_description, CommandCategory};
    use super::*;

    #[test]
    fn test_command_registry_not_empty() {
        let commands = get_all_commands();
        assert!(!commands.is_empty(), "Command registry should not be empty");
    }

    #[test]
    fn test_all_categories_represented() {
        let commands = get_all_commands();
        let categories: std::collections::HashSet<_> = commands
            .iter()
            .map(|cmd| cmd.category.clone())
            .collect();

        assert!(categories.contains(&CommandCategory::Core));
        assert!(categories.contains(&CommandCategory::Info));
    }

    #[test]
    fn test_truncate_description() {
        assert_eq!(truncate_description("short", 10), "short");
        assert_eq!(truncate_description("this is a very long description", 10), "this is...");
    }
}
