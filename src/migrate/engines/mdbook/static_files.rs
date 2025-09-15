use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_static(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating static files...");
    }

    // Create static assets directory
    let static_dir = dest_dir.join("assets");
    fs::create_dir_all(&static_dir)
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;

    // Migrate static files from src directory
    migrate_src_static_files(source_dir, dest_dir, verbose, result)?;

    // Migrate theme static files
    migrate_theme_static_files(source_dir, dest_dir, verbose, result)?;

    // Migrate additional static files from book.toml configuration
    migrate_configured_static_files(source_dir, dest_dir, verbose, result)?;

    Ok(())
}

fn migrate_src_static_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let src_dir = source_dir.join("src");
    if !src_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(&src_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Skip markdown files and SUMMARY.md as they're handled separately
        if path.extension().map_or(false, |ext| ext == "md") {
            continue;
        }

        if path.is_file() {
            let relative_path = path.strip_prefix(&src_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            let dest_path = dest_dir.join("assets").join(relative_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            fs::copy(path, &dest_path)
                .map_err(|e| format!("Failed to copy static file: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: format!("assets/{}", relative_path.display()),
                description: format!("Copied static file from {}", path.display()),
            });
        }
    }

    Ok(())
}

fn migrate_theme_static_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let theme_dir = source_dir.join("theme");
    if !theme_dir.exists() {
        return Ok(());
    }

    // Define static asset directories to migrate
    let static_dirs = ["fonts", "images", "icons", "favicon"];

    for dir_name in static_dirs.iter() {
        let source_static_dir = theme_dir.join(dir_name);
        if !source_static_dir.exists() {
            continue;
        }

        let dest_static_dir = dest_dir.join("assets").join(dir_name);
        fs::create_dir_all(&dest_static_dir)
            .map_err(|e| format!("Failed to create {} directory: {}", dir_name, e))?;

        for entry in WalkDir::new(&source_static_dir).min_depth(1) {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() {
                let relative_path = path.strip_prefix(&source_static_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = dest_static_dir.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                fs::copy(path, &dest_path)
                    .map_err(|e| format!("Failed to copy static file: {}", e))?;

                result.changes.push(MigrationChange {
                    change_type: ChangeType::Copied,
                    file_path: format!("assets/{}/{}", dir_name, relative_path.display()),
                    description: format!("Copied theme static file from {}", path.display()),
                });
            }
        }
    }

    Ok(())
}

fn migrate_configured_static_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let book_toml_path = source_dir.join("book.toml");
    if !book_toml_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&book_toml_path)
        .map_err(|e| format!("Failed to read book.toml: {}", e))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse book.toml: {}", e))?;

    if let Some(static_files) = toml_value.get("output").and_then(|o| o.get("static-files")) {
        if let Some(files) = static_files.as_array() {
            for file in files {
                if let Some(path_str) = file.as_str() {
                    let source_path = source_dir.join(path_str);
                    if !source_path.exists() {
                        if verbose {
                            log::warn!("Configured static file not found: {}", path_str);
                        }
                        result.warnings.push(format!("Static file not found: {}", path_str));
                        continue;
                    }

                    let relative_path = source_path.strip_prefix(source_dir)
                        .map_err(|e| format!("Failed to get relative path: {}", e))?;
                    let dest_path = dest_dir.join("assets").join(relative_path);

                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                    }

                    if source_path.is_file() {
                        fs::copy(&source_path, &dest_path)
                            .map_err(|e| format!("Failed to copy static file: {}", e))?;

                        result.changes.push(MigrationChange {
                            change_type: ChangeType::Copied,
                            file_path: format!("assets/{}", relative_path.display()),
                            description: format!("Copied configured static file from {}", source_path.display()),
                        });
                    } else if source_path.is_dir() {
                        for entry in WalkDir::new(&source_path).min_depth(1) {
                            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                            let path = entry.path();

                            if path.is_file() {
                                let file_relative_path = path.strip_prefix(&source_path)
                                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                                let file_dest_path = dest_path.join(file_relative_path);

                                if let Some(parent) = file_dest_path.parent() {
                                    fs::create_dir_all(parent)
                                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                                }

                                fs::copy(path, &file_dest_path)
                                    .map_err(|e| format!("Failed to copy static file: {}", e))?;

                                result.changes.push(MigrationChange {
                                    change_type: ChangeType::Copied,
                                    file_path: format!("assets/{}/{}", relative_path.display(), file_relative_path.display()),
                                    description: format!("Copied configured static file from {}", path.display()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
} 