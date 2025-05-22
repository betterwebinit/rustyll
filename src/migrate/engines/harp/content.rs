use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::HarpMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, content is typically directly in the source directory or in public/
        // Files can be .jade, .ejs, .md, or .html
        
        let mut content_dir = source_dir.to_path_buf();
        
        // Check if content is in the public/ directory
        let public_dir = source_dir.join("public");
        if public_dir.exists() && public_dir.is_dir() {
            content_dir = public_dir;
            if verbose {
                log::info!("Content found in public/ directory");
            }
        }
        
        // Create destination directories
        let dest_posts_dir = dest_dir.join("_posts");
        let dest_pages_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&dest_posts_dir)?;
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        let mut found_content = false;
        
        // Harp doesn't have a standard directory for blog posts, but some common patterns
        let blog_dirs = vec![
            content_dir.join("blog"),
            content_dir.join("posts"),
            content_dir.join("articles"),
        ];
        
        // Check for blog posts in specific directories
        for blog_dir in blog_dirs {
            if blog_dir.exists() && blog_dir.is_dir() {
                if verbose {
                    log::info!("Migrating blog posts from {}", blog_dir.display());
                }
                
                self.migrate_posts(&blog_dir, &dest_posts_dir, result)?;
                found_content = true;
            }
        }
        
        // Process regular content (pages)
        if verbose {
            log::info!("Migrating pages from {}", content_dir.display());
        }
        
        self.migrate_pages(&content_dir, &dest_pages_dir, result)?;
        found_content = true;
        
        // Create sample content if no content was found
        if !found_content {
            if verbose {
                log::info!("No content found. Creating sample content.");
            }
            
            self.create_sample_content(&dest_posts_dir, &dest_pages_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_posts(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process content files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "jade" || ext == "ejs" {
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
        
        // Check for EJS or Jade format and convert as needed
        let processed_content = match file_path.extension().and_then(|e| e.to_str()) {
            Some("jade") => convert_jade_to_liquid(&content),
            Some("ejs") => convert_ejs_to_liquid(&content),
            _ => content,
        };
        
        // Extract metadata from _data.json file if it exists
        let metadata = extract_metadata_from_data_json(file_path, source_dir)?;
        
        // Get file name
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        
        // Check for date pattern in filename
        let date_regex = Regex::new(r"^(\d{4}-\d{2}-\d{2})-(.*)\.(md|markdown|html|jade|ejs)$").unwrap();
        
        let (date, title) = if let Some(captures) = date_regex.captures(&file_name) {
            // Already has date prefix
            let date = captures.get(1).unwrap().as_str().to_string();
            let title_part = captures.get(2).unwrap().as_str();
            
            // Convert kebab-case to Title Case
            let title = title_part.replace('-', " ").split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
                
            (date, title)
        } else {
            // No date prefix, use metadata or default date
            let date = metadata.get("date")
                .cloned()
                .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
                
            // Extract title from metadata or filename
            let title = metadata.get("title")
                .cloned()
                .unwrap_or_else(|| {
                    // Convert filename to title
                    file_path.file_stem().unwrap().to_string_lossy()
                        .replace('-', " ").split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                });
                
            (date, title)
        };
        
        // Create destination filename with date prefix
        let dest_file_name = format!("{}-{}.md", 
            date, 
            file_path.file_stem().unwrap().to_string_lossy()
                .to_string().to_lowercase().replace(' ', "-")
        );
        
        // Prepare Jekyll/Rustyll front matter
        let mut front_matter = String::from("---\n");
        front_matter.push_str(&format!("title: \"{}\"\n", title));
        front_matter.push_str("layout: post\n");
        front_matter.push_str(&format!("date: {}\n", date));
        
        // Add other metadata from _data.json
        for (key, value) in &metadata {
            if key != "title" && key != "layout" && key != "date" {
                front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        front_matter.push_str("---\n\n");
        
        // Create final content
        let final_content = if processed_content.starts_with("---") {
            // Content already has front matter, replace it
            if let Some(end_index) = processed_content[3..].find("---") {
                front_matter + &processed_content[(end_index + 6)..]
            } else {
                front_matter + &processed_content
            }
        } else {
            // Add front matter to content
            front_matter + &processed_content
        };
        
        // Write to destination file
        let dest_path = dest_dir.join(&dest_file_name);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write post file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "Blog post migrated from Harp".to_string(),
        });
        
        Ok(())
    }
    
    fn migrate_pages(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip files in special directories or underscore directories
                let rel_path = file_path.strip_prefix(source_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                let path_str = rel_path.to_string_lossy();
                
                if path_str.starts_with("_") || 
                   path_str.contains("/_") || 
                   path_str.starts_with(".") {
                    continue;
                }
                
                // Skip blog directories we already processed
                if path_str.starts_with("blog/") || 
                   path_str.starts_with("posts/") || 
                   path_str.starts_with("articles/") {
                    continue;
                }
                
                // Process content files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "jade" || ext == "ejs" {
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
        
        // Check for EJS or Jade format and convert as needed
        let processed_content = match file_path.extension().and_then(|e| e.to_str()) {
            Some("jade") => convert_jade_to_liquid(&content),
            Some("ejs") => convert_ejs_to_liquid(&content),
            _ => content,
        };
        
        // Extract metadata from _data.json file if it exists
        let metadata = extract_metadata_from_data_json(file_path, source_dir)?;
        
        // Get the relative path for permalink
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Determine a clean file name for the destination
        let file_name = if rel_path.components().count() > 1 {
            // For files in subdirectories, reflect the path in the name
            let path_parts: Vec<_> = rel_path.components()
                .map(|comp| comp.as_os_str().to_string_lossy().to_string())
                .collect();
                
            let file_stem = path_parts.last().unwrap()
                .split('.')
                .next()
                .unwrap()
                .to_string();
                
            if path_parts.len() > 1 {
                let dir_parts = &path_parts[0..path_parts.len()-1];
                let dir_name = dir_parts.join("-");
                
                if file_stem == "index" {
                    format!("{}.md", dir_name)
                } else {
                    format!("{}-{}.md", dir_name, file_stem)
                }
            } else {
                format!("{}.md", file_stem)
            }
        } else {
            // Simple case: just use the file stem
            format!("{}.md", file_path.file_stem().unwrap().to_string_lossy())
        };
        
        // Extract title from metadata or filename
        let title = metadata.get("title")
            .cloned()
            .unwrap_or_else(|| {
                // Use filename or "Home" for index
                if file_path.file_stem().unwrap().to_string_lossy() == "index" {
                    "Home".to_string()
                } else {
                    // Convert filename to title
                    file_path.file_stem().unwrap().to_string_lossy()
                        .replace('-', " ").split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                }
            });
            
        // Determine permalink
        let permalink = if file_path.file_stem().unwrap().to_string_lossy() == "index" {
            // Special handling for index pages
            if rel_path.parent().is_some() && !rel_path.parent().unwrap().as_os_str().is_empty() {
                format!("/{}/", rel_path.parent().unwrap().to_string_lossy())
            } else {
                "/".to_string()
            }
        } else {
            // Regular page permalink
            let permalink_path = rel_path.with_extension("");
            format!("/{}/", permalink_path.to_string_lossy())
        };
        
        // Prepare Jekyll/Rustyll front matter
        let mut front_matter = String::from("---\n");
        front_matter.push_str(&format!("title: \"{}\"\n", title));
        front_matter.push_str("layout: page\n");
        front_matter.push_str(&format!("permalink: {}\n", permalink));
        
        // Add other metadata from _data.json
        for (key, value) in &metadata {
            if key != "title" && key != "layout" && key != "permalink" {
                front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        front_matter.push_str("---\n\n");
        
        // Create final content
        let final_content = if processed_content.starts_with("---") {
            // Content already has front matter, replace it
            if let Some(end_index) = processed_content[3..].find("---") {
                front_matter + &processed_content[(end_index + 6)..]
            } else {
                front_matter + &processed_content
            }
        } else {
            // Add front matter to content
            front_matter + &processed_content
        };
        
        // Write to destination file
        let dest_path = dest_dir.join(&file_name);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write page file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_pages/{}", file_name),
            change_type: ChangeType::Converted,
            description: "Page migrated from Harp".to_string(),
        });
        
        Ok(())
    }
    
    fn create_sample_content(&self, posts_dir: &Path, pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a sample index page
        let index_content = r#"---
title: "Home"
layout: home
permalink: /
---

# Welcome to Your Migrated Site

This is a sample home page created during migration from Harp to Rustyll.

## Recent Posts

{% for post in site.posts limit:5 %}
* [{{ post.title }}]({{ post.url | relative_url }}) - {{ post.date | date: "%b %-d, %Y" }}
{% endfor %}

## About This Site

This site was migrated from Harp to Rustyll. Edit this page in `_pages/index.md` to customize it.
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

This is a sample About page created during migration from Harp to Rustyll.

## Migration Details

This site was automatically migrated from a Harp site to Rustyll format.

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
tags: [rustyll, harp, migration]
---

# Welcome to Rustyll!

This is a sample blog post created during the migration from Harp to Rustyll.

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

// Helper function to extract metadata from _data.json file
fn extract_metadata_from_data_json(file_path: &Path, source_dir: &Path) -> Result<std::collections::HashMap<String, String>, String> {
    let mut metadata = std::collections::HashMap::new();
    
    // Get the relative directory of the file
    let parent_dir = if let Some(parent) = file_path.parent() {
        parent.to_path_buf()
    } else {
        return Ok(metadata);
    };
    
    // Check for _data.json in the same directory
    let data_json_path = parent_dir.join("_data.json");
    if !data_json_path.exists() {
        return Ok(metadata);
    }
    
    // Read and parse the _data.json file
    let data_content = fs::read_to_string(&data_json_path)
        .map_err(|e| format!("Failed to read _data.json: {}", e))?;
        
    let data_json: serde_json::Value = serde_json::from_str(&data_content)
        .map_err(|e| format!("Failed to parse _data.json: {}", e))?;
        
    // Get the file's basename to look up in the data
    let file_base = file_path.file_stem().unwrap().to_string_lossy();
    
    // Check if there's data for this file
    if let Some(obj) = data_json.get(&*file_base).and_then(|v| v.as_object()) {
        for (key, value) in obj {
            // Convert JSON values to strings for the metadata
            let value_str = match value {
                serde_json::Value::Null => continue,
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => serde_json::to_string(value)
                    .unwrap_or_else(|_| "".to_string())
                    .trim_matches('"')
                    .to_string(),
            };
            
            metadata.insert(key.clone(), value_str);
        }
    }
    
    Ok(metadata)
}

// Helper function to convert EJS syntax to Liquid
pub(super) fn convert_ejs_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert EJS includes: <% include('path/to/file') %> -> {% include 'path/to/file' %}
    result = Regex::new(r#"<%\s*include\(['"]([^'"]*)['"]\)\s*%>"#)
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{% include '{}' %}}", &caps[1])
        })
        .to_string();
    
    // Convert EJS variables: <%= variable %> -> {{ variable }}
    result = Regex::new(r"<%=\s*(.*?)\s*%>")
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{ {} }}", &caps[1])
        })
        .to_string();
    
    // Convert EJS if statements: <% if (condition) { %> -> {% if condition %}
    result = Regex::new(r"<%\s*if\s*\((.*?)\)\s*{\s*%>")
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{% if {} %}}", &caps[1])
        })
        .to_string();
    
    // Convert EJS else if: <% } else if (condition) { %> -> {% elsif condition %}
    result = Regex::new(r"<%\s*}\s*else\s*if\s*\((.*?)\)\s*{\s*%>")
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{% elsif {} %}}", &caps[1])
        })
        .to_string();
    
    // Convert EJS else: <% } else { %> -> {% else %}
    result = result.replace("<% } else { %>", "{% else %}");
    
    // Convert EJS end blocks: <% } %> -> {% endif %}, etc.
    result = result.replace("<% } %>", "{% endif %}");
    
    // Convert EJS for loops: <% for (var i = 0; i < items.length; i++) { %> -> {% for item in items %}
    // This is more complex and might need manual adjustment
    result = Regex::new(r"<%\s*for\s*\(.*?in\s*(.*?)\)\s*{\s*%>")
        .unwrap()
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{{% for item in {} %}}", &caps[1])
        })
        .to_string();
    
    result
}

