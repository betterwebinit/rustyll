use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::NanocMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Nanoc, layouts are in the layouts/ directory
        let layouts_dir = source_dir.join("layouts");
        
        if !layouts_dir.exists() || !layouts_dir.is_dir() {
            result.warnings.push("Could not find Nanoc layouts directory.".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating layouts from Nanoc to Rustyll format");
        }
        
        // Create destination directory
        let dest_layouts_dir = dest_dir.join("_layouts");
        create_dir_if_not_exists(&dest_layouts_dir)?;
        
        // Process all layout files
        for entry in WalkDir::new(&layouts_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Get the file extension to determine the type
                if let Some(extension) = file_path.extension() {
                    let file_name = file_path.file_name()
                        .ok_or_else(|| "Invalid file name".to_string())?
                        .to_string_lossy();
                        
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    // Handle different template types
                    match ext.as_ref() {
                        "html" | "htm" => {
                            // HTML layouts can be directly copied or converted
                            self.process_html_layout(file_path, &file_name, &dest_layouts_dir, result)?;
                        },
                        "erb" => {
                            // ERB layouts need to be converted to Liquid
                            self.process_erb_layout(file_path, &file_name, &dest_layouts_dir, result)?;
                        },
                        "haml" => {
                            // HAML layouts need to be converted to HTML first
                            result.warnings.push(format!(
                                "HAML layout {} detected. HAML layouts need manual conversion to HTML/Liquid.", 
                                file_path.display()
                            ));
                            
                            // Copy the file for reference
                            let ref_file = dest_dir.join("nanoc_layouts").join(&*file_name);
                            create_dir_if_not_exists(&ref_file.parent().unwrap())?;
                            copy_file(file_path, &ref_file)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("nanoc_layouts/{}", file_name),
                                change_type: ChangeType::Copied,
                                description: "HAML layout preserved for reference".to_string(),
                            });
                        },
                        _ => {
                            // Other types are just copied for reference
                            let ref_file = dest_dir.join("nanoc_layouts").join(&*file_name);
                            create_dir_if_not_exists(&ref_file.parent().unwrap())?;
                            copy_file(file_path, &ref_file)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("nanoc_layouts/{}", file_name),
                                change_type: ChangeType::Copied,
                                description: format!("Unsupported layout type ({}) preserved for reference", ext).to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Create basic layouts if none were found
        if fs::read_dir(&dest_layouts_dir).map(|entries| entries.count()).unwrap_or(0) == 0 {
            self.create_basic_layouts(&dest_layouts_dir, result)?;
        }
        
        // Create README for layouts directory
        let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Nanoc.

## Layout Format

Layouts in Rustyll:
- Use Liquid templating instead of ERB or other engines
- Files are typically HTML with Liquid templating code
- Layouts can inherit from other layouts using the `layout` front matter variable

## Conversion Details

- Nanoc ERB layouts have been converted to Liquid syntax
- `<%= yield %>` has been changed to `{{ content }}`
- `<%%= ...  %>` has been changed to `{{ ... }}`
- `<% ... %>` has been changed to `{% ... %}`
- HAML layouts (if any) may need manual conversion
"#;
        
        write_readme(&dest_layouts_dir, layouts_readme)?;
        
        Ok(())
    }
    
    fn process_html_layout(&self, file_path: &Path, file_name: &str, dest_layouts_dir: &Path, 
                          result: &mut MigrationResult) -> Result<(), String> {
        // Read the layout file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
            
        // HTML layouts might contain Nanoc-specific code that needs conversion
        let converted_content = convert_nanoc_html_to_liquid(&content);
        
        // Determine the destination filename - strip the extension and add .html
        let base_name = file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        
        let dest_file_name = format!("{}.html", base_name);
        let dest_path = dest_layouts_dir.join(&dest_file_name);
        
        // Write the converted file
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write layout file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_layouts/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "HTML layout converted to Liquid format".to_string(),
        });
        
        Ok(())
    }
    
    fn process_erb_layout(&self, file_path: &Path, file_name: &str, dest_layouts_dir: &Path, 
                         result: &mut MigrationResult) -> Result<(), String> {
        // Read the layout file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
            
        // Convert ERB to Liquid
        let converted_content = convert_erb_to_liquid(&content);
        
        // Determine the destination filename - strip the extension and add .html
        let base_name = file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        
        let dest_file_name = format!("{}.html", base_name);
        let dest_path = dest_layouts_dir.join(&dest_file_name);
        
        // Write the converted file
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write layout file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_layouts/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "ERB layout converted to Liquid format".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_layouts(&self, dest_layouts_dir: &Path, 
                          result: &mut MigrationResult) -> Result<(), String> {
        // Default layout
        let default_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title | escape }} - {% endif %}{{ site.title | escape }}</title>
    <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
    <header class="site-header">
        <div class="container">
            <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title | escape }}</a>
            <nav class="site-nav">
                <ul>
                    <li><a href="{{ '/' | relative_url }}">Home</a></li>
                    <li><a href="{{ '/about/' | relative_url }}">About</a></li>
                    <li><a href="{{ '/blog/' | relative_url }}">Blog</a></li>
                </ul>
            </nav>
        </div>
    </header>
    
    <main class="site-content">
        <div class="container">
            {{ content }}
        </div>
    </main>
    
    <footer class="site-footer">
        <div class="container">
            <p>&copy; {{ site.time | date: '%Y' }} {{ site.title | escape }}</p>
        </div>
    </footer>
