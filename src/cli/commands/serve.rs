use log::{info, error, LevelFilter};
use std::path::PathBuf;

use crate::builder;
use crate::config;
use crate::server;
use crate::cli::types::Commands;
use crate::cli::logging::set_log_level;
use crate::server::config::ServerConfig;

/// Handle the serve command
pub async fn handle_serve_command(
    command: &Commands,
    source: Option<&PathBuf>,
    destination: Option<&PathBuf>,
    layouts: Option<&PathBuf>,
    safe_mode: bool,
) {
    if let Commands::Serve {
        host,
        port,
        open_url,
        watch,
        livereload,
        verbose,
        config: cfg_files,
        drafts,
        unpublished,
        source: serve_source,
        destination: serve_destination,
        baseurl,
    } = command {
        if *verbose {
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
        // Command-specific options take precedence over global options
        if serve_source.is_some() {
            config.source = serve_source.clone().unwrap();
        } else if source.is_some() {
            config.source = source.unwrap().clone();
        }
        
        if serve_destination.is_some() {
            config.destination = serve_destination.clone().unwrap();
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

        // First build the site
        info!("Building site before serving...");
        match builder::build_site(&config, *drafts, *unpublished) {
            Ok(_) => info!("Site built successfully at {}", config.destination.display()),
            Err(e) => {
                error!("Failed to build site: {}", e);
                return;
            }
        }

        // Start server
        info!("Starting server at http://{}:{}", host, port);
        let server_config = ServerConfig::new(host, *port, *livereload)
            .with_open_url(*open_url);

        // If watching for changes, start a watcher thread
        if *watch {
            if let Err(e) = server::serve_with_watch(&server_config, &config, *drafts, *unpublished).await {
                error!("Server error: {}", e);
            }
        } else {
            if let Err(e) = server::serve(&server_config, &config, *drafts, *unpublished).await {
                error!("Server error: {}", e);
            }
        }
    }
} 