use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header, Request},
    routing::get,
    response::{IntoResponse, Response},
    body::Body,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tower::ServiceExt;
use std::path::{Path as FilePath, PathBuf};
use std::sync::Arc;
use std::fs;
use log::{info, debug};

use crate::server::handlers::{handle_not_found, create_directory_listing};
use crate::server::config::ServerConfig;

// App state that will be shared with handlers
#[derive(Clone)]
struct AppState {
    show_dir_listing: bool,
    base_url: String,
    destination: PathBuf,
}

// Custom handler for serving files with proper MIME types
async fn serve_file_handler(
    req: Request<Body>,
    state: Arc<AppState>,
) -> impl IntoResponse {
    let uri_path = req.uri().path();
    info!("🔍 [SERVER] Request received for: {}", uri_path);

    // URL decode the path
    let decoded_path = urlencoding::decode(uri_path)
        .unwrap_or_else(|_| std::borrow::Cow::Borrowed(uri_path))
        .into_owned();

    info!("🔗 [SERVER] Decoded path: {}", decoded_path);

    let clean_path = decoded_path.trim_start_matches('/').trim_end_matches('/');

    // Build the requested file path
    let requested_file = if clean_path.is_empty() {
        state.destination.join("index.html")
    } else {
        state.destination.join(clean_path)
    };

    info!("📁 [SERVER] Looking for file at: {}", requested_file.display());

    // Try different variations in order
    let variations = if clean_path.ends_with(".html") {
        // If path ends with .html, try without it first
        let without_html = clean_path.trim_end_matches(".html");
        vec![
            // 1. Try without .html extension (Jekyll style)
            state.destination.join(without_html),
            // 2. Try exact path with .html
            requested_file.clone(),
            // 3. Try as directory with index.html
            requested_file.join("index.html"),
        ]
    } else {
        vec![
            // 1. Exact path as file
            requested_file.clone(),
            // 2. Path with .html extension
            state.destination.join(format!("{}.html", clean_path)),
            // 3. Path as directory with index.html
            requested_file.join("index.html"),
        ]
    };

    for variation in variations {
        info!("🔎 [SERVER] Checking variation: {}", variation.display());

        if variation.exists() && variation.is_file() {
            info!("✅ [SERVER] Found file: {}", variation.display());

            match fs::read(&variation) {
                Ok(content) => {
                    // Determine content type based on extension
                    let content_type = match variation.extension().and_then(|s| s.to_str()) {
                        Some("html") => "text/html; charset=utf-8",
                        Some("css") => "text/css; charset=utf-8",
                        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
                        Some("json") => "application/json; charset=utf-8",
                        Some("xml") => "application/xml; charset=utf-8",
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        Some("gif") => "image/gif",
                        Some("svg") => "image/svg+xml; charset=utf-8",
                        Some("ico") => "image/x-icon",
                        Some("woff") => "font/woff",
                        Some("woff2") => "font/woff2",
                        Some("ttf") => "font/ttf",
                        Some("eot") => "application/vnd.ms-fontobject",
                        Some("txt") => "text/plain; charset=utf-8",
                        Some("md") => "text/markdown; charset=utf-8",
                        _ => {
                            // No extension or unknown - check content for HTML
                            let content_slice = if content.len() > 512 {
                                &content[..512]
                            } else {
                                &content[..]
                            };

                            // Check for HTML signatures
                            if content_slice.starts_with(b"<!DOCTYPE") ||
                               content_slice.starts_with(b"<!doctype") ||
                               content_slice.starts_with(b"<html") ||
                               content_slice.starts_with(b"<HTML") ||
                               (content_slice.starts_with(b"<") &&
                                (content_slice.windows(6).any(|w| w == b"<head>" || w == b"<HEAD>") ||
                                 content_slice.windows(6).any(|w| w == b"<body>" || w == b"<BODY>"))) {
                                "text/html; charset=utf-8"
                            } else {
                                // Default to HTML for extensionless files in Jekyll sites
                                "text/html; charset=utf-8"
                            }
                        }
                    };

                    info!("📄 [SERVER] Serving with content-type: {}", content_type);

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, content_type)
                        .header(header::CONTENT_DISPOSITION, "inline")
                        .header(header::CACHE_CONTROL, "public, max-age=0, must-revalidate")
                        .body(Body::from(content))
                        .unwrap()
                        .into_response();
                }
                Err(e) => {
                    info!("❌ [SERVER] Error reading file {}: {}", variation.display(), e);
                }
            }
        }
    }

    // Check if it's a directory and show listing if enabled
    if requested_file.is_dir() && state.show_dir_listing {
        info!("📂 [SERVER] Showing directory listing for: {}", requested_file.display());
        let uri_path_obj = FilePath::new(uri_path);
        match create_directory_listing(&requested_file, uri_path_obj) {
            Ok(response) => return response.into_response(),
            Err(e) => {
                info!("❌ [SERVER] Error creating directory listing: {}", e);
                return handle_not_found(&state.destination).into_response();
            }
        }
    }

    info!("🚫 [SERVER] No file found for path: {} - serving 404", uri_path);
    handle_not_found(&state.destination).into_response()
}

// Helper function to build the router with all routes and middleware
fn build_router(state: AppState) -> Router {
    let shared_state = Arc::new(state.clone());

    // Create router with catch-all route
    let router = Router::new()
        .fallback({
            let state = shared_state.clone();
            move |req: Request<Body>| serve_file_handler(req, state.clone())
        });

    // Add middleware
    let router = router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // If base_url is set, nest the router under that path
    if !state.base_url.is_empty() {
        let base_path = format!("/{}", state.base_url.trim_matches('/'));
        Router::new().nest(&base_path, router)
    } else {
        router
    }
}

/// Create the Axum Router for serving Jekyll-style static sites
pub fn create_app(
    destination: PathBuf,
    config: &ServerConfig,
) -> impl Fn() -> Router + Clone + Send + 'static {
    info!("🚀 [VERBOSE] Creating server app for destination: {}", destination.display());
    info!("🚀 [VERBOSE] Server config - Show directory listing: {}, Base URL: '{}'",
          config.show_dir_listing, config.baseurl);

    // Create the app state
    let state = AppState {
        show_dir_listing: config.show_dir_listing,
        base_url: config.baseurl.clone(),
        destination,
    };

    // Return a closure that creates a new router with the app state
    move || build_router(state.clone())
}