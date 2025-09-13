use log::{info, error};
use std::path::PathBuf;

use crate::config;
use crate::directory;

/// Handle the clean command
pub fn handle_clean_command() {
    // Create configuration
    let config = match config::load_config(PathBuf::from("."), None) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load config: {}", e);
            return;
        }
    };

    info!("Cleaning site at {}", config.destination.display());
    match directory::clean_destination(&config) {
        Ok(_) => info!("Site cleaned successfully"),
        Err(e) => error!("Failed to clean site: {}", e),
    }
} 