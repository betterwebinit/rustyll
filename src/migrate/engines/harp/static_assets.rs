use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::HarpMigrator {
    pub(super) fn migrate_static_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, static assets can be anywhere in the content directory
        // Common patterns are 'images/', 'css/', 'js/', etc.
        
        // Determine content directory
        let content_dir = if source_dir.join("public").exists() && source_dir.join("public").is_dir() {
            source_dir.join("public")
        } else {
            source_dir.to_path_buf()
        };
        
        // Create destination assets directory
        let dest_assets_dir = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets_dir)?;
        
        // Create subdirectories for assets
        let dest_css_dir = dest_assets_dir.join("css");
        let dest_js_dir = dest_assets_dir.join("js");
        let dest_images_dir = dest_assets_dir.join("images");
        
        create_dir_if_not_exists(&dest_css_dir)?;
        create_dir_if_not_exists(&dest_js_dir)?;
        create_dir_if_not_exists(&dest_images_dir)?;
        
        // Track if we found any assets
        let mut found_assets = false;
        
        // Find and copy static assets
        for entry in WalkDir::new(&content_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip files in special directories or underscore directories
                let rel_path = match file_path.strip_prefix(&content_dir) {
                    Ok(path) => path,
                    Err(_) => continue,
                };
                
                let path_str = rel_path.to_string_lossy();
                
                if path_str.starts_with('_') || path_str.contains("/_") {
                    continue; // Skip files in underscore directories
                }
                
                // Process file based on extension
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    match ext.as_ref() {
                        // CSS files
                        "css" | "less" | "scss" | "sass" => {
                            self.copy_asset_to_dir(file_path, &content_dir, &dest_css_dir, "css", result)?;
                            found_assets = true;
                        },
                        
                        // JavaScript files
                        "js" | "jsx" | "ts" | "coffee" => {
                            self.copy_asset_to_dir(file_path, &content_dir, &dest_js_dir, "js", result)?;
                            found_assets = true;
                        },
                        
                        // Image files
                        "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "ico" => {
                            self.copy_asset_to_dir(file_path, &content_dir, &dest_images_dir, "images", result)?;
                            found_assets = true;
                        },
                        
                        // Skip content files
                        "md" | "markdown" | "html" | "jade" | "ejs" | "json" => continue,
                        
                        // Other files, copy to assets root
                        _ => {
                            // But only if they're in already in an assets-like directory
                            if path_str.contains("/images/") || 
                               path_str.contains("/css/") || 
                               path_str.contains("/js/") || 
                               path_str.contains("/img/") || 
                               path_str.contains("/assets/") || 
                               path_str.contains("/fonts/") {
                                self.copy_asset_preserving_structure(file_path, &content_dir, &dest_assets_dir, result)?;
                                found_assets = true;
                            }
                        }
                    }
                }
            }
        }
        
        // Also check for common asset directories and copy them directly
        let asset_dirs = vec![
            ("images", "images"),
            ("img", "images"),
            ("css", "css"),
            ("styles", "css"),
            ("js", "js"),
            ("javascript", "js"),
            ("fonts", "fonts"),
            ("assets", ""),
        ];
        
        for (src_name, dest_name) in asset_dirs {
            let src_dir = content_dir.join(src_name);
            if src_dir.exists() && src_dir.is_dir() {
                if verbose {
                    log::info!("Migrating assets from directory: {}", src_dir.display());
                }
                
                let dest_subdir = if dest_name.is_empty() {
                    dest_assets_dir.clone()
                } else {
                    let dir = dest_assets_dir.join(dest_name);
                    create_dir_if_not_exists(&dir)?;
                    dir
                };
                
                self.copy_asset_directory(&src_dir, &dest_subdir, result)?;
                found_assets = true;
            }
        }
        
        // Create basic assets if none were found
        if !found_assets {
            if verbose {
                log::info!("No assets found. Creating basic asset files.");
            }
            
            self.create_basic_assets(&dest_assets_dir, result)?;
        }
        
        // Create README for assets directory
        write_assets_readme(&dest_assets_dir, result)?;
        
        Ok(())
    }
    
    fn copy_asset_to_dir(&self, file_path: &Path, source_dir: &Path, dest_dir: &Path, asset_type: &str, result: &mut MigrationResult) -> Result<(), String> {
        let file_name = file_path.file_name().unwrap_or_default();
        let dest_path = dest_dir.join(file_name);
        
        copy_file(file_path, &dest_path)?;
        
        result.changes.push(MigrationChange {
            file_path: format!("assets/{}/{}", asset_type, file_name.to_string_lossy()),
            change_type: ChangeType::Converted,
            description: format!("{} file copied", asset_type),
        });
        
        Ok(())
    }
    
    fn copy_asset_preserving_structure(&self, file_path: &Path, source_dir: &Path, dest_assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Get the relative path from source directory
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
        // Create destination path preserving directory structure
        let dest_path = dest_assets_dir.join(rel_path);
        
        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        result.changes.push(MigrationChange {
            file_path: format!("assets/{}", rel_path.to_string_lossy()),
            change_type: ChangeType::Converted,
            description: "Asset file copied".to_string(),
        });
        
        Ok(())
    }
    
    fn copy_asset_directory(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Get the relative path from source directory
                let rel_path = file_path.strip_prefix(source_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                    
                // Create destination path preserving directory structure
                let dest_path = dest_dir.join(rel_path);
                
                // Create parent directories if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Copy the file
                copy_file(file_path, &dest_path)?;
                
                // Get the path relative to assets directory for the changelog
                let assets_rel_path = dest_path.strip_prefix(dest_dir.parent().unwrap())
                    .map_err(|_| format!("Failed to get assets relative path for {}", dest_path.display()))?;
                
                result.changes.push(MigrationChange {
                    file_path: assets_rel_path.to_string_lossy().to_string(),
                    change_type: ChangeType::Converted,
                    description: "Asset file copied".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn create_basic_assets(&self, assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create CSS directory and basic stylesheet
        let css_dir = assets_dir.join("css");
        create_dir_if_not_exists(&css_dir)?;
        
        let css_content = r#"/**
 * Main stylesheet for the migrated site
 */

/* Global styles */
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  line-height: 1.6;
  color: #333;
  margin: 0;
  padding: 0;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 15px;
}

/* Header styles */
.site-header {
  background-color: #f8f9fa;
  padding: 1rem 0;
  border-bottom: 1px solid #e9ecef;
}

.site-title {
  font-size: 1.5rem;
  font-weight: bold;
  color: #333;
  text-decoration: none;
}

.site-nav ul {
  list-style: none;
  display: flex;
  margin: 0;
  padding: 0;
}

.site-nav li {
  margin-right: 1rem;
}

.site-nav a {
  color: #333;
  text-decoration: none;
}

.site-nav a:hover {
  color: #007bff;
}

/* Content styles */
.content {
  padding: 2rem 0;
}

/* Footer styles */
.site-footer {
  background-color: #f8f9fa;
  padding: 1rem 0;
  border-top: 1px solid #e9ecef;
  margin-top: 2rem;
}
"#;
        
        let css_file = css_dir.join("main.css");
        fs::write(&css_file, css_content)
            .map_err(|e| format!("Failed to write CSS file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/css/main.css".to_string(),
            change_type: ChangeType::Created,
            description: "Basic CSS file created".to_string(),
        });
        
        // Create JS directory and basic script
        let js_dir = assets_dir.join("js");
        create_dir_if_not_exists(&js_dir)?;
        
        let js_content = r#"/**
 * Main JavaScript file for the migrated site
 */

document.addEventListener('DOMContentLoaded', function() {
  console.log('Site loaded');
  
  // Add your JavaScript here
});
"#;
        
        let js_file = js_dir.join("main.js");
        fs::write(&js_file, js_content)
            .map_err(|e| format!("Failed to write JavaScript file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/js/main.js".to_string(),
            change_type: ChangeType::Created,
            description: "Basic JavaScript file created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to write a README for the assets directory
fn write_assets_readme(assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    let readme_content = r#"# Assets Directory

This directory contains static assets for your Rustyll site:

- `css/` - Stylesheets
- `js/` - JavaScript files
- `images/` - Image files
- `fonts/` - Font files (if any)

## Migrated Assets

Assets in this directory were migrated from the Harp site:
- Files were categorized based on their extensions
- Directory structure was preserved where it made sense
- Some assets might have been moved to better organize them

## Using Assets in Templates

To reference assets in your templates:

```liquid
<link rel="stylesheet" href="{{ "/assets/css/main.css" | relative_url }}">
<script src="{{ "/assets/js/main.js" | relative_url }}"></script>
<img src="{{ "/assets/images/logo.png" | relative_url }}" alt="Logo">
```
"#;

    write_readme(assets_dir, readme_content)?;
    
    result.changes.push(MigrationChange {
        file_path: "assets/README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Assets directory README created".to_string(),
    });
    
    Ok(())
} 