</body>
</html>"#;

        fs::write(dest_layouts_dir.join("default.html"), default_html)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic default layout created".to_string(),
        });
        
        // Post layout
        let post_html = r#"---
layout: default
---
<article class="post">
    <header class="post-header">
        <h1 class="post-title">{{ page.title | escape }}</h1>
        <div class="post-meta">
            <time datetime="{{ page.date | date_to_xmlschema }}">
                {{ page.date | date: "%b %-d, %Y" }}
            </time>
            {% if page.author %}
                • {{ page.author }}
            {% endif %}
        </div>
    </header>

    <div class="post-content">
        {{ content }}
    </div>
</article>"#;

        fs::write(dest_layouts_dir.join("post.html"), post_html)
            .map_err(|e| format!("Failed to write post layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/post.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic post layout created".to_string(),
        });
        
        // Page layout
        let page_html = r#"---
layout: default
---
<article class="page">
    <header class="page-header">
        <h1 class="page-title">{{ page.title | escape }}</h1>
    </header>

    <div class="page-content">
        {{ content }}
    </div>
</article>"#;

        fs::write(dest_layouts_dir.join("page.html"), page_html)
            .map_err(|e| format!("Failed to write page layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/page.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic page layout created".to_string(),
        });
        
        Ok(())
    }
}

// Helper functions

// Convert Nanoc HTML templates to Liquid
fn convert_nanoc_html_to_liquid(content: &str) -> String {
    // Replace "<%= yield %>" with "{{ content }}" for the main content
    let with_content = content.replace("<%= yield %>", "{{ content }}");
    
    // Convert any other Nanoc/ERB code
    convert_erb_to_liquid(&with_content)
}

// Convert ERB to Liquid
fn convert_erb_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert <%= expression %> to {{ expression }}
    let output_regex = Regex::new(r"<%=\s*(.*?)\s*%>").unwrap();
    result = output_regex.replace_all(&result, |caps: &regex::Captures| {
        let expr = caps.get(1).map_or("", |m| m.as_str());
        format!("{{ {} }}", expr)
    }).to_string();
    
    // Convert <% if condition %> to {% if condition %}
    let if_regex = Regex::new(r"<%\s*if\s+(.*?)\s*%>").unwrap();
    result = if_regex.replace_all(&result, |caps: &regex::Captures| {
        let condition = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% if {} %}}", condition)
    }).to_string();
    
    // Convert <% else %> to {% else %}
    result = result.replace("<% else %>", "{% else %}");
    
    // Convert <% end %> to {% endif %} or {% endfor %}
    // Since we can't easily tell which one it is, we'll use {% endif %}
    // and let the user adjust if needed
    result = result.replace("<% end %>", "{% endif %}");
    
    // Convert <% for item in items %> to {% for item in items %}
    let for_regex = Regex::new(r"<%\s*for\s+(.*?)\s*%>").unwrap();
    result = for_regex.replace_all(&result, |caps: &regex::Captures| {
        let loop_expr = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% for {} %}}", loop_expr)
    }).to_string();
    
    // Convert end for loop
    let end_for_regex = Regex::new(r"<%\s*end\s*%>").unwrap();
    result = end_for_regex.replace_all(&result, "{% endfor %}").to_string();
    
    result
} 