use axum::Router;
use std::path::PathBuf;
use std::sync::{Arc, mpsc::{channel, Receiver}};
use std::time::Duration;
use log::{info, error};
use std::io;
use std::net::SocketAddr;
use tokio::signal;
use axum_server::tls_rustls::RustlsConfig;

use crate::config::Config;
use crate::builder::build_site;
use crate::server::types::BoxResult;
use crate::server::config::ServerConfig as ServerOpts;
use crate::server::app::create_app;
use crate::server::livereload::watch_directory;
use crate::server::utils::browser::open_browser;
use crate::server::core::watcher::handle_file_changes;

/// Start a server with watching for file changes
pub async fn serve_with_watch(
    server_config: &ServerOpts, 
    config: &Config,
    include_drafts: bool,
    include_unpublished: bool
) -> BoxResult<()> {
    // Clone the destination to avoid the borrowed data escaping outside of function
    let destination = config.destination.clone();
    let address = server_config.address_string();
    
    // First perform initial build unless skipped
    if !server_config.skip_initial_build {
        info!("Building site before serving...");
        build_site(config, include_drafts, include_unpublished)?;
    } else {
        info!("Skipping initial site build as requested");
    }
    
    info!("Starting server at {}", server_config.url());
    info!("Serving files from {}", destination.display());
    
    // Create a channel for file change events
    let (tx, rx) = channel();
    
    // Create a file watcher with appropriate ignore patterns
    let patterns_to_watch = server_config.livereload_ignore.clone();
    let min_delay = server_config.livereload_min_delay.unwrap_or(500);
    let _watcher = watch_directory(&config.source, tx, Duration::from_millis(min_delay), &patterns_to_watch)?;
    
    // Create a router factory
    let app_factory = create_app(destination.clone(), server_config);
    let app = app_factory();
    
    // Create a server
    let addr: SocketAddr = address.parse()?;
    
    // Configure server with TLS if needed
    if let (Some(cert_path), Some(key_path)) = (&server_config.ssl_cert, &server_config.ssl_key) {
        #[cfg(feature = "tls")]
        {
            let tls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            
            let server = axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service());
            
            // Open in browser if requested
            if server_config.open_url {
                let url = server_config.url();
                info!("Opening browser at {}", url);
                if !open_browser(&url) {
                    error!("Failed to open browser automatically");
                }
            }
            
            // Process detachment if requested
            if server_config.detach {
                info!("Server running in detached mode. Press Ctrl+C to stop.");
            }
            
            // Create a thread to handle file change events and rebuild
            let config_clone = config.clone();
            let max_delay = server_config.livereload_max_delay.unwrap_or(2000);
            let _rebuild_thread = std::thread::spawn(move || {
                handle_file_changes(
                    rx, 
                    &config_clone, 
                    include_drafts, 
                    include_unpublished, 
                    Duration::from_millis(min_delay),
                    Duration::from_millis(max_delay)
                );
            });
            
            // Run the server with graceful shutdown
            tokio::select! {
                _ = server => {},
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down server...");
                },
            }
        }
        
        #[cfg(not(feature = "tls"))]
        {
            error!("TLS support was not enabled at build time. Please rebuild with 'tls' feature.");
            return Err("TLS support not available".into());
        }
    } else {
        // Plain HTTP server
        let server = axum_server::bind(addr)
            .serve(app.into_make_service());
            
        // Open in browser if requested
        if server_config.open_url {
            let url = server_config.url();
            info!("Opening browser at {}", url);
            if !open_browser(&url) {
                error!("Failed to open browser automatically");
            }
        }
        
        // Process detachment if requested
        if server_config.detach {
            // Note: Full detachment works differently in Rust than Ruby
            // This is a simpler approach that doesn't create a daemon
            info!("Server running in detached mode. Press Ctrl+C to stop.");
        }
        
        // Create a thread to handle file change events and rebuild
        let config_clone = config.clone();
        let max_delay = server_config.livereload_max_delay.unwrap_or(2000);
        let _rebuild_thread = std::thread::spawn(move || {
            handle_file_changes(
                rx, 
                &config_clone, 
                include_drafts, 
                include_unpublished, 
                Duration::from_millis(min_delay),
                Duration::from_millis(max_delay)
            );
        });
        
        // Run the server with graceful shutdown
        tokio::select! {
            _ = server => {},
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down server...");
            },
        }
    }
    
    Ok(())
}

/// Start a server without watching for file changes
pub async fn serve(
    server_config: &ServerOpts,
    config: &Config,
    include_drafts: bool,
    include_unpublished: bool
) -> BoxResult<()> {
    let address = server_config.address_string();
    let destination = config.destination.clone();
    
    // Build the site first unless skipped
    if !server_config.skip_initial_build {
        info!("Building site before serving...");
        build_site(config, include_drafts, include_unpublished)?;
    } else {
        info!("Skipping initial site build as requested");
    }
    
    info!("Starting server at {}", server_config.url());
    info!("Serving files from {}", destination.display());
    
    // Create an app factory
    let app_factory = create_app(destination.clone(), server_config);
    let app = app_factory();
    
    // Create a server
    let addr: SocketAddr = address.parse()?;
    
    // Configure server with TLS if needed
    if let (Some(cert_path), Some(key_path)) = (&server_config.ssl_cert, &server_config.ssl_key) {
        #[cfg(feature = "tls")]
        {
            let tls_config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            
            let server = axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service());
            
            // Open in browser if requested
            if server_config.open_url {
                let url = server_config.url();
                info!("Opening browser at {}", url);
                if !open_browser(&url) {
                    error!("Failed to open browser automatically");
                }
            }
            
            // Run the server with graceful shutdown
            tokio::select! {
                _ = server => {},
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down server...");
                },
            }
        }
        
        #[cfg(not(feature = "tls"))]
        {
            error!("TLS support was not enabled at build time. Please rebuild with 'tls' feature.");
            return Err("TLS support not available".into());
        }
    } else {
        // Plain HTTP server
        let server = axum_server::bind(addr)
            .serve(app.into_make_service());
            
        // Open in browser if requested
        if server_config.open_url {
            let url = server_config.url();
            info!("Opening browser at {}", url);
            if !open_browser(&url) {
                error!("Failed to open browser automatically");
            }
        }
        
        // Run the server with graceful shutdown
        tokio::select! {
            _ = server => {},
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down server...");
            },
        }
    }
    
    Ok(())
} 