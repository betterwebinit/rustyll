use std::path::Path;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress assets...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // Migrate images
    migrate_images(source_dir, dest_dir, verbose, result)?;
    
    // Migrate JavaScript
    migrate_javascript(source_dir, dest_dir, verbose, result)?;
    
    // Migrate other assets
    migrate_other_assets(source_dir, dest_dir, verbose, result)?;

    Ok(())
}

fn migrate_images(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let dest_images_dir = dest_dir.join("assets/images");
    create_dir_if_not_exists(&dest_images_dir)?;
    
    // Check potential image directories
    let image_dirs = [
        source_dir.join("images"),
        source_dir.join("source/images"),
        source_dir.join("public/images"),
        source_dir.join("source/img"),
        source_dir.join("img"),
    ];
    
    for source_images_dir in image_dirs.iter() {
        if source_images_dir.exists() && source_images_dir.is_dir() {
            if verbose {
                log::info!("Found images directory: {}", source_images_dir.display());
            }
            
            // Process all image files
            for entry in WalkDir::new(source_images_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Check if it's an image file
                    if is_image_file(file_path) {
                        // Copy the image file
                        copy_image_file(file_path, source_images_dir, &dest_images_dir, result)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_javascript(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let dest_js_dir = dest_dir.join("assets/js");
    create_dir_if_not_exists(&dest_js_dir)?;
    
    // Check potential JavaScript directories
    let js_dirs = [
        source_dir.join("javascripts"),
        source_dir.join("source/javascripts"),
        source_dir.join("public/javascripts"),
        source_dir.join("source/js"),
        source_dir.join("js"),
    ];
    
    for source_js_dir in js_dirs.iter() {
        if source_js_dir.exists() && source_js_dir.is_dir() {
            if verbose {
                log::info!("Found JavaScript directory: {}", source_js_dir.display());
            }
            
            // Process all JavaScript files
            for entry in WalkDir::new(source_js_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Check if it's a JavaScript file
                    if is_javascript_file(file_path) {
                        // Copy the JavaScript file
                        copy_js_file(file_path, source_js_dir, &dest_js_dir, result)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_other_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // This function handles any other assets like fonts, PDFs, etc.
    let dest_other_dir = dest_dir.join("assets/files");
    create_dir_if_not_exists(&dest_other_dir)?;
    
    // Check potential directories for other assets
    let other_dirs = [
        source_dir.join("files"),
        source_dir.join("source/files"),
        source_dir.join("public/files"),
        source_dir.join("assets"),
        source_dir.join("source/assets"),
    ];
    
    for source_other_dir in other_dirs.iter() {
        if source_other_dir.exists() && source_other_dir.is_dir() {
            if verbose {
                log::info!("Found other assets directory: {}", source_other_dir.display());
            }
            
            // Process all files
            for entry in WalkDir::new(source_other_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Skip image and JavaScript files as they're handled separately
                    if !is_image_file(file_path) && !is_javascript_file(file_path) {
                        // Copy the file
                        copy_other_file(file_path, source_other_dir, &dest_other_dir, result)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn copy_image_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
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
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: format!("assets/images/{}", rel_path.display()),
        description: format!("Copied image from {}", file_path.display()),
    });
    
    Ok(())
}

fn copy_js_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
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
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: format!("assets/js/{}", rel_path.display()),
        description: format!("Copied JavaScript file from {}", file_path.display()),
    });
    
    Ok(())
}

fn copy_other_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
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
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: format!("assets/files/{}", rel_path.display()),
        description: format!("Copied file from {}", file_path.display()),
    });
    
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" | "ico" | "bmp")
    } else {
        false
    }
}

fn is_javascript_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "js" | "jsx" | "mjs")
    } else {
        false
    }
} 