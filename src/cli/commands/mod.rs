mod build;
mod serve;
mod clean;
mod report;
mod migrate;
mod new;
mod config;
mod cache;
mod theme;
mod plugin;
mod completions;

pub use build::handle_build_command;
pub use serve::handle_serve_command;
pub use clean::handle_clean_command;
pub use report::handle_report_command;
pub use migrate::handle_migrate_command;
pub use new::handle_new_command;
pub use config::handle_config_command;
pub use cache::handle_cache_command;
pub use theme::handle_theme_command;
pub use plugin::handle_plugin_command;
pub use completions::handle_completions_command;

