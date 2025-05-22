use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::HugoMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Hugo, layouts are in themes/theme-name/layouts or layouts/ at root
        let mut layouts_dirs = vec![
            source_dir.join("layouts"),
        ];
        
        // Check for theme layouts
        let themes_dir = source_dir.join("themes");
        if themes_dir.exists() && themes_dir.is_dir() {
            for theme_entry in fs::read_dir(&themes_dir)
                .map_err(|e| format!("Failed to read themes directory: {}", e))? {
                
                if let Ok(theme_entry) = theme_entry {
                    let theme_path = theme_entry.path();
                    if theme_path.is_dir() {
                        layouts_dirs.push(theme_path.join("layouts"));
                    }
                }
            }
        }
        
        let layouts_dest_dir = dest_dir.join("_layouts");
        create_dir_if_not_exists(&layouts_dest_dir)?;
        
        let includes_dest_dir = dest_dir.join("_includes");
        create_dir_if_not_exists(&includes_dest_dir)?;
        
        // Keep track of processed templates to avoid duplicates
        let mut processed_templates = HashSet::new();
        
        // Process layouts from each directory (theme or project)
        for layouts_dir in layouts_dirs {
            if layouts_dir.exists() && layouts_dir.is_dir() {
                if verbose {
                    log::info!("Migrating layouts from {}", layouts_dir.display());
                }
                
                // First process partials (they become includes)
                let partials_dir = layouts_dir.join("partials");
                if partials_dir.exists() && partials_dir.is_dir() {
                    self.process_partials(&partials_dir, &includes_dest_dir, verbose, result)?;
                }
                
                // Process base templates (_default/baseof.html)
                let default_dir = layouts_dir.join("_default");
                if default_dir.exists() && default_dir.is_dir() {
                    let baseof_path = default_dir.join("baseof.html");
                    if baseof_path.exists() {
                        self.process_layout_template(&baseof_path, "default.html", &layouts_dest_dir, &mut processed_templates, verbose, result)?;
                    }
                }
                
                // Process standard layouts
                let layout_patterns = vec![
                    ("_default/single.html", "post.html"),
                    ("_default/list.html", "archive.html"),
                    ("index.html", "home.html"),
                    ("posts/single.html", "post.html"),
                    ("post/single.html", "post.html"),
                    ("blog/single.html", "post.html"),
                    ("page/single.html", "page.html"),
                ];
                
                for (source_pattern, dest_name) in layout_patterns {
                    let source_path = layouts_dir.join(source_pattern);
                    if source_path.exists() && !processed_templates.contains(dest_name) {
                        self.process_layout_template(&source_path, dest_name, &layouts_dest_dir, &mut processed_templates, verbose, result)?;
                    }
                }
                
                // Process remaining templates
                for entry in WalkDir::new(&layouts_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        // Only process HTML files that aren't partials
                        if let Some(extension) = file_path.extension() {
                            if extension == "html" && !file_path.starts_with(&partials_dir) {
                                // Skip files already processed
                                let file_name = file_path.file_name()
                                    .ok_or_else(|| "Invalid file name".to_string())?
                                    .to_string_lossy()
                                    .to_string();
                                
                                // Skip if already processed
                                if processed_templates.contains(&file_name) {
                                    continue;
                                }
                                
                                // Determine template type from path
                                let rel_path = file_path.strip_prefix(&layouts_dir)
                                    .map_err(|_| "Failed to get relative path".to_string())?;
                                
                                // Is this a section template?
                                let is_section = rel_path.to_string_lossy().contains("section.html");
                                let is_single = rel_path.to_string_lossy().contains("single.html");
                                let is_list = rel_path.to_string_lossy().contains("list.html");
                                
                                let dest_name = if is_section || is_list {
                                    "archive.html".to_string()
                                } else if is_single {
                                    "post.html".to_string()
                                } else {
                                    // Use the original name
                                    file_name
                                };
                                
                                if !processed_templates.contains(&dest_name) {
                                    self.process_layout_template(file_path, &dest_name, &layouts_dest_dir, &mut processed_templates, verbose, result)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create some basic templates if none were found
        if processed_templates.is_empty() {
            self.create_basic_templates(&layouts_dest_dir, &includes_dest_dir, result)?;
        }
        
        // Create README for layouts directory
        let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Hugo.

## Layout Files

- `default.html` - The base template (from baseof.html)
- `post.html` - Template for blog posts
- `page.html` - Template for regular pages
- `home.html` - Template for the home page
- `archive.html` - Template for list/section pages

## Changes from Hugo

- Hugo templates using Go templating have been converted to Liquid
- `{{ block "main" . }}` calls have been changed to `{{ content }}`
- Partials are now includes and live in the _includes directory
- Some Hugo functions may need manual adaptation
"#;
        
        write_readme(&layouts_dest_dir, layouts_readme)?;
        
        result.warnings.push(
            "Hugo layouts use Go templates which have been partially converted to Liquid. Some manual adjustments may be needed for complex templates.".to_string()
        );
        
        Ok(())
    }
    
    fn process_partials(&self, partials_dir: &Path, includes_dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        if verbose {
            log::info!("Processing Hugo partials from {}", partials_dir.display());
        }
        
        for entry in WalkDir::new(partials_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                if let Some(extension) = file_path.extension() {
                    if extension == "html" {
                        let file_name = file_path.file_name()
                            .ok_or_else(|| "Invalid file name".to_string())?
                            .to_string_lossy()
                            .to_string();
                            
                        let dest_path = includes_dest_dir.join(&file_name);
                        
                        // Read the template
                        let content = fs::read_to_string(file_path)
                            .map_err(|e| format!("Failed to read partial template {}: {}", file_path.display(), e))?;
                        
                        // Convert Go template to Liquid
                        let converted_content = self.convert_hugo_template(&content);
                        
                        // Write the converted template
                        fs::write(&dest_path, converted_content)
                            .map_err(|e| format!("Failed to write converted include: {}", e))?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_includes/{}", file_name),
                            change_type: ChangeType::Converted,
                            description: "Partial template converted from Hugo to Liquid".to_string(),
                        });
                    }
                }
            }
        }
        
        // Create README for includes
        let includes_readme = r#"# Includes Directory

This directory contains partial templates migrated from Hugo.

## Usage

In Rustyll, partials are included using:

```liquid
{% include "filename.html" %}
```

Instead of Hugo's:

```go
{{ partial "filename.html" . }}
```

## Changes from Hugo

- Hugo partials have been converted to Liquid includes
- Context handling is different - parameters need to be passed explicitly
- Some Hugo functions may need manual adaptation
"#;
        
        write_readme(includes_dest_dir, includes_readme)?;
        
        Ok(())
    }
    
    fn process_layout_template(&self, source_path: &Path, dest_name: &str, layouts_dest_dir: &Path, 
                              processed_templates: &mut HashSet<String>, verbose: bool, 
                              result: &mut MigrationResult) -> Result<(), String> {
        if verbose {
            log::info!("Converting Hugo template {} to {}", source_path.display(), dest_name);
        }
        
        // Read the template
        let content = fs::read_to_string(source_path)
            .map_err(|e| format!("Failed to read template {}: {}", source_path.display(), e))?;
        
        // Convert Go template to Liquid
        let converted_content = self.convert_hugo_template(&content);
        
        // Write the converted template
        let dest_path = layouts_dest_dir.join(dest_name);
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write converted layout: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_layouts/{}", dest_name),
            change_type: ChangeType::Converted,
            description: "Layout template converted from Hugo to Liquid".to_string(),
        });
        
        // Mark as processed
        processed_templates.insert(dest_name.to_string());
        
        Ok(())
    }
    
    fn create_basic_templates(&self, layouts_dest_dir: &Path, includes_dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Default template
        let default_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title | escape }} - {% endif %}{{ site.title | escape }}</title>
    <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
    {% include head.html %}
