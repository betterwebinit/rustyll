use std::path::Path;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::HugoMigrator {
    pub(super) fn migrate_static(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let static_source_dir = source_dir.join("static");
        
        if static_source_dir.exists() && static_source_dir.is_dir() {
            if verbose {
                log::info!("Migrating static assets");
            }
            
            // In Hugo, static files go to site root, same in Rustyll
            // Process all static files
            for entry in WalkDir::new(&static_source_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    // Get the relative path from static directory
                    let rel_path = entry.path().strip_prefix(&static_source_dir)
                        .map_err(|_| "Failed to get relative path".to_string())?;
                    
                    let dest_path = dest_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the file
                    copy_file(entry.path(), &dest_path)?;
                    
                    let file_path_str = rel_path.to_string_lossy().to_string();
                    result.changes.push(MigrationChange {
                        file_path: file_path_str,
                        change_type: ChangeType::Copied,
                        description: "Static asset copied".to_string(),
                    });
                }
            }
        }
        
        // Also check for assets directory, which is another convention in Hugo
        let assets_source_dir = source_dir.join("assets");
        if assets_source_dir.exists() && assets_source_dir.is_dir() {
            if verbose {
                log::info!("Migrating from assets directory to assets in destination");
            }
            
            let assets_dest_dir = dest_dir.join("assets");
            create_dir_if_not_exists(&assets_dest_dir)?;
            
            // Process all asset files
            for entry in WalkDir::new(&assets_source_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    // Get the relative path from assets directory
                    let rel_path = entry.path().strip_prefix(&assets_source_dir)
                        .map_err(|_| "Failed to get relative path".to_string())?;
                    
                    let dest_path = assets_dest_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the file
                    copy_file(entry.path(), &dest_path)?;
                    
                    let file_path_str = format!("assets/{}", rel_path.to_string_lossy());
                    result.changes.push(MigrationChange {
                        file_path: file_path_str,
                        change_type: ChangeType::Copied,
                        description: "Asset file copied".to_string(),
                    });
                }
            }
            
            result.warnings.push(
                "Hugo's asset pipeline uses different approaches than Rustyll. Manual adjustments may be needed.".to_string()
            );
        }
        
        Ok(())
    }
} 