use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;
use log::{info, debug, error};
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event, EventKind, Config as NotifyConfig, RecommendedWatcher, event::DataChange};

use crate::server::types::BoxResult;
use crate::server::core::watcher::should_ignore_path;

/// Watch a directory for file changes and send events to a channel
pub fn watch_directory<P: AsRef<Path>>(
    directory: P,
    event_sender: Sender<Event>,
    debounce_duration: Duration,
    ignore_patterns: &[String]
) -> BoxResult<RecommendedWatcher> {
    let ignore_patterns = ignore_patterns.to_vec(); // Clone for move into closure
    
    // Configure the watcher with better performance settings
    let _config = NotifyConfig::default()
        .with_poll_interval(debounce_duration)
        .with_compare_contents(false); // Only check timestamps, not file contents
    
    let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
        match res {
            Ok(event) => {
                // Only trigger rebuild on file modifications
                match event.kind {
                    EventKind::Create(_) | 
                    EventKind::Modify(ModifyKind::Data(DataChange::Any)) | 
                    EventKind::Modify(ModifyKind::Name(_)) | 
                    EventKind::Remove(_) => {
                        // Check if the file should be ignored
                        let should_ignore = event.paths.iter().any(|p| should_ignore_path(p, &ignore_patterns));
                        
                        if !should_ignore {
                            // Send event to channel
                            if let Err(e) = event_sender.send(event) {
                                error!("Error sending file event: {}", e);
                            }
                        } else {
                            debug!("Ignoring file change event due to pattern match");
                        }
                    },
                    _ => {}
                }
            },
            Err(e) => error!("Watch error: {}", e),
        }
    })?;
    
    // Watch the directory
    watcher.watch(directory.as_ref(), RecursiveMode::Recursive)?;
    info!("Watching for changes in {}", directory.as_ref().display());
    
    Ok(watcher)
}

/// Generate a websocket URL for LiveReload
pub fn livereload_url(host: &str, port: u16) -> String {
    // Jekyll uses ws://HOST:PORT/livereload
    format!("ws://{}:{}/livereload", host, port)
}

use notify::event::ModifyKind; 