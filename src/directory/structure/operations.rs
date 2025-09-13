use std::fs;
use log::{debug, warn};
use walkdir::WalkDir;
use crate::directory::types::BoxResult;
use crate::directory::utils::is_convertible_file;
use super::directory_structure::DirectoryStructure;

impl DirectoryStructure {
    /// Create required directories for site generation
    pub fn create_site_directories(&self) -> BoxResult<()> {
        // Create destination directory if it doesn't exist
        if !self.destination.exists() {
            fs::create_dir_all(&self.destination)?;
            debug!("Created destination directory: {}", self.destination.display());
        }
        
        Ok(())
    }

    /// Copy static files from source to destination
    pub fn copy_static_files(&self) -> BoxResult<usize> {
        let mut copied_count = 0;
        
        for entry in WalkDir::new(&self.source)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            // Skip special directories and excluded files
            if self.is_excluded(path) || self.is_special_directory(path) {
                continue;
            }
            
            // Skip files that will be processed as pages/collections
            if is_convertible_file(path) {
                continue;
            }
            
            // Calculate destination path
            let rel_path = path.strip_prefix(&self.source).unwrap();
            let dest_path = self.destination.join(rel_path);
            
            // Create parent directory if it doesn't exist
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            
            // Copy the file
            match fs::copy(path, &dest_path) {
                Ok(_) => {
                    debug!("Copied static file: {} -> {}", 
                           path.display(), 
                           dest_path.display());
                    copied_count += 1;
                },
                Err(e) => {
                    warn!("Failed to copy file {}: {}", path.display(), e);
                }
            }
        }
        
        Ok(copied_count)
    }
} 