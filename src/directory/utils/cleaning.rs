use std::path::Path;
use std::fs;
use std::io;
use log::{info, warn};
use crate::config::Config;
use super::path_helpers::{is_safe_delete_path, path_matches_pattern};

/// Clean destination directory
pub fn clean_destination(config: &Config) -> io::Result<()> {
    let destination = &config.destination;
    
    if destination.exists() {
        // Check if destination is outside of source in safe mode
        if config.safe_mode && !is_safe_delete_path(destination, config) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, 
                format!("Safe mode: Cannot delete directory outside of source: {}", 
                destination.display())
            ));
        }
        
        info!("Cleaning destination directory: {}", destination.display());
        
        // If keep_files is defined, don't delete the directory entirely
        if !config.keep_files.is_empty() {
            // Remove all files except those in keep_files
            for entry in fs::read_dir(destination)? {
                let entry = entry?;
                let path = entry.path();
                let rel_path = path.strip_prefix(destination).unwrap_or(&path);
                let path_str = rel_path.to_string_lossy().to_string();
                
                // Check if this path should be kept
                let should_keep = config.keep_files.iter().any(|pattern| {
                    path_matches_pattern(&path_str, pattern)
                });
                
                if !should_keep {
                    if path.is_dir() {
                        fs::remove_dir_all(&path)?;
                    } else {
                        fs::remove_file(&path)?;
                    }
                }
            }
        } else {
            // Remove the entire destination directory
            fs::remove_dir_all(destination)?;
            fs::create_dir_all(destination)?;
        }
    }
    
    Ok(())
}

/// Clean a directory by removing its contents
pub fn clean_directory(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    info!("Cleaning directory: {}", path.display());
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                fs::remove_dir_all(entry_path)?;
            } else {
                fs::remove_file(entry_path)?;
            }
        }
    } else {
        warn!("Attempted to clean a non-directory: {}", path.display());
    }
    
    Ok(())
} 