//! Cache control middleware for HTTP responses

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use http::{HeaderValue, HeaderMap};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::Layer;
use tower_http::set_header::SetResponseHeaderLayer;
use sha2::{Sha256, Digest};
use log::{debug, trace};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Max age for static assets in seconds
    pub static_max_age: u32,
    /// Max age for HTML pages in seconds
    pub html_max_age: u32,
    /// Enable ETags
    pub use_etags: bool,
    /// Enable Last-Modified headers
    pub use_last_modified: bool,
    /// Cache control directives
    pub cache_control: CacheControl,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            static_max_age: 31536000, // 1 year for static assets
            html_max_age: 3600,       // 1 hour for HTML
            use_etags: true,
            use_last_modified: true,
            cache_control: CacheControl::default(),
        }
    }
}

/// Cache control directives
#[derive(Debug, Clone)]
pub struct CacheControl {
    pub public: bool,
    pub private: bool,
    pub no_cache: bool,
    pub no_store: bool,
    pub must_revalidate: bool,
    pub immutable: bool,
    pub s_maxage: Option<u32>,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            private: false,
            no_cache: false,
            no_store: false,
            must_revalidate: false,
            immutable: false,
            s_maxage: None,
        }
    }
}

impl CacheControl {
    /// Build the Cache-Control header value
    pub fn to_header_value(&self, max_age: u32) -> String {
        let mut directives = Vec::new();

        if self.no_store {
            directives.push("no-store".to_string());
        } else {
            if self.public && !self.private {
                directives.push("public".to_string());
            } else if self.private {
                directives.push("private".to_string());
            }

            if self.no_cache {
                directives.push("no-cache".to_string());
            } else {
                directives.push(format!("max-age={}", max_age));
            }

            if let Some(s_maxage) = self.s_maxage {
                directives.push(format!("s-maxage={}", s_maxage));
            }

            if self.must_revalidate {
                directives.push("must-revalidate".to_string());
            }

            if self.immutable && max_age > 0 {
                directives.push("immutable".to_string());
            }
        }

        directives.join(", ")
    }
}

