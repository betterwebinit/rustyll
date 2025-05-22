use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy assets...");
    }

    // Create assets directory
    let assets_dir = dest_dir.join("assets");
    fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;

    // Create subdirectories
    let css_dir = assets_dir.join("css");
    let js_dir = assets_dir.join("js");
    let images_dir = assets_dir.join("images");
    let fonts_dir = assets_dir.join("fonts");

    for dir in [&css_dir, &js_dir, &images_dir, &fonts_dir] {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;
    }

    // Look for assets in various locations
    let static_dir = source_dir.join("static");
    let theme_static_dir = source_dir.join("themes/docsy/static");
    let assets_source_dir = source_dir.join("assets");

    // Migrate static files (user overrides first)
    if static_dir.exists() {
        migrate_static_files(&static_dir, &assets_dir, verbose, result)?;
    }

    // Migrate theme static files
    if theme_static_dir.exists() {
        migrate_static_files(&theme_static_dir, &assets_dir, verbose, result)?;
    }

    // Migrate assets directory if it exists (newer Hugo structure)
    if assets_source_dir.exists() {
        migrate_assets_directory(&assets_source_dir, &assets_dir, verbose, result)?;
    }

    // Create default assets if none found
    if !static_dir.exists() && !theme_static_dir.exists() && !assets_source_dir.exists() {
        create_default_assets(&assets_dir, result)?;
    }

    Ok(())
}

fn migrate_static_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix(source_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            let dest_path = dest_dir.join(relative_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            fs::copy(path, &dest_path)
                .map_err(|e| format!("Failed to copy static file: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("assets/{}", relative_path.display()).into(),
                description: format!("Copied static file from {}", path.display()).into(),
            });
        }
    }

    Ok(())
}

fn migrate_assets_directory(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process different asset types differently
    let scss_dir = source_dir.join("scss");
    let js_dir = source_dir.join("js");
    let images_dir = source_dir.join("images");
    let vendor_dir = source_dir.join("vendor");

    // Migrate SCSS/CSS files
    if scss_dir.exists() {
        migrate_scss_files(&scss_dir, &dest_dir.join("css"), verbose, result)?;
    }

    // Migrate JS files
    if js_dir.exists() {
        migrate_js_files(&js_dir, &dest_dir.join("js"), verbose, result)?;
    }

    // Copy images directly
    if images_dir.exists() {
        migrate_static_files(&images_dir, &dest_dir.join("images"), verbose, result)?;
    }

    // Copy vendor files
    if vendor_dir.exists() {
        migrate_static_files(&vendor_dir, &dest_dir.join("vendor"), verbose, result)?;
    }

    // Also check for any files in the root assets directory
    for entry in fs::read_dir(source_dir).map_err(|e| format!("Failed to read assets directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            // Determine destination based on file type
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            let dest_subdir = match extension {
                "css" | "scss" => "css",
                "js" => "js",
                "jpg" | "jpeg" | "png" | "gif" | "svg" => "images",
                "ttf" | "woff" | "woff2" | "eot" => "fonts",
                _ => "other",
            };

            let file_name = path.file_name().unwrap();
            let dest_path = dest_dir.join(dest_subdir).join(file_name);

            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy asset file: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("assets/{}/{}", dest_subdir, file_name.to_string_lossy()).into(),
                description: format!("Copied asset file from {}", path.display()).into(),
            });
        }
    }

    Ok(())
}

fn migrate_scss_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create the destination directory
    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create CSS directory: {}", e))?;

    // Copy SCSS files, converting @import paths if needed
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "scss" || extension == "css" {
                let relative_path = path.strip_prefix(source_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = dest_dir.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                // Read the file and convert paths if necessary
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read SCSS file: {}", e))?;
                
                // Convert Hugo-style imports to Jekyll-style imports
                let converted_content = convert_scss_imports(&content);
                
                fs::write(&dest_path, converted_content)
                    .map_err(|e| format!("Failed to write SCSS file: {}", e))?;

                result.changes.push(MigrationChange {
                    change_type: ChangeType::Created,
                    file_path: format!("assets/css/{}", relative_path.display()).into(),
                    description: format!("Converted SCSS file from {}", path.display()).into(),
                });
            }
        }
    }

    // Create a main.scss file that imports all the partials
    create_main_scss(dest_dir, result)?;

    Ok(())
}

