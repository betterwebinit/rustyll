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
        log::info!("Migrating Fresh static assets...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // In Fresh, static assets are typically in the static directory
    let static_dir = source_dir.join("static");
    if !static_dir.exists() || !static_dir.is_dir() {
        result.warnings.push("No static directory found.".into());
        return Ok(());
    }

    // Copy static assets
    for entry in WalkDir::new(&static_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the static directory
            let rel_path = file_path.strip_prefix(&static_dir)
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
    
    // Create CSS directory
    let css_dir = dest_assets_dir.join("css");
    create_dir_if_not_exists(&css_dir)?;
    
    // Check if a main.css file exists already
    let main_css_path = css_dir.join("main.css");
    if !main_css_path.exists() {
        // Create a basic CSS file for the Jekyll site
        let css_content = r#"/* Styles for migrated Fresh site */
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 1rem;
}

header, footer {
    margin: 2rem 0;
}

a {
    color: #3273dc;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5rem;
    margin-bottom: 0.5rem;
    color: #363636;
}

img {
    max-width: 100%;
}

pre {
    padding: 1rem;
    overflow: auto;
    background-color: #f5f5f5;
    border-radius: 4px;
}

code {
    background-color: #f5f5f5;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: monospace;
}

.btn {
    display: inline-block;
    background-color: #3273dc;
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    text-decoration: none;
    cursor: pointer;
}

.btn:hover {
    background-color: #2366d1;
    text-decoration: none;
}

.island {
    border: 1px solid #eee;
    border-radius: 4px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}
"#;
        
        fs::write(&main_css_path, css_content)
            .map_err(|e| format!("Failed to write CSS file: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "assets/css/main.css".into(),
            description: "Created CSS file for Fresh site".into(),
        });
    }
    
    Ok(())
} 