</head>
<body>
    <header class="site-header">
        {% include header.html %}
    </header>
    
    <main class="site-content">
        {{ content }}
    </main>
    
    <footer class="site-footer">
        {% include footer.html %}
    </footer>
</body>
</html>"#;
        
        fs::write(layouts_dest_dir.join("default.html"), default_html)
            .map_err(|e| format!("Failed to write default template: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic default layout created".to_string(),
        });
        
        // Post template
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
            
            {% if page.categories.size > 0 %}
            <div class="post-categories">
                Categories: 
                {% for category in page.categories %}
                <span class="post-category">{{ category }}</span>
                {% endfor %}
            </div>
            {% endif %}
            
            {% if page.tags.size > 0 %}
            <div class="post-tags">
                Tags: 
                {% for tag in page.tags %}
                <span class="post-tag">{{ tag }}</span>
                {% endfor %}
            </div>
            {% endif %}
        </div>
    </header>

    <div class="post-content">
        {{ content }}
    </div>
</article>"#;
        
        fs::write(layouts_dest_dir.join("post.html"), post_html)
            .map_err(|e| format!("Failed to write post template: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/post.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic post layout created".to_string(),
        });
        
        // Page template
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
        
        fs::write(layouts_dest_dir.join("page.html"), page_html)
            .map_err(|e| format!("Failed to write page template: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/page.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic page layout created".to_string(),
        });
        
        // Home template
        let home_html = r#"---
