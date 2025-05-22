use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_theme(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook theme...");
    }

    // Create theme directories
    let theme_dir = dest_dir.join("_includes/theme");
    let layouts_dir = dest_dir.join("_layouts");
    let assets_dir = dest_dir.join("assets/theme");

    for dir in [&theme_dir, &layouts_dir, &assets_dir] {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {}", dir.display(), e))?;
    }

    // Migrate default theme if no custom theme exists
    let custom_theme_dir = source_dir.join("theme");
    if !custom_theme_dir.exists() {
        migrate_default_theme(dest_dir, verbose, result)?;
        return Ok(());
    }

    // Migrate custom theme files
    for entry in WalkDir::new(&custom_theme_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix(&custom_theme_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("hbs") => {
                    // Convert Handlebars templates to Liquid
                    let dest_path = if relative_path.to_string_lossy() == "index.hbs" {
                        layouts_dir.join("default.html")
                    } else {
                        theme_dir.join(relative_path).with_extension("html")
                    };

                    migrate_template(path, &dest_path, verbose, result)?;
                },
                Some("css") | Some("scss") => {
                    // Copy stylesheets
                    let dest_path = assets_dir.join(relative_path);
                    migrate_stylesheet(path, &dest_path, verbose, result)?;
                },
                Some("js") => {
                    // Copy JavaScript files
                    let dest_path = assets_dir.join(relative_path);
                    migrate_javascript(path, &dest_path, verbose, result)?;
                },
                _ => {
                    // Copy other assets as-is
                    let dest_path = assets_dir.join(relative_path);
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                    }
                    fs::copy(path, &dest_path)
                        .map_err(|e| format!("Failed to copy asset: {}", e))?;

                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Copied,
                        file_path: format!("assets/theme/{}", relative_path.display()),
                        description: format!("Copied theme asset from {}", path.display()),
                    });
                }
            }
        }
    }

    Ok(())
}

fn migrate_default_theme(
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default layout
    let layout_content = r#"<!DOCTYPE html>
<html lang="{{ site.lang | default: "en" }}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% if page.title %}{{ page.title }} - {% endif %}{{ site.title }}</title>
  <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
  <link rel="stylesheet" href="{{ '/assets/css/book.css' | relative_url }}">
  {% if site.use_katex %}
  <link rel="stylesheet" href="{{ '/assets/css/katex.min.css' | relative_url }}">
  {% endif %}
</head>
<body>
  <div class="book">
    <div class="book-summary">
      {% include sidebar.html %}
    </div>
    
    <div class="book-body">
      <div class="body-inner">
        <div class="page-wrapper" tabindex="-1">
          <div class="page-inner">
            <section class="normal markdown-section">
              <h1 class="page-title">{{ page.title }}</h1>
              {{ content }}
            </section>
          </div>
        </div>
      </div>
    </div>
  </div>
  
  <script src="{{ '/assets/js/book.js' | relative_url }}"></script>
</body>
</html>"#;

    // Create default stylesheet
    let style_content = r#"/* MDBook default stylesheet migrated to Jekyll */
.book {
  display: flex;
  flex-direction: row;
  height: 100vh;
  margin: 0;
}

.book-summary {
  flex: 0 0 300px;
  overflow-y: auto;
  padding: 1rem;
  background-color: #f5f5f5;
  border-right: 1px solid #e0e0e0;
}

.book-body {
  flex: 1 1 auto;
  overflow-y: auto;
}

.page-inner {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}

.page-title {
  margin-top: 0;
  border-bottom: 1px solid #e0e0e0;
  padding-bottom: 0.5rem;
}

.markdown-section {
  line-height: 1.6;
  font-size: 16px;
}

/* Sidebar navigation */
.sidebar-nav {
  list-style: none;
  padding-left: 0;
}

.sidebar-nav li {
  margin-bottom: 0.5rem;
}

.sidebar-nav a {
  text-decoration: none;
  color: #333;
}

.sidebar-nav a:hover {
  color: #0074D9;
}

.sidebar-nav .active {
  font-weight: bold;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .book {
    flex-direction: column;
  }
  
  .book-summary {
    flex: 0 0 auto;
    height: auto;
    max-height: 40vh;
  }
}"#;

    fs::write(
        dest_dir.join("_layouts/default.html"),
        layout_content,
    ).map_err(|e| format!("Failed to write default layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".into(),
        description: "Created default theme layout".into(),
    });

    fs::write(
        dest_dir.join("assets/theme/book.scss"),
        style_content,
    ).map_err(|e| format!("Failed to write default styles: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/theme/book.scss".into(),
        description: "Created default theme styles".into(),
    });

    Ok(())
}

fn migrate_template(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    // Convert Handlebars to Liquid syntax
    let converted_content = convert_handlebars_to_liquid(&content)?;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write template: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: dest_path.to_string_lossy().into(),
        description: format!("Converted template from {}", source_path.display()),
    });

    Ok(())
}

fn convert_handlebars_to_liquid(content: &str) -> Result<String, String> {
    let mut converted = content.to_string();

    // Convert basic Handlebars syntax to Liquid
    converted = converted.replace("{{", "{{");
    converted = converted.replace("}}", "}}");
    converted = converted.replace("{{#if", "{% if");
    converted = converted.replace("{{/if}}", "{% endif %}");
    converted = converted.replace("{{#each", "{% for item in");
    converted = converted.replace("{{/each}}", "{% endfor %}");
    converted = converted.replace("{{else}}", "{% else %}");

    // Convert MDBook-specific helpers
    converted = converted.replace("{{chapter_title}}", "{{ page.title }}");
    converted = converted.replace("{{path}}", "{{ page.path }}");
    converted = converted.replace("{{content}}", "{{ content }}");

    Ok(converted)
}

fn migrate_stylesheet(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read stylesheet: {}", e))?;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(dest_path, content)
        .map_err(|e| format!("Failed to write stylesheet: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: dest_path.to_string_lossy().into(),
        description: format!("Copied stylesheet from {}", source_path.display()),
    });

    Ok(())
}

fn migrate_javascript(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read JavaScript file: {}", e))?;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(dest_path, content)
        .map_err(|e| format!("Failed to write JavaScript file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: dest_path.to_string_lossy().into(),
        description: format!("Copied JavaScript file from {}", source_path.display()),
    });

    Ok(())
} 