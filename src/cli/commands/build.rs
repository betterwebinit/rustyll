use log::{info, error, LevelFilter};
use std::path::PathBuf;

use crate::builder;
use crate::config;
use crate::cli::types::Commands;
use crate::cli::logging::set_log_level;

/// Handle the build command
pub async fn handle_build_command(
    command: &Commands,
    source: Option<&PathBuf>,
    destination: Option<&PathBuf>,
    layouts: Option<&PathBuf>,
    safe_mode: bool,
) {
    if let Commands::Build {
        watch,
        baseurl,
        drafts,
        unpublished,
        verbose,
        quiet,
        config: cfg_files,
        source: build_source,
        destination: build_destination,
        debug,
        ..
    } = command {
        // Set log level based on command line options
        if *verbose {
            set_log_level(LevelFilter::Debug);
        } else if *quiet {
            set_log_level(LevelFilter::Error);
        } else if *debug {
            set_log_level(LevelFilter::Debug);
        }

        // Create configuration
        let mut config_paths: Vec<&str> = Vec::new();
        if let Some(cfg_files) = cfg_files {
            config_paths = cfg_files.iter().map(|s| s.as_str()).collect();
        }

        let mut config = match config::load_config(PathBuf::from("."), Some(config_paths.iter().map(|p| PathBuf::from(*p)).collect())) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load config: {}", e);
                return;
            }
        };

        // Override config with command line arguments if provided
        // Build-specific options take precedence over global options
        if build_source.is_some() {
            config.source = build_source.clone().unwrap();
        } else if source.is_some() {
            config.source = source.unwrap().clone();
        }
        
        if build_destination.is_some() {
            config.destination = build_destination.clone().unwrap();
        } else if destination.is_some() {
            config.destination = destination.unwrap().clone();
        }
        
        if let Some(layouts_dir) = layouts { 
            config.layouts_dir = layouts_dir.clone(); 
        }
        
        config.safe_mode = safe_mode;
        
        // Apply baseurl if provided
        if let Some(base) = baseurl {
            config.base_url = base.clone();
        }

        // Build the site
        info!("Building site...");
        match builder::build_site(&config, *drafts, *unpublished) {
            Ok(_) => info!("Site built successfully at {}", config.destination.display()),
            Err(e) => error!("Failed to build site: {}", e),
        }

        // Watch for changes if requested
        if *watch {
            info!("Watching for changes...");
            match builder::watch_site(&config, *drafts, *unpublished) {
                Ok(_) => {},
                Err(e) => error!("Error watching for changes: {}", e),
            }
        }
    }
} 