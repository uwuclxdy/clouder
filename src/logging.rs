use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default to info level for all crates
        EnvFilter::new("info")
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .compact(),
        )
        .init();
}

/// Log a debug-level message.
pub use tracing::debug;

/// Log an info-level message.
pub use tracing::info;

/// Log a warn-level message.
pub use tracing::warn;

/// Log an error-level message.
pub use tracing::error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_filter_parsing() {
        // Test that various RUST_LOG formats are valid
        let filters = [
            "debug",
            "info",
            "warn",
            "error",
            "clouder=debug",
            "clouder=debug,serenity=warn",
        ];

        for filter_str in filters {
            let result = EnvFilter::try_new(filter_str);
            assert!(
                result.is_ok(),
                "Filter '{}' should be valid: {:?}",
                filter_str,
                result.err()
            );
        }
    }

    #[test]
    fn test_default_filter() {
        // Verify the default filter string is valid
        let default_filter = "info";
        let result = EnvFilter::try_new(default_filter);
        assert!(result.is_ok(), "Default filter should be valid");
    }
}