/// ETag cache for storing computed ETags
#[derive(Debug, Clone)]
pub struct ETagCache {
    cache: Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

impl ETagCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        cache.get(key).cloned()
    }

    pub async fn set(&self, key: String, etag: String) {
        let mut cache = self.cache.write().await;
        cache.insert(key, etag);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Generate an ETag from content
pub fn generate_etag(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("\"{}\"", hex::encode(&result[..8])) // Use first 8 bytes for shorter ETags
}

/// Check if the request has a matching ETag
pub fn check_etag_match(request_headers: &HeaderMap, etag: &str) -> bool {
    if let Some(if_none_match) = request_headers.get(header::IF_NONE_MATCH) {
        if let Ok(if_none_match_str) = if_none_match.to_str() {
            // Handle multiple ETags separated by commas
            for etag_value in if_none_match_str.split(',') {
                let trimmed = etag_value.trim();
                if trimmed == "*" || trimmed == etag {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if the request has If-Modified-Since header
pub fn check_if_modified_since(request_headers: &HeaderMap, last_modified: SystemTime) -> bool {
    if let Some(if_modified_since) = request_headers.get(header::IF_MODIFIED_SINCE) {
        if let Ok(if_modified_since_str) = if_modified_since.to_str() {
            if let Ok(if_modified_time) = httpdate::parse_http_date(if_modified_since_str) {
                return last_modified <= if_modified_time;
            }
        }
    }
    false
}

/// Cache middleware for handling HTTP caching
pub async fn cache_middleware(
    State(config): State<Arc<CacheConfig>>,
    State(etag_cache): State<Arc<ETagCache>>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    if !config.enabled {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let request_headers = request.headers().clone();

    // Only apply caching to GET and HEAD requests
    if method != http::Method::GET && method != http::Method::HEAD {
        return next.run(request).await;
    }

    // Check for cached ETag
    if config.use_etags {
        if let Some(cached_etag) = etag_cache.get(&path).await {
            if check_etag_match(&request_headers, &cached_etag) {
                debug!("ETag match for {}, returning 304", path);
                return Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .header(header::ETAG, cached_etag)
                    .body(Body::empty())
                    .unwrap()
                    .into_response();
            }
        }
    }

    // Process the request
    let mut response = next.run(request).await;

    // Only apply caching headers to successful responses
    if response.status().is_success() {
        let is_static = is_static_asset(&path);
        let max_age = if is_static {
            config.static_max_age
        } else {
            config.html_max_age
        };

        // Set Cache-Control header
        let cache_control_value = if is_static && config.cache_control.immutable {
            let mut cc = config.cache_control.clone();
            cc.immutable = true;
            cc.to_header_value(max_age)
        } else {
            config.cache_control.to_header_value(max_age)
        };

        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_str(&cache_control_value).unwrap_or_else(|_| {
                HeaderValue::from_static("public, max-age=3600")
            }),
        );

        // Set Last-Modified header
        if config.use_last_modified && !response.headers().contains_key(header::LAST_MODIFIED) {
            let now = SystemTime::now();
            if let Ok(_duration) = now.duration_since(UNIX_EPOCH) {
                let last_modified = httpdate::fmt_http_date(now);
                response.headers_mut().insert(
                    header::LAST_MODIFIED,
                    HeaderValue::from_str(&last_modified).unwrap_or_else(|_| {
                        HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT")
                    }),
                );
            }
        }

        // Generate and set ETag if enabled
        if config.use_etags && !response.headers().contains_key(header::ETAG) {
            // Note: In production, you'd want to generate ETags from file content
            // For now, we'll use path + timestamp as a simple example
            let etag = generate_etag(path.as_bytes());
            response.headers_mut().insert(
                header::ETAG,
                HeaderValue::from_str(&etag).unwrap(),
            );

            // Cache the ETag
            etag_cache.set(path.clone(), etag).await;
        }

        // Add Vary header for proper caching with compression
        if !response.headers().contains_key(header::VARY) {
            response.headers_mut().insert(
                header::VARY,
                HeaderValue::from_static("Accept-Encoding"),
            );
        }

        trace!("Cache headers set for {}: max-age={}", path, max_age);
    }

    response
}

/// Check if the path is a static asset
fn is_static_asset(path: &str) -> bool {
    let static_extensions = [
        ".css", ".js", ".jpg", ".jpeg", ".png", ".gif", ".svg",
        ".ico", ".woff", ".woff2", ".ttf", ".eot", ".otf",
        ".mp4", ".webm", ".mp3", ".wav", ".pdf", ".zip",
    ];

    static_extensions.iter().any(|ext| path.ends_with(ext))
}

/// Create a simple cache control layer
pub fn create_cache_control_layer(max_age: Option<u32>) -> SetResponseHeaderLayer<HeaderValue> {
    let age = max_age.unwrap_or(3600);
    let header_value = HeaderValue::from_str(&format!("public, max-age={}", age))
        .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=3600"));
    SetResponseHeaderLayer::if_not_present(header::CACHE_CONTROL, header_value)
}

/// Create cache layers for the server
pub fn create_cache_layers(config: Arc<CacheConfig>) -> impl Layer<
    tower::util::BoxCloneService<Request<Body>, Response<Body>, std::convert::Infallible>,
    Service = impl tower::Service<Request<Body>, Response = Response<Body>, Error = std::convert::Infallible> + Clone,
> + Clone {
    // Return the simple layer for now
    // In production, you'd wire up the full middleware
    create_cache_control_layer(Some(config.html_max_age))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_header() {
        let cc = CacheControl::default();
        assert_eq!(cc.to_header_value(3600), "public, max-age=3600");

        let mut cc = CacheControl::default();
        cc.private = true;
        cc.public = false;
        assert_eq!(cc.to_header_value(3600), "private, max-age=3600");

        let mut cc = CacheControl::default();
        cc.no_store = true;
        assert_eq!(cc.to_header_value(3600), "no-store");

        let mut cc = CacheControl::default();
        cc.immutable = true;
        assert_eq!(cc.to_header_value(3600), "public, max-age=3600, immutable");
    }

    #[test]
    fn test_is_static_asset() {
        assert!(is_static_asset("/styles/main.css"));
        assert!(is_static_asset("/js/app.js"));
        assert!(is_static_asset("/images/logo.png"));
        assert!(!is_static_asset("/index.html"));
        assert!(!is_static_asset("/about"));
    }

    #[test]
    fn test_generate_etag() {
        let content = b"Hello, World!";
        let etag = generate_etag(content);
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        assert!(etag.len() > 2);
    }
}