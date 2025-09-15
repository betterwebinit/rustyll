use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub fn migrate_static(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MkDocs static files...");
    }

    // In MkDocs, static files can be in multiple locations:
    // 1. docs/img/ or docs/images/
    // 2. docs/static/
    // 3. docs/ (root level files)
    // 4. static/ in root

    let potential_static_dirs = vec![
        source_dir.join("docs").join("img"),
        source_dir.join("docs").join("images"),
        source_dir.join("docs").join("static"),
        source_dir.join("static"),
    ];

    // Create destination directories
    let dest_static_dir = dest_dir.join("static");
    create_dir_if_not_exists(&dest_static_dir)?;

    let dest_images_dir = dest_dir.join("assets").join("images");
    create_dir_if_not_exists(&dest_images_dir)?;

    // Migrate from each potential directory
    for static_dir in potential_static_dirs {
        if static_dir.exists() && static_dir.is_dir() {
            if verbose {
                log::info!("Found static directory: {}", static_dir.display());
            }

            // Determine destination based on directory name
            let dir_name = static_dir.file_name().unwrap().to_string_lossy().to_lowercase();
            let is_image_dir = dir_name == "img" || dir_name == "images";

            let destination = if is_image_dir {
                dest_images_dir.as_path()
            } else {
                dest_static_dir.as_path()
            };

            // Copy all files from this directory
            copy_static_directory(&static_dir, destination, result)?;
        }
    }

    // Check for files directly in docs/ that should be copied
    let docs_dir = source_dir.join("docs");
    if docs_dir.exists() && docs_dir.is_dir() {
        for entry in fs::read_dir(&docs_dir).map_err(|e| format!("Failed to read docs directory: {}", e))? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // Skip markdown files, they're handled by content migration
                    if path.extension().map_or(false, |ext| ext == "md" || ext == "markdown") {
                        continue;
                    }

                    // Copy the file to the appropriate location
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let dest_path = dest_static_dir.join(&*file_name);

                    copy_file(&path, &dest_path)?;

                    result.changes.push(MigrationChange {
                        file_path: format!("static/{}", file_name),
                        change_type: ChangeType::Copied,
                        description: format!("Copied static file from {}", path.display()),
                    });
                }
            }
        }
    }

    Ok(())
}

fn copy_static_directory(
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
            
            // Get the relative path from the dest_dir's parent (for reporting purposes)
            let rel_dest_path = if let Ok(path) = dest_path.strip_prefix(dest_dir.parent().unwrap_or(dest_dir)) {
                path.display().to_string()
            } else {
                dest_path.file_name().unwrap().to_string_lossy().to_string()
            };
            
            result.changes.push(MigrationChange {
                file_path: rel_dest_path,
                change_type: ChangeType::Copied,
                description: format!("Copied static file from {}", file_path.display()),
            });
        }
    }
    
    Ok(())
} 