// Helper function to convert Jade to Liquid (simplified)
pub(super) fn convert_jade_to_liquid(content: &str) -> String {
    // Note: A complete Jade to Liquid converter would be complex
    // This is a simplified version that handles basic cases
    
    let mut result = String::new();
    let mut in_code_block = false;
    
    // Simple line-by-line conversion
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("//") {
            // Comment line
            result.push_str(&format!("<!-- {} -->\n", &trimmed[2..]));
        } else if trimmed.starts_with("extends ") {
            // Layout extension
            let layout = trimmed.strip_prefix("extends ").unwrap().trim_matches('"').trim_matches('\'');
            result.push_str("---\n");
            result.push_str(&format!("layout: {}\n", layout));
            result.push_str("---\n\n");
        } else if trimmed.starts_with("block content") {
            // Start of content block
            in_code_block = true;
        } else if trimmed.starts_with("if ") {
            // If statement
            let condition = trimmed.strip_prefix("if ").unwrap();
            result.push_str(&format!("{{% if {} %}}\n", condition));
        } else if trimmed == "else" {
            // Else statement
            result.push_str("{% else %}\n");
        } else if trimmed.starts_with("each ") {
            // Each loop
            let loop_parts: Vec<&str> = trimmed.split_whitespace().collect();
            if loop_parts.len() >= 4 && loop_parts[2] == "in" {
                result.push_str(&format!("{{% for {} in {} %}}\n", loop_parts[1], loop_parts[3]));
            }
        } else if in_code_block && trimmed.contains("#{") {
            // Variable interpolation in content
            let processed = Regex::new(r#"#\{([^}]*)\}"#)
                .unwrap()
                .replace_all(line, |caps: &regex::Captures| {
                    format!("{{ {} }}", &caps[1])
                })
                .to_string();
            result.push_str(&processed);
            result.push('\n');
        } else if trimmed.contains("=") && !trimmed.starts_with("=") {
            // Attributes - convert to HTML
            // This is very simplified and won't handle complex Jade
            result.push_str(line);
            result.push('\n');
        } else {
            // Copy line as is
            result.push_str(line);
            result.push('\n');
        }
    }
    
    // Add closing tags for any open blocks
    // This is simplified and might not handle all cases
    result = result.replace("block content", "");
    
    result
} 