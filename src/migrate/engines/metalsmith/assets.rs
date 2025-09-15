use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Metalsmith assets...");
    }
    
    // In Metalsmith, assets can be in several locations
    let asset_dirs = [
        source_dir.join("assets"),
        source_dir.join("src").join("assets"),
        source_dir.join("public"),
        source_dir.join("static"),
    ];
    
    let mut found_assets = false;
    
    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;
    
    // Process CSS/SCSS/SASS files separately to work with Jekyll
    let dest_sass_dir = dest_dir.join("_sass");
    create_dir_if_not_exists(&dest_sass_dir)?;
    
    let dest_css_dir = dest_assets_dir.join("css");
    create_dir_if_not_exists(&dest_css_dir)?;
    
    // Main asset directory processing
    for asset_dir in asset_dirs.iter() {
        if !asset_dir.exists() {
            continue;
        }
        
        found_assets = true;
        
        // Process all asset files
        process_asset_directory(asset_dir, &dest_assets_dir, &dest_sass_dir, &dest_css_dir, result)?;
    }
    
    // Also check for common asset directories like images, js, fonts
    let common_asset_dirs = [
        ("images", source_dir.join("images")),
        ("img", source_dir.join("img")),
        ("js", source_dir.join("js")),
        ("javascript", source_dir.join("javascript")),
        ("scripts", source_dir.join("scripts")),
        ("fonts", source_dir.join("fonts")),
        ("css", source_dir.join("css")),
        ("sass", source_dir.join("sass")),
        ("scss", source_dir.join("scss")),
        ("styles", source_dir.join("styles")),
    ];
    
    for (asset_type, asset_dir) in common_asset_dirs.iter() {
        if !asset_dir.exists() {
            continue;
        }
        
        found_assets = true;
        
        // Create and store the directory path with a longer lifetime
        let dest_specific_dir = match *asset_type {
            "sass" | "scss" => dest_sass_dir.clone(),
            "css" | "styles" => dest_css_dir.clone(),
            _ => {
                let dest_dir = dest_assets_dir.join(asset_type);
                create_dir_if_not_exists(&dest_dir)?;
                dest_dir
            }
        };
        
        // For each asset type, copy all files
        copy_asset_files(asset_dir, &dest_specific_dir, asset_type, result)?;
    }
    
    if !found_assets {
        result.warnings.push("Could not find Metalsmith assets directory".into());
        
        // Create some defaults to help the user
        create_default_assets(&dest_assets_dir, &dest_sass_dir, &dest_css_dir, result)?;
    }
    
    // Set up scss main file for Jekyll
    setup_jekyll_scss(&dest_sass_dir, &dest_css_dir, dest_dir, result)?;
    
    Ok(())
}

