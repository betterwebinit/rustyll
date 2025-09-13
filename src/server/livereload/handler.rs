use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::Path;
use log::{info, debug};
use glob::Pattern;

/// LiveReload handler for managing browser refresh
pub struct LiveReloadHandler {
    /// Port for LiveReload server
    port: u16,
    /// Set of ignore patterns
    ignore_patterns: Vec<String>,
    /// Minimum delay between reloads
    min_delay: Duration,
    /// Last reload time
    last_reload: Arc<Mutex<Instant>>,
    /// Whether the LiveReload server is running
    is_running: Arc<Mutex<bool>>,
}

impl LiveReloadHandler {
    /// Create a new LiveReload handler
    pub fn new(port: u16, ignore_patterns: Vec<String>, min_delay: Duration) -> Self {
        Self {
            port,
            ignore_patterns,
            min_delay,
            last_reload: Arc::new(Mutex::new(Instant::now())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Check if a path should be ignored
    pub fn should_ignore(&self, path: &Path) -> bool {
        if self.ignore_patterns.is_empty() {
            return false;
        }
        
        let path_str = path.to_string_lossy();
        
        for pattern in &self.ignore_patterns {
            // Convert Jekyll glob patterns to Rust glob patterns
            if let Ok(glob) = Pattern::new(pattern) {
                if glob.matches(&path_str) {
                    debug!("LiveReload ignoring {} (matched pattern {})", path_str, pattern);
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Start the LiveReload server
    pub fn start(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = true;
        info!("LiveReload server started on port {}", self.port);
    }
    
    /// Stop the LiveReload server
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
        info!("LiveReload server stopped");
    }
    
    /// Check if the LiveReload server is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
    
    /// Trigger a reload if enough time has passed since the last one
    pub fn trigger_reload(&self, paths: &[&Path]) -> bool {
        if !self.is_running() {
            return false;
        }
        
        // Check if all paths should be ignored
        if !paths.is_empty() && paths.iter().all(|p| self.should_ignore(p)) {
            debug!("All changed paths are ignored, not triggering reload");
            return false;
        }
        
        // Check if enough time has passed since the last reload
        let mut last_reload = self.last_reload.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_reload);
        
        if elapsed < self.min_delay {
            debug!("Not enough time since last reload, skipping");
            return false;
        }
        
        // Update the last reload time
        *last_reload = now;
        
        // Log the reload
        if !paths.is_empty() {
            info!("LiveReload triggered for {} changed files", paths.len());
        } else {
            info!("LiveReload triggered");
        }
        
        true
    }
    
    /// Get the LiveReload URL
    pub fn url(&self, host: &str) -> String {
        format!("ws://{}:{}/livereload", host, self.port)
    }
} 