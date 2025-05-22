use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::AssembleMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, layouts can be in various locations:
        // - layouts/ directory
        // - templates/layouts/ directory
        // - src/layouts/ directory
        
        let layout_dirs = vec![
            source_dir.join("layouts"),
            source_dir.join("templates").join("layouts"),
            source_dir.join("src").join("layouts"),
        ];
        
        // Create destination directory
        let dest_layouts_dir = dest_dir.join("_layouts");
        create_dir_if_not_exists(&dest_layouts_dir)?;
        
        // Create destination includes directory
        let dest_includes_dir = dest_dir.join("_includes");
        create_dir_if_not_exists(&dest_includes_dir)?;
        
        let mut found_layouts = false;
        
        // Process layouts from potential locations
        for layout_dir in layout_dirs {
            if layout_dir.exists() && layout_dir.is_dir() {
                if verbose {
                    log::info!("Migrating layouts from {}", layout_dir.display());
                }
                
                self.process_layout_directory(&layout_dir, &dest_layouts_dir, result)?;
                found_layouts = true;
            }
        }
        
        // Look for partials (to convert to includes)
        self.migrate_partials(source_dir, &dest_includes_dir, verbose, result)?;
        
        // Create basic layouts if none were found
        if !found_layouts {
            if verbose {
                log::info!("No layouts found. Creating basic layout templates.");
            }
            
            self.create_basic_layouts(&dest_layouts_dir, &dest_includes_dir, result)?;
        }
        
        // Create README for layouts directory
        let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Assemble.

## Layout Format

Layouts in Rustyll:
- Use Liquid templating instead of Handlebars
- Files are typically HTML with Liquid templating code
- Layouts can inherit from other layouts using the `layout` front matter variable

## Conversion Details

- Assemble Handlebars templates have been converted to Liquid syntax
- `{{> partial}}` has been changed to `{% include "partial.html" %}`
- `{{#if condition}}` has been changed to `{% if condition %}`
- `{{#each items}}` has been changed to `{% for item in items %}`
- `{{variable}}` has been changed to `{{ variable }}`
"#;
        
        write_readme(&dest_layouts_dir, layouts_readme)?;
        
        Ok(())
    }
    
    fn process_layout_directory(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process layout files based on extension
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "hbs" || ext == "handlebars" || ext == "html" {
                        self.convert_layout_file(file_path, dest_dir, result)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn convert_layout_file(&self, file_path: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
        
        // Convert Handlebars syntax to Liquid
        let converted_content = convert_handlebars_layout(&content);
        
        // Determine the destination filename - always use .html extension
        let file_stem = file_path.file_stem().unwrap().to_string_lossy();
        let dest_filename = format!("{}.html", file_stem);
        let dest_path = dest_dir.join(&dest_filename);
        
        // Write the converted file
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write layout file: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_layouts/{}", dest_filename),
            change_type: ChangeType::Converted,
            description: "Layout template converted from Assemble Handlebars to Liquid".to_string(),
        });
        
        Ok(())
    }
    
    fn migrate_partials(&self, source_dir: &Path, dest_includes_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, partials can be in various locations
        let partials_dirs = vec![
            source_dir.join("partials"),
            source_dir.join("templates").join("partials"),
            source_dir.join("src").join("partials"),
            source_dir.join("includes"),
        ];
        
        let mut found_partials = false;
        
        for partials_dir in partials_dirs {
            if partials_dir.exists() && partials_dir.is_dir() {
                if verbose {
                    log::info!("Migrating partials from {}", partials_dir.display());
                }
                
                self.process_partials_directory(&partials_dir, dest_includes_dir, result)?;
                found_partials = true;
            }
        }
        
        if !found_partials {
            if verbose {
                log::info!("No partials found. Creating basic include templates.");
            }
            
            // Create basic includes
            self.create_basic_includes(dest_includes_dir, result)?;
        }
        
        // Create README for includes directory
        let includes_readme = r#"# Includes Directory

This directory contains reusable template fragments that can be included in layouts and pages.

## Includes Format

Includes in Rustyll:
- Are referenced using `{% include "filename.html" %}`
- Can accept parameters: `{% include "product.html" with product=item %}`
- Are converted from Assemble partials

## Conversion Details

- Assemble partials (`{{> partial}}`) have been converted to Liquid includes
- Partials with context (`{{> partial context}}`) need manual adjustment
"#;
        
        write_readme(dest_includes_dir, includes_readme)?;
        
        Ok(())
    }
    
    fn process_partials_directory(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process partial files based on extension
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "hbs" || ext == "handlebars" || ext == "html" {
                        self.convert_partial_file(file_path, dest_dir, result)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn convert_partial_file(&self, file_path: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read partial file {}: {}", file_path.display(), e))?;
        
        // Convert Handlebars syntax to Liquid
        let converted_content = convert_handlebars_partial(&content);
        
        // Determine the destination filename - always use .html extension
        let file_stem = file_path.file_stem().unwrap().to_string_lossy();
        let dest_filename = format!("{}.html", file_stem);
        let dest_path = dest_dir.join(&dest_filename);
        
        // Write the converted file
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write include file: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_includes/{}", dest_filename),
            change_type: ChangeType::Converted,
            description: "Partial template converted to Liquid include".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_layouts(&self, layouts_dir: &Path, includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create default layout
        let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title | escape }} - {% endif %}{{ site.title | escape }}</title>
    <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
    {% include header.html %}
    
    <main class="content">
        <div class="container">
            {{ content }}
        </div>
    </main>
    
    {% include footer.html %}
    
    <script src="{{ '/assets/js/main.js' | relative_url }}"></script>
</body>
</html>"#;
        
        fs::write(layouts_dir.join("default.html"), default_layout)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic default layout created".to_string(),
        });
        
        // Create home layout
        let home_layout = r#"---
