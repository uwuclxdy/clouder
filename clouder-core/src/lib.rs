//! Clouder Core
//!
//! Shared code between the Discord bot and web dashboard.
//! Contains configuration, database models, and shared business logic.

pub mod config;
pub mod database;
pub mod shared;
pub mod utils;

// Re-export commonly used types
pub use config::{AppState, Config};
pub use database::dashboard_users::DashboardUser;
