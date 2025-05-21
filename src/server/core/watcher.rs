use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::path::Path;
use log::{info, debug, error};
use notify::Event;
use glob::Pattern;

use crate::config::Config;
use crate::builder::build_site;

/// Handle file change events and rebuild the site
pub fn handle_file_changes(
    rx: Receiver<Event>,
    config: &Config,
    include_drafts: bool,
    include_unpublished: bool,
    min_delay: Duration,
    _max_delay: Duration   // Not used currently but kept for future debouncing improvements
) {
    let mut last_build = std::time::Instant::now();
    
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                debug!("File event: {:?}", event);
                
                // If we already built recently, wait at least the minimum delay
                let elapsed = last_build.elapsed();
                if elapsed < min_delay {
                    // Sleep for the remaining debounce time
                    std::thread::sleep(min_delay - elapsed);
                    
                    // Drain any additional events that came in during sleep
                    while let Ok(_) = rx.try_recv() {}
                }
                
                // Rebuild the site
                info!("File change detected, rebuilding site...");
                if let Err(e) = build_site(config, include_drafts, include_unpublished) {
                    error!("Error rebuilding site: {}", e);
                }
                
                // Update the last build time
                last_build = std::time::Instant::now();
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No events, continue waiting
            },
            Err(e) => {
                error!("Error receiving file events: {}", e);
                break;
            }
        }
    }
}

/// Check if a file path should be ignored based on patterns
pub fn should_ignore_path(path: &Path, ignore_patterns: &[String]) -> bool {
    if ignore_patterns.is_empty() {
        return false;
    }
    
    let path_str = path.to_string_lossy();
    
    for pattern in ignore_patterns {
        // Convert Jekyll glob patterns to Rust glob patterns
        match Pattern::new(pattern) {
            Ok(glob) => {
                if glob.matches(&path_str) {
                    debug!("Ignoring file {} (matched pattern {})", path_str, pattern);
                    return true;
                }
            },
            Err(e) => {
                error!("Invalid glob pattern '{}': {}", pattern, e);
            }
        }
    }
    
    false
} 