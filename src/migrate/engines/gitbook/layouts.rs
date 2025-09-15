use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::GitbookMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In GitBook, layouts might be in _layouts/ directory (for custom themes)
        // Otherwise, we'll need to create basic layouts compatible with Rustyll/Jekyll
        
        // Create destination layouts directory
        let dest_layouts_dir = dest_dir.join("_layouts");
        create_dir_if_not_exists(&dest_layouts_dir)?;
        
        // Check for existing layouts
        let source_layouts_dir = source_dir.join("_layouts");
        let theme_dir = source_dir.join("theme");
        
        let mut found_layouts = false;
        
        // First check _layouts directory
        if source_layouts_dir.exists() && source_layouts_dir.is_dir() {
            if verbose {
                log::info!("Found _layouts directory, migrating layouts");
            }
            
            found_layouts = self.migrate_layout_directory(&source_layouts_dir, &dest_layouts_dir, result)?;
        }
        
        // Then check theme directory
        if theme_dir.exists() && theme_dir.is_dir() {
            if verbose {
                log::info!("Found theme directory, looking for layout files");
            }
            
            // Look for common layout files in theme directory
            for entry in WalkDir::new(&theme_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                    
                    if file_name.ends_with(".html") {
                        // This might be a layout file
                        if verbose {
                            log::info!("Migrating possible layout file: {}", file_path.display());
                        }
                        
                        // Copy the file and adapt it for Rustyll
                        self.migrate_layout_file(file_path, &dest_layouts_dir, result)?;
                        found_layouts = true;
                    }
                }
            }
        }
        
        // Create basic layouts if none were found
        if !found_layouts {
            if verbose {
                log::info!("No layouts found, creating basic layouts");
            }
            
            self.create_basic_layouts(&dest_layouts_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_layout_directory(&self, source_layouts_dir: &Path, dest_layouts_dir: &Path, result: &mut MigrationResult) -> Result<bool, String> {
        let mut found_layouts = false;
        
        for entry in WalkDir::new(source_layouts_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Only process HTML files
                if let Some(extension) = file_path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "html" {
                        // This is a layout file
                        self.migrate_layout_file(file_path, dest_layouts_dir, result)?;
                        found_layouts = true;
                    }
                }
            }
        }
        
        Ok(found_layouts)
    }
    
    fn migrate_layout_file(&self, file_path: &Path, dest_layouts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the layout file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
            
        // Convert GitBook template syntax to Liquid syntax
        let converted_content = convert_gitbook_to_liquid(&content);
        
        // Determine the layout name
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        let layout_name = if file_name.to_lowercase() == "website.html" || file_name.to_lowercase() == "page.html" {
            // Standard GitBook layouts, rename to default
            "default.html".to_string()
        } else {
            file_name.to_string()
        };
        
        // Write to destination
        let dest_path = dest_layouts_dir.join(&layout_name);
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write layout file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_layouts/{}", layout_name),
            change_type: ChangeType::Converted,
            description: "Layout file migrated from GitBook".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_layouts(&self, dest_layouts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create default layout
        let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <meta name="description" content="{% if page.excerpt %}{{ page.excerpt | strip_html | strip_newlines | truncate: 160 }}{% else %}{{ site.description }}{% endif %}">
  <link rel="stylesheet" href="{{ "/assets/css/style.css" | relative_url }}">
</head>
<body>
  <div class="book">
    <div class="book-summary">
      <nav role="navigation">
        <ul class="summary">
          <li>
            <a href="{{ "/" | relative_url }}" class="custom-link">{{ site.title }}</a>
          </li>
          
          {% if site.data.navigation %}
            {% for section in site.data.navigation %}
              {% if section.title %}
                <li class="divider"></li>
                <li class="header">{{ section.title }}</li>
                {% for item in section.items %}
                  <li>
                    <a href="{{ item.url | relative_url }}">{{ item.title }}</a>
                  </li>
                {% endfor %}
              {% endif %}
            {% endfor %}
          {% else %}
            <li class="divider"></li>
            {% for page in site.pages %}
              {% if page.title %}
                <li>
                  <a href="{{ page.url | relative_url }}">{{ page.title }}</a>
                </li>
              {% endif %}
            {% endfor %}
          {% endif %}
        </ul>
      </nav>
    </div>
    
    <div class="book-body">
      <div class="body-inner">
        <div class="page-wrapper" tabindex="-1" role="main">
          <div class="page-inner">
            <div class="search-plus" id="book-search-results">
              <div class="search-noresults">
                <section class="normal markdown-section">
                  {{ content }}
                </section>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
  
  <script src="{{ "/assets/js/main.js" | relative_url }}"></script>
</body>
</html>"#;

        fs::write(dest_layouts_dir.join("default.html"), default_layout)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Default layout created (GitBook-style)".to_string(),
        });
        
        // Create page layout (extends default)
        let page_layout = r#"---
