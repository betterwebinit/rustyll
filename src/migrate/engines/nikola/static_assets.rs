use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_static_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Nikola static assets...");
    }

    // Create destination asset directories
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;
    
    let dest_images_dir = dest_assets_dir.join("images");
    create_dir_if_not_exists(&dest_images_dir)?;
    
    let dest_css_dir = dest_assets_dir.join("css");
    create_dir_if_not_exists(&dest_css_dir)?;
    
    let dest_js_dir = dest_assets_dir.join("js");
    create_dir_if_not_exists(&dest_js_dir)?;

    // In Nikola, static assets are typically in files/ or static/ directory
    let static_dirs = [
        source_dir.join("files"),
        source_dir.join("static"),
        source_dir.join("assets"),
    ];
    
    for static_dir in static_dirs.iter() {
        if static_dir.exists() && static_dir.is_dir() {
            if verbose {
                log::info!("Found static assets directory: {}", static_dir.display());
            }
            
            // Process images
            migrate_images(static_dir, &dest_images_dir, result)?;
            
            // Process CSS
            migrate_css(static_dir, &dest_css_dir, result)?;
            
            // Process JavaScript
            migrate_js(static_dir, &dest_js_dir, result)?;
            
            // Process other asset files
            migrate_other_assets(static_dir, &dest_assets_dir, result)?;
        }
    }
    
    // Create a default CSS file if none exists
    if is_dir_empty(&dest_css_dir)? {
        create_default_css(&dest_css_dir, result)?;
    }

    Ok(())
}

fn migrate_images(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Walk through the static directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process image files
            if is_image_file(file_path) {
                copy_asset_file(file_path, source_dir, dest_dir, result, "image")?;
            }
        }
    }
    
    // Also check for specific image directories
    let image_dirs = [
        source_dir.join("images"),
        source_dir.join("img"),
        source_dir.join("image"),
    ];
    
    for image_dir in image_dirs.iter() {
        if image_dir.exists() && image_dir.is_dir() {
            for entry in WalkDir::new(image_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    copy_asset_file(file_path, image_dir, dest_dir, result, "image")?;
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_css(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Walk through the static directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process CSS files
            if is_css_file(file_path) {
                copy_asset_file(file_path, source_dir, dest_dir, result, "CSS")?;
            }
        }
    }
    
    // Also check for specific CSS directories
    let css_dirs = [
        source_dir.join("css"),
        source_dir.join("styles"),
        source_dir.join("style"),
    ];
    
    for css_dir in css_dirs.iter() {
        if css_dir.exists() && css_dir.is_dir() {
            for entry in WalkDir::new(css_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    copy_asset_file(file_path, css_dir, dest_dir, result, "CSS")?;
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_js(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Walk through the static directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process JavaScript files
            if is_js_file(file_path) {
                copy_asset_file(file_path, source_dir, dest_dir, result, "JavaScript")?;
            }
        }
    }
    
    // Also check for specific JS directories
    let js_dirs = [
        source_dir.join("js"),
        source_dir.join("javascript"),
        source_dir.join("scripts"),
    ];
    
    for js_dir in js_dirs.iter() {
        if js_dir.exists() && js_dir.is_dir() {
            for entry in WalkDir::new(js_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    copy_asset_file(file_path, js_dir, dest_dir, result, "JavaScript")?;
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_other_assets(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let other_files_dir = dest_dir.join("files");
    create_dir_if_not_exists(&other_files_dir)?;
    
    // Walk through the static directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip files that are already handled by other functions
            if !is_image_file(file_path) && !is_css_file(file_path) && !is_js_file(file_path) {
                copy_asset_file(file_path, source_dir, &other_files_dir, result, "asset")?;
            }
        }
    }
    
    Ok(())
}

fn copy_asset_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
    asset_type: &str,
) -> Result<(), String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path preserving directory structure
    let dest_path = dest_dir.join(rel_path);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Copy the file
    copy_file(file_path, &dest_path)?;
    
    // Determine the destination path relative to the Jekyll root
    let assets_subdir = match asset_type {
        "image" => "images",
        "CSS" => "css",
        "JavaScript" => "js",
        _ => "files",
    };
    
    let jekyll_rel_path = format!("assets/{}/{}", assets_subdir, rel_path.display());
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: jekyll_rel_path,
        description: format!("Copied {} from {}", asset_type, file_path.display()),
    });
    
    Ok(())
}

fn create_default_css(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a basic main.css file
    let css_content = r#"/* Main CSS file for the Jekyll site converted from Nikola */

/* Base styles */
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  line-height: 1.6;
  color: #333;
  max-width: 800px;
  margin: 0 auto;
  padding: 0 20px;
}

header, footer {
  margin: 20px 0;
  padding: 20px 0;
  border-bottom: 1px solid #eee;
}

footer {
  border-top: 1px solid #eee;
  border-bottom: none;
  color: #777;
  font-size: 0.9em;
}

h1, h2, h3, h4, h5, h6 {
  margin-top: 1.5em;
  margin-bottom: 0.5em;
}

a {
  color: #0366d6;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* Navigation */
nav ul {
  list-style: none;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
}

nav li {
  margin-right: 15px;
}

/* Posts and Pages */
.post, .page {
  margin-bottom: 40px;
}

.post-meta {
  color: #777;
  font-size: 0.9em;
  margin-bottom: 20px;
}

.post-tags {
  margin-top: 20px;
}

.post-tags ul {
  list-style: none;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
}

.post-tags li {
  margin-right: 10px;
  background: #f1f8ff;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 0.9em;
}
"#;

    let css_path = dest_dir.join("main.css");
    fs::write(&css_path, css_content)
        .map_err(|e| format!("Failed to create main.css: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.css".to_string(),
        description: "Created default CSS file".to_string(),
    });
    
    Ok(())
}

fn is_dir_empty(dir: &Path) -> Result<bool, String> {
    if !dir.exists() {
        return Ok(true);
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    Ok(entries.count() == 0)
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "ico" | "bmp")
    } else {
        false
    }
}

fn is_css_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "css" | "scss" | "sass")
    } else {
        false
    }
}

fn is_js_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "js" | "mjs" | "jsx")
    } else {
        false
    }
} 