layout: default
---
<div class="home">
    <h1 class="page-heading">{{ page.title }}</h1>
    
    {{ content }}
    
    <div class="post-list">
        <h2>Recent Posts</h2>
        
        {% for post in site.posts limit:5 %}
        <div class="post-item">
            <h3>
                <a href="{{ post.url | relative_url }}">{{ post.title | escape }}</a>
            </h3>
            <p class="post-meta">{{ post.date | date: "%b %-d, %Y" }}</p>
            <p>{{ post.excerpt }}</p>
            <p><a href="{{ post.url | relative_url }}">Read more...</a></p>
        </div>
        {% endfor %}
    </div>
</div>"#;
        
        fs::write(layouts_dir.join("home.html"), home_layout)
            .map_err(|e| format!("Failed to write home layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/home.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic home layout created".to_string(),
        });
        
        // Create post layout
        let post_layout = r#"---
layout: default
---
<article class="post">
    <header class="post-header">
        <h1 class="post-title">{{ page.title | escape }}</h1>
        <p class="post-meta">
            <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%b %-d, %Y" }}</time>
            {% if page.author %}
                • <span>{{ page.author }}</span>
            {% endif %}
        </p>
        
        {% if page.categories.size > 0 or page.tags.size > 0 %}
        <div class="post-taxonomy">
            {% if page.categories.size > 0 %}
            <div class="post-categories">
                <strong>Categories:</strong>
                {% for category in page.categories %}
                <span class="post-category">{{ category }}</span>
                {% endfor %}
            </div>
            {% endif %}
            
            {% if page.tags.size > 0 %}
            <div class="post-tags">
                <strong>Tags:</strong>
                {% for tag in page.tags %}
                <span class="post-tag">{{ tag }}</span>
                {% endfor %}
            </div>
            {% endif %}
        </div>
        {% endif %}
    </header>

    <div class="post-content">
        {{ content }}
    </div>
