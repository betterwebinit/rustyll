use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use log::{debug, error, warn};
use walkdir::WalkDir;

/// Copy a file or directory, preserving its structure
pub fn copy(source: &Path, destination: &Path) -> io::Result<()> {
    debug!("Copying from {} to {}", source.display(), destination.display());
    
    if source.is_dir() {
        // Copy directory
        if !destination.exists() {
            fs::create_dir_all(destination)?;
        }
        
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let target_path = destination.join(entry_path.file_name().unwrap());
            
            if entry_path.is_dir() {
                copy(&entry_path, &target_path)?;
            } else {
                copy_file(&entry_path, &target_path)?;
            }
        }
        
        Ok(())
    } else {
        // Copy file
        copy_file(source, destination)
    }
}

/// Copy a file with better error handling
pub fn copy_file(source: &Path, destination: &Path) -> io::Result<()> {
    debug!("Copying file from {} to {}", source.display(), destination.display());
    
    // Ensure parent directory exists
    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    // Use a binary copy approach to handle all file types
    match fs::copy(source, destination) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to copy file from {} to {}: {}", 
                source.display(), destination.display(), e);
            Err(e)
        }
    }
}

/// Copy static files from one directory to another
pub fn copy_static_files(
    source: &Path,
    destination: &Path,
    exclude_dirs: &[PathBuf]
) -> io::Result<usize> {
    let mut copied_count = 0;
    
    for entry in WalkDir::new(source)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories in the exclude list
        if exclude_dirs.iter().any(|exclude| path.starts_with(exclude)) {
            continue;
        }
        
        if path.is_file() {
            // Calculate the relative path from source
            if let Ok(rel_path) = path.strip_prefix(source) {
                let dest_path = destination.join(rel_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                
                // Copy the file
                match fs::copy(path, &dest_path) {
                    Ok(_) => {
                        debug!("Copied static file: {} -> {}", 
                            path.display(), dest_path.display());
                        copied_count += 1;
                    },
                    Err(e) => {
                        warn!("Failed to copy file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    Ok(copied_count)
} 