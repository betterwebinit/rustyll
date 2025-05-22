use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::JekyllMigrator {
    pub(super) fn migrate_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Handle _sass directory
        let sass_source_dir = source_dir.join("_sass");
        if sass_source_dir.exists() {
            let sass_dest_dir = dest_dir.join("_sass");
            
            if verbose {
                log::info!("Migrating Sass files");
            }
            
            create_dir_if_not_exists(&sass_dest_dir)?;
            
            // Process all sass files recursively
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
                        change_type: ChangeType::Converted,
                        description: "Sass file migrated".to_string(),
                    });
                }
            }
            
            // Create README for sass directory
            let sass_readme = r#"# Sass Directory

This directory contains Sass files migrated from Jekyll.

## Sass Usage

Sass files in Rustyll work the same way as in Jekyll:
- Files are preprocessed and compiled to CSS
- The main Sass file is typically in the `assets/css` directory
- Partials (files starting with `_`) can be imported into other Sass files

## Changes from Jekyll

Sass processing in Rustyll is compatible with Jekyll, but some advanced
configurations might require adjustments.
"#;
            
            write_readme(&sass_dest_dir, sass_readme)?;
        }
        
        // Handle assets directory
        let assets_dirs = vec![
            source_dir.join("assets"),
            source_dir.join("css"),
            source_dir.join("js"),
            source_dir.join("images"),
            source_dir.join("img"),
        ];
        
        for assets_dir in assets_dirs {
            if assets_dir.exists() && assets_dir.is_dir() {
                let dir_name = assets_dir.file_name()
                    .ok_or_else(|| "Invalid directory name".to_string())?
                    .to_string_lossy();
                
                let dest_asset_dir = if dir_name == "assets" {
                    dest_dir.join("assets")
                } else {
                    dest_dir.join("assets").join(dir_name.to_string())
                };
                
                if verbose {
                    log::info!("Migrating assets from {}", assets_dir.display());
                }
                
                create_dir_if_not_exists(&dest_asset_dir)?;
                
                // Process all asset files recursively
                for entry in WalkDir::new(&assets_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        let rel_path = file_path.strip_prefix(&assets_dir)
                            .map_err(|_| "Failed to get relative path".to_string())?;
                        
                        let dest_path = dest_asset_dir.join(rel_path);
                        
                        // Create parent directory if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the file
                        copy_file(file_path, &dest_path)?;
                        
                        let file_path_str = if dir_name == "assets" {
                            format!("assets/{}", rel_path.to_string_lossy())
                        } else {
                            format!("assets/{}/{}", dir_name, rel_path.to_string_lossy())
                        };
                        
                        result.changes.push(MigrationChange {
                            file_path: file_path_str,
                            change_type: ChangeType::Converted,
                            description: "Asset file migrated".to_string(),
                        });
                    }
                }
            }
        }
        
        // Create assets README if it doesn't exist yet
        let assets_dest_dir = dest_dir.join("assets");
        if !assets_dest_dir.exists() {
            create_dir_if_not_exists(&assets_dest_dir)?;
        }
        
        let assets_readme = r#"# Assets Directory

This directory contains static assets migrated from Jekyll.

## Assets Organization

Assets in Rustyll:
- CSS, JavaScript, images, and other static files go here
- Can be organized in subdirectories (css, js, images, etc.)
- Are referenced in templates relative to the site root

## Examples

Referencing assets in templates:
```html
<link rel="stylesheet" href="/assets/css/main.css">
<script src="/assets/js/script.js"></script>
<img src="/assets/images/logo.png" alt="Logo">
```
"#;
        
        write_readme(&assets_dest_dir, assets_readme)?;
        
        Ok(())
    }
} 