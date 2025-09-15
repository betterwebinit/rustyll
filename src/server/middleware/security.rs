//! Security header middleware for HTTP responses

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, HeaderValue},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;
use tower::Layer;
use tower_http::set_header::SetResponseHeaderLayer;
use log::trace;

/// Security configuration for the server
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable security headers
    pub enabled: bool,
    /// Content Security Policy
    pub csp: Option<ContentSecurityPolicy>,
    /// X-Frame-Options header value
    pub x_frame_options: XFrameOptions,
    /// X-Content-Type-Options header
    pub x_content_type_options: bool,
    /// X-XSS-Protection header
    pub x_xss_protection: bool,
    /// Referrer-Policy header
    pub referrer_policy: ReferrerPolicy,
    /// Permissions-Policy header
    pub permissions_policy: Option<String>,
    /// Strict-Transport-Security header (HSTS)
    pub hsts: Option<HstsConfig>,
    /// Cross-Origin-Opener-Policy header
    pub coop: CrossOriginOpenerPolicy,
    /// Cross-Origin-Embedder-Policy header
    pub coep: CrossOriginEmbedderPolicy,
    /// Cross-Origin-Resource-Policy header
    pub corp: CrossOriginResourcePolicy,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            csp: Some(ContentSecurityPolicy::default()),
            x_frame_options: XFrameOptions::Deny,
            x_content_type_options: true,
            x_xss_protection: true,
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            permissions_policy: Some("geolocation=(), microphone=(), camera=()".to_string()),
            hsts: Some(HstsConfig::default()),
            coop: CrossOriginOpenerPolicy::SameOrigin,
            coep: CrossOriginEmbedderPolicy::RequireCorp,
            corp: CrossOriginResourcePolicy::SameOrigin,
        }
    }
}

/// Content Security Policy configuration
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub media_src: Vec<String>,
    pub object_src: Vec<String>,
    pub frame_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub upgrade_insecure_requests: bool,
    pub block_all_mixed_content: bool,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
            font_src: vec!["'self'".to_string(), "data:".to_string()],
            connect_src: vec!["'self'".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'none'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: false,
            block_all_mixed_content: false,
        }
    }
}

