use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use log::{info, debug, error};
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event, EventKind};

use crate::config::Config;
use crate::builder::site::build_site;
use crate::builder::types::BoxResult;

/// Watch the site source directory for changes and rebuild when necessary
pub fn watch_site(config: &Config, include_drafts: bool, include_unpublished: bool) -> BoxResult<()> {
    info!("Watching source directory: {}", config.source.display());
    
    // Create a channel to receive filesystem events
    let (tx, rx) = channel();
    
    // Create a watcher object
    let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
        match res {
            Ok(event) => {
                // Only trigger rebuild on file modifications
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Send event to channel
                        tx.send(event).unwrap_or_else(|e| {
                            error!("Error sending file event: {}", e);
                        });
                    },
                    _ => {}
                }
            },
            Err(e) => error!("Watch error: {}", e),
        }
    })?;
    
    // Watch the source directory
    watcher.watch(&config.source, RecursiveMode::Recursive)?;
    
    // Initial build
    build_site(config, include_drafts, include_unpublished)?;
    
    info!("Watching for changes. Press Ctrl+C to stop.");
    
    // Track the last build time to avoid rebuilding too frequently
    let mut last_build = std::time::Instant::now();
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);
    
    // Wait for events
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                debug!("File event: {:?}", event);
                
                // If we already built recently, wait a bit
                let elapsed = last_build.elapsed();
                if elapsed < DEBOUNCE_DURATION {
                    // Sleep for the remaining debounce time
                    std::thread::sleep(DEBOUNCE_DURATION - elapsed);
                    
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
    
    Ok(())
} 