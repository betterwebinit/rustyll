use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_static_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Pelican static files...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // Pelican static files can be in multiple locations:
    // 1. content/images (and other media)
    // 2. theme/static/
    // 3. static/ (in project root)
    // 4. {STATIC_PATHS} in pelicanconf.py

    // Check for static directory in source root
    let static_dir = source_dir.join("static");
    if static_dir.exists() && static_dir.is_dir() {
        migrate_static_directory(&static_dir, &dest_assets_dir, result)?;
    }

    // Check for content/images directory
    let content_images_dir = source_dir.join("content/images");
    if content_images_dir.exists() && content_images_dir.is_dir() {
        let dest_images_dir = dest_assets_dir.join("images");
        create_dir_if_not_exists(&dest_images_dir)?;
        migrate_static_directory(&content_images_dir, &dest_images_dir, result)?;
    }

    // Check for theme static directories
    let theme_dirs = find_theme_directories(source_dir);
    for theme_dir in theme_dirs {
        let theme_static_dir = theme_dir.join("static");
        if theme_static_dir.exists() && theme_static_dir.is_dir() {
            migrate_theme_static_directory(&theme_static_dir, &dest_assets_dir, result)?;
        }
    }

    // Check pelicanconf.py for STATIC_PATHS
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        let static_paths = parse_static_paths(&pelicanconf_path);
        for path in static_paths {
            let static_path = source_dir.join(&path);
            if static_path.exists() {
                // Decide on destination directory based on path
                if path.contains("images") || path.contains("img") {
                    let dest_path = dest_assets_dir.join("images");
                    create_dir_if_not_exists(&dest_path)?;
                    migrate_static_directory(&static_path, &dest_path, result)?;
                } else if path.contains("css") || path.contains("styles") {
                    let dest_path = dest_assets_dir.join("css");
                    create_dir_if_not_exists(&dest_path)?;
                    migrate_static_directory(&static_path, &dest_path, result)?;
                } else if path.contains("js") || path.contains("javascript") {
                    let dest_path = dest_assets_dir.join("js");
                    create_dir_if_not_exists(&dest_path)?;
                    migrate_static_directory(&static_path, &dest_path, result)?;
                } else {
                    // Default to copying to assets/[dirname]
                    let dirname = static_path.file_name().unwrap().to_string_lossy();
                    let dest_path = dest_assets_dir.join(dirname.to_string());
                    create_dir_if_not_exists(&dest_path)?;
                    migrate_static_directory(&static_path, &dest_path, result)?;
                }
            }
        }
    }

    Ok(())
}

fn find_theme_directories(source_dir: &Path) -> Vec<PathBuf> {
    let mut theme_dirs = Vec::new();
    
    // Check for theme directory in the source root
    let theme_dir = source_dir.join("theme");
    if theme_dir.exists() && theme_dir.is_dir() {
        theme_dirs.push(theme_dir);
    }
    
    // Check for themes directory in the source root
    let themes_dir = source_dir.join("themes");
    if themes_dir.exists() && themes_dir.is_dir() {
        // Add each subdirectory in themes/
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        theme_dirs.push(path);
                    }
                }
            }
        }
    }
    
    // Check if pelicanconf.py specifies a theme
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(config_content) = fs::read_to_string(&pelicanconf_path) {
            // Look for THEME = "theme_name" in the config
            let theme_regex = regex::Regex::new(r#"THEME\s*=\s*["']([^"']+)["']"#).unwrap();
            if let Some(captures) = theme_regex.captures(&config_content) {
                let theme_name = &captures[1];
                
                // Check if it's an absolute path
                let theme_path = PathBuf::from(theme_name);
                if theme_path.is_absolute() && theme_path.exists() {
                    theme_dirs.push(theme_path);
                } else {
                    // Check if it's in the themes directory
                    let in_themes_dir = themes_dir.join(theme_name);
                    if in_themes_dir.exists() {
                        theme_dirs.push(in_themes_dir);
                    }
                }
            }
        }
    }
    
    theme_dirs
}

fn migrate_static_directory(
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
            fs::copy(file_path, &dest_path)
                .map_err(|e| format!("Failed to copy static file: {}", e))?;
            
            // Get the relative path from the dest_dir's parent (for reporting purposes)
            let rel_dest_path = if let Ok(path) = dest_path.strip_prefix(dest_dir.parent().unwrap_or(dest_dir)) {
                path.display().to_string()
            } else {
                dest_path.file_name().unwrap().to_string_lossy().to_string()
            };
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Copied,
                file_path: rel_dest_path,
                description: format!("Copied static file from {}", file_path.display()),
            });
        }
    }
    
    Ok(())
}

fn migrate_theme_static_directory(
    theme_static_dir: &Path,
    dest_assets_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Theme static directory typically has subdirectories like css, js, images
    if let Ok(entries) = fs::read_dir(theme_static_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy();
                    
                    let dest_dir = match dir_name.as_ref() {
                        "css" | "styles" => dest_assets_dir.join("css"),
                        "js" | "javascript" => dest_assets_dir.join("js"),
                        "images" | "img" => dest_assets_dir.join("images"),
                        "fonts" => dest_assets_dir.join("fonts"),
                        _ => dest_assets_dir.join(dir_name.to_string()),
                    };
                    
                    create_dir_if_not_exists(&dest_dir)?;
                    migrate_static_directory(&path, &dest_dir, result)?;
                } else if path.is_file() {
                    // Files directly in static/ go to assets/
                    let filename = path.file_name().unwrap().to_string_lossy();
                    let dest_path = dest_assets_dir.join(filename.to_string());
                    
                    fs::copy(&path, &dest_path)
                        .map_err(|e| format!("Failed to copy theme static file: {}", e))?;
                    
                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Copied,
                        file_path: format!("assets/{}", filename),
                        description: format!("Copied theme static file from {}", path.display()),
                    });
                }
            }
        }
    }
    
    Ok(())
}

fn parse_static_paths(pelicanconf_path: &Path) -> Vec<String> {
    let mut static_paths = Vec::new();
    
    if let Ok(content) = fs::read_to_string(pelicanconf_path) {
        // Simple regex to extract STATIC_PATHS from pelicanconf.py
        // This is a simplification and might not handle all Python syntax correctly
        let static_paths_regex = regex::Regex::new(r"STATIC_PATHS\s*=\s*\[(.*?)\]").unwrap();
        
        if let Some(captures) = static_paths_regex.captures(&content) {
            let paths_str = &captures[1];
            
            // Extract individual paths
            let path_regex = regex::Regex::new(r#"["']([^"']+)["']"#).unwrap();
            for path_match in path_regex.captures_iter(paths_str) {
                static_paths.push(path_match[1].to_string());
            }
        }
    }
    
    // If no paths were specified, use default paths
    if static_paths.is_empty() {
        static_paths.push("images".to_string());
    }
    
    static_paths
} 