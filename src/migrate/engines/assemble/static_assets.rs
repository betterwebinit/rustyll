use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::AssembleMigrator {
    pub(super) fn migrate_static_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, static assets could be in various locations:
        // - assets/ directory
        // - public/ directory
        // - static/ directory
        // - src/assets/ directory
        // - dist/ directory (built assets)
        
        let possible_asset_dirs = vec![
            source_dir.join("assets"),
            source_dir.join("public"),
            source_dir.join("static"),
            source_dir.join("src").join("assets"),
            source_dir.join("dist"),
        ];
        
        // Create destination assets directory
        let dest_assets_dir = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets_dir)?;
        
        let mut found_assets = false;
        
        // Process each potential assets directory
        for asset_dir in possible_asset_dirs {
            if asset_dir.exists() && asset_dir.is_dir() {
                if verbose {
                    log::info!("Migrating static assets from {}", asset_dir.display());
                }
                
                self.copy_assets(&asset_dir, &dest_assets_dir, result)?;
                found_assets = true;
            }
        }
        
        // Also look for common asset files directly in the source directory
        let common_asset_patterns = vec![
            "*.css", "*.js", "*.png", "*.jpg", "*.jpeg", "*.gif", "*.svg", "*.ico", "favicon.ico"
        ];
        
        for pattern in common_asset_patterns {
            for entry in glob::glob(&format!("{}/{}", source_dir.display(), pattern))
                .map_err(|e| format!("Failed to read glob pattern: {}", e))? {
                
                if let Ok(path) = entry {
                    if path.is_file() {
                        let file_name = path.file_name().unwrap();
                        let dest_path = dest_assets_dir.join(file_name);
                        
                        copy_file(&path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/{}", file_name.to_string_lossy()),
                            change_type: ChangeType::Copied,
                            description: "Static asset file copied".to_string(),
                        });
                        
                        found_assets = true;
                    }
                }
            }
        }
        
        // Create standard assets structure if no assets were found
        if !found_assets {
            if verbose {
                log::info!("No static assets found. Creating basic asset structure.");
            }
            
            self.create_basic_assets(&dest_assets_dir, result)?;
        }
        
        Ok(())
    }
    
    fn copy_assets(&self, asset_dir: &Path, dest_assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create subdirectories for common asset types
        let dest_css_dir = dest_assets_dir.join("css");
        let dest_js_dir = dest_assets_dir.join("js");
        let dest_images_dir = dest_assets_dir.join("images");
        let dest_fonts_dir = dest_assets_dir.join("fonts");
        
        create_dir_if_not_exists(&dest_css_dir)?;
        create_dir_if_not_exists(&dest_js_dir)?;
        create_dir_if_not_exists(&dest_images_dir)?;
        create_dir_if_not_exists(&dest_fonts_dir)?;
        
        // Walk through all files in the assets directory
        for entry in WalkDir::new(asset_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let rel_path = file_path.strip_prefix(asset_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                // Determine the appropriate destination directory based on file extension
                let dest_path = if let Some(ext) = file_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    
                    match ext_str.as_ref() {
                        "css" | "scss" | "sass" | "less" => {
                            // CSS files go to css/
                            if rel_path.parent().is_some() && rel_path.parent().unwrap().to_string_lossy().contains("css") {
                                // Keep original structure if it already has css in the path
                                dest_assets_dir.join(rel_path)
                            } else {
                                dest_css_dir.join(rel_path.file_name().unwrap())
                            }
                        },
                        "js" | "jsx" | "ts" | "tsx" => {
                            // JS files go to js/
                            if rel_path.parent().is_some() && rel_path.parent().unwrap().to_string_lossy().contains("js") {
                                // Keep original structure if it already has js in the path
                                dest_assets_dir.join(rel_path)
                            } else {
                                dest_js_dir.join(rel_path.file_name().unwrap())
                            }
                        },
                        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => {
                            // Image files go to images/
                            if rel_path.parent().is_some() && (
                                rel_path.parent().unwrap().to_string_lossy().contains("img") || 
                                rel_path.parent().unwrap().to_string_lossy().contains("image")
                            ) {
                                // Keep original structure if it already has image in the path
                                dest_assets_dir.join(rel_path)
                            } else {
                                dest_images_dir.join(rel_path.file_name().unwrap())
                            }
                        },
                        "ttf" | "otf" | "woff" | "woff2" | "eot" => {
                            // Font files go to fonts/
                            if rel_path.parent().is_some() && rel_path.parent().unwrap().to_string_lossy().contains("font") {
                                // Keep original structure if it already has font in the path
                                dest_assets_dir.join(rel_path)
                            } else {
                                dest_fonts_dir.join(rel_path.file_name().unwrap())
                            }
                        },
                        _ => {
                            // Other files go directly to assets/ or maintain their structure
                            dest_assets_dir.join(rel_path)
                        }
                    }
                } else {
                    // Files without extension go directly to assets/
                    dest_assets_dir.join(rel_path)
                };
                
                // Create parent directories if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Copy the file
                copy_file(file_path, &dest_path)?;
                
                // Create the relative path for the migration result
                let rel_dest_path = dest_path.strip_prefix(dest_assets_dir.parent().unwrap())
                    .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
                
                result.changes.push(MigrationChange {
                    file_path: rel_dest_path.to_string_lossy().to_string(),
                    change_type: ChangeType::Copied,
                    description: "Static asset file copied".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn create_basic_assets(&self, assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create basic CSS, JS, and image directories
        let css_dir = assets_dir.join("css");
        let js_dir = assets_dir.join("js");
        let images_dir = assets_dir.join("images");
        
        create_dir_if_not_exists(&css_dir)?;
        create_dir_if_not_exists(&js_dir)?;
        create_dir_if_not_exists(&images_dir)?;
        
        // Create a basic CSS file
        let css_content = r#"/**
 * Main stylesheet for the Rustyll site
 * Migrated from Assemble
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

/* Post styles */
.post-list {
  margin-top: 2rem;
}

.post-item {
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #e9ecef;
}

.post-meta {
  color: #6c757d;
  font-size: 0.875rem;
}

/* Footer styles */
.site-footer {
  background-color: #f8f9fa;
  padding: 1rem 0;
  border-top: 1px solid #e9ecef;
  margin-top: 2rem;
}
"#;
        
        fs::write(css_dir.join("main.css"), css_content)
            .map_err(|e| format!("Failed to write main.css: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/css/main.css".to_string(),
            change_type: ChangeType::Created,
            description: "Basic CSS file created".to_string(),
        });
        
        // Create a basic JavaScript file
        let js_content = r#"/**
 * Main JavaScript file for the Rustyll site
 * Migrated from Assemble
 */

document.addEventListener('DOMContentLoaded', function() {
  console.log('Rustyll site loaded successfully');
  
  // Add your JavaScript here
});
"#;
        
        fs::write(js_dir.join("main.js"), js_content)
            .map_err(|e| format!("Failed to write main.js: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/js/main.js".to_string(),
            change_type: ChangeType::Created,
            description: "Basic JavaScript file created".to_string(),
        });
        
        Ok(())
    }
} 