layout: default
---
{{ content }}"#;

        fs::write(dest_layouts_dir.join("page.html"), page_layout)
            .map_err(|e| format!("Failed to write page layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/page.html".to_string(),
            change_type: ChangeType::Created,
            description: "Page layout created".to_string(),
        });
        
        // Create home layout (extends default)
        let home_layout = r#"---
layout: default
---
<div class="home-content">
  {{ content }}
  
  {% if site.pages.size > 0 %}
    <div class="pages-list">
      <h2>Contents</h2>
      <ul>
        {% for page in site.pages %}
          {% if page.title and page.title != page.title %}
            <li>
              <a href="{{ page.url | relative_url }}">{{ page.title }}</a>
              {% if page.description %}
                <p>{{ page.description }}</p>
              {% endif %}
            </li>
          {% endif %}
        {% endfor %}
      </ul>
    </div>
  {% endif %}
</div>"#;

        fs::write(dest_layouts_dir.join("home.html"), home_layout)
            .map_err(|e| format!("Failed to write home layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/home.html".to_string(),
            change_type: ChangeType::Created,
            description: "Home layout created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to convert GitBook template syntax to Liquid
fn convert_gitbook_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // GitBook uses a mix of different templating systems (Nunjucks, Jinja2, etc.)
    // Here we convert the most common patterns to Liquid syntax
    
    // Replace {{ content }} with {{ content }} (no change needed if already using this syntax)
    
    // Replace blocks
    // {% block body %}...{% endblock %} -> {{ content }}
    result = regex::Regex::new(r"(?s)\{%\s*block\s+body\s*%\}.*?\{%\s*endblock\s*%\}")
        .unwrap()
        .replace_all(&result, "{{ content }}")
        .to_string();
        
    // Replace extends
    // {% extends "layout.html" %} -> ---\nlayout: layout\n---
    result = regex::Regex::new(r#"(?s)\{%\s*extends\s+"([^"]+)"\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let layout_name = caps.get(1).unwrap().as_str();
            let layout_name = layout_name.trim_end_matches(".html");
            format!("---\nlayout: {}\n---", layout_name)
        })
        .to_string();
    
    // Also handle single quotes in extends
    result = regex::Regex::new(r#"(?s)\{%\s*extends\s+'([^']+)'\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let layout_name = caps.get(1).unwrap().as_str();
            let layout_name = layout_name.trim_end_matches(".html");
            format!("---\nlayout: {}\n---", layout_name)
        })
        .to_string();
    
    // Replace include with double quotes
    // {% include "head.html" %} -> {% include head.html %}
    result = regex::Regex::new(r#"(?s)\{%\s*include\s+"([^"]+)"\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let include_name = caps.get(1).unwrap().as_str();
            format!("{{% include {} %}}", include_name)
        })
        .to_string();
    
    // Replace include with single quotes
    result = regex::Regex::new(r#"(?s)\{%\s*include\s+'([^']+)'\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let include_name = caps.get(1).unwrap().as_str();
            format!("{{% include {} %}}", include_name)
        })
        .to_string();
    
    // Replace if statements
    // {% if condition %}...{% endif %} (no change needed if already using this syntax)
    
    // Replace for loops
    // {% for item in items %}...{% endfor %} (no change needed if already using this syntax)
    
    // Check for Gitbook specific tags and add warning
    if result.contains("gitbook.") || result.contains("book.") {
        // These are GitBook specific and need special handling
        result = format!("<!-- Warning: This layout contains GitBook-specific tags that may need manual adjustment -->\n\n{}", result);
    }
    
    result
} 