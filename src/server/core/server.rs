use std::path::PathBuf;
use std::sync::{Arc, mpsc::channel};
use std::time::Duration;
use log::{info, error, warn};
use std::net::SocketAddr;
use tokio::signal;
// use axum_server::tls_rustls::RustlsConfig;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};
use tower_http::decompression::DecompressionLayer;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::timeout::TimeoutLayer;
use std::sync::atomic::AtomicBool;

use crate::config::Config;
use crate::builder::build_site;
use crate::server::types::BoxResult;
use crate::server::config::ServerConfig as ServerOpts;
use crate::server::app::create_app;
use crate::server::livereload::watch_directory;
use crate::server::utils::browser::open_browser;
use crate::server::core::watcher::handle_file_changes;
use crate::server::middleware::compression::create_compression_layer;
use crate::server::middleware::cache::create_cache_control_layer;
use crate::server::middleware::security::create_security_headers_layer;

/// Shared state for server control
#[derive(Debug, Clone)]
struct ServerState {
    reload_requested: Arc<AtomicBool>,
    config: Arc<Config>,
    serve_dir: PathBuf,
}

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
    
    // Create a shared state for server control
    let state = ServerState {
        reload_requested: Arc::new(AtomicBool::new(false)),
        config: Arc::new(config.clone()),
        serve_dir: destination.clone(),
    };
    
    // First perform initial build unless skipped
    if !server_config.skip_initial_build {
        info!("Building site before serving...");
        match build_site(config, include_drafts, include_unpublished) {
            Ok(_) => info!("Site built successfully"),
            Err(e) => {
                error!("Failed to build site: {}", e);
                if true { // Always rebuild on errors for now
                    warn!("Ignoring build errors and serving anyway");
                }
            }
        }
    } else {
        info!("Skipping initial site build as requested");
    }
    
    // Check if the destination directory exists
    if !destination.exists() {
        error!("Destination directory {} does not exist", destination.display());
        return Err("Destination directory not found".into());
    }
    
    info!("Starting server at {}", server_config.url());
    info!("Serving files from {}", destination.display());
    
    // Create a channel for file change events
    let (tx, rx) = channel();
    
    // Create a file watcher with appropriate ignore patterns
    let patterns_to_watch = server_config.livereload_ignore.clone();
    let min_delay = server_config.livereload_min_delay.unwrap_or(500);
    let _watcher = watch_directory(&config.source, tx, Duration::from_millis(min_delay), &patterns_to_watch)?;
    
    // Create advanced middleware stack
    let _compression = create_compression_layer(None);
    let cache_control = create_cache_control_layer(None);
    let security_headers = create_security_headers_layer();
    
    // Enable CORS for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Add timeout to prevent hanging requests
    let timeout = TimeoutLayer::new(Duration::from_secs(30));
    
    // Create a router factory with advanced middleware
    let app_factory = || {
        let base_app = create_app(destination.clone(), server_config);
        
        base_app()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(DecompressionLayer::new())
            .layer(cache_control)
            .layer(security_headers)
            .layer(cors)
            .layer(timeout)
            .layer(CatchPanicLayer::new())
    };
    
    let app = app_factory();
    
    // Create a server
    let addr: SocketAddr = address.parse()?;
    
    // Configure server with TLS if needed
    if let (Some(_cert_path), Some(_key_path)) = (&server_config.ssl_cert, &server_config.ssl_key) {
        #[cfg(feature = "tls")]
        {
            info!("Starting HTTPS server with TLS");
            // Load TLS configuration
            let tls_config = match RustlsConfig::from_pem_file(cert_path, key_path).await {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load TLS certificates: {}", e);
                    return Err(format!("TLS configuration error: {}", e).into());
                }
            };
            
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
            let _reload_requested = state.reload_requested.clone();
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
                _ = server => {
                    info!("Server stopped");
                },
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down server (received Ctrl+C)...");
                },
            }
        }
        
        #[cfg(not(feature = "tls"))]
        {
            error!("TLS support was not enabled at build time. Please rebuild with 'tls' feature.");
            return Err("TLS support not available".into());
        }
    } else {
        // Plain HTTP server with HTTP/2 support
        info!("Starting HTTP server with HTTP/2 support");
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
        let _reload_requested = state.reload_requested.clone();
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
        
        // Print server startup information
        print_server_banner(server_config);
        
        // Run the server with graceful shutdown
        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    error!("Server error: {}", e);
                } else {
                    info!("Server stopped");
                }
            },
            _ = signal::ctrl_c() => {
                info!("Shutting down server (received Ctrl+C)...");
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
        match build_site(config, include_drafts, include_unpublished) {
            Ok(_) => info!("Site built successfully"),
            Err(e) => {
                error!("Failed to build site: {}", e);
                if true { // Always rebuild on errors for now
                    warn!("Ignoring build errors and serving anyway");
                }
            }
        }
    } else {
        info!("Skipping initial site build as requested");
    }
    
    // Check if the destination directory exists
    if !destination.exists() {
        error!("Destination directory {} does not exist", destination.display());
        return Err("Destination directory not found".into());
    }
    
    info!("Starting server at {}", server_config.url());
    info!("Serving files from {}", destination.display());
    
    // Create advanced middleware stack
    let _compression = create_compression_layer(None);
    let cache_control = create_cache_control_layer(None);
    let security_headers = create_security_headers_layer();
    
    // Create an app factory with advanced middleware
    let app_factory = || {
        let base_app = create_app(destination.clone(), server_config);
        
        base_app()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(cache_control)
            .layer(security_headers)
            .layer(CatchPanicLayer::new())
    };
    
    let app = app_factory();
    
    // Create a server
    let addr: SocketAddr = address.parse()?;
    
    // Configure server with TLS if needed
    if let (Some(_cert_path), Some(_key_path)) = (&server_config.ssl_cert, &server_config.ssl_key) {
        #[cfg(feature = "tls")]
        {
            info!("Starting HTTPS server with TLS");
            let tls_config = match RustlsConfig::from_pem_file(cert_path, key_path).await {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load TLS certificates: {}", e);
                    return Err(format!("TLS configuration error: {}", e).into());
                }
            };
            
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
            
            // Print server banner
            print_server_banner(server_config);
            
            // Run the server with graceful shutdown
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("Server error: {}", e);
                    } else {
                        info!("Server stopped");
                    }
                },
                _ = signal::ctrl_c() => {
                    info!("Shutting down server (received Ctrl+C)...");
                },
            }
        }
        
        #[cfg(not(feature = "tls"))]
        {
            error!("TLS support was not enabled at build time. Please rebuild with 'tls' feature.");
            return Err("TLS support not available".into());
        }
    } else {
        // Plain HTTP server with HTTP/2 support
        info!("Starting HTTP server with HTTP/2 support");
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
        
        // Print server banner
        print_server_banner(server_config);
        
        // Run the server with graceful shutdown
        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    error!("Server error: {}", e);
                } else {
                    info!("Server stopped");
                }
            },
            _ = signal::ctrl_c() => {
                info!("Shutting down server (received Ctrl+C)...");
            },
        }
    }
    
    Ok(())
}

/// Print a banner with server information
fn print_server_banner(server_config: &ServerOpts) {
    println!("\n{}", "-".repeat(60));
    println!(" Rustyll Server");
    println!(" - URL: {}", server_config.url());
    println!(" - Livereload: {}", if server_config.livereload { "Enabled" } else { "Disabled" });
    if server_config.livereload {
        println!(" - Livereload URL: {}/livereload", server_config.url());
    }
    println!(" - HTTP/2: Enabled");
    println!(" - Compression: Enabled");
    println!(" - Press Ctrl+C to stop");
    println!("{}\n", "-".repeat(60));
} 