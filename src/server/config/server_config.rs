use std::path::PathBuf;

/// Server configuration with Jekyll-compatible options
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind to (e.g., "127.0.0.1", "0.0.0.0")
    pub address: String,
    
    /// Port to listen on (e.g., 4000)
    pub port: u16,
    
    /// Whether to enable LiveReload
    pub livereload: bool,
    
    /// LiveReload port (typically 35729)
    pub livereload_port: Option<u16>,
    
    /// Whether to open the browser automatically
    pub open_url: bool,
    
    /// Whether to detach the server process (run in background)
    pub detach: bool,
    
    /// Base URL to use for serving the site
    pub baseurl: String,
    
    /// SSL certificate path
    pub ssl_cert: Option<PathBuf>,
    
    /// SSL key path
    pub ssl_key: Option<PathBuf>,
    
    /// Whether to show directory listings instead of index files
    pub show_dir_listing: bool,
    
    /// Skip initial site build
    pub skip_initial_build: bool,
    
    /// List of patterns for LiveReload to ignore
    pub livereload_ignore: Vec<String>,
    
    /// Minimum delay for LiveReload
    pub livereload_min_delay: Option<u64>,
    
    /// Maximum delay for LiveReload
    pub livereload_max_delay: Option<u64>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "127.0.0.1".to_string(),
            port: 4000,
            livereload: false,
            livereload_port: Some(35729),
            open_url: false,
            detach: false,
            baseurl: "".to_string(),
            ssl_cert: None,
            ssl_key: None,
            show_dir_listing: false,
            skip_initial_build: false,
            livereload_ignore: Vec::new(),
            livereload_min_delay: None,
            livereload_max_delay: None,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with basic options
    pub fn new(address: &str, port: u16, livereload: bool) -> Self {
        ServerConfig {
            address: address.to_string(),
            port,
            livereload,
            livereload_port: if livereload { Some(35729) } else { None },
            ..Default::default()
        }
    }
    
    /// Set whether to open the URL in a browser
    pub fn with_open_url(mut self, open_url: bool) -> Self {
        self.open_url = open_url;
        self
    }
    
    /// Set whether to detach the server process
    pub fn with_detach(mut self, detach: bool) -> Self {
        self.detach = detach;
        self
    }
    
    /// Set the base URL for serving
    pub fn with_baseurl(mut self, baseurl: &str) -> Self {
        self.baseurl = baseurl.to_string();
        self
    }
    
    /// Set the LiveReload port
    pub fn with_livereload_port(mut self, port: u16) -> Self {
        self.livereload_port = Some(port);
        self
    }
    
    /// Set the SSL certificate and key
    pub fn with_ssl(mut self, cert: PathBuf, key: PathBuf) -> Self {
        self.ssl_cert = Some(cert);
        self.ssl_key = Some(key);
        self
    }
    
    /// Set whether to show directory listings
    pub fn with_dir_listing(mut self, show: bool) -> Self {
        self.show_dir_listing = show;
        self
    }
    
    /// Set LiveReload ignore patterns
    pub fn with_livereload_ignore(mut self, patterns: Vec<String>) -> Self {
        self.livereload_ignore = patterns;
        self
    }
    
    /// Set LiveReload delay bounds
    pub fn with_livereload_delays(mut self, min: u64, max: u64) -> Self {
        self.livereload_min_delay = Some(min);
        self.livereload_max_delay = Some(max);
        self
    }
    
    /// Get the full address string (e.g., "127.0.0.1:4000")
    pub fn address_string(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
    
    /// Get the full URL (e.g., "http://127.0.0.1:4000")
    pub fn url(&self) -> String {
        let protocol = if self.ssl_cert.is_some() && self.ssl_key.is_some() {
            "https"
        } else {
            "http"
        };
        
        let address = if self.address == "127.0.0.1" {
            "localhost"
        } else if self.address.contains(':') {
            &format!("[{}]", self.address)
        } else {
            &self.address
        };
        
        let baseurl = if self.baseurl.is_empty() {
            "".to_string()
        } else {
            format!("/{}", self.baseurl.trim_matches('/'))
        };
        
        format!("{}://{}:{}{}/", protocol, address, self.port, baseurl)
    }
} 