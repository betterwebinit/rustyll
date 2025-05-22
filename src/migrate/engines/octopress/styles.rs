use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_styles(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress styles...");
    }

    // Create destination style directories
    let dest_css_dir = dest_dir.join("assets/css");
    create_dir_if_not_exists(&dest_css_dir)?;
    
    let dest_sass_dir = dest_dir.join("_sass");
    create_dir_if_not_exists(&dest_sass_dir)?;

    // Migrate Sass files from sass/ directory
    let sass_dir = source_dir.join("sass");
    if sass_dir.exists() && sass_dir.is_dir() {
        migrate_sass_directory(&sass_dir, &dest_sass_dir, result)?;
    }

    // Migrate CSS files from stylesheets/ or public/stylesheets/ directory
    let stylesheets_dir = source_dir.join("stylesheets");
    if stylesheets_dir.exists() && stylesheets_dir.is_dir() {
        migrate_css_directory(&stylesheets_dir, &dest_css_dir, result)?;
    } else {
        let public_stylesheets_dir = source_dir.join("public/stylesheets");
        if public_stylesheets_dir.exists() && public_stylesheets_dir.is_dir() {
            migrate_css_directory(&public_stylesheets_dir, &dest_css_dir, result)?;
        }
    }

    // Create main CSS file if it doesn't exist
    create_main_css_file(&dest_css_dir, result)?;

    Ok(())
}

fn migrate_sass_directory(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process Sass/SCSS files
            if let Some(ext) = file_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "scss" || ext_str == "sass" {
                    migrate_sass_file(file_path, source_dir, dest_dir, result)?;
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_css_directory(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process CSS files
            if let Some(ext) = file_path.extension() {
                if ext == "css" {
                    migrate_css_file(file_path, source_dir, dest_dir, result)?;
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_sass_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Determine if it's a partial (starting with underscore)
    let file_name = rel_path.file_name().unwrap().to_string_lossy();
    let is_partial = file_name.starts_with('_');
    
    // Create destination path
    let dest_path = if is_partial {
        // Keep partials as-is
        dest_dir.join(rel_path)
    } else {
        // Convert main Sass files to partials
        let parent = rel_path.parent().unwrap_or_else(|| Path::new(""));
        let new_name = format!("_{}", file_name);
        dest_dir.join(parent).join(new_name)
    };
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Read and process the Sass file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read Sass file {}: {}", file_path.display(), e))?;
    
    // Update imports in the Sass content
    let updated_content = update_sass_imports(&content);
    
    // Write to destination
    fs::write(&dest_path, updated_content)
        .map_err(|e| format!("Failed to write Sass file {}: {}", dest_path.display(), e))?;
    
    let change_path = format!("_sass/{}", dest_path.file_name().unwrap().to_string_lossy());
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: change_path,
        description: format!("Converted Sass file from {}", file_path.display()),
    });
    
    Ok(())
}

fn migrate_css_file(
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
    
    // Read and copy the CSS file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read CSS file {}: {}", file_path.display(), e))?;
    
    // Write to destination
    fs::write(&dest_path, content)
        .map_err(|e| format!("Failed to write CSS file {}: {}", dest_path.display(), e))?;
    
    let change_path = format!("assets/css/{}", dest_path.file_name().unwrap().to_string_lossy());
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: change_path,
        description: format!("Copied CSS file from {}", file_path.display()),
    });
    
    Ok(())
}

fn update_sass_imports(content: &str) -> String {
    let mut updated_content = content.to_string();
    
    // Replace any old-style Sass imports
    updated_content = updated_content.replace("@import url(", "@import \"");
    updated_content = updated_content.replace(");", "\";");
    
    // Replace any relative paths in imports if needed
    
    updated_content
}

fn create_main_css_file(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let main_css_path = dest_dir.join("main.css");
    
    // Only create if it doesn't exist
    if main_css_path.exists() {
        return Ok(());
    }
    
    // Create a minimal main CSS file with Jekyll front matter
    let content = r#"---
---

@import "base";
@import "layout";
@import "syntax";
@import "custom";
"#;
    
    fs::write(&main_css_path, content)
        .map_err(|e| format!("Failed to create main CSS file: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.css".to_string(),
        description: "Created main CSS file".to_string(),
    });
    
    Ok(())
} 