use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::GitbookMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In GitBook, content is organized based on SUMMARY.md
        // We need to parse SUMMARY.md to understand the structure and convert it to Jekyll/Rustyll format
        
        // Create destination directories
        let dest_pages_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        // Extract structure from SUMMARY.md
        let summary_path = source_dir.join("SUMMARY.md");
        let mut found_summary = false;
        
        if summary_path.exists() {
            found_summary = true;
            if verbose {
                log::info!("Found SUMMARY.md, parsing it for navigation structure");
            }
            
            // Parse SUMMARY.md to get page structure
            let (pages, navigation) = self.parse_summary(&summary_path, source_dir, verbose)?;
            
            // Migrate content based on the structure
            self.migrate_structured_content(&pages, source_dir, &dest_pages_dir, result)?;
            
            // Save navigation to _data/navigation.yml
            self.save_navigation(navigation, dest_dir, result)?;
        } else {
            if verbose {
                log::info!("No SUMMARY.md found, looking for content files directly");
            }
            
            // If no SUMMARY.md, just migrate all markdown files
            self.migrate_all_content(source_dir, &dest_pages_dir, result)?;
        }
        
        // Handle README.md specially - convert to index.md
        let readme_path = source_dir.join("README.md");
        if readme_path.exists() {
            if verbose {
                log::info!("Migrating README.md to index.md");
            }
            
            self.migrate_readme(&readme_path, &dest_pages_dir, result)?;
        }
        
        // Create sample content if nothing was found
        let has_content = !dest_pages_dir.read_dir().map_or(true, |mut r| r.next().is_none());
        if !has_content {
            if verbose {
                log::info!("No content found, creating sample content");
            }
            
            self.create_sample_content(&dest_pages_dir, result)?;
        }
        
        Ok(())
    }
    
    fn parse_summary(&self, summary_path: &Path, source_dir: &Path, verbose: bool) -> Result<(Vec<(PathBuf, String, u32)>, Vec<NavigationItem>), String> {
        // Read SUMMARY.md
        let content = fs::read_to_string(summary_path)
            .map_err(|e| format!("Failed to read SUMMARY.md: {}", e))?;
            
        // Parse links in SUMMARY.md to get pages and their relative order
        let link_regex = Regex::new(r"\*\s*\[(.*?)\]\((.*?)\)").unwrap();
        let section_regex = Regex::new(r"#{1,6}\s+(.+)").unwrap();
        
        let mut pages = Vec::new();
        let mut navigation = Vec::new();
        let mut current_section: Option<String> = None;
        let mut current_level = 0;
        let mut items_in_section = Vec::new();
        
        for line in content.lines() {
            // Check if this is a section header
            if let Some(captures) = section_regex.captures(line) {
                // Process previous section if it exists
                if let Some(section) = &current_section {
                    if !items_in_section.is_empty() {
                        navigation.push(NavigationItem {
                            title: section.clone(),
                            url: None,
                            children: items_in_section.clone(),
                        });
                        items_in_section.clear();
                    }
                }
                
                // Start new section
                current_section = Some(captures.get(1).unwrap().as_str().trim().to_string());
                current_level = line.chars().take_while(|&c| c == '#').count() as u32;
                continue;
            }
            
            // Check if this is a link entry
            if let Some(captures) = link_regex.captures(line) {
                let title = captures.get(1).unwrap().as_str().trim();
                let link = captures.get(2).unwrap().as_str().trim();
                
                // Calculate indentation level based on spaces/tabs before the *
                let indent_level = current_level + line.chars()
                    .take_while(|&c| c == ' ' || c == '\t')
                    .count() as u32 / 2;
                
                // Add to navigation
                let nav_item = NavigationItem {
                    title: title.to_string(),
                    url: Some(link.to_string()),
                    children: Vec::new(),
                };
                
                items_in_section.push(nav_item);
                
                // Construct the full path
                let file_path = source_dir.join(link);
                
                // Only add if it's a file that exists
                if file_path.exists() && file_path.is_file() {
                    pages.push((file_path, title.to_string(), indent_level));
                } else if verbose {
                    log::warn!("Referenced file not found: {}", file_path.display());
                }
            }
        }
        
        // Add the last section if it exists
        if let Some(section) = &current_section {
            if !items_in_section.is_empty() {
                navigation.push(NavigationItem {
                    title: section.clone(),
                    url: None,
                    children: items_in_section,
                });
            }
        }
        
        Ok((pages, navigation))
    }
    
    fn migrate_structured_content(&self, pages: &[(PathBuf, String, u32)], source_dir: &Path, dest_pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for (file_path, title, level) in pages {
            // Skip README.md (handled separately)
            if file_path.file_name().unwrap_or_default().to_string_lossy() == "README.md" {
                continue;
            }
            
            // Determine the destination file name
            let rel_path = file_path.strip_prefix(source_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
            // Clean the path for use in Jekyll
            let dest_file_name = format!("{}.md", self.clean_path_for_jekyll(rel_path));
            
            // Read the file
            let content = fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
                
            // Determine if it already has front matter
            let (existing_front_matter, content_text) = extract_front_matter(&content);
            
            // Create front matter
            let mut front_matter = String::from("---\n");
            front_matter.push_str(&format!("title: \"{}\"\n", title));
            front_matter.push_str("layout: page\n");
            
            // Generate permalink from path
            let permalink = format!("/{}/", rel_path.with_extension("").to_string_lossy().replace('\\', "/"));
            front_matter.push_str(&format!("permalink: {}\n", permalink));
            
            // Add weight/order based on level in SUMMARY.md
            front_matter.push_str(&format!("weight: {}\n", level));
            
            // Add any existing front matter that doesn't conflict
            for (key, value) in &existing_front_matter {
                if key != "title" && key != "layout" && key != "permalink" && key != "weight" {
                    front_matter.push_str(&format!("{}: {}\n", key, value));
                }
            }
            
            front_matter.push_str("---\n\n");
            
            // Create the final content
            let final_content = format!("{}{}", front_matter, content_text);
            
            // Write the file
            let dest_path = dest_pages_dir.join(&dest_file_name);
            fs::write(&dest_path, final_content)
                .map_err(|e| format!("Failed to write page file: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: format!("_pages/{}", dest_file_name),
                change_type: ChangeType::Converted,
                description: format!("Content page migrated from GitBook: {}", rel_path.display()),
            });
        }
        
        Ok(())
    }
    
    fn save_navigation(&self, navigation: Vec<NavigationItem>, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create _data directory
        let data_dir = dest_dir.join("_data");
        create_dir_if_not_exists(&data_dir)?;
        
        // Convert navigation to YAML
        let mut yaml_content = String::from("# Navigation structure extracted from SUMMARY.md\n");
        
        for (i, section) in navigation.iter().enumerate() {
            if i > 0 {
                yaml_content.push('\n');
            }
            
            if section.url.is_none() {
                // This is a section header
                yaml_content.push_str(&format!("{}:\n", self.clean_string_for_yaml(&section.title)));
                
                for item in &section.children {
                    let default_url = String::from("/");
                    let url = item.url.as_ref().unwrap_or(&default_url);
                    yaml_content.push_str(&format!("  - title: \"{}\"\n", item.title));
                    yaml_content.push_str(&format!("    url: {}\n", url));
                }
            } else {
                // This is a top-level item
                yaml_content.push_str(&format!("- title: \"{}\"\n", section.title));
                yaml_content.push_str(&format!("  url: {}\n", section.url.as_ref().unwrap()));
            }
        }
        
        // Write navigation to _data/navigation.yml
        fs::write(data_dir.join("navigation.yml"), yaml_content)
            .map_err(|e| format!("Failed to write navigation data: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_data/navigation.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Navigation structure extracted from SUMMARY.md".to_string(),
        });
        
        Ok(())
    }
    
    fn migrate_all_content(&self, source_dir: &Path, dest_pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Migrate all Markdown files
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip non-markdown files
                if !file_path.extension().map_or(false, |ext| ext == "md") {
                    continue;
                }
                
                // Skip README.md and SUMMARY.md (handled separately)
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                if file_name == "README.md" || file_name == "SUMMARY.md" {
                    continue;
                }
                
                // Determine destination file path
                let rel_path = file_path.strip_prefix(source_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                // Clean the path for Jekyll
                let dest_file_name = format!("{}.md", self.clean_path_for_jekyll(rel_path));
                
                // Read the content
                let content = fs::read_to_string(file_path)
                    .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
                
                // Extract existing front matter
                let (existing_front_matter, content_text) = extract_front_matter(&content);
                
                // Extract title from first heading or use filename
                let title = extract_title_from_content(&content_text)
                    .unwrap_or_else(|| {
                        // Use file name as title
                        file_path.file_stem().unwrap_or_default().to_string_lossy()
                            .replace('-', " ").replace('_', " ")
                            .split_whitespace()
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
                
                // Create front matter
                let mut front_matter = String::from("---\n");
                front_matter.push_str(&format!("title: \"{}\"\n", title));
                front_matter.push_str("layout: page\n");
                
                // Generate permalink
                let permalink = format!("/{}/", rel_path.with_extension("").to_string_lossy().replace('\\', "/"));
                front_matter.push_str(&format!("permalink: {}\n", permalink));
                
                // Add any existing front matter that doesn't conflict
                for (key, value) in &existing_front_matter {
                    if key != "title" && key != "layout" && key != "permalink" {
                        front_matter.push_str(&format!("{}: {}\n", key, value));
                    }
                }
                
                front_matter.push_str("---\n\n");
                
                // Create final content
                let final_content = format!("{}{}", front_matter, content_text);
                
                // Write to destination
                let dest_path = dest_pages_dir.join(&dest_file_name);
                fs::write(&dest_path, final_content)
                    .map_err(|e| format!("Failed to write page file: {}", e))?;
                    
                result.changes.push(MigrationChange {
                    file_path: format!("_pages/{}", dest_file_name),
                    change_type: ChangeType::Converted,
                    description: format!("Content page migrated from GitBook: {}", rel_path.display()),
                });
            }
        }
        
        Ok(())
    }
    
    fn migrate_readme(&self, readme_path: &Path, dest_pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read README.md
        let content = fs::read_to_string(readme_path)
            .map_err(|e| format!("Failed to read README.md: {}", e))?;
            
        // Extract any existing front matter
        let (existing_front_matter, content_text) = extract_front_matter(&content);
        
        // Extract title from first heading or use default
        let title = extract_title_from_content(&content_text)
            .unwrap_or_else(|| "Home".to_string());
        
        // Create front matter
        let mut front_matter = String::from("---\n");
        front_matter.push_str(&format!("title: \"{}\"\n", title));
        front_matter.push_str("layout: home\n");
        front_matter.push_str("permalink: /\n");
        
        // Add any existing front matter that doesn't conflict
        for (key, value) in &existing_front_matter {
            if key != "title" && key != "layout" && key != "permalink" {
                front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        front_matter.push_str("---\n\n");
        
        // Create final content
        let final_content = format!("{}{}", front_matter, content_text);
        
        // Write to destination
        let dest_path = dest_pages_dir.join("index.md");
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write index page: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_pages/index.md".to_string(),
            change_type: ChangeType::Converted,
            description: "Home page migrated from GitBook README.md".to_string(),
        });
        
        Ok(())
    }
    
    fn create_sample_content(&self, dest_pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a sample index page
        let index_content = r#"---
title: "Home"
layout: home
permalink: /
---

# Welcome to Your Migrated GitBook Site

This is a sample home page created during migration from GitBook to Rustyll.

## Contents

{% for page in site.pages %}
{% if page.title and page.title != "Home" %}
* [{{ page.title }}]({{ page.url | relative_url }})
{% endif %}
{% endfor %}

## About This Site

This site was migrated from GitBook to Rustyll. You can edit this page in `_pages/index.md` to customize it.
"#;

        fs::write(dest_pages_dir.join("index.md"), index_content)
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

This is a sample About page created during migration from GitBook to Rustyll.

## Migration Details

This site was automatically migrated from a GitBook site to Rustyll format.

## Content Organization

In GitBook, content was organized using `SUMMARY.md`. In Rustyll, we've preserved this 
structure and converted it to Jekyll/Rustyll's page and collection system.

## Documentation

For more information about Rustyll, visit [https://rustyll.org](https://rustyll.org).
"#;

        fs::write(dest_pages_dir.join("about.md"), about_content)
            .map_err(|e| format!("Failed to write sample about page: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_pages/about.md".to_string(),
            change_type: ChangeType::Created,
            description: "Sample about page created".to_string(),
        });
        
        Ok(())
    }
    
    fn clean_path_for_jekyll(&self, path: &Path) -> String {
        // Convert path to a Jekyll-friendly format
        let path_str = path.with_extension("").to_string_lossy().to_string();
        
        // Replace slashes with hyphens
        let clean_path = path_str.replace('/', "-").replace('\\', "-");
        
        clean_path
    }
    
    fn clean_string_for_yaml(&self, s: &str) -> String {
        // Clean a string for use as a YAML key
        s.to_lowercase()
            .replace(' ', "_")
            .replace('-', "_")
            .replace('.', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }
}

// Helper struct for navigation items
#[derive(Clone)]
struct NavigationItem {
    title: String,
    url: Option<String>,
    children: Vec<NavigationItem>,
}

// Helper function to extract front matter from content
fn extract_front_matter(content: &str) -> (Vec<(String, String)>, &str) {
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

// Helper function to extract title from content
fn extract_title_from_content(content: &str) -> Option<String> {
    // Look for the first Markdown heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed.trim_start_matches("# ").trim().to_string());
        }
    }
    None
} 