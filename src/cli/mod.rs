pub mod types;
pub mod commands;
pub mod logging;

use clap::Parser;
use crate::config;
use std::path::PathBuf;

/// Run the command-line interface
pub async fn run() {
    let cli = types::Cli::parse();
    
    // Initialize logging system
    logging::init_logging(cli.debug || cli.verbose);
    
    // Configure backtrace
    logging::configure_backtrace(cli.trace);
    

    match &cli.command {
        Some(types::Commands::Build { .. }) => {
            commands::handle_build_command(
                &cli.command.as_ref().unwrap(),
                cli.source.as_ref(),
                cli.destination.as_ref(),
                cli.layouts.as_ref(),
                cli.safe
            ).await;
        },
        Some(types::Commands::Serve { .. }) => {
            commands::handle_serve_command(
                &cli.command.as_ref().unwrap(),
                cli.source.as_ref(),
                cli.destination.as_ref(),
                cli.layouts.as_ref(),
                cli.safe
            ).await;
        },
        Some(types::Commands::Clean {}) => {
            commands::handle_clean_command();
        },
        Some(types::Commands::Report { .. }) => {
            commands::handle_report_command(
                &cli.command.as_ref().unwrap(),
                cli.source.as_ref()
            ).await;
        },
        Some(types::Commands::Migrate(_)) => {
            commands::handle_migrate_command(
                &cli.command.as_ref().unwrap(),
                cli.source.as_ref(),
                cli.destination.as_ref()
            ).await;
        },
        Some(types::Commands::Config { .. }) => {
            commands::handle_config_command(&cli.command.as_ref().unwrap()).await;
        },
        Some(types::Commands::Cache { .. }) => {
            commands::handle_cache_command(&cli.command.as_ref().unwrap()).await;
        },
        Some(types::Commands::Theme { .. }) => {
            commands::handle_theme_command(&cli.command.as_ref().unwrap()).await;
        },
        Some(types::Commands::Plugin { .. }) => {
            commands::handle_plugin_command(&cli.command.as_ref().unwrap()).await;
        },
        Some(types::Commands::Completions { .. }) => {
            commands::handle_completions_command(&cli.command.as_ref().unwrap()).await;
        },
        Some(types::Commands::New { .. }) => {
            commands::handle_new_command(
                &cli.command.as_ref().unwrap()
            ).await;
        },
        None => {
            // Default to build command if none provided
            let config = match config::load_config(PathBuf::from("."), None) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::error!("Failed to load config: {}", e);
                    return;
                }
            };

            log::info!("Building site...");
            match crate::builder::build_site(&config, false, false) {
                Ok(_) => log::info!("Site built successfully at {}", config.destination.display()),
                Err(e) => log::error!("Failed to build site: {}", e),
            }
        }
    }
} 
