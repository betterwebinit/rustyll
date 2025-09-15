use std::path::Path;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::NanocMigrator {
    pub(super) fn migrate_static_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Nanoc, static files can be in several places:
        // - static/ directory
        // - output/ directory (compiled site)
        // - content/ directory with certain extensions
        
        // Check for static/ directory
        let static_dir = source_dir.join("static");
        if static_dir.exists() && static_dir.is_dir() {
            if verbose {
                log::info!("Migrating static assets from static/ directory");
            }
            
            // Create destination directory
            let dest_assets_dir = dest_dir.join("assets");
            create_dir_if_not_exists(&dest_assets_dir)?;
            
            // Copy all static files
            self.copy_static_directory(&static_dir, &dest_assets_dir, result)?;
        }
        
        // Check for output/ directory
        let output_dir = source_dir.join("output");
        if output_dir.exists() && output_dir.is_dir() {
            // Only copy specific asset types from output/
            if verbose {
                log::info!("Looking for assets in output/ directory");
            }
            
            // Create destination assets directory
            let dest_assets_dir = dest_dir.join("assets");
            create_dir_if_not_exists(&dest_assets_dir)?;
            
            // Copy only asset files (CSS, JS, images)
            self.copy_asset_files(&output_dir, &dest_assets_dir, result)?;
        }
        
        // Handle special content directories that might contain assets
        self.process_content_assets(source_dir, dest_dir, verbose, result)?;
        
        Ok(())
    }
    
    fn copy_static_directory(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Get the relative path from source directory
                let rel_path = file_path.strip_prefix(source_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                let dest_path = dest_dir.join(rel_path);
                
                // Create parent directories if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Copy the file
                copy_file(file_path, &dest_path)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("assets/{}", rel_path.display()),
                    change_type: ChangeType::Copied,
                    description: "Static asset copied".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn copy_asset_files(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Only copy specific asset types (CSS, JS, images)
        let asset_extensions = [
            "css", "js", "jpg", "jpeg", "png", "gif", "svg", "webp", 
            "ttf", "woff", "woff2", "eot", "ico", "pdf"
        ];
        
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    // Check if this is an asset file
                    if asset_extensions.contains(&ext.as_ref()) {
                        // Get the relative path from source directory
                        let rel_path = file_path.strip_prefix(source_dir)
                            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                        
                        // Organize assets by type
                        let dest_subdir = match ext.as_ref() {
                            "css" => "css",
                            "js" => "js",
                            "ttf" | "woff" | "woff2" | "eot" => "fonts",
                            "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "ico" => "images",
                            _ => "other",
                        };
                        
                        let dest_path = dest_dir.join(dest_subdir).join(rel_path.file_name().unwrap());
                        
                        // Create parent directories if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the file
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/{}/{}", dest_subdir, rel_path.file_name().unwrap().to_string_lossy()),
                            change_type: ChangeType::Copied,
                            description: format!("Asset file copied from output directory").to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn process_content_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for content subdirectories that might contain assets
        let content_dir = source_dir.join("content");
        
        if !content_dir.exists() || !content_dir.is_dir() {
            return Ok(());
        }
        
        let asset_dirs = [
            "assets", "images", "img", "css", "js", "javascript", 
            "stylesheets", "fonts", "media", "files"
        ];
        
        let dest_assets_dir = dest_dir.join("assets");
        
        for asset_dir_name in asset_dirs.iter() {
            let asset_dir = content_dir.join(asset_dir_name);
            
            if asset_dir.exists() && asset_dir.is_dir() {
                if verbose {
                    log::info!("Found asset directory in content: {}", asset_dir_name);
                }
                
                // Determine destination subdirectory
                let dest_subdir = match *asset_dir_name {
                    "css" | "stylesheets" => dest_assets_dir.join("css"),
                    "js" | "javascript" => dest_assets_dir.join("js"),
                    "fonts" => dest_assets_dir.join("fonts"),
                    "images" | "img" => dest_assets_dir.join("images"),
                    _ => dest_assets_dir.join(asset_dir_name),
                };
                
                create_dir_if_not_exists(&dest_subdir)?;
                
                // Copy all files from this directory
                for entry in WalkDir::new(&asset_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        // Get the relative path from asset directory
                        let rel_path = file_path.strip_prefix(&asset_dir)
                            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                        
                        let dest_path = dest_subdir.join(rel_path);
                        
                        // Create parent directories if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the file
                        copy_file(file_path, &dest_path)?;
                        
                        let subdir_name = dest_subdir.file_name().unwrap().to_string_lossy();
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/{}/{}", subdir_name, rel_path.display()),
                            change_type: ChangeType::Copied,
                            description: format!("Asset file copied from content/{}", asset_dir_name).to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
} 