layout: default
---
<div class="home">
    <h1 class="page-heading">{{ site.title }}</h1>
    
    {{ content }}
    
    <h2 class="post-list-heading">Latest Posts</h2>
    <ul class="post-list">
        {% for post in site.posts limit:5 %}
        <li>
            <span class="post-meta">{{ post.date | date: "%b %-d, %Y" }}</span>
            <h3>
                <a class="post-link" href="{{ post.url | relative_url }}">
                    {{ post.title | escape }}
                </a>
            </h3>
            {% if post.excerpt %}
            <div class="post-excerpt">
                {{ post.excerpt }}
                <a href="{{ post.url | relative_url }}">Read more</a>
            </div>
            {% endif %}
        </li>
        {% endfor %}
    </ul>
    
    <p class="rss-subscribe">Subscribe <a href="{{ "/feed.xml" | relative_url }}">via RSS</a></p>
</div>"#;
        
        fs::write(layouts_dest_dir.join("home.html"), home_html)
            .map_err(|e| format!("Failed to write home template: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/home.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic home layout created".to_string(),
        });
        
        // Archive template
        let archive_html = r#"---
layout: default
---
<div class="archive">
    <h1 class="page-heading">{{ page.title | default: "Archive" }}</h1>
    
    {{ content }}
    
    <ul class="post-list">
        {% for post in site.posts %}
        <li>
            <span class="post-meta">{{ post.date | date: "%b %-d, %Y" }}</span>
            <h3>
                <a class="post-link" href="{{ post.url | relative_url }}">
                    {{ post.title | escape }}
                </a>
            </h3>
        </li>
        {% endfor %}
    </ul>
</div>"#;
        
        fs::write(layouts_dest_dir.join("archive.html"), archive_html)
            .map_err(|e| format!("Failed to write archive template: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_layouts/archive.html".to_string(),
            change_type: ChangeType::Created,
            description: "Basic archive layout created".to_string(),
        });
        
        // Create basic includes
        let includes = vec![
            ("header.html", r#"<div class="container">
    <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title | escape }}</a>
    <nav class="site-nav">
        <ul>
            <li><a href="{{ '/' | relative_url }}">Home</a></li>
            <li><a href="{{ '/about/' | relative_url }}">About</a></li>
            <li><a href="{{ '/posts/' | relative_url }}">Blog</a></li>
        </ul>
    </nav>
