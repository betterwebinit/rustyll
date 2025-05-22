use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::MiddlemanMigrator {
    pub(super) fn migrate_static(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Middleman, static assets are in various locations:
        // - source/images/
        // - source/javascripts/
        // - source/stylesheets/
        // - source/fonts/
        // - And potentially other directories in source/
        
        let source_content_dir = source_dir.join("source");
        if !source_content_dir.exists() || !source_content_dir.is_dir() {
            result.warnings.push("Could not find standard Middleman source directory.".to_string());
            return Ok(());
        }
        
        // Create destination assets directory
        let dest_assets = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets)?;
        
        // Define asset directories to look for
        let asset_dirs = [
            ("images", "images"),
            ("javascripts", "js"),
            ("stylesheets", "css"),
            ("fonts", "fonts"),
            ("css", "css"),
            ("js", "js"),
            ("img", "images"),
        ];
        
        let mut found_assets = false;
        
        // Process each asset directory
        for (source_subdir, dest_subdir) in &asset_dirs {
            let source_asset_dir = source_content_dir.join(source_subdir);
            if source_asset_dir.exists() && source_asset_dir.is_dir() {
                found_assets = true;
                
                if verbose {
                    log::info!("Migrating assets from {}", source_asset_dir.display());
                }
                
                let dest_asset_dir = dest_assets.join(dest_subdir);
                create_dir_if_not_exists(&dest_asset_dir)?;
                
                // Copy all files in the asset directory
                for entry in WalkDir::new(&source_asset_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        let relative_path = file_path.strip_prefix(&source_asset_dir)
                            .map_err(|e| format!("Failed to get relative path: {}", e))?;
                        
                        let dest_path = dest_asset_dir.join(relative_path);
                        
                        // Create parent directories if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Special handling for SCSS and SASS files
                        if let Some(extension) = file_path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            
                            if ext == "scss" || ext == "sass" {
                                // For SCSS/SASS files, we might need to adjust imports
                                // This is a simplified placeholder - real implementation would be more complex
                                let content = fs::read_to_string(file_path)
                                    .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                                
                                // Convert Middleman-style imports to standard SCSS/SASS imports
                                let converted_content = content
                                    .replace("@import \"", "@import \"../")  // Adjust import paths if needed
                                    .replace("asset_path(", "url(")         // Middleman asset helpers
                                    .replace("asset_url(", "url(")          // to standard CSS functions
                                    .replace("image_path(", "url(")
                                    .replace("image_url(", "url(");
                                
                                fs::write(&dest_path, converted_content)
                                    .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()),
                                    change_type: ChangeType::Converted,
                                    description: format!("{} file converted to standard format", ext),
                                });
                            } else if ext == "coffee" {
                                // For CoffeeScript files, we should warn about conversion
                                // Copy the original for reference
                                copy_file(file_path, &dest_path)?;
                                
                                // And create a JavaScript version with placeholder
                                let js_path = dest_path.with_extension("js");
                                let js_content = "/* This is a placeholder converted from CoffeeScript.\n * Manual conversion is required.\n * Original file: {} \n */\n\nconsole.log('Convert CoffeeScript to JavaScript');\n";
                                
                                fs::write(&js_path, js_content)
                                    .map_err(|e| format!("Failed to write JS placeholder: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()),
                                    change_type: ChangeType::Copied,
                                    description: "Original CoffeeScript file copied for reference".to_string(),
                                });
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/{}/{}.js", dest_subdir, relative_path.with_extension("").display()),
                                    change_type: ChangeType::Created,
                                    description: "Placeholder JavaScript file created".to_string(),
                                });
                                
                                result.warnings.push(format!(
                                    "CoffeeScript file assets/{}/{} requires manual conversion to JavaScript",
                                    dest_subdir, relative_path.display()
                                ));
                            } else {
                                // For regular asset files, just copy
                                copy_file(file_path, &dest_path)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()),
                                    change_type: ChangeType::Copied,
                                    description: "Asset file copied".to_string(),
                                });
                            }
                        } else {
                            // For files without extension, just copy
                            copy_file(file_path, &dest_path)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()),
                                change_type: ChangeType::Copied,
                                description: "Asset file copied".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Create README for assets directory
        let assets_readme = r#"# Assets Directory

This directory contains static assets migrated from Middleman.

## Asset Structure

- `/assets/css/` - Stylesheets (from Middleman's stylesheets directory)
- `/assets/js/` - JavaScript files (from Middleman's javascripts directory)
- `/assets/images/` - Images (from Middleman's images directory)
- `/assets/fonts/` - Font files (from Middleman's fonts directory)

## Changes from Middleman

- Middleman uses the asset pipeline, while Rustyll has a simpler asset structure
- Asset paths in templates need to be updated to use the new structure
- Middleman helpers like `asset_path` should be replaced with direct paths
- CSS/SCSS imports may need path adjustments
"#;
        
        write_readme(&dest_assets, assets_readme)?;
        
        if !found_assets {
            result.warnings.push(
                "Could not identify standard asset directories in the Middleman site.".to_string()
            );
        }
        
        Ok(())
    }
} 