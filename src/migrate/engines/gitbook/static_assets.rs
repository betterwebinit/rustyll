use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::GitbookMigrator {
    pub(super) fn migrate_static_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In GitBook, static assets can be in various places
        // Common patterns: assets/ or files in the root directory
        
        // Create destination assets directory
        let dest_assets_dir = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets_dir)?;
        
        // Create subdirectories for different asset types
        let dest_images_dir = dest_assets_dir.join("images");
        create_dir_if_not_exists(&dest_images_dir)?;
        
        // Look for assets in common directories
        let source_assets_dirs = [
            source_dir.join("assets"),
            source_dir.join("static"),
            source_dir.join("images"),
            source_dir.join("img"),
            source_dir.join("media"),
            source_dir.join("files"),
        ];
        
        let mut found_assets = false;
        
        // Migrate assets from standard directories
        for assets_dir in &source_assets_dirs {
            if assets_dir.exists() && assets_dir.is_dir() {
                if verbose {
                    log::info!("Found assets directory: {}", assets_dir.display());
                }
                
                self.migrate_assets_directory(assets_dir, source_dir, &dest_assets_dir, result)?;
                found_assets = true;
            }
        }
        
        // Look for images in content directories
        // GitBook often has images mixed with content
        self.find_images_in_content(source_dir, &dest_images_dir, result)?;
        
        // Also look for any root-level assets that might be referenced
        self.migrate_root_assets(source_dir, &dest_assets_dir, result, verbose)?;
        
        // Create a favicon if none exists
        let favicon_path = dest_assets_dir.join("favicon.ico");
        if !favicon_path.exists() {
            self.create_favicon(&dest_assets_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_assets_directory(&self, assets_dir: &Path, source_dir: &Path, dest_assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(assets_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip files that we handle elsewhere (CSS, JS, etc.)
                let extension = file_path.extension().map(|e| e.to_string_lossy().to_lowercase());
                if let Some(ext) = &extension {
                    if ext == "css" || ext == "js" || ext == "scss" || ext == "less" {
                        continue;
                    }
                }
                
                // Get the relative path from the assets directory
                let rel_path = file_path.strip_prefix(assets_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                    
                // Create destination path preserving the directory structure
                let dest_path = dest_assets_dir.join(rel_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Copy the file
                copy_file(file_path, &dest_path)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("assets/{}", rel_path.to_string_lossy()),
                    change_type: ChangeType::Copied,
                    description: "Asset file copied".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn find_images_in_content(&self, source_dir: &Path, dest_images_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Look for image files in content directories that might be referenced
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let extension = file_path.extension().map(|e| e.to_string_lossy().to_lowercase());
                
                if let Some(ext) = extension {
                    // Check for image file extensions
                    if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif" || ext == "svg" || ext == "webp" {
                        // Skip files in already processed asset directories
                        let rel_path = file_path.strip_prefix(source_dir)
                            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                            
                        let path_str = rel_path.to_string_lossy();
                        if path_str.starts_with("assets/") || 
                           path_str.starts_with("static/") || 
                           path_str.starts_with("images/") || 
                           path_str.starts_with("img/") || 
                           path_str.starts_with("media/") || 
                           path_str.starts_with("files/") {
                            continue;
                        }
                        
                        // Copy image to images directory
                        let dest_file = dest_images_dir.join(file_path.file_name().unwrap());
                        copy_file(file_path, &dest_file)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("assets/images/{}", file_path.file_name().unwrap().to_string_lossy()),
                            change_type: ChangeType::Copied,
                            description: "Image file copied from content directory".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn migrate_root_assets(&self, source_dir: &Path, dest_assets_dir: &Path, result: &mut MigrationResult, verbose: bool) -> Result<(), String> {
        // Look for common asset files in the root directory
        let common_assets = [
            "favicon.ico", "favicon.png", "logo.png", "logo.svg", 
            "CNAME", "robots.txt", ".nojekyll"
        ];
        
        for asset in &common_assets {
            let asset_path = source_dir.join(asset);
            if asset_path.exists() && asset_path.is_file() {
                if verbose {
                    log::info!("Found root asset: {}", asset_path.display());
                }
                
                // Determine destination - some files should go to root, others to assets
                if *asset == "CNAME" || *asset == "robots.txt" || *asset == ".nojekyll" {
                    // These should go in the root directory
                    let dest_path = dest_assets_dir.parent().unwrap().join(asset);
                    copy_file(&asset_path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: asset.to_string(),
                        change_type: ChangeType::Copied,
                        description: "Root file copied".to_string(),
                    });
                } else {
                    // Other assets go to the assets directory
                    let dest_path = dest_assets_dir.join(asset);
                    copy_file(&asset_path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("assets/{}", asset),
                        change_type: ChangeType::Copied,
                        description: "Root asset copied".to_string(),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn create_favicon(&self, dest_assets_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a very simple favicon.ico (1x1 transparent pixel)
        // This is a minimal valid .ico file
        let favicon_data: [u8; 70] = [
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 
            0x0A, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x01, 0x00, 
            0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4e, 0x55
        ];
        
        let favicon_path = dest_assets_dir.join("favicon.ico");
        fs::write(&favicon_path, &favicon_data)
            .map_err(|e| format!("Failed to create favicon: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/favicon.ico".to_string(),
            change_type: ChangeType::Created,
            description: "Default favicon created".to_string(),
        });
        
        Ok(())
    }
} 