</div>"#),
            ("footer.html", r#"<div class="container">
    <div class="footer-col-wrapper">
        <div class="footer-col">
            <p>{{ site.description | escape }}</p>
        </div>
    </div>
    <div class="copyright">
        &copy; {{ site.time | date: '%Y' }} {{ site.title | escape }}
    </div>
</div>"#),
            ("head.html", r#"<!-- Additional head elements -->
<link rel="canonical" href="{{ page.url | replace:'index.html','' | absolute_url }}">
<link rel="alternate" type="application/rss+xml" title="{{ site.title | escape }}" href="{{ "/feed.xml" | relative_url }}">"#)
        ];
        
        for (name, content) in includes {
            fs::write(includes_dest_dir.join(name), content)
                .map_err(|e| format!("Failed to write include {}: {}", name, e))?;
            
            result.changes.push(MigrationChange {
                file_path: format!("_includes/{}", name),
                change_type: ChangeType::Created,
                description: format!("Basic {} include created", name).to_string(),
            });
        }
        
        Ok(())
    }
    
    pub(super) fn convert_hugo_template(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Convert Go template syntax to Liquid
        
        // 1. Handle basic variable substitution
        // {{ .Title }} -> {{ page.title }}
        let var_regex = Regex::new(r"\{\{\s*\.([A-Za-z0-9_]+)\s*\}\}").unwrap();
        result = var_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = caps.get(1).map_or("", |m| m.as_str());
            let liquid_var = match var_name.to_lowercase().as_str() {
                "title" => "page.title",
                "content" => "content",
                "date" => "page.date",
                "description" => "page.description",
                "summary" => "page.excerpt",
                "permalink" => "page.url",
                "sitename" => "site.title",
                "sitedescription" => "site.description",
                _ => {
                    // Default to page variable
                    &format!("page.{}", var_name.to_lowercase())[..]
                }
            };
            
            format!("{{ {} }}", liquid_var)
        }).to_string();
        
        // 2. Handle site variables
        // {{ .Site.Title }} -> {{ site.title }}
        let site_var_regex = Regex::new(r"\{\{\s*\.Site\.([A-Za-z0-9_]+)\s*\}\}").unwrap();
        result = site_var_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = caps.get(1).map_or("", |m| m.as_str());
            format!("{{ site.{} }}", var_name.to_lowercase())
        }).to_string();
        
        // 3. Handle partials
        // {{ partial "header.html" . }} -> {% include header.html %}
        let partial_regex = Regex::new(r#"\{\{\s*partial\s+"([^"]+)"\s+\.\s*\}\}"#).unwrap();
        result = partial_regex.replace_all(&result, |caps: &regex::Captures| {
            let partial_name = caps.get(1).map_or("", |m| m.as_str());
            format!("{{% include \"{}\" %}}", partial_name)
        }).to_string();
        
        // 4. Handle if statements
        // {{ if .Title }} -> {% if page.title %}
        let if_regex = Regex::new(r"\{\{\s*if\s+\.([A-Za-z0-9_]+)\s*\}\}").unwrap();
        result = if_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = caps.get(1).map_or("", |m| m.as_str());
            format!("{{% if page.{} %}}", var_name.to_lowercase())
        }).to_string();
        
        // {{ end }} -> {% endif %}
        result = result.replace("{{ end }}", "{% endif %}");
        
        // 5. Handle range loops
        // {{ range .Pages }} -> {% for page in site.pages %}
        let range_regex = Regex::new(r"\{\{\s*range\s+\.([A-Za-z0-9_]+)\s*\}\}").unwrap();
        result = range_regex.replace_all(&result, |caps: &regex::Captures| {
            let collection = caps.get(1).map_or("", |m| m.as_str());
            let liquid_collection = match collection.to_lowercase().as_str() {
                "pages" => "site.pages",
                "posts" => "site.posts",
                "site.pages" => "site.pages",
                "site.posts" => "site.posts",
                _ => {
                    // Default to site variable
                    &format!("site.{}", collection.to_lowercase())[..]
                }
            };
            
            format!("{{% for item in {} %}}", liquid_collection)
        }).to_string();
        
        // {{ end }} -> {% endfor %}
        result = result.replace("{{ end }}", "{% endfor %}");
        
        // 6. Handle block statements
        // {{ block "main" . }} -> {{ content }}
        let re_main_block = Regex::new(r#"\{\{\s*block\s+"main"\s+\.\s*\}\}(.*?)\{\{\s*end\s*\}\}"#).unwrap();
        result = re_main_block.replace_all(&result, "{{ content }}").to_string();
        
        // 7. Handle with statements
        // {{ with .Params.author }} -> {% if page.author %}
        let with_regex = Regex::new(r"\{\{\s*with\s+\.([A-Za-z0-9_.]+)\s*\}\}").unwrap();
        result = with_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_path = caps.get(1).map_or("", |m| m.as_str());
            let liquid_var = if var_path.starts_with("Params.") {
                format!("page.{}", &var_path[7..].to_lowercase())
            } else {
                format!("page.{}", var_path.to_lowercase())
            };
            
            format!("{{% if {} %}}", liquid_var)
        }).to_string();
        
        // 8. Handle define statements (become capture in Liquid)
        // {{ define "title" }} -> {% capture title %}
        let define_regex = Regex::new(r#"\{\{\s*define\s+"([^"]+)"\s*\}\}"#).unwrap();
        result = define_regex.replace_all(&result, |caps: &regex::Captures| {
            let block_name = caps.get(1).map_or("", |m| m.as_str());
            format!("{{% capture {} %}}", block_name)
        }).to_string();
        
        // {{ end }} -> {% endcapture %}
        result = result.replace("{{ end }}", "{% endcapture %}");
        
        // Return the converted template
        result
    }
} 