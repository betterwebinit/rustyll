use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

// Public module function for external access
pub fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Delegate to the struct implementation
    let migrator = super::CobaltMigrator::new();
    migrator.migrate_content(source_dir, dest_dir, verbose, result)
}

impl super::CobaltMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Cobalt, content is typically in:
        // - posts/ directory for blog posts
        // - _posts/ directory for blog posts (alternatively)
        // - pages/ directory or root for other content
        
        let post_dirs = vec![
            source_dir.join("posts"),
            source_dir.join("_posts"),
            source_dir.join("blog"),
        ];
        
        let page_dirs = vec![
            source_dir.join("pages"),
            source_dir.join("_pages"),
        ];
        
        // Create destination directories
        let dest_posts_dir = dest_dir.join("_posts");
        let dest_pages_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&dest_posts_dir)?;
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        let mut found_posts = false;
        let mut found_pages = false;
        
        // Process blog posts
        for post_dir in post_dirs {
            if post_dir.exists() && post_dir.is_dir() {
                if verbose {
                    log::info!("Migrating blog posts from {}", post_dir.display());
                }
                
                self.migrate_posts(&post_dir, &dest_posts_dir, result)?;
                found_posts = true;
            }
        }
        
        // Process pages from specific page directories
        for page_dir in page_dirs {
            if page_dir.exists() && page_dir.is_dir() {
                if verbose {
                    log::info!("Migrating pages from {}", page_dir.display());
                }
                
                self.migrate_pages(&page_dir, &dest_pages_dir, result)?;
                found_pages = true;
            }
        }
        
        // Also check for pages in the root directory
        if !found_pages {
            if verbose {
                log::info!("Looking for pages in root directory");
            }
            
            self.migrate_root_pages(source_dir, &dest_pages_dir, result)?;
            found_pages = true;
        }
        
        // Create sample content if no content was found
        if !found_posts && !found_pages {
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
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "liquid" {
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
        
        // Get the file name
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        
        // Determine if the filename already has date prefix (YYYY-MM-DD-*)
        let date_regex = Regex::new(r"^(\d{4}-\d{2}-\d{2})-(.*)\.(md|markdown|html|liquid)$").unwrap();
        
        let dest_file_name = if let Some(captures) = date_regex.captures(&file_name) {
            // Already has date prefix, keep the same name but ensure .md extension
            format!("{}-{}.md", 
                    captures.get(1).unwrap().as_str(), 
                    captures.get(2).unwrap().as_str())
        } else {
            // No date prefix, extract date from front matter or use current date
            let date = front_matter.iter()
                .find(|(k, _)| k.to_lowercase() == "date")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
                
            // Format date to YYYY-MM-DD if necessary
            let formatted_date = if date.contains("T") {
                // ISO format, extract just the date part
                date.split('T').next().unwrap_or(&date).to_string()
            } else {
                date
            };
            
            // Create new filename with date prefix
            format!("{}-{}.md", formatted_date, file_path.file_stem().unwrap().to_string_lossy())
        };
        
        // Prepare the new content with adjusted front matter
        let mut new_content = String::from("---\n");
        
        // Add title if it exists, or create one from the filename
        let title = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                let stem = file_path.file_stem().unwrap().to_string_lossy();
                // Remove date prefix if present
                if let Some(captures) = date_regex.captures(&stem) {
                    if let Some(title_part) = captures.get(2) {
                        // Convert kebab-case to Title Case
                        title_part.as_str().replace('-', " ").split_whitespace()
                            .map(|word| {
                                let mut chars = word.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(" ")
                    } else {
                        stem.to_string()
                    }
                } else {
                    stem.to_string()
                }
            });
            
        new_content.push_str(&format!("title: \"{}\"\n", title));
        
        // Add layout (default to post)
        let layout = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "layout" || k.to_lowercase() == "extends")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "post".to_string());
            
        new_content.push_str(&format!("layout: {}\n", layout));
        
        // Add or modify date
        let date = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "date")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
            
        new_content.push_str(&format!("date: {}\n", date));
        
        // Add other front matter
        for (key, value) in &front_matter {
            if key.to_lowercase() != "title" && key.to_lowercase() != "layout" && 
               key.to_lowercase() != "extends" && key.to_lowercase() != "date" {
                new_content.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        new_content.push_str("---\n\n");
        
        // Add the content, converting any Liquid tags if needed
        let processed_content = convert_liquid_if_needed(content_text);
        new_content.push_str(&processed_content);
        
        // Write the file
        let dest_path = dest_dir.join(&dest_file_name);
        fs::write(&dest_path, new_content)
            .map_err(|e| format!("Failed to write post file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "Blog post migrated from Cobalt".to_string(),
        });
        
        Ok(())
    }
    
    fn migrate_pages(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process content files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ext == "md" || ext == "markdown" || ext == "html" || ext == "liquid" {
                        self.process_page_file(file_path, source_dir, dest_dir, result)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn migrate_root_pages(&self, source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Look for content files directly in the root directory
        for entry in fs::read_dir(source_dir)
            .map_err(|e| format!("Failed to read source directory: {}", e))? {
            
            if let Ok(entry) = entry {
                let file_path = entry.path();
                
                if file_path.is_file() {
                    // Skip special files and directories
                    let file_name = file_path.file_name().unwrap().to_string_lossy();
                    if file_name.starts_with('_') || file_name == "CNAME" || 
                       file_name == "README.md" || file_name == "LICENSE" {
                        continue;
                    }
                    
                    // Process content files
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        
                        if ext == "md" || ext == "markdown" || ext == "html" || ext == "liquid" {
                            self.process_page_file(&file_path, source_dir, dest_dir, result)?;
                        }
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
        
        // Get the relative path
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Determine the destination filename
        let dest_file_name = if rel_path.components().count() > 1 {
            // For files in subdirectories, flatten the path with hyphens
            let path_str = rel_path.with_extension("").to_string_lossy().to_string();
            format!("{}.md", path_str.replace("/", "-"))
        } else {
            // For root files, keep the name but ensure .md extension
            format!("{}.md", file_path.file_stem().unwrap().to_string_lossy())
        };
        
        // Prepare the new content with adjusted front matter
        let mut new_content = String::from("---\n");
        
        // Add title if it exists, or create one from the filename
        let title = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                if file_path.file_stem().unwrap().to_string_lossy() == "index" {
                    "Home".to_string()
                } else {
                    // Convert kebab-case to Title Case
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
            
        new_content.push_str(&format!("title: \"{}\"\n", title));
        
        // Add layout (default to page)
        let layout = front_matter.iter()
            .find(|(k, _)| k.to_lowercase() == "layout" || k.to_lowercase() == "extends")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                if file_path.file_stem().unwrap().to_string_lossy() == "index" {
                    "home".to_string()
                } else {
                    "page".to_string()
                }
            });
            
        new_content.push_str(&format!("layout: {}\n", layout));
        
        // Add permalink based on the file path
        let permalink = if file_path.file_stem().unwrap().to_string_lossy() == "index" {
            // Special case for index pages
            if rel_path.parent().is_some() && !rel_path.parent().unwrap().as_os_str().is_empty() {
                format!("/{}/", rel_path.parent().unwrap().to_string_lossy())
            } else {
                "/".to_string()
            }
        } else {
            // Check if there's a permalink in the front matter
            if let Some((_, permalink)) = front_matter.iter()
                .find(|(k, _)| k.to_lowercase() == "permalink") {
                permalink.clone()
            } else {
                // Regular pages
                if rel_path.components().count() > 1 {
                    // For files in subdirectories
                    format!("/{}/", rel_path.with_extension("").to_string_lossy())
                } else {
                    // For root files
                    format!("/{}/", file_path.file_stem().unwrap().to_string_lossy())
                }
            }
        };
        
        new_content.push_str(&format!("permalink: {}\n", permalink));
        
        // Add other front matter
        for (key, value) in &front_matter {
            if key.to_lowercase() != "title" && key.to_lowercase() != "layout" && 
               key.to_lowercase() != "extends" && key.to_lowercase() != "permalink" {
                new_content.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        new_content.push_str("---\n\n");
        
        // Add the content, converting any Liquid tags if needed
        let processed_content = convert_liquid_if_needed(content_text);
        new_content.push_str(&processed_content);
        
        // Write the file
        let dest_path = dest_dir.join(&dest_file_name);
        fs::write(&dest_path, new_content)
            .map_err(|e| format!("Failed to write page file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_pages/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "Page migrated from Cobalt".to_string(),
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

This is a sample home page created during migration from Cobalt.rs to Rustyll.

## Recent Posts

{% for post in site.posts limit:5 %}
* [{{ post.title }}]({{ post.url | relative_url }}) - {{ post.date | date: "%b %-d, %Y" }}
{% endfor %}

## About This Site

This site was migrated from Cobalt.rs to Rustyll. Edit this page in `_pages/index.md` to customize it.
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

This is a sample About page created during migration from Cobalt.rs to Rustyll.

## Migration Details

This site was automatically migrated from a Cobalt.rs site to Rustyll format.

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
tags: [rustyll, cobalt, migration]
---

# Welcome to Rustyll!

This is a sample blog post created during the migration from Cobalt.rs to Rustyll.

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

// Helper function to extract front matter from content
fn extract_front_matter(content: &str) -> (Vec<(String, String)>, &str) {
    let mut front_matter = Vec::new();
    let content_text: &str;
    
    // Cobalt uses ---/--- for YAML front matter
    if content.starts_with("---") {
        if let Some(end_index) = content[3..].find("---") {
            let yaml_part = &content[3..(end_index + 3)];
            content_text = &content[(end_index + 6)..];
            
            // Simple line-by-line parsing of YAML front matter
            for line in yaml_part.lines() {
                if line.trim().is_empty() || line.trim().starts_with('#') {
                    continue; // Skip empty lines and comments
                }
                
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

// Helper function to convert/fix Liquid syntax if needed
fn convert_liquid_if_needed(content: &str) -> String {
    // Cobalt already uses Liquid-compatible syntax, but 
    // we might need to fix some specific issues
    
    // For now, we'll just return the content as-is
    content.to_string()
} 