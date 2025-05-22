use std::path::{Path, PathBuf};
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
        log::info!("Migrating Bridgetown static assets...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // Look for static assets in Bridgetown's static directory
    let source_static_dir = source_dir.join("src/static");
    if !source_static_dir.exists() || !source_static_dir.is_dir() {
        if verbose {
            log::info!("No static directory found in Bridgetown project");
        }
        result.warnings.push("No static assets directory found.".into());
        return Ok(());
    }

    // Copy static assets
    for entry in WalkDir::new(&source_static_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the static directory
            let rel_path = file_path.strip_prefix(&source_static_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
            // Create destination path
            let dest_path = dest_assets_dir.join(rel_path);
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Copy the static asset
            copy_file(file_path, &dest_path)?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: format!("assets/{}", rel_path.display()),
                description: format!("Copied static asset from {}", file_path.display()),
            });
        }
    }
    
    // Also check for frontend directory (for JS, CSS, etc.)
    let source_frontend_dir = source_dir.join("frontend");
    if source_frontend_dir.exists() && source_frontend_dir.is_dir() {
        // Create destination assets/js and assets/css directories
        let dest_js_dir = dest_assets_dir.join("js");
        let dest_css_dir = dest_assets_dir.join("css");
        create_dir_if_not_exists(&dest_js_dir)?;
        create_dir_if_not_exists(&dest_css_dir)?;
        
        // Copy JavaScript and CSS files
        for entry in WalkDir::new(&source_frontend_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                
                // Skip non-relevant files
                if !["js", "jsx", "ts", "tsx", "css", "scss", "sass"].contains(&extension) {
                    continue;
                }
                
                // Get destination based on file type
                let dest_path = if ["js", "jsx", "ts", "tsx"].contains(&extension) {
                    let file_name = file_path.file_stem().unwrap().to_string_lossy();
                    dest_js_dir.join(format!("{}.js", file_name))
                } else {
                    let file_name = file_path.file_stem().unwrap().to_string_lossy();
                    dest_css_dir.join(format!("{}.css", file_name))
                };
                
                // Copy or convert the file based on type
                if extension == "js" || extension == "css" {
                    copy_file(file_path, &dest_path)?;
                } else {
                    // For non-JS/CSS files, we'd need to convert them
                    // For simplicity, just copy with a warning
                    copy_file(file_path, &dest_path)?;
                    result.warnings.push(format!(
                        "Copied {} file without conversion: {}. Manual review may be needed.",
                        extension.to_uppercase(),
                        file_path.display()
                    ));
                }
                
                result.changes.push(MigrationChange {
                    change_type: ChangeType::Copied,
                    file_path: format!("assets/{}/{}.{}", 
                        if ["js", "jsx", "ts", "tsx"].contains(&extension) { "js" } else { "css" },
                        file_path.file_stem().unwrap().to_string_lossy(),
                        if ["js", "jsx", "ts", "tsx"].contains(&extension) { "js" } else { "css" }
                    ),
                    description: format!("Copied frontend asset from {}", file_path.display()),
                });
            }
        }
    }
    
    Ok(())
} 