use std::path::Path;
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
        log::info!("Migrating Cobalt static assets...");
    }

    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // In Cobalt, static assets are typically in the public directory
    let source_static_dirs = [
        source_dir.join("assets"),
        source_dir.join("public"),
        source_dir.join("static"),
    ];

    let mut found_assets = false;

    for source_static_dir in &source_static_dirs {
        if source_static_dir.exists() && source_static_dir.is_dir() {
            if verbose {
                log::info!("Found static assets directory: {}", source_static_dir.display());
            }
            
            found_assets = true;
            
            // Copy static assets
            for entry in WalkDir::new(source_static_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Get the relative path from the static directory
                    let rel_path = file_path.strip_prefix(source_static_dir)
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
        }
    }
    
    if !found_assets && verbose {
        log::info!("No static assets directories found.");
    }
    
    // Create basic CSS file if none exists
    let css_dir = dest_assets_dir.join("css");
    create_dir_if_not_exists(&css_dir)?;
    
    let main_css_path = css_dir.join("main.css");
    if !main_css_path.exists() {
        // Create a simple default CSS file
        let default_css = r#"/* Default styles for migrated Cobalt site */
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 1rem;
}

header, footer {
    margin: 2rem 0;
}

a {
    color: #0366d6;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 2rem;
    margin-bottom: 1rem;
}

img {
    max-width: 100%;
}

code {
    background-color: #f6f8fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
}

pre {
    background-color: #f6f8fa;
    padding: 1rem;
    overflow: auto;
    border-radius: 3px;
}

blockquote {
    border-left: 4px solid #ddd;
    padding-left: 1rem;
    margin-left: 0;
    color: #666;
}
"#;
        
        fs::write(&main_css_path, default_css)
            .map_err(|e| format!("Failed to write default CSS file: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "assets/css/main.css".into(),
            description: "Created default CSS file".into(),
        });
    }
    
    Ok(())
} 