</article>"#;
        
        fs::write(layouts_dir.join("post.html"), post_layout)
            .map_err(|e| format!("Failed to write post layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/post.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic post layout created".to_string(),
        });
        
        // Create page layout
        let page_layout = r#"---
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
        
        fs::write(layouts_dir.join("page.html"), page_layout)
            .map_err(|e| format!("Failed to write page layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/page.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic page layout created".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_includes(&self, includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create header include
        let header_include = r#"<header class="site-header">
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
</header>"#;
        
        fs::write(includes_dir.join("header.html"), header_include)
            .map_err(|e| format!("Failed to write header include: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_includes/header.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic header include created".to_string(),
        });
        
        // Create footer include
        let footer_include = r#"<footer class="site-footer">
    <div class="container">
        <p>&copy; {{ site.time | date: '%Y' }} {{ site.title | escape }}</p>
    </div>
</footer>"#;
        
        fs::write(includes_dir.join("footer.html"), footer_include)
            .map_err(|e| format!("Failed to write footer include: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_includes/footer.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic footer include created".to_string(),
        });
        
        Ok(())
    }
}

// Helper functions

// Convert Handlebars layout templates to Liquid
fn convert_handlebars_layout(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert basic Handlebars syntax
    result = convert_basic_handlebars(&result);
    
    // Handle body content (Handlebars typically uses {{body}} for main content)
    result = result.replace("{{body}}", "{{ content }}");
    
    // Convert nested layouts
    // {{#extend "layout"}} -> inheritance is handled in YAML front matter
    let extend_regex = Regex::new(r#"\{\{#extend\s+"([^"]+)"\s*\}\}"#).unwrap();
    result = extend_regex.replace_all(&result, |caps: &regex::Captures| {
        let layout_name = caps.get(1).map_or("", |m| m.as_str());
        format!("---\nlayout: {}\n---\n", layout_name)
    }).to_string();
    
    // Also handle single quotes
    let extend_regex_sq = Regex::new(r#"\{\{#extend\s+'([^']+)'\s*\}\}"#).unwrap();
    result = extend_regex_sq.replace_all(&result, |caps: &regex::Captures| {
        let layout_name = caps.get(1).map_or("", |m| m.as_str());
        format!("---\nlayout: {}\n---\n", layout_name)
    }).to_string();
    
    // Remove {{/extend}}
    result = result.replace("{{/extend}}", "");
    
    // Handle block statements
    // {{#block "blockName"}} -> {% block blockName %}
    let block_regex = Regex::new(r#"\{\{#block\s+"([^"]+)"\s*\}\}"#).unwrap();
    result = block_regex.replace_all(&result, |caps: &regex::Captures| {
        let block_name = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% block {} %}}", block_name)
    }).to_string();
    
    // Also handle single quotes
    let block_regex_sq = Regex::new(r#"\{\{#block\s+'([^']+)'\s*\}\}"#).unwrap();
    result = block_regex_sq.replace_all(&result, |caps: &regex::Captures| {
        let block_name = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% block {} %}}", block_name)
    }).to_string();
    
    // {{/block}} -> {% endblock %}
    result = result.replace("{{/block}}", "{% endblock %}");
    
    // Handle content blocks
    // {{#content "blockName"}} -> {% capture blockName %}
    let content_regex = Regex::new(r#"\{\{#content\s+"([^"]+)"\s*\}\}"#).unwrap();
    result = content_regex.replace_all(&result, |caps: &regex::Captures| {
        let block_name = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% capture {} %}}", block_name)
    }).to_string();
    
    // Also handle single quotes
    let content_regex_sq = Regex::new(r#"\{\{#content\s+'([^']+)'\s*\}\}"#).unwrap();
    result = content_regex_sq.replace_all(&result, |caps: &regex::Captures| {
        let block_name = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% capture {} %}}", block_name)
    }).to_string();
    
    // {{/content}} -> {% endcapture %}
    result = result.replace("{{/content}}", "{% endcapture %}");
    
    result
}

