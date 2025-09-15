use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, write_readme};

pub fn migrate_layouts(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MkDocs layouts...");
    }

    // Create layouts directory
    let layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&layouts_dir)?;

    // Check if there are custom templates in the MkDocs project
    let custom_dir = source_dir.join("custom_theme");
    // MkDocs themes are usually not in the project directory but installed via pip
    // However, some projects have custom themes or theme_dir specified
    
    // Common locations for custom themes
    let potential_theme_dirs = vec![
        source_dir.join("theme"),
        source_dir.join("custom_theme"),
        source_dir.join("custom_dir"),
        source_dir.join("overrides"),
        source_dir.join("material"),
    ];
    
    // Create destination layouts directory
    let dest_layouts = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts)?;
    
    // Create destination includes directory for partials
    let dest_includes = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes)?;
    
    // Check for custom theme in mkdocs.yml
    let mut custom_theme_dir = None;
    let mkdocs_yml = source_dir.join("mkdocs.yml");
    if mkdocs_yml.exists() {
        if verbose {
            log::info!("Checking mkdocs.yml for custom theme configuration");
        }
        
        let config_content = fs::read_to_string(&mkdocs_yml)
            .map_err(|e| format!("Failed to read mkdocs.yml: {}", e))?;
        
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&config_content) {
            // Check for theme_dir or custom_dir
            if let Some(theme_dir) = yaml.get("theme_dir")
                .and_then(|v| v.as_str())
                .or_else(|| yaml.get("theme")
                    .and_then(|v| v.get("custom_dir"))
                    .and_then(|v| v.as_str())) {
                
                custom_theme_dir = Some(source_dir.join(theme_dir));
                
                if verbose {
                    log::info!("Found custom theme directory in mkdocs.yml: {}", theme_dir);
                }
            }
        }
    }
    
    // Add custom theme directory to potential dirs if found
    let theme_dirs = if let Some(dir) = custom_theme_dir {
        let mut dirs = potential_theme_dirs.clone();
        dirs.insert(0, dir);
        dirs
    } else {
        potential_theme_dirs.clone()
    };
    
    let mut found_theme = false;
    
    // Check each potential theme directory
    for theme_dir in theme_dirs {
        if theme_dir.exists() && theme_dir.is_dir() {
            found_theme = true;
            
            if verbose {
                log::info!("Found theme directory: {}", theme_dir.display());
            }
            
            // Look for template files
            let template_extensions = ["html", "htm", "j2", "jinja", "jinja2"];
            
            for entry in WalkDir::new(&theme_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = entry.file_name().to_string_lossy();
                    
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        
                        if template_extensions.contains(&ext.as_ref()) {
                            // Determine if this is a layout or include
                            let is_layout = file_name == "base.html" || 
                                           file_name == "main.html" || 
                                           file_path.to_string_lossy().contains("layout") ||
                                           file_path.to_string_lossy().contains("base");
                            
                            let is_include = file_name.starts_with("_") || 
                                            file_path.to_string_lossy().contains("partials") ||
                                            file_path.to_string_lossy().contains("includes");
                            
                            // Get file content
                            let content = fs::read_to_string(file_path)
                                .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                            
                            // Process the template
                            let (dest_dir, file_type) = if is_layout {
                                // Use a standard name for the layout file
                                let layout_name = if file_name == "base.html" || file_name == "main.html" {
                                    "doc.html".to_string()
                                } else {
                                    file_name.to_string()
                                };
                                
                                (dest_layouts.clone(), format!("_layouts/{}", layout_name))
                            } else if is_include {
                                // Use the original name for includes
                                (dest_includes.clone(), format!("_includes/{}", file_name))
                            } else {
                                // If not sure, treat as include
                                (dest_includes.clone(), format!("_includes/{}", file_name))
                            };
                            
                            let dest_path = dest_dir.join(file_name.to_string());
                            
                            // Convert Jinja2 template to Liquid
                            let converted_content = convert_jinja_to_liquid(&content);
                            
                            // Save the converted file
                            fs::write(&dest_path, converted_content)
                                .map_err(|e| format!("Failed to write file {}: {}", dest_path.display(), e))?;
                            
                            result.changes.push(MigrationChange {
                                file_path: file_type,
                                change_type: ChangeType::Converted,
                                description: "Template converted from Jinja2 to Liquid".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // If no theme files found, create basic templates
    if !found_theme {
        // Create a basic doc layout
        let doc_layout = dest_layouts.join("doc.html");
        let doc_layout_content = r#"---
layout: default
---
<div class="doc-container">
  <div class="doc-sidebar">
    {% include sidebar.html %}
  </div>
  <div class="doc-content">
    <h1>{{ page.title }}</h1>
    {{ content }}
  </div>
</div>
"#;
        
        fs::write(&doc_layout, doc_layout_content)
            .map_err(|e| format!("Failed to write doc layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/doc.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic document layout created".to_string(),
        });
        
        // Create a default layout
        let default_layout = dest_layouts.join("default.html");
        let default_layout_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
    <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
    <header class="site-header">
        <div class="wrapper">
            <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title }}</a>
            <nav class="site-nav">
                <a href="{{ '/docs/' | relative_url }}">Documentation</a>
            </nav>
        </div>
    </header>

    <main class="page-content">
        <div class="wrapper">
            {{ content }}
        </div>
    </main>

    <footer class="site-footer">
        <div class="wrapper">
            <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
        </div>
    </footer>
</body>
</html>"#;
        
        fs::write(&default_layout, default_layout_content)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Created default layout for MkDocs site".to_string(),
        });
        
        // Create a home layout
        let home_layout = dest_layouts.join("home.html");
        let home_layout_content = r#"---
layout: default
---
<div class="home-container">
    <div class="hero">
        <h1>{{ site.title }}</h1>
        <p>{{ site.description }}</p>
        <a href="/docs/" class="btn">View Documentation</a>
    </div>
    
    {{ content }}
</div>
"#;
        
        fs::write(&home_layout, home_layout_content)
            .map_err(|e| format!("Failed to write home layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/home.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic home layout created".to_string(),
        });
        
        // Create a basic sidebar include
        let sidebar_include = dest_includes.join("sidebar.html");
        let sidebar_include_content = r#"<div class="sidebar">
    <h3>Documentation</h3>
    <ul class="nav-items">
        {% for item in site.data.navigation.docs_nav %}
            {% if item.title %}
                <li class="nav-item">
                    <a href="/docs/{{ item.url }}">{{ item.title }}</a>
                    {% if item.children %}
                        <ul>
                            {% for child in item.children %}
                                <li><a href="/docs/{{ child.url }}">{{ child.title }}</a></li>
                            {% endfor %}
                        </ul>
                    {% endif %}
                </li>
            {% endif %}
        {% endfor %}
    </ul>
</div>
"#;
        
        fs::write(&sidebar_include, sidebar_include_content)
            .map_err(|e| format!("Failed to write sidebar include: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_includes/sidebar.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic sidebar navigation include created".to_string(),
        });
        
        result.warnings.push(
            "No custom theme found. Basic layouts and templates have been created.".to_string()
        );
    }
    
    // Create README for layouts directory
    let layouts_readme = r#"# Layouts Directory

This directory contains layout templates for the Rustyll site migrated from MkDocs.

## Layout Structure

- `default.html` - The base layout for all pages
- `doc.html` - Layout for documentation pages
- `home.html` - Layout for the home page

## Changes from MkDocs

- MkDocs uses Jinja2 templates, which have been converted to Liquid syntax
- Some manual adjustments may be needed for complex template logic
- MkDocs extensions and plugins need to be reimplemented as Rustyll features
"#;
    
    write_readme(&dest_layouts, layouts_readme)?;
    
    // Create README for includes directory
    let includes_readme = r#"# Includes Directory

This directory contains partial templates for the Rustyll site migrated from MkDocs.

## Includes Structure

- `sidebar.html` - Navigation sidebar for documentation pages
- Other includes converted from MkDocs theme

## Changes from MkDocs

- MkDocs uses Jinja2 templates, which have been converted to Liquid syntax
- Some MkDocs macros may need manual conversion
- Include usage is similar: `{% include "filename.html" %}` in Liquid
"#;
    
    write_readme(&dest_includes, includes_readme)?;
    
    // Create CSS for the layouts
    let dest_css_dir = dest_dir.join("assets/css");
    create_dir_if_not_exists(&dest_css_dir)?;

    let main_css = r#"/* Main styles for migrated MkDocs site */

/* Base styles */
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    margin: 0;
    padding: 0;
}

.wrapper {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Header styles */
.site-header {
    background-color: #2c3e50;
    color: white;
    padding: 1rem 0;
}

.site-title {
    font-size: 1.5rem;
    font-weight: bold;
    color: white;
    text-decoration: none;
}

.site-nav {
    float: right;
}

.site-nav a {
    color: #ecf0f1;
    margin-left: 20px;
    text-decoration: none;
}

.site-nav a:hover {
    text-decoration: underline;
}

/* Footer styles */
.site-footer {
    border-top: 1px solid #e8e8e8;
    padding: 20px 0;
    color: #828282;
    margin-top: 40px;
}

/* Documentation layout */
.doc-layout {
    display: flex;
    margin-top: 20px;
}

.doc-sidebar {
    width: 250px;
    padding-right: 30px;
}

.doc-nav {
    display: flex;
    flex-direction: column;
}

.doc-nav a {
    padding: 8px 0;
    color: #2c3e50;
    text-decoration: none;
    border-bottom: 1px solid #eee;
}

.doc-nav a.active {
    font-weight: bold;
    color: #1e88e5;
}

.doc-content {
    flex: 1;
    max-width: 800px;
}

.doc-footer {
    color: #828282;
    margin-top: 40px;
    padding-top: 10px;
    border-top: 1px solid #e8e8e8;
}

/* Content styles */
h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    color: #2c3e50;
}

a {
    color: #1e88e5;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

code {
    background-color: #f8f8f8;
    padding: 2px 5px;
    border-radius: 3px;
    font-family: Consolas, Monaco, "Andale Mono", monospace;
    font-size: 0.9em;
}

pre {
    background-color: #f8f8f8;
    padding: 10px;
    border-radius: 3px;
    overflow-x: auto;
}

pre code {
    padding: 0;
    background-color: transparent;
}

blockquote {
    border-left: 4px solid #ddd;
    padding-left: 20px;
    margin-left: 0;
    color: #666;
}

img {
    max-width: 100%;
}

/* Admonitions / Callouts (common in MkDocs) */
.alert {
    padding: 15px;
    margin-bottom: 20px;
    border-radius: 4px;
}

.alert-info {
    background-color: #d9edf7;
    border-left: 5px solid #31708f;
}

.alert-success {
    background-color: #dff0d8;
    border-left: 5px solid #3c763d;
}

.alert-warning {
    background-color: #fcf8e3;
    border-left: 5px solid #8a6d3b;
}

.alert-danger {
    background-color: #f2dede;
    border-left: 5px solid #a94442;
}

/* Code tabs (common in MkDocs) */
.code-tabs {
    margin-bottom: 20px;
}

.code-tabs .nav {
    padding-left: 0;
    margin-bottom: 0;
    list-style: none;
    display: flex;
    border-bottom: 1px solid #ddd;
}

.code-tabs .nav-item {
    margin-bottom: -1px;
}

.code-tabs .nav-link {
    padding: 8px 15px;
    text-decoration: none;
    color: #555;
}

.code-tabs .nav-link.active {
    color: #1e88e5;
    background-color: #fff;
    border: 1px solid #ddd;
    border-bottom-color: transparent;
    border-top-left-radius: 3px;
    border-top-right-radius: 3px;
}

.code-tabs .tab-content {
    padding: 15px;
    border: 1px solid #ddd;
    border-top: none;
}"#;

    fs::write(dest_css_dir.join("main.css"), main_css)
        .map_err(|e| format!("Failed to write main CSS file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/css/main.css".into(),
        description: "Created stylesheet for migrated MkDocs site".into(),
    });

    Ok(())
}

// Helper function to convert Jinja2 templates to Liquid
fn convert_jinja_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Replace common Jinja2 syntax with Liquid equivalents
    
    // Comments
    result = result.replace("{#", "{% comment %}").replace("#}", "{% endcomment %}");
    
    // For loops - basic format
    let mut for_loop_converted = result.replace("{% for ", "{% for ");
    for_loop_converted = for_loop_converted.replace("{% endfor %}", "{% endfor %}");
    
    // If statements - basic format
    let mut if_converted = for_loop_converted.replace("{% if ", "{% if ");
    if_converted = if_converted.replace("{% endif %}", "{% endif %}");
    if_converted = if_converted.replace("{% else %}", "{% else %}");
    if_converted = if_converted.replace("{% elif ", "{% elsif ");
    
    // Jinja2 filters that need to be converted
    if_converted = if_converted.replace("|tojson", "| json");
    if_converted = if_converted.replace("|striptags", "| strip_html");
    if_converted = if_converted.replace("|safe", "");  // 'safe' is default in Liquid
    
    // Basic variable access
    let var_converted = if_converted.replace("{{ config.", "{{ site.");
    
    result = var_converted;
    return result;
} 