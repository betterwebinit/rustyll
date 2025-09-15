use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::AssembleMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, content is typically in:
        // - pages/ directory
        // - content/ directory
        // - posts/ or blog/ directory for blog posts
        
        let page_dirs = vec![
            source_dir.join("pages"),
            source_dir.join("content"),
        ];
        
        let post_dirs = vec![
            source_dir.join("posts"),
            source_dir.join("blog"),
            source_dir.join("_posts"),
        ];
        
        // Create destination directories
        let dest_pages_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        let dest_posts_dir = dest_dir.join("_posts");
        create_dir_if_not_exists(&dest_posts_dir)?;
        
        let mut found_content = false;
        
        // Process page content
        for page_dir in page_dirs {
            if page_dir.exists() && page_dir.is_dir() {
                if verbose {
                    log::info!("Migrating content from {}", page_dir.display());
                }
                
                self.migrate_pages(&page_dir, &dest_pages_dir, result)?;
                found_content = true;
            }
        }
        
        // Process blog posts
        for post_dir in post_dirs {
            if post_dir.exists() && post_dir.is_dir() {
                if verbose {
                    log::info!("Migrating blog posts from {}", post_dir.display());
                }
                
                self.migrate_posts(&post_dir, &dest_posts_dir, result)?;
                found_content = true;
            }
        }
        
        if !found_content {
            result.warnings.push("No content directories found. Creating sample content.".to_string());
            self.create_sample_content(dest_dir, &dest_pages_dir, &dest_posts_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_pages(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Only process Markdown and HTML files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "hbs" || ext == "handlebars" {
                        self.process_page_file(file_path, source_dir, dest_dir, result)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn process_page_file(&self, file_path: &Path, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read page file {}: {}", file_path.display(), e))?;
        
        // Extract front matter and content
        let (front_matter, content_text) = extract_front_matter(&content);
        
        // Get the relative path from source directory
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Determine destination path
        let page_name = if rel_path.components().count() > 1 {
            // Files in subdirectories get renamed with hyphens
            let path_str = rel_path.with_extension("").to_string_lossy().to_string();
            format!("{}.{}", path_str.replace("/", "-"), "md")
        } else {
            // Root-level files keep their names
            format!("{}.{}", file_path.file_stem().unwrap().to_string_lossy(), "md")
        };
        
        let dest_path = dest_dir.join(&page_name);
        
        // Create the front matter for Rustyll
        let mut new_front_matter = String::from("---\n");
        
        // Extract title from front matter or use filename
        let title = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                file_path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
        
        new_front_matter.push_str(&format!("title: \"{}\"\n", title));
        new_front_matter.push_str("layout: page\n");
        
        // Generate a permalink based on the file path
        let permalink = if rel_path.components().count() > 1 {
            let path_str = rel_path.with_extension("").to_string_lossy().to_string();
            format!("/{}/", path_str)
        } else {
            let stem = file_path.file_stem().unwrap().to_string_lossy();
            if stem == "index" {
                "/".to_string()
            } else {
                format!("/{}/", stem)
            }
        };
        
        new_front_matter.push_str(&format!("permalink: {}\n", permalink));
        
        // Add any other front matter
        for (key, value) in &front_matter {
            if key.to_lowercase() != "title" && key.to_lowercase() != "layout" && key.to_lowercase() != "permalink" {
                new_front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        new_front_matter.push_str("---\n\n");
        
        // Convert Handlebars syntax to Liquid if needed
        let converted_content = convert_handlebars_to_liquid(content_text);
        
        // Write the converted file
        let final_content = format!("{}{}", new_front_matter, converted_content);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write page file: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_pages/{}", page_name),
            change_type: ChangeType::Converted,
            description: "Page converted from Assemble format".to_string(),
        });
        
        Ok(())
    }
    
    fn migrate_posts(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Only process Markdown and HTML files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "hbs" || ext == "handlebars" {
                        self.process_post_file(file_path, source_dir, dest_dir, result)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn process_post_file(&self, file_path: &Path, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read post file {}: {}", file_path.display(), e))?;
        
        // Extract front matter and content
        let (front_matter, content_text) = extract_front_matter(&content);
        
        // Extract title from front matter or use filename
        let title = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                file_path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
        
        // Try to extract date from front matter or filename
        let date = extract_date_from_post(&file_path, &front_matter);
        
        // Create Jekyll-style post filename with date prefix
        let post_name = format!("{}-{}.md", 
                              date.replace(" ", "-").split('T').next().unwrap_or(&date),
                              file_path.file_stem().unwrap().to_string_lossy());
        
        let dest_path = dest_dir.join(&post_name);
        
        // Create the front matter for Rustyll
        let mut new_front_matter = String::from("---\n");
        new_front_matter.push_str(&format!("title: \"{}\"\n", title));
        new_front_matter.push_str(&format!("date: {}\n", date));
        new_front_matter.push_str("layout: post\n");
        
        // Add any other front matter
        for (key, value) in &front_matter {
            if key.to_lowercase() != "title" && key.to_lowercase() != "date" && key.to_lowercase() != "layout" {
                new_front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        new_front_matter.push_str("---\n\n");
        
        // Convert Handlebars syntax to Liquid if needed
        let converted_content = convert_handlebars_to_liquid(content_text);
        
        // Write the converted file
        let final_content = format!("{}{}", new_front_matter, converted_content);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write post file: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", post_name),
            change_type: ChangeType::Converted,
            description: "Blog post converted from Assemble format".to_string(),
        });
        
        Ok(())
    }
    
    fn create_sample_content(&self, dest_dir: &Path, pages_dir: &Path, posts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a sample index page
        let index_content = r#"---
title: "Home"
layout: home
permalink: /
---

# Welcome to Your Migrated Site

This is a sample home page created during migration from Assemble to Rustyll.

## Recent Posts

{% for post in site.posts limit:5 %}
* [{{ post.title }}]({{ post.url | relative_url }}) - {{ post.date | date: "%b %-d, %Y" }}
{% endfor %}

## About This Site

This site was migrated from Assemble to Rustyll. Edit this page in `_pages/index.md` to customize it.
"#;
        
        fs::write(pages_dir.join("index.md"), index_content)
            .map_err(|e| format!("Failed to write sample index page: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_pages/index.md".to_string(),
            change_type: ChangeType::Created,
            description: "Sample index page created".to_string(),
        });
        
        // Create a sample about page
        let about_content = r#"---
title: "About"
layout: page
permalink: /about/
---

# About This Site

This is a sample About page created during migration from Assemble to Rustyll.

## Migration Details

This site was automatically migrated from an Assemble site to Rustyll format.

## Contact

You can edit this page in `_pages/about.md` to add your contact information.
"#;
        
        fs::write(pages_dir.join("about.md"), about_content)
            .map_err(|e| format!("Failed to write sample about page: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_pages/about.md".to_string(),
            change_type: ChangeType::Created,
            description: "Sample about page created".to_string(),
        });
        
        // Create a sample blog post
        let date = chrono::Local::now().format("%Y-%m-%d");
        let post_content = format!(r#"---
title: "Welcome to Rustyll"
date: {}
layout: post
categories: [welcome, migration]
tags: [rustyll, assemble, migration]
---

# Welcome to Rustyll!

This is a sample blog post created during the migration from Assemble to Rustyll.

## Migration Complete

Your site has been migrated to Rustyll format. You can now:

1. Customize your layouts in the `_layouts` directory
2. Add more blog posts to the `_posts` directory
3. Edit pages in the `_pages` directory
4. Configure your site in `_config.yml`

## Next Steps

Edit this post or create new posts in the `_posts` directory with the format `YYYY-MM-DD-title.md`.
"#, date);
        
        let post_filename = format!("{}-welcome-to-rustyll.md", date);
        fs::write(posts_dir.join(&post_filename), post_content)
            .map_err(|e| format!("Failed to write sample blog post: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", post_filename),
            change_type: ChangeType::Created,
            description: "Sample blog post created".to_string(),
        });
        
        Ok(())
    }
}

// Helper functions

// Extract YAML front matter from content
fn extract_front_matter(content: &str) -> (Vec<(String, String)>, &str) {
    // Assemble typically uses YAML front matter between --- delimiters
    let mut front_matter = Vec::new();
    let content_text: &str;
    
    if content.starts_with("---") {
        if let Some(end_index) = content[3..].find("---") {
            let yaml_part = &content[3..(end_index + 3)];
            content_text = &content[(end_index + 6)..];
            
            // Simple line-by-line parsing of YAML front matter
            for line in yaml_part.lines() {
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_string();
                    let value = line[(colon_pos + 1)..].trim().to_string();
                    if !key.is_empty() {
                        front_matter.push((key, value));
                    }
                }
            }
        } else {
            content_text = content;
        }
    } else {
        content_text = content;
    }
    
    (front_matter, content_text)
}

// Extract date from post filename or front matter
fn extract_date_from_post(file_path: &Path, front_matter: &[(String, String)]) -> String {
    // First try to get date from front matter
    if let Some((_, date)) = front_matter.iter().find(|(k, _)| k.to_lowercase() == "date") {
        return date.clone();
    }
    
    // Next, try to extract date from filename (YYYY-MM-DD-title.md format)
    let filename = file_path.file_stem().unwrap().to_string_lossy();
    let date_regex = Regex::new(r"^(\d{4}-\d{2}-\d{2})").unwrap();
    
    if let Some(captures) = date_regex.captures(&filename) {
        if let Some(date_match) = captures.get(1) {
            return date_match.as_str().to_string();
        }
    }
    
    // If no date found, use current date
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// Convert Handlebars syntax to Liquid syntax
fn convert_handlebars_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert Handlebars {{variable}} to Liquid {{ variable }}
    // This is a simplistic conversion - real implementation would need to handle more cases
    let var_regex = Regex::new(r"\{\{\s*([^#\^/].*?)\s*\}\}").unwrap();
    result = var_regex.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map_or("", |m| m.as_str());
        format!("{{ {} }}", var_name)
    }).to_string();
    
    // Convert Handlebars {{#if condition}} to Liquid {% if condition %}
    let if_regex = Regex::new(r"\{\{#if\s+(.*?)\s*\}\}").unwrap();
    result = if_regex.replace_all(&result, |caps: &regex::Captures| {
        let condition = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% if {} %}}", condition)
    }).to_string();
    
    // Convert Handlebars {{else}} to Liquid {% else %}
    result = result.replace("{{else}}", "{% else %}");
    
    // Convert Handlebars {{/if}} to Liquid {% endif %}
    result = result.replace("{{/if}}", "{% endif %}");
    
    // Convert Handlebars {{#each items}} to Liquid {% for item in items %}
    let each_regex = Regex::new(r"\{\{#each\s+(.*?)\s*\}\}").unwrap();
    result = each_regex.replace_all(&result, |caps: &regex::Captures| {
        let collection = caps.get(1).map_or("", |m| m.as_str());
        format!("{{% for item in {} %}}", collection)
    }).to_string();
    
    // Convert Handlebars {{/each}} to Liquid {% endfor %}
    result = result.replace("{{/each}}", "{% endfor %}");
    
    // Convert Handlebars {{#with obj}} to Liquid (we'll use a comment to remind manual conversion)
    let with_regex = Regex::new(r"\{\{#with\s+(.*?)\s*\}\}").unwrap();
    result = with_regex.replace_all(&result, |caps: &regex::Captures| {
        let object = caps.get(1).map_or("", |m| m.as_str());
        format!("<!-- TODO: Convert Handlebars #with to Liquid. Original: {{{{#with {} }}}} -->\n", object)
    }).to_string();
    
    // Convert Handlebars {{/with}} to Liquid (reminder comment)
    result = result.replace("{{/with}}", "<!-- TODO: End of Handlebars #with block -->");
    
    result
} 