//! Compression middleware for HTTP responses

use tower_http::compression::{CompressionLayer, CompressionLevel};
use std::sync::Arc;
use log::debug;

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Compression level (0-9, where 0 is no compression and 9 is best)
    pub level: CompressionLevel,
    /// Minimum response size to compress (in bytes)
    pub min_size: u16,
    /// Enable gzip compression
    pub gzip: bool,
    /// Enable brotli compression
    pub brotli: bool,
    /// Enable deflate compression
    pub deflate: bool,
    /// Content types to exclude from compression
    pub exclude_types: Vec<String>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: CompressionLevel::Default,
            min_size: 1024, // Don't compress responses smaller than 1KB
            gzip: true,
            brotli: true,
            deflate: true,
            exclude_types: vec![
                // Already compressed formats
                "image/jpeg".to_string(),
                "image/jpg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/avif".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "audio/mpeg".to_string(),
                "audio/mp3".to_string(),
                "audio/ogg".to_string(),
                "application/zip".to_string(),
                "application/x-gzip".to_string(),
                "application/x-bzip2".to_string(),
                "application/x-7z-compressed".to_string(),
                "application/x-rar-compressed".to_string(),
            ],
        }
    }
}

impl CompressionConfig {
    /// Create configuration for maximum compression
    pub fn best() -> Self {
        Self {
            level: CompressionLevel::Best,
            ..Default::default()
        }
    }

    /// Create configuration for fastest compression
    pub fn fastest() -> Self {
        Self {
            level: CompressionLevel::Fastest,
            ..Default::default()
        }
    }

    /// Create configuration with balanced compression
    pub fn balanced() -> Self {
        Self {
            level: CompressionLevel::Default,
            ..Default::default()
        }
    }

    /// Set compression level (0-9)
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = match level {
            0 => CompressionLevel::Fastest,
            1..=3 => CompressionLevel::Fastest,
            4..=6 => CompressionLevel::Default,
            7..=9 => CompressionLevel::Best,
            _ => CompressionLevel::Default,
        };
        self
    }

    /// Set minimum size for compression
    pub fn with_min_size(mut self, min_size: u16) -> Self {
        self.min_size = min_size;
        self
    }

    /// Add content type to exclude from compression
    pub fn exclude_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.exclude_types.push(content_type.into());
        self
    }
}

/// Create a compression layer with the specified configuration
pub fn create_compression_layer(config: Option<Arc<CompressionConfig>>) -> CompressionLayer {
    if let Some(config) = config {
        if !config.enabled {
            debug!("Compression is disabled");
            return CompressionLayer::new()
                .no_gzip()
                .no_br()
                .no_deflate();
        }

        let mut layer = CompressionLayer::new()
            .quality(config.level.clone());

        // Configure compression algorithms
        if !config.gzip {
            layer = layer.no_gzip();
        }
        if !config.brotli {
            layer = layer.no_br();
        }
        if !config.deflate {
            layer = layer.no_deflate();
        }

        debug!(
            "Compression enabled: gzip={}, brotli={}, deflate={}, min_size={} bytes",
            config.gzip, config.brotli, config.deflate, config.min_size
        );

        layer
    } else {
        // Default compression when no config provided
        CompressionLayer::new()
    }
}

/// Create a simple compression layer with default settings
pub fn create_default_compression_layer() -> CompressionLayer {
    CompressionLayer::new()
        .quality(CompressionLevel::Default)
}

/// Create a compression layer for static assets
pub fn create_static_compression_layer() -> CompressionLayer {
    CompressionLayer::new()
        .quality(CompressionLevel::Best) // Best compression for static assets
}

/// Create a compression layer for dynamic content
pub fn create_dynamic_compression_layer() -> CompressionLayer {
    CompressionLayer::new()
        .quality(CompressionLevel::Fastest) // Fast compression for dynamic content
}

/// Check if content should be compressed based on content type
pub fn should_compress_content_type(content_type: &str, exclude_types: &[String]) -> bool {
    // Check if it's in the exclude list
    if exclude_types.iter().any(|t| content_type.starts_with(t)) {
        return false;
    }

    // Compress text-based content types
    let compressible_prefixes = [
        "text/",
        "application/json",
        "application/xml",
        "application/javascript",
        "application/x-javascript",
        "application/ecmascript",
        "application/rss+xml",
        "application/atom+xml",
        "application/xhtml+xml",
        "application/x-font-",
        "font/",
        "image/svg+xml",
        "image/x-icon",
    ];

    compressible_prefixes.iter().any(|prefix| content_type.starts_with(prefix))
}

/// Compression statistics for monitoring
#[derive(Debug, Default)]
pub struct CompressionStats {
    pub total_requests: u64,
    pub compressed_requests: u64,
    pub bytes_saved: u64,
    pub compression_ratio: f32,
}

impl CompressionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_compression(&mut self, original_size: u64, compressed_size: u64) {
        self.total_requests += 1;
        if compressed_size < original_size {
            self.compressed_requests += 1;
            self.bytes_saved += original_size - compressed_size;

            // Update running average of compression ratio
            let ratio = compressed_size as f32 / original_size as f32;
            self.compression_ratio = if self.compressed_requests == 1 {
                ratio
            } else {
                (self.compression_ratio * (self.compressed_requests - 1) as f32 + ratio)
                    / self.compressed_requests as f32
            };
        }
    }

    pub fn get_savings_percentage(&self) -> f32 {
        if self.compression_ratio > 0.0 {
            (1.0 - self.compression_ratio) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_size, 1024);

        let config = CompressionConfig::best();
        assert!(matches!(config.level, CompressionLevel::Best));

        let config = CompressionConfig::fastest();
        assert!(matches!(config.level, CompressionLevel::Fastest));
    }

    #[test]
    fn test_should_compress_content_type() {
        let exclude_types = vec!["image/jpeg".to_string(), "video/mp4".to_string()];

        assert!(should_compress_content_type("text/html", &exclude_types));
        assert!(should_compress_content_type("application/json", &exclude_types));
        assert!(should_compress_content_type("text/css", &exclude_types));
        assert!(should_compress_content_type("application/javascript", &exclude_types));

        assert!(!should_compress_content_type("image/jpeg", &exclude_types));
        assert!(!should_compress_content_type("video/mp4", &exclude_types));
        assert!(!should_compress_content_type("image/png", &[])); // PNG is in default exclude
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::new();

        stats.record_compression(1000, 300);
        assert_eq!(stats.compressed_requests, 1);
        assert_eq!(stats.bytes_saved, 700);
        assert_eq!(stats.compression_ratio, 0.3);
        assert_eq!(stats.get_savings_percentage(), 70.0);

        stats.record_compression(2000, 800);
        assert_eq!(stats.compressed_requests, 2);
        assert_eq!(stats.bytes_saved, 1900);
    }
}