fn convert_scss_imports(content: &str) -> String {
    // Convert Hugo-style imports to Jekyll-style
    // For example: @import "variables"; -> @import "variables";
    // This function can be expanded if more complex transformations are needed
    content.to_string()
}

fn create_main_scss(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a main.scss file that will be the entry point
    let main_scss = r#"---
# Only the main Sass file needs front matter (the dashes are enough)
---

// Import partials
@import "variables";
@import "base";
@import "components";
@import "utilities";
"#;

    let dest_path = dest_dir.join("main.scss");
    fs::write(&dest_path, main_scss)
        .map_err(|e| format!("Failed to write main.scss: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.scss".into(),
        description: "Created main SCSS file".into(),
    });

    Ok(())
}

fn migrate_js_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create the destination directory
    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create JS directory: {}", e))?;

    // Copy JS files
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "js" {
                let relative_path = path.strip_prefix(source_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = dest_dir.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                fs::copy(path, &dest_path)
                    .map_err(|e| format!("Failed to copy JS file: {}", e))?;

                result.changes.push(MigrationChange {
                    change_type: ChangeType::Created,
                    file_path: format!("assets/js/{}", relative_path.display()).into(),
                    description: format!("Copied JS file from {}", path.display()).into(),
                });
            }
        }
    }

    Ok(())
}

fn create_default_assets(
    assets_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default CSS
    let css_content = r#"/* Main styles for migrated Docsy site */
:root {
  --primary-color: #30638E;
  --secondary-color: #FFA630;
  --text-color: #333;
  --light-color: #f8f9fa;
  --dark-color: #343a40;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: var(--text-color);
  line-height: 1.6;
}

a {
  color: var(--primary-color);
}

a:hover {
  color: #1a3c56;
}

.td-navbar {
  background-color: var(--primary-color);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.td-sidebar {
  padding-top: 2rem;
}

.td-content {
  padding: 2rem 0;
}

.td-footer {
  background-color: var(--dark-color);
  color: var(--light-color);
  padding: 2rem 0;
}
"#;

    let css_path = assets_dir.join("css/main.css");
    fs::write(&css_path, css_content)
        .map_err(|e| format!("Failed to write default CSS: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.css".into(),
        description: "Created default CSS styles".into(),
    });

    // Create default JS
    let js_content = r#"// Main JavaScript for migrated Docsy site
document.addEventListener('DOMContentLoaded', function() {
  // Initialize any components that need JavaScript
  initNavbar();
  initSearch();
});

function initNavbar() {
  // Handle mobile navigation
  const navToggle = document.querySelector('.td-navbar-toggle');
  if (navToggle) {
    navToggle.addEventListener('click', function() {
      const navbar = document.querySelector('.td-navbar-nav-scroll');
      navbar.classList.toggle('show');
    });
  }
}

function initSearch() {
  // Basic search functionality
  const searchInput = document.querySelector('#search-input');
  if (searchInput) {
    searchInput.addEventListener('keypress', function(e) {
      if (e.key === 'Enter') {
        window.location.href = '/search?q=' + encodeURIComponent(searchInput.value);
      }
    });
  }
}
"#;

    let js_path = assets_dir.join("js/main.js");
    fs::write(&js_path, js_content)
        .map_err(|e| format!("Failed to write default JS: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/js/main.js".into(),
        description: "Created default JavaScript".into(),
    });

    // Create a simple logo
    let svg_content = r###"<svg width="200" height="60" xmlns="http://www.w3.org/2000/svg">
  <rect width="200" height="60" fill="#30638E" />
  <text x="20" y="40" font-family="Arial" font-size="24" fill="white">Migrated Site</text>
</svg>"###;

    let logo_path = assets_dir.join("images/logo.svg");
    fs::write(&logo_path, svg_content)
        .map_err(|e| format!("Failed to write default logo: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/images/logo.svg".into(),
        description: "Created default logo".into(),
    });

    Ok(())
} 