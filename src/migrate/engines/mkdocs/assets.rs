use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MkDocs assets...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;
    
    // Create subdirectories
    let css_dir = dest_assets_dir.join("css");
    let js_dir = dest_assets_dir.join("js");
    let images_dir = dest_assets_dir.join("images");
    
    create_dir_if_not_exists(&css_dir)?;
    create_dir_if_not_exists(&js_dir)?;
    create_dir_if_not_exists(&images_dir)?;
    
    // Check for various asset directories in MkDocs
    let potential_asset_dirs = vec![
        source_dir.join("docs/assets"),
        source_dir.join("docs/css"),
        source_dir.join("docs/js"),
        source_dir.join("docs/images"),
        source_dir.join("docs/img"),
        source_dir.join("docs/stylesheets"),
        source_dir.join("docs/javascripts"),
        source_dir.join("docs/static"),
        source_dir.join("assets"),
        source_dir.join("css"),
        source_dir.join("js"),
        source_dir.join("images"),
        source_dir.join("img"),
        source_dir.join("static"),
    ];
    
    // Process each potential asset directory
    for asset_dir in &potential_asset_dirs {
        if asset_dir.exists() && asset_dir.is_dir() {
            if verbose {
                log::info!("Processing asset directory: {}", asset_dir.display());
            }
            
            // Determine asset type based on directory name
            let dirname = asset_dir.file_name().unwrap().to_string_lossy().to_lowercase();
            
            let dest_subdir = if dirname.contains("css") || dirname.contains("style") {
                &css_dir
            } else if dirname.contains("js") || dirname.contains("javascript") {
                &js_dir
            } else if dirname.contains("img") || dirname.contains("image") {
                &images_dir
            } else {
                // For general asset directories, we need to look at file extensions
                // Just continue and we'll handle files individually
                continue;
            };
            
            // Copy all files from this directory
            copy_asset_directory(&asset_dir, dest_subdir, result)?;
        }
    }
    
    // Look for general asset directories and process files by type
    for asset_dir in potential_asset_dirs.iter().filter(|dir| dir.exists() && dir.is_dir() && {
        let name = dir.file_name().unwrap().to_string_lossy().to_lowercase();
        name == "assets" || name == "static"
    }) {
        process_general_asset_directory(asset_dir, &css_dir, &js_dir, &images_dir, result)?;
    }
    
    // Create a basic CSS file if none exists
    if is_directory_empty(&css_dir)? {
        create_default_css(&css_dir, result)?;
    }
    
    Ok(())
}

fn copy_asset_directory(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the source directory
            let rel_path = file_path.strip_prefix(source_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create destination path
            let dest_path = dest_dir.join(rel_path);
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Copy the file
            copy_file(file_path, &dest_path)?;
            
            // Report the change
            let rel_dest = dest_path.strip_prefix(dest_dir.parent().unwrap_or(dest_dir))
                .unwrap_or(dest_path.as_path());
            
            result.changes.push(MigrationChange {
                file_path: rel_dest.to_string_lossy().to_string(),
                change_type: ChangeType::Copied,
                description: format!("Copied asset from {}", file_path.display()),
            });
        }
    }
    
    Ok(())
}

fn process_general_asset_directory(
    asset_dir: &Path,
    css_dir: &Path,
    js_dir: &Path,
    images_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(asset_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Determine the file type based on extension
            let extension = file_path.extension().map(|ext| ext.to_string_lossy().to_lowercase());
            
            let dest_dir = match extension.as_deref() {
                Some("css") | Some("scss") | Some("sass") => css_dir,
                Some("js") | Some("mjs") => js_dir,
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("svg") | Some("webp") => images_dir,
                _ => {
                    // Skip unknown file types
                    continue;
                }
            };
            
            // Get the filename
            let filename = file_path.file_name().unwrap();
            let dest_path = dest_dir.join(filename);
            
            // Copy the file
            copy_file(file_path, &dest_path)?;
            
            // Report the change
            let rel_dest = dest_path.strip_prefix(dest_dir.parent().unwrap_or(dest_dir))
                .unwrap_or(dest_path.as_path());
            
            result.changes.push(MigrationChange {
                file_path: rel_dest.to_string_lossy().to_string(),
                change_type: ChangeType::Copied,
                description: format!("Copied asset from {}", file_path.display()),
            });
        }
    }
    
    Ok(())
}

fn is_directory_empty(dir: &Path) -> Result<bool, String> {
    if !dir.exists() {
        return Ok(true);
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    Ok(entries.count() == 0)
}

fn create_default_css(css_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    let main_css_path = css_dir.join("main.css");
    let css_content = r#"/* Default CSS generated by Rustyll */
:root {
  --primary-color: #007bff;
  --secondary-color: #6c757d;
  --bg-color: #ffffff;
  --text-color: #333333;
  --link-color: #0056b3;
  --header-bg: #f8f9fa;
  --footer-bg: #f8f9fa;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-color);
  background-color: var(--bg-color);
  line-height: 1.6;
  margin: 0;
  padding: 0;
}

a {
  color: var(--link-color);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* Layout */
.wrapper {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

/* Header */
.site-header {
  background-color: var(--header-bg);
  padding: 20px 0;
  border-bottom: 1px solid #e9ecef;
}

.site-title {
  font-size: 1.5rem;
  font-weight: bold;
  color: var(--primary-color);
}

.site-nav {
  display: inline-block;
  float: right;
}

.site-nav a {
  margin-left: 20px;
}

/* Footer */
.site-footer {
  background-color: var(--footer-bg);
  padding: 20px 0;
  margin-top: 40px;
  border-top: 1px solid #e9ecef;
  text-align: center;
}

/* Documentation Styles */
.doc-container {
  display: flex;
  flex-wrap: wrap;
}

.doc-sidebar {
  flex: 0 0 250px;
  padding: 20px;
  border-right: 1px solid #e9ecef;
}

.doc-content {
  flex: 1;
  min-width: 0;
  padding: 20px;
}

.doc-content h1 {
  margin-top: 0;
}

/* Navigation */
.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-list li {
  margin-bottom: 10px;
}

.nav-list .active {
  font-weight: bold;
  color: var(--primary-color);
}

/* Responsive */
@media (max-width: 768px) {
  .doc-container {
    flex-direction: column;
  }
  
  .doc-sidebar {
    flex: 0 0 100%;
    border-right: none;
    border-bottom: 1px solid #e9ecef;
  }
}
"#;

    fs::write(&main_css_path, css_content)
        .map_err(|e| format!("Failed to write main.css: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "assets/css/main.css".to_string(),
        change_type: ChangeType::Created,
        description: "Created default CSS file".to_string(),
    });
    
    Ok(())
} 