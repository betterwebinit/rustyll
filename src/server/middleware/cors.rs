use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

/// CORS middleware for Jekyll-style static sites
pub struct CorsMiddleware;

impl CorsMiddleware {
    /// Create a CORS middleware that allows any origin
    pub fn allow_all() -> CorsLayer {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ])
            .allow_headers(Any)
    }
    
    /// Create a CORS middleware with specific allowed origins
    pub fn new(origins: Vec<String>) -> CorsLayer {
        let origins = origins.into_iter()
            .filter_map(|origin| origin.parse().ok())
            .collect::<Vec<_>>();
        
        if origins.is_empty() {
            return Self::allow_all();
        }
        
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ])
            .allow_headers(Any)
    }
} 