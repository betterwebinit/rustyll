use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_static(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Jigsaw static files...");
    }
    
    // In Jigsaw, static files are typically in the source directory root
    // or in specific directories that aren't prefixed with underscore
    let source_content_dir = source_dir.join("source");
    
    if !source_content_dir.exists() {
        result.warnings.push("Could not find Jigsaw source directory".into());
        return Ok(());
    }
    
    // Create a list of directories to skip (these are handled by other migrations)
    let skip_dirs = [
        "_assets",
        "_layouts",
        "_posts",
        "_docs",
        "_components",
        "_helpers",
        "_templates",
    ];
    
    // Find all files in the source directory that aren't in special directories
    let static_files = WalkDir::new(&source_content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            if !e.path().is_file() {
                return false;
            }
            
            // Check if the file is in a directory we should skip
            for path in e.path().ancestors() {
                if path == source_content_dir {
                    break;
                }
                
                if let Some(dir_name) = path.file_name() {
                    let dir_name_str = dir_name.to_string_lossy();
                    if dir_name_str.starts_with('_') || skip_dirs.contains(&dir_name_str.as_ref()) {
                        return false;
                    }
                }
            }
            
            // Skip special files like blade templates, etc.
            let extension = e.path().extension().unwrap_or_default().to_string_lossy().to_lowercase();
            if extension == "blade" || 
               extension == "php" ||
               e.path().to_string_lossy().contains(".blade.") {
                return false;
            }
            
            true
        });
    
    for entry in static_files {
        let relative_path = entry.path().strip_prefix(&source_content_dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        let dest_file = dest_dir.join(relative_path);
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Copy the file
        fs::copy(entry.path(), &dest_file)
            .map_err(|e| format!("Failed to copy static file {}: {}", entry.path().display(), e))?;
        
        // Add to migration results
        result.changes.push(MigrationChange {
            file_path: format!("{}", relative_path.display()),
            change_type: ChangeType::Copied,
            description: "Copied static file".into(),
        });
    }
    
    // Special handling for source/favicon.ico and similar files
    let common_static_files = [
        "favicon.ico",
        "robots.txt",
        "sitemap.xml",
        ".htaccess",
    ];
    
    for file_name in common_static_files.iter() {
        let source_file = source_content_dir.join(file_name);
        if source_file.exists() && source_file.is_file() {
            let dest_file = dest_dir.join(file_name);
            
            fs::copy(&source_file, &dest_file)
                .map_err(|e| format!("Failed to copy file {}: {}", source_file.display(), e))?;
            
            result.changes.push(MigrationChange {
                file_path: file_name.to_string(),
                change_type: ChangeType::Copied,
                description: "Copied static file from source root".into(),
            });
        }
    }
    
    // Also check for a 'public' directory, which may contain built/compiled assets
    let public_dir = source_dir.join("public");
    if public_dir.exists() && public_dir.is_dir() {
        // Add a note about the public directory
        result.warnings.push(
            "Found 'public' directory. This may contain compiled assets that shouldn't be copied directly.".into()
        );
    }
    
    Ok(())
} 