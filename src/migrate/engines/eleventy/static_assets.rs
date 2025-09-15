use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::EleventyMigrator {
    pub(super) fn migrate_static(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Eleventy copies any non-template files to the output directory
        // Common patterns include public/, static/, assets/ directories
        let potential_static_dirs = vec![
            source_dir.join("public"),
            source_dir.join("static"),
            source_dir.join("assets"),
            source_dir.join("img"),
            source_dir.join("images"),
            source_dir.join("css"),
            source_dir.join("js"),
        ];
        
        // Create destination assets directory
        let dest_assets = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets)?;
        
        // Track if we found any static assets
        let mut found_static = false;
        
        // Process each potential static directory
        for static_dir in &potential_static_dirs {
            if static_dir.exists() && static_dir.is_dir() {
                found_static = true;
                
                let dir_name = static_dir.file_name()
                    .ok_or_else(|| "Invalid directory name".to_string())?
                    .to_string_lossy();
                
                if verbose {
                    log::info!("Migrating static assets from {}", static_dir.display());
                }
                
                // Create subdirectory for this category
                let subdir = match dir_name.as_ref() {
                    "css" | "styles" => "css",
                    "js" | "javascript" | "scripts" => "js", 
                    "img" | "images" => "images",
                    _ => dir_name.as_ref()
                };
                
                let dest_subdir = dest_assets.join(subdir);
                create_dir_if_not_exists(&dest_subdir)?;
                
                // Copy all files
                for entry in WalkDir::new(static_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        let relative_path = file_path.strip_prefix(static_dir)
                            .map_err(|e| format!("Failed to get relative path: {}", e))?;
                        
                        let dest_path = dest_subdir.join(relative_path);
                        
                        // Create parent directories if they don't exist
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the asset file
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/{}/{}", subdir, relative_path.display()),
                            change_type: ChangeType::Copied,
                            description: format!("Static asset copied from {}", dir_name),
                        });
                    }
                }
            }
        }
        
        // Check for root-level assets (common in Eleventy sites)
        let asset_extensions = vec!["css", "js", "png", "jpg", "jpeg", "gif", "svg", "ico", "woff", "woff2", "ttf", "eot"];
        
        for entry in fs::read_dir(source_dir)
            .map_err(|e| format!("Failed to read directory {}: {}", source_dir.display(), e))? {
            
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    
                    if asset_extensions.contains(&ext_str.as_str()) {
                        found_static = true;
                        
                        // Determine the asset type
                        let asset_type = match ext_str.as_str() {
                            "css" => "css",
                            "js" => "js",
                            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" => "images",
                            "woff" | "woff2" | "ttf" | "eot" => "fonts",
                            _ => "misc",
                        };
                        
                        let dest_subdir = dest_assets.join(asset_type);
                        create_dir_if_not_exists(&dest_subdir)?;
                        
                        let file_name = path.file_name()
                            .ok_or_else(|| "Invalid file name".to_string())?;
                        
                        let dest_path = dest_subdir.join(file_name);
                        
                        if verbose {
                            log::info!("Migrating root asset: {}", file_name.to_string_lossy());
                        }
                        
                        // Copy the asset file
                        copy_file(&path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/{}/{}", asset_type, file_name.to_string_lossy()),
                            change_type: ChangeType::Copied,
                            description: "Root-level asset file copied".to_string(),
                        });
                    }
                }
            }
        }
        
        // Create README for assets directory
        let assets_readme = r#"# Assets Directory

This directory contains static assets migrated from Eleventy.

## Asset Structure

- `/assets/css/` - Stylesheets 
- `/assets/js/` - JavaScript files
- `/assets/images/` - Images
- `/assets/fonts/` - Font files
- `/assets/public/` - Other public files

## Changes from Eleventy

- Eleventy can serve any file in the source directory as a static asset
- In Rustyll, static assets should be organized in the assets directory
- Update references in templates and content to use the new asset paths
"#;
        
        write_readme(&dest_assets, assets_readme)?;
        
        if !found_static {
            result.warnings.push(
                "Could not identify clear static asset directories in the Eleventy site.".to_string()
            );
        }
        
        Ok(())
    }
} 