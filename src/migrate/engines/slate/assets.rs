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
        log::info!("Migrating Slate assets...");
    }

    // Create destination directories
    let dest_images_dir = dest_dir.join("assets/images");
    create_dir_if_not_exists(&dest_images_dir)?;
    
    let dest_fonts_dir = dest_dir.join("assets/fonts");
    create_dir_if_not_exists(&dest_fonts_dir)?;

    // Migrate images
    migrate_images(source_dir, dest_dir, &dest_images_dir, result)?;
    
    // Migrate fonts
    migrate_fonts(source_dir, dest_dir, &dest_fonts_dir, result)?;
    
    // Migrate other assets (like favicon, etc.)
    migrate_other_assets(source_dir, dest_dir, result)?;

    Ok(())
}

fn migrate_images(
    source_dir: &Path,
    dest_dir: &Path,
    dest_images_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Check common image directories in Slate projects
    let image_dirs = [
        source_dir.join("source/images"),
        source_dir.join("source/img"),
        source_dir.join("images"),
        source_dir.join("img"),
    ];
    
    for img_dir in image_dirs.iter() {
        if img_dir.exists() && img_dir.is_dir() {
            // Process all files in the directory
            for entry in WalkDir::new(img_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Check if it's an image file
                    if is_image_file(file_path) {
                        // Copy to destination
                        let rel_path = file_path.strip_prefix(img_dir)
                            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                        
                        let dest_path = dest_images_dir.join(rel_path);
                        
                        // Create parent directory if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy file
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            change_type: ChangeType::Copied,
                            file_path: format!("assets/images/{}", rel_path.display()),
                            description: format!("Copied image from {}", file_path.display()),
                        });
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_fonts(
    source_dir: &Path,
    dest_dir: &Path,
    dest_fonts_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Check common font directories in Slate projects
    let font_dirs = [
        source_dir.join("source/fonts"),
        source_dir.join("fonts"),
    ];
    
    for font_dir in font_dirs.iter() {
        if font_dir.exists() && font_dir.is_dir() {
            // Process all files in the directory
            for entry in WalkDir::new(font_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Check if it's a font file
                    if is_font_file(file_path) {
                        // Copy to destination
                        let rel_path = file_path.strip_prefix(font_dir)
                            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                        
                        let dest_path = dest_fonts_dir.join(rel_path);
                        
                        // Create parent directory if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy file
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            change_type: ChangeType::Copied,
                            file_path: format!("assets/fonts/{}", rel_path.display()),
                            description: format!("Copied font from {}", file_path.display()),
                        });
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
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Check for favicon and other common assets in the root or source directory
    let common_assets = [
        ("favicon.ico", "favicon.ico"),
        ("favicon.png", "favicon.png"),
        ("robots.txt", "robots.txt"),
        ("site.webmanifest", "site.webmanifest"),
        ("browserconfig.xml", "browserconfig.xml"),
    ];
    
    for (asset_name, dest_name) in common_assets.iter() {
        // Check in source root
        let source_path = source_dir.join(asset_name);
        if source_path.exists() && source_path.is_file() {
            let dest_path = dest_dir.join(dest_name);
            copy_file(&source_path, &dest_path)?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: dest_name.to_string(),
                description: format!("Copied {} from source root", asset_name),
            });
            
            continue;
        }
        
        // Check in source/images or source directory
        let source_img_path = source_dir.join("source/images").join(asset_name);
        if source_img_path.exists() && source_img_path.is_file() {
            let dest_path = dest_dir.join(dest_name);
            copy_file(&source_img_path, &dest_path)?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: dest_name.to_string(),
                description: format!("Copied {} from source/images", asset_name),
            });
            
            continue;
        }
        
        let source_src_path = source_dir.join("source").join(asset_name);
        if source_src_path.exists() && source_src_path.is_file() {
            let dest_path = dest_dir.join(dest_name);
            copy_file(&source_src_path, &dest_path)?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: dest_name.to_string(),
                description: format!("Copied {} from source directory", asset_name),
            });
        }
    }
    
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

fn is_font_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "ttf" | "otf" | "woff" | "woff2" | "eot")
    } else {
        false
    }
} 