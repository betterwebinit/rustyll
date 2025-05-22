use std::path::Path;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

// Public module function that can be called from anywhere
pub fn migrate_static(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let migrator = super::ZolaMigrator::new();
    migrator.migrate_static(source_dir, dest_dir, verbose, result)
}

impl super::ZolaMigrator {
    pub(super) fn migrate_static(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let static_source_dir = source_dir.join("static");
        
        if !static_source_dir.exists() {
            result.warnings.push("No static directory found in Zola site".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating static assets from Zola to Rustyll format");
        }
        
        // In Zola, static files are in the /static directory
        // In Rustyll, we'll put them in /assets
        let assets_dest_dir = dest_dir.join("assets");
        create_dir_if_not_exists(&assets_dest_dir)?;
        
        // Process all static files recursively
        for entry in WalkDir::new(&static_source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let rel_path = file_path.strip_prefix(&static_source_dir)
                    .map_err(|_| "Failed to get relative path".to_string())?;
                
                let dest_path = assets_dest_dir.join(rel_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Copy the file
                copy_file(file_path, &dest_path)?;
                
                let file_path_str = format!("assets/{}", rel_path.to_string_lossy());
                result.changes.push(MigrationChange {
                    file_path: file_path_str,
                    change_type: ChangeType::Copied,
                    description: "Static asset copied".to_string(),
                });
            }
        }
        
        // Also check for sass directory, which is another convention in Zola
        let sass_source_dir = source_dir.join("sass");
        if sass_source_dir.exists() && sass_source_dir.is_dir() {
            if verbose {
                log::info!("Migrating Sass files from Zola to Rustyll");
            }
            
            let sass_dest_dir = dest_dir.join("_sass");
            create_dir_if_not_exists(&sass_dest_dir)?;
            
            // Process all sass files
            for entry in WalkDir::new(&sass_source_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let rel_path = file_path.strip_prefix(&sass_source_dir)
                        .map_err(|_| "Failed to get relative path".to_string())?;
                    
                    let dest_path = sass_dest_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the file
                    copy_file(file_path, &dest_path)?;
                    
                    let file_path_str = format!("_sass/{}", rel_path.to_string_lossy());
                    result.changes.push(MigrationChange {
                        file_path: file_path_str,
                        change_type: ChangeType::Copied,
                        description: "Sass file copied".to_string(),
                    });
                }
            }
            
            result.warnings.push(
                "Zola's Sass processing is different from Rustyll's. Manual adjustments may be needed.".to_string()
            );
        }
        
        Ok(())
    }
} 