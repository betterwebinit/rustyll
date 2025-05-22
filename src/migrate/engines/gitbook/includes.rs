use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::GitbookMigrator {
    pub(super) fn migrate_includes(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In GitBook, includes might be in _includes/ directory (for custom themes)
        // or in theme/_includes/ or other theme directories
        
        // Create destination includes directory
        let dest_includes_dir = dest_dir.join("_includes");
        create_dir_if_not_exists(&dest_includes_dir)?;
        
        // Check for existing includes
        let source_includes_dir = source_dir.join("_includes");
        let theme_includes_dir = source_dir.join("theme").join("_includes");
        
        let mut found_includes = false;
        
        // First check _includes directory
        if source_includes_dir.exists() && source_includes_dir.is_dir() {
            if verbose {
                log::info!("Found _includes directory, migrating includes");
            }
            
            found_includes = self.migrate_includes_directory(&source_includes_dir, &dest_includes_dir, result)?;
        }
        
        // Then check theme/_includes directory
        if theme_includes_dir.exists() && theme_includes_dir.is_dir() {
            if verbose {
                log::info!("Found theme/_includes directory, migrating includes");
            }
            
            let theme_includes_found = self.migrate_includes_directory(&theme_includes_dir, &dest_includes_dir, result)?;
            found_includes = found_includes || theme_includes_found;
        }
        
        // Also look for partials in theme directory
        let theme_dir = source_dir.join("theme");
        if theme_dir.exists() && theme_dir.is_dir() {
            // Look for common partial patterns in theme directory
            for entry in WalkDir::new(&theme_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                    
                    // Check for common partial file patterns (header.html, footer.html, etc.)
                    if file_name.ends_with(".html") && 
                       (file_name.starts_with("_") || 
                        ["header", "footer", "head", "nav", "sidebar", "toc", "analytics"]
                            .iter()
                            .any(|&pattern| file_name.starts_with(pattern))) {
                        
                        if verbose {
                            log::info!("Migrating possible include file: {}", file_path.display());
                        }
                        
                        self.migrate_include_file(file_path, &dest_includes_dir, result)?;
                        found_includes = true;
                    }
                }
            }
        }
        
        // Create basic includes if none were found
        if !found_includes {
            if verbose {
                log::info!("No includes found, creating basic includes");
            }
            
            self.create_basic_includes(&dest_includes_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_includes_directory(&self, source_includes_dir: &Path, dest_includes_dir: &Path, result: &mut MigrationResult) -> Result<bool, String> {
        let mut found_includes = false;
        
        for entry in WalkDir::new(source_includes_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process all files in the includes directory
                self.migrate_include_file(file_path, dest_includes_dir, result)?;
                found_includes = true;
            }
        }
        
        Ok(found_includes)
    }
    
    fn migrate_include_file(&self, file_path: &Path, dest_includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the include file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read include file {}: {}", file_path.display(), e))?;
            
        // Convert GitBook template syntax to Liquid syntax
        let converted_content = convert_include_to_liquid(&content);
        
        // Determine the include name
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        let include_name = if file_name.starts_with("_") && file_name.len() > 1 {
            // Remove leading underscore for Jekyll includes
            let mut chars = file_name.chars();
            chars.next(); // Skip the underscore
            chars.as_str().to_string()
        } else {
            file_name.to_string()
        };
        
        // Make sure it has .html extension for Jekyll includes
        let include_name = if !include_name.ends_with(".html") && include_name.contains('.') {
            // Change extension to .html
            let dot_pos = include_name.rfind('.').unwrap();
            format!("{}.html", &include_name[0..dot_pos])
        } else if !include_name.contains('.') {
            // Add .html extension
            format!("{}.html", include_name)
        } else {
            include_name.to_string()
        };
        
        // Write to destination
        let dest_path = dest_includes_dir.join(&include_name);
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write include file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_includes/{}", include_name),
            change_type: ChangeType::Converted,
            description: "Include file migrated from GitBook".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_includes(&self, dest_includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create header include
        let header_content = r#"<header class="book-header" role="navigation">
  <h1>
    <i class="fa fa-circle-o-notch fa-spin"></i>
    <a href="{{ "/" | relative_url }}" >{{ site.title }}</a>
  </h1>
</header>"#;

        fs::write(dest_includes_dir.join("header.html"), header_content)
            .map_err(|e| format!("Failed to write header include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/header.html".to_string(),
            change_type: ChangeType::Created,
            description: "Header include created".to_string(),
        });
        
        // Create head include
        let head_content = r#"<meta charset="UTF-8">
<meta content="text/html; charset=utf-8" http-equiv="Content-Type">
<title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
<meta http-equiv="X-UA-Compatible" content="IE=edge" />
<meta name="description" content="{% if page.excerpt %}{{ page.excerpt | strip_html | strip_newlines | truncate: 160 }}{% else %}{{ site.description }}{% endif %}">
<meta name="generator" content="Rustyll">
<meta name="author" content="{{ site.author }}">

<link rel="stylesheet" href="{{ "/assets/css/style.css" | relative_url }}">
<link rel="stylesheet" href="{{ "/assets/css/website.css" | relative_url }}">

<!-- Custom head content -->
{% if site.extra_head %}
  {{ site.extra_head }}
{% endif %}"#;

        fs::write(dest_includes_dir.join("head.html"), head_content)
            .map_err(|e| format!("Failed to write head include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/head.html".to_string(),
            change_type: ChangeType::Created,
            description: "Head include created".to_string(),
        });
        
        // Create navigation include
        let navigation_content = r#"<nav role="navigation">
  <ul class="summary">
    <li{% if page.url == "/" %} class="active"{% endif %}>
      <a href="{{ "/" | relative_url }}">{{ site.title }}</a>
    </li>
    
    <li class="divider"></li>
    
    {% if site.data.navigation %}
      {% for section in site.data.navigation %}
        {% if section.title %}
          <li class="header">{{ section.title }}</li>
          {% for item in section.items %}
            <li{% if page.url == item.url %} class="active"{% endif %}>
              <a href="{{ item.url | relative_url }}">{{ item.title }}</a>
            </li>
          {% endfor %}
          <li class="divider"></li>
        {% endif %}
      {% endfor %}
    {% else %}
      {% assign sorted_pages = site.pages | sort: "weight" %}
      {% for page in sorted_pages %}
        {% if page.title %}
          <li{% if page.url == item.url %} class="active"{% endif %}>
            <a href="{{ page.url | relative_url }}">{{ page.title }}</a>
          </li>
        {% endif %}
      {% endfor %}
    {% endif %}
  </ul>
</nav>"#;

        fs::write(dest_includes_dir.join("navigation.html"), navigation_content)
            .map_err(|e| format!("Failed to write navigation include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/navigation.html".to_string(),
            change_type: ChangeType::Created,
            description: "Navigation include created".to_string(),
        });
        
        // Create footer include
        let footer_content = r#"<footer class="book-footer">
  <div class="footer-wrapper">
    <div class="footer-content">
      <p>
        Published with 
        <a href="https://rustyll.org" target="_blank">Rustyll</a>
        (migrated from GitBook)
      </p>
      <p>&copy; {{ site.time | date: '%Y' }} {{ site.author }}</p>
    </div>
  </div>
</footer>"#;

        fs::write(dest_includes_dir.join("footer.html"), footer_content)
            .map_err(|e| format!("Failed to write footer include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/footer.html".to_string(),
            change_type: ChangeType::Created,
            description: "Footer include created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to convert GitBook include syntax to Liquid
fn convert_include_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // GitBook uses Nunjucks/Jinja2 style includes
    // Most of these are the same in Liquid, but some special cases need conversion
    
    // Replace include with parameters
    // {% include "./file.html", param1="value1", param2="value2" %} -> {% include file.html param1="value1" param2="value2" %}
    result = regex::Regex::new(r#"\{%\s*include\s+["']([^"']+)["']\s*,\s*(.+?)\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let file = caps.get(1).unwrap().as_str().trim();
            let file = if file.starts_with("./") { &file[2..] } else { file };
            
            let params = caps.get(2).unwrap().as_str().trim()
                .replace(',', " ")
                .replace('=', "=");
                
            format!("{{% include {} {} %}}", file, params)
        })
        .to_string();
        
    // Replace regular includes
    // {% include "./file.html" %} -> {% include file.html %}
    result = regex::Regex::new(r#"\{%\s*include\s+["']([^"']+)["']\s*%\}"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            let file = caps.get(1).unwrap().as_str().trim();
            // Remove "./" from the beginning
            let file = if file.starts_with("./") { &file[2..] } else { file };
            format!("{{% include {} %}}", file)
        })
        .to_string();
        
    // Check for GitBook specific tags and add warning
    if result.contains("gitbook.") || result.contains("book.") {
        result = format!("<!-- Warning: This include contains GitBook-specific tags that may need manual adjustment -->\n\n{}", result);
    }
    
    result
} 