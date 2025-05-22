use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_static_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating static assets...");
    }

    // In MkDocs, static files are typically in the docs directory
    // or defined in the mkdocs.yml configuration
    let assets_dirs = [
        source_dir.join("docs/assets"),
        source_dir.join("docs/static"),
        source_dir.join("docs/images"),
        source_dir.join("docs/css"),
        source_dir.join("docs/js"),
        source_dir.join("site/assets"),
    ];

    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;

    // Ensure css and js directories exist in the destination
    let dest_css_dir = dest_assets_dir.join("css");
    let dest_js_dir = dest_assets_dir.join("js");
    let dest_images_dir = dest_assets_dir.join("images");

    create_dir_if_not_exists(&dest_css_dir)?;
    create_dir_if_not_exists(&dest_js_dir)?;
    create_dir_if_not_exists(&dest_images_dir)?;

    let mut found_assets = false;

    // Copy all static files from possible asset directories
    for assets_dir in assets_dirs.iter() {
        if assets_dir.exists() && assets_dir.is_dir() {
            found_assets = true;

            if verbose {
                log::info!("Copying assets from {}", assets_dir.display());
            }

            for entry in WalkDir::new(assets_dir)
                .into_iter()
                .filter_map(Result::ok) {

                if entry.file_type().is_file() {
                    let file_path = entry.path();

                    // Get the relative path from the assets directory
                    let relative_path = file_path.strip_prefix(assets_dir)
                        .map_err(|e| format!("Failed to get relative path: {}", e))?;

                    // Determine destination based on file extension
                    let extension = file_path.extension().map(|ext| ext.to_string_lossy().to_lowercase());
                    let dest_subdir = match extension.as_deref() {
                        Some("css") => "css",
                        Some("js") => "js",
                        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("svg") => "images",
                        _ => "other",
                    };

                    let dest_path = dest_assets_dir.join(dest_subdir).join(relative_path);

                    // Create parent directories if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }

                    // Copy the file
                    copy_file(file_path, &dest_path)?;

                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Created,
                        file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()),
                        description: "Copied static asset".to_string(),
                    });
                }
            }
        }
    }

    // If no assets were found, create default assets
    if !found_assets {
        create_default_assets(&dest_assets_dir, result)?;
    }

    Ok(())
}

fn create_default_assets(
    assets_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a default CSS file
    let css_content = r#"/* Default styles for MkDocs migrated site */
:root {
  --primary-color: #2980b9;
  --nav-background: #343131;
  --text-color: #333;
  --link-color: #2980b9;
  --link-hover-color: #3091d1;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Fira Sans", "Droid Sans", "Helvetica Neue", Arial, sans-serif;
  color: var(--text-color);
  line-height: 1.6;
}

a {
  color: var(--link-color);
}

a:hover {
  color: var(--link-hover-color);
}

.header {
  background-color: var(--primary-color);
  color: white;
  padding: 1rem 0;
}

.sidebar {
  background-color: #f8f8f8;
  padding: 1rem;
}

.content {
  padding: 1rem;
}

.admonition {
  padding: 1rem;
  margin: 1rem 0;
  border-left: 0.25rem solid var(--primary-color);
  background-color: #f8f9fa;
}

.admonition-title {
  font-weight: bold;
  margin-bottom: 0.5rem;
}
"#;

    let css_path = assets_dir.join("css/main.css");
    fs::write(&css_path, css_content)
        .map_err(|e| format!("Failed to write default CSS: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.css".into(),
        description: "Created default CSS".into(),
    });

    // Create a default JavaScript file
    let js_content = r#"// Default JavaScript for MkDocs migrated site
document.addEventListener('DOMContentLoaded', function() {
  // Mobile navigation toggle
  const navToggle = document.querySelector('.nav-toggle');
  if (navToggle) {
    navToggle.addEventListener('click', function() {
      const nav = document.querySelector('.nav');
      nav.classList.toggle('nav--open');
    });
  }

  // Highlight current page in navigation
  const currentPath = window.location.pathname;
  const navLinks = document.querySelectorAll('.nav a');
  navLinks.forEach(function(link) {
    if (link.getAttribute('href') === currentPath) {
      link.classList.add('active');
    }
  });
});
"#;

    let js_path = assets_dir.join("js/main.js");
    fs::write(&js_path, js_content)
        .map_err(|e| format!("Failed to write default JavaScript: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/js/main.js".into(),
        description: "Created default JavaScript".into(),
    });

    Ok(())
} 