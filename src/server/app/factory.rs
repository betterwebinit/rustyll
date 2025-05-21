use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    routing::get,
    response::IntoResponse,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use std::path::PathBuf;
use std::sync::Arc;

use crate::server::handlers::{create_static_files_handler, handle_not_found, create_directory_listing};
use crate::server::config::ServerConfig;

// App state that will be shared with handlers
#[derive(Clone)]
struct AppState {
    show_dir_listing: bool,
    base_url: String,
    destination: PathBuf,
}

// Helper function to build the router with all routes and middleware
fn build_router(state: AppState) -> Router {
    // Create the shared state
    let shared_state = Arc::new(state.clone());
    
    // Create a router with all routes
    let mut router = Router::new()
        .merge(create_static_files_handler(state.destination.clone(), state.show_dir_listing));
    
    // Add directory listing if enabled
    if state.show_dir_listing {
        router = router.route(
            "/*path/", 
            get(move |Path(path): Path<String>| {
                let state = shared_state.clone();
                async move {
                    let full_path = state.destination.join(&path);
                    if full_path.is_dir() {
                        create_directory_listing(&full_path, &state.destination)
                            .unwrap_or_else(|_| {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create directory listing").into_response()
                            })
                    } else {
                        (StatusCode::NOT_FOUND, "Not Found").into_response()
                    }
                }
            })
        );
    }
    
    // Add fallback handler for not found
    let destination = state.destination.clone();
    router = router.fallback(move || async move {
        handle_not_found(&destination)
    });
    
    // Add middleware
    router = router
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
    // Create the app state
    let state = AppState {
        show_dir_listing: config.show_dir_listing,
        base_url: config.baseurl.clone(),
        destination,
    };
    
    // Return a closure that creates a new router with the app state
    move || build_router(state.clone())
} 