fn process_asset_directory(
    asset_dir: &Path,
    dest_assets_dir: &Path,
    dest_sass_dir: &Path,
    dest_css_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let asset_files = WalkDir::new(asset_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file());
    
    for entry in asset_files {
        let relative_path = entry.path().strip_prefix(asset_dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        // Determine the destination based on file type
        let extension = entry.path().extension().unwrap_or_default().to_string_lossy().to_lowercase();
        
        let dest_file = if extension == "scss" || extension == "sass" {
            // SASS/SCSS files go to _sass directory
            let sass_file = if relative_path.starts_with("_") {
                // Keep underscore prefix in sass partials
                dest_sass_dir.join(relative_path)
            } else {
                // Ensure base sass files are properly formatted for Jekyll
                let file_name = relative_path.file_name().unwrap_or_default();
                let file_stem = relative_path.file_stem().unwrap_or_default().to_string_lossy();
                
                if file_stem.starts_with('_') {
                    dest_sass_dir.join(relative_path)
                } else {
                    // Make sure partial files start with underscore
                    let mut comp = relative_path.components();
                    // Skip the first component if it's just a directory name
                    if comp.clone().count() > 1 {
                        let _ = comp.next();
                    }
                    let partial_path = Path::new("_").join(file_name);
                    dest_sass_dir.join(partial_path)
                }
            };
            
            sass_file
        } else if extension == "css" {
            // CSS files go to assets/css directory
            dest_css_dir.join(relative_path)
        } else if extension == "js" || extension == "jsx" || extension == "ts" || extension == "tsx" {
            // JS files go to assets/js directory
            let js_dir = dest_assets_dir.join("js");
            create_dir_if_not_exists(&js_dir)?;
            js_dir.join(relative_path)
        } else if extension == "jpg" || extension == "jpeg" || extension == "png" || 
                  extension == "gif" || extension == "svg" || extension == "webp" {
            // Image files go to assets/images directory
            let img_dir = dest_assets_dir.join("images");
            create_dir_if_not_exists(&img_dir)?;
            img_dir.join(relative_path)
        } else if extension == "woff" || extension == "woff2" || extension == "ttf" || 
                  extension == "eot" || extension == "otf" {
            // Font files go to assets/fonts directory
            let font_dir = dest_assets_dir.join("fonts");
            create_dir_if_not_exists(&font_dir)?;
            font_dir.join(relative_path)
        } else {
            // Other files go directly to assets directory
            dest_assets_dir.join(relative_path)
        };
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Copy the file
        fs::copy(entry.path(), &dest_file)
            .map_err(|e| format!("Failed to copy asset file {}: {}", entry.path().display(), e))?;
        
        // Add to migration results
        let rel_path = dest_file.strip_prefix(dest_assets_dir.parent().unwrap_or(Path::new("")))
            .unwrap_or(relative_path);
        
        result.changes.push(MigrationChange {
            file_path: format!("{}", rel_path.display()),
            change_type: ChangeType::Copied,
            description: format!("Copied asset file ({})", extension),
        });
    }
    
    Ok(())
}

fn copy_asset_files(
    source_dir: &Path,
    dest_dir: &Path,
    asset_type: &str,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let files = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file());
    
    for entry in files {
        let relative_path = entry.path().strip_prefix(source_dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        let dest_file = dest_dir.join(relative_path);
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Copy the file
        fs::copy(entry.path(), &dest_file)
            .map_err(|e| format!("Failed to copy {} file {}: {}", asset_type, entry.path().display(), e))?;
        
        // Add to migration results
        let rel_path = format!("assets/{}/{}", asset_type, relative_path.display());
        
        result.changes.push(MigrationChange {
            file_path: rel_path,
            change_type: ChangeType::Copied,
            description: format!("Copied {} file", asset_type),
        });
    }
    
    Ok(())
}

fn create_default_assets(
    dest_assets_dir: &Path,
    dest_sass_dir: &Path,
    dest_css_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default main.scss in css directory
    let main_css = dest_css_dir.join("main.scss");
    let main_css_content = r#"---
---

@import "main";
"#;
    
    fs::write(&main_css, main_css_content)
        .map_err(|e| format!("Failed to write main.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "assets/css/main.scss".into(),
        change_type: ChangeType::Created,
        description: "Created default main.scss file".into(),
    });
    
    // Create default _main.scss in _sass directory
    let main_sass = dest_sass_dir.join("_main.scss");
    let main_sass_content = r#"// Main SASS file for the migrated Metalsmith site

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  line-height: 1.6;
  color: #333;
  max-width: 800px;
  margin: 0 auto;
  padding: 1rem;
}

h1, h2, h3, h4, h5, h6 {
  margin-top: 1.5rem;
  margin-bottom: 1rem;
}

a {
  color: #0366d6;
  text-decoration: none;
  
  &:hover {
    text-decoration: underline;
  }
}

// You can add more styles here
"#;
    
    fs::write(&main_sass, main_sass_content)
        .map_err(|e| format!("Failed to write _main.scss: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_sass/_main.scss".into(),
        change_type: ChangeType::Created,
        description: "Created default _main.scss file".into(),
    });
    
    // Create assets README to guide the user
    let assets_readme = dest_assets_dir.join("README.md");
    let assets_readme_content = r#"# Asset Migration from Metalsmith

## Structure

Jekyll processes assets differently than Metalsmith:

- CSS: 
  - SCSS files should be in the `_sass` directory
  - The main SCSS file importing all partials should be in `assets/css/main.scss` with front matter
- JS: JavaScript files are in `assets/js`
- Images: Image files are in `assets/images`
- Fonts: Font files are in `assets/fonts`

## Stylesheet Processing

Jekyll uses Sass by default. For proper processing:

1. Ensure `_sass` directory contains your partial Sass files (prefixed with `_`)
2. The main Sass file in `assets/css/main.scss` should have empty front matter and import your partials

Example:
```scss
---
---

@import "main";
```

## Configuration

Check the `_config.yml` file for Sass settings:

```yaml
sass:
  style: compressed
```
"#;
    
    fs::write(&assets_readme, assets_readme_content)
        .map_err(|e| format!("Failed to write assets README: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "assets/README.md".into(),
        change_type: ChangeType::Created,
        description: "Created assets README guide".into(),
    });
    
    Ok(())
}

fn setup_jekyll_scss(
    sass_dir: &Path,
    css_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Update _config.yml to include Sass settings
    let config_path = dest_dir.join("_config.yml");
    let mut config_content = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read _config.yml: {}", e))?
    } else {
        "# Site settings\ntitle: Migrated Metalsmith Site\n".to_string()
    };
    
    // Only add if not already present
    if !config_content.contains("sass:") {
        config_content.push_str("\n# Sass settings (migrated from Metalsmith)\nsass:\n  sass_dir: _sass\n  style: compressed\n");
        
        fs::write(&config_path, &config_content)
            .map_err(|e| format!("Failed to update _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".into(),
            change_type: ChangeType::Modified,
            description: "Added Sass configuration".into(),
        });
    }
    
    // Check for a main.scss file in css_dir, create if not exists
    let main_scss = css_dir.join("main.scss");
    if !main_scss.exists() {
        // Look for potential main SCSS files in the sass directory
        let sass_files: Vec<_> = WalkDir::new(sass_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                let file_name = e.file_name().to_string_lossy().to_lowercase();
                e.path().is_file() && (file_name == "main.scss" || file_name == "styles.scss" || file_name == "style.scss")
            })
            .collect();
        
        let sass_imports = if !sass_files.is_empty() {
            // Use existing SCSS files as imports
            let imports: Vec<_> = sass_files.iter()
                .map(|entry| {
                    let file_stem = entry.path().file_stem().unwrap_or_default().to_string_lossy();
                    let import_name = if file_stem.starts_with('_') {
                        file_stem[1..].to_string()
                    } else {
                        file_stem.to_string()
                    };
                    format!("@import \"{}\";\n", import_name)
                })
                .collect();
            imports.join("")
        } else {
            // Look for any SCSS files and import them
            "@import \"main\";\n".to_string()
        };
        
        let main_scss_content = format!("---\n---\n\n{}", sass_imports);
        
        fs::write(&main_scss, &main_scss_content)
            .map_err(|e| format!("Failed to write main.scss: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "assets/css/main.scss".into(),
            change_type: ChangeType::Created,
            description: "Created main SCSS file for Jekyll".into(),
        });
    }
    
    Ok(())
} 