// Convert Handlebars partial templates to Liquid
fn convert_handlebars_partial(content: &str) -> String {
    convert_basic_handlebars(&content)
}

// Common conversions for Handlebars to Liquid
fn convert_basic_handlebars(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert variables
    // {{variable}} -> {{ variable }}
    let var_regex = Regex::new(r"\{\{\s*([^#\^/><].*?)\s*\}\}").unwrap();
    result = var_regex.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map_or("", |m| m.as_str());
        
        // Skip if it's a Handlebars helper that we'll convert separately
        if var_name.contains(" ") && !var_name.starts_with("this.") && !var_name.contains(".") {
            format!("{{{{{}}}}}", var_name) // Keep as is, will be processed by helper conversions
        } else {
            format!("{{ {} }}", var_name)
        }
    }).to_string();
    
    // Convert partials
    // {{> partial}} -> {% include "partial.html" %}
    let partial_regex = Regex::new(r"\{\{\s*>\s*([^\s}]+)(?:\s+([^}]*))?\s*\}\}").unwrap();
    result = partial_regex.replace_all(&result, |caps: &regex::Captures| {
        let partial_name = caps.get(1).map_or("", |m| m.as_str());
        
        // Handle extensions - add .html if missing
        let include_name = if partial_name.contains('.') {
            partial_name.to_string()
        } else {
            format!("{}.html", partial_name)
        };
        
        // Check if there's context passed to the partial
        if let Some(context) = caps.get(2) {
            // Add a warning comment for partial with context
            format!(r#"<!-- TODO: Handlebars partial with context needs manual adjustment -->
{{% include "{}" %}}"#, include_name)
        } else {
            format!("{{% include \"{}\" %}}", include_name)
        }
    }).to_string();
    
    // Convert if statements
    // {{#if condition}} -> {% if condition %}
    let if_regex = Regex::new(r"\{\{#if\s+(.*?)\s*\}\}").unwrap();
    result = if_regex.replace_all(&result, |caps: &regex::Captures| {
        let condition = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% if {} %}}", condition)
    }).to_string();
    
    // {{else}} -> {% else %}
    result = result.replace("{{else}}", "{% else %}");
    
    // {{/if}} -> {% endif %}
    result = result.replace("{{/if}}", "{% endif %}");
    
    // Convert unless statements
    // {{#unless condition}} -> {% unless condition %}
    let unless_regex = Regex::new(r"\{\{#unless\s+(.*?)\s*\}\}").unwrap();
    result = unless_regex.replace_all(&result, |caps: &regex::Captures| {
        let condition = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% unless {} %}}", condition)
    }).to_string();
    
    // {{/unless}} -> {% endunless %}
    result = result.replace("{{/unless}}", "{% endunless %}");
    
    // Convert each loops
    // {{#each items}} -> {% for item in items %}
    let each_regex = Regex::new(r"\{\{#each\s+(.*?)\s*\}\}").unwrap();
    result = each_regex.replace_all(&result, |caps: &regex::Captures| {
        let collection = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% for item in {} %}}", collection)
    }).to_string();
    
    // {{this}} in each loop -> {{ item }}
    result = result.replace("{{this}}", "{{ item }}");
    result = result.replace("{{this.", "{{ item.");
    
    // {{/each}} -> {% endfor %}
    result = result.replace("{{/each}}", "{% endfor %}");
    
    // Mark more complex handlebars helpers for manual review
    let helper_regex = Regex::new(r"\{\{\s*([a-zA-Z0-9_-]+)\s+([^}]*)\}\}").unwrap();
    result = helper_regex.replace_all(&result, |caps: &regex::Captures| {
        let helper_name = caps.get(1).map_or("", |m| m.as_str());
        let helper_args = caps.get(2).map_or("", |m| m.as_str());
        
        format!(r#"<!-- TODO: Handlebars helper needs manual conversion - Original: {{{{ {} {} }}}} -->"#, 
               helper_name, helper_args)
    }).to_string();
    
    result
} 