impl ContentSecurityPolicy {
    /// Build the CSP header value
    pub fn to_header_value(&self) -> String {
        let mut directives = Vec::new();

        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }
        if !self.script_src.is_empty() {
            directives.push(format!("script-src {}", self.script_src.join(" ")));
        }
        if !self.style_src.is_empty() {
            directives.push(format!("style-src {}", self.style_src.join(" ")));
        }
        if !self.img_src.is_empty() {
            directives.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.font_src.is_empty() {
            directives.push(format!("font-src {}", self.font_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            directives.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.media_src.is_empty() {
            directives.push(format!("media-src {}", self.media_src.join(" ")));
        }
        if !self.object_src.is_empty() {
            directives.push(format!("object-src {}", self.object_src.join(" ")));
        }
        if !self.frame_src.is_empty() {
            directives.push(format!("frame-src {}", self.frame_src.join(" ")));
        }
        if !self.frame_ancestors.is_empty() {
            directives.push(format!("frame-ancestors {}", self.frame_ancestors.join(" ")));
        }
        if !self.base_uri.is_empty() {
            directives.push(format!("base-uri {}", self.base_uri.join(" ")));
        }
        if !self.form_action.is_empty() {
            directives.push(format!("form-action {}", self.form_action.join(" ")));
        }
        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }
        if self.block_all_mixed_content {
            directives.push("block-all-mixed-content".to_string());
        }

        directives.join("; ")
    }

    /// Create a strict CSP policy
    pub fn strict() -> Self {
        Self {
            default_src: vec!["'none'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string()],
            img_src: vec!["'self'".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string()],
            media_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            frame_src: vec!["'none'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
        }
    }

    /// Create a relaxed CSP policy for development
    pub fn development() -> Self {
        Self {
            default_src: vec!["*".to_string()],
            script_src: vec!["*".to_string(), "'unsafe-inline'".to_string(), "'unsafe-eval'".to_string()],
            style_src: vec!["*".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["*".to_string(), "data:".to_string(), "blob:".to_string()],
            font_src: vec!["*".to_string(), "data:".to_string()],
            connect_src: vec!["*".to_string()],
            media_src: vec!["*".to_string()],
            object_src: vec!["*".to_string()],
            frame_src: vec!["*".to_string()],
            frame_ancestors: vec!["*".to_string()],
            base_uri: vec!["*".to_string()],
            form_action: vec!["*".to_string()],
            upgrade_insecure_requests: false,
            block_all_mixed_content: false,
        }
    }
}

/// X-Frame-Options header values
#[derive(Debug, Clone)]
pub enum XFrameOptions {
    Deny,
    SameOrigin,
    AllowFrom(String),
}

impl XFrameOptions {
    pub fn to_header_value(&self) -> &str {
        match self {
            XFrameOptions::Deny => "DENY",
            XFrameOptions::SameOrigin => "SAMEORIGIN",
            XFrameOptions::AllowFrom(uri) => {
                // Note: ALLOW-FROM is deprecated, but included for completeness
                Box::leak(format!("ALLOW-FROM {}", uri).into_boxed_str())
            }
        }
    }
}

/// Referrer-Policy header values
#[derive(Debug, Clone)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

impl ReferrerPolicy {
    pub fn to_header_value(&self) -> &str {
        match self {
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
        }
    }
}

/// HSTS (HTTP Strict Transport Security) configuration
#[derive(Debug, Clone)]
pub struct HstsConfig {
    pub max_age: u32,
    pub include_subdomains: bool,
    pub preload: bool,
}

impl Default for HstsConfig {
    fn default() -> Self {
        Self {
            max_age: 31536000, // 1 year
            include_subdomains: true,
            preload: false,
        }
    }
}

impl HstsConfig {
    pub fn to_header_value(&self) -> String {
        let mut value = format!("max-age={}", self.max_age);
        if self.include_subdomains {
            value.push_str("; includeSubDomains");
        }
        if self.preload {
            value.push_str("; preload");
        }
        value
    }
}

/// Cross-Origin-Opener-Policy header values
#[derive(Debug, Clone)]
pub enum CrossOriginOpenerPolicy {
    UnsafeNone,
    SameOrigin,
    SameOriginAllowPopups,
}

impl CrossOriginOpenerPolicy {
    pub fn to_header_value(&self) -> &str {
        match self {
            CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
            CrossOriginOpenerPolicy::SameOrigin => "same-origin",
            CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
        }
    }
}

/// Cross-Origin-Embedder-Policy header values
#[derive(Debug, Clone)]
pub enum CrossOriginEmbedderPolicy {
    UnsafeNone,
    RequireCorp,
    Credentialless,
}

impl CrossOriginEmbedderPolicy {
    pub fn to_header_value(&self) -> &str {
        match self {
            CrossOriginEmbedderPolicy::UnsafeNone => "unsafe-none",
            CrossOriginEmbedderPolicy::RequireCorp => "require-corp",
            CrossOriginEmbedderPolicy::Credentialless => "credentialless",
        }
    }
}

/// Cross-Origin-Resource-Policy header values
#[derive(Debug, Clone)]
pub enum CrossOriginResourcePolicy {
    SameSite,
    SameOrigin,
    CrossOrigin,
}

impl CrossOriginResourcePolicy {
    pub fn to_header_value(&self) -> &str {
        match self {
            CrossOriginResourcePolicy::SameSite => "same-site",
            CrossOriginResourcePolicy::SameOrigin => "same-origin",
            CrossOriginResourcePolicy::CrossOrigin => "cross-origin",
        }
    }
}

/// Security middleware for adding security headers
pub async fn security_middleware(
    State(config): State<Arc<SecurityConfig>>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    if !config.enabled {
        return next.run(request).await;
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Content Security Policy
    if let Some(ref csp) = config.csp {
        headers.insert(
            "content-security-policy",
            HeaderValue::from_str(&csp.to_header_value()).unwrap(),
        );
    }

    // X-Frame-Options
    headers.insert(
        "x-frame-options",
        HeaderValue::from_str(&config.x_frame_options.to_header_value()).unwrap(),
    );

    // X-Content-Type-Options
    if config.x_content_type_options {
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
    }

    // X-XSS-Protection
    if config.x_xss_protection {
        headers.insert(
            "x-xss-protection",
            HeaderValue::from_static("1; mode=block"),
        );
    }

    // Referrer-Policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_str(&config.referrer_policy.to_header_value()).unwrap(),
    );

    // Permissions-Policy
    if let Some(ref permissions) = config.permissions_policy {
        headers.insert(
            "permissions-policy",
            HeaderValue::from_str(permissions).unwrap(),
        );
    }

    // Strict-Transport-Security (HSTS)
    if let Some(ref hsts) = config.hsts {
        headers.insert(
            "strict-transport-security",
            HeaderValue::from_str(&hsts.to_header_value()).unwrap(),
        );
    }

    // Cross-Origin-Opener-Policy
    headers.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_str(&config.coop.to_header_value()).unwrap(),
    );

    // Cross-Origin-Embedder-Policy
    headers.insert(
        "cross-origin-embedder-policy",
        HeaderValue::from_str(config.coep.to_header_value()).unwrap(),
    );

    // Cross-Origin-Resource-Policy
    headers.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_str(config.corp.to_header_value()).unwrap(),
    );

    trace!("Security headers applied");

    response
}

/// Create a simple security headers layer
pub fn create_security_headers_layer() -> SetResponseHeaderLayer<HeaderValue> {
    let csp = ContentSecurityPolicy::default();
    let header_value = HeaderValue::from_str(&csp.to_header_value())
        .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'"));
    SetResponseHeaderLayer::if_not_present("content-security-policy".parse().unwrap(), header_value)
}

/// Create all security layers for production
pub fn create_production_security_layers() -> impl Layer<
    tower::util::BoxCloneService<Request<Body>, Response<Body>, std::convert::Infallible>,
    Service = impl tower::Service<Request<Body>, Response = Response<Body>, Error = std::convert::Infallible> + Clone,
> + Clone {
    create_security_headers_layer()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_header() {
        let csp = ContentSecurityPolicy::default();
        let header = csp.to_header_value();
        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self' 'unsafe-inline'"));

        let strict_csp = ContentSecurityPolicy::strict();
        let header = strict_csp.to_header_value();
        assert!(header.contains("default-src 'none'"));
        assert!(header.contains("script-src 'self'"));
        assert!(!header.contains("'unsafe-inline'"));
    }

    #[test]
    fn test_x_frame_options() {
        assert_eq!(XFrameOptions::Deny.to_header_value(), "DENY");
        assert_eq!(XFrameOptions::SameOrigin.to_header_value(), "SAMEORIGIN");
    }

    #[test]
    fn test_hsts_header() {
        let hsts = HstsConfig::default();
        let header = hsts.to_header_value();
        assert!(header.contains("max-age=31536000"));
        assert!(header.contains("includeSubDomains"));
        assert!(!header.contains("preload"));

        let hsts_with_preload = HstsConfig {
            max_age: 63072000,
            include_subdomains: true,
            preload: true,
        };
        let header = hsts_with_preload.to_header_value();
        assert!(header.contains("max-age=63072000"));
        assert!(header.contains("preload"));
    }

    #[test]
    fn test_referrer_policy() {
        assert_eq!(ReferrerPolicy::NoReferrer.to_header_value(), "no-referrer");
        assert_eq!(ReferrerPolicy::StrictOrigin.to_header_value(), "strict-origin");
    }
}