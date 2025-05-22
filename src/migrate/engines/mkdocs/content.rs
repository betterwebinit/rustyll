use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

// Public wrapper function
pub fn migrate_content(source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
    let migrator = super::MkdocsMigrator::new();
    migrator.migrate_content(source_dir, dest_dir, verbose, result)
}

impl super::MkdocsMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        if verbose {
            log::info!("Migrating MkDocs content...");
        }

        // In MkDocs, content is typically in the docs/ directory
        let docs_dir = source_dir.join("docs");
        if !docs_dir.exists() || !docs_dir.is_dir() {
            result.warnings.push("No docs directory found.".into());
            return Ok(());
        }

        // Create destination directories
        let dest_docs_dir = dest_dir.join("_docs");
        create_dir_if_not_exists(&dest_docs_dir)?;

        // Process all markdown files in the docs directory
        for entry in WalkDir::new(&docs_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process only markdown files
                let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if extension == "md" || extension == "markdown" {
                    // Get the relative path from the docs directory
                    let rel_path = file_path.strip_prefix(&docs_dir)
                        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                    
                    // Create destination path
                    let dest_path = dest_docs_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Process the markdown file
                    migrate_markdown_file(file_path, &dest_path, result)?;
                }
            }
        }
        
        // Create an index.md in the root if it doesn't exist
        let index_path = dest_dir.join("index.md");
        if !index_path.exists() {
            create_index_file(&index_path, result)?;
        }
        
        Ok(())
    }
}

fn migrate_markdown_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the markdown file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read markdown file {}: {}", source_path.display(), e))?;
    
    // Extract any existing front matter or create new one
    let (front_matter, body) = extract_front_matter(&content);
    
    // Get file name without extension for the title
    let file_stem = source_path.file_stem().unwrap().to_string_lossy();
    
    // Get permalink from path
    let permalink = get_permalink_from_path(source_path, dest_path)?;
    
    // Create or update front matter
    let new_front_matter = if front_matter.is_empty() {
        format!("---\nlayout: doc\ntitle: {}\npermalink: {}\n---\n\n", 
                title_case(&file_stem), permalink)
    } else {
        // Check if layout is specified
        let layout_regex = Regex::new(r"(?m)^layout:").unwrap();
        let title_regex = Regex::new(r"(?m)^title:").unwrap();
        let permalink_regex = Regex::new(r"(?m)^permalink:").unwrap();
        
        let mut updated_front_matter = front_matter.clone();
        
        if !layout_regex.is_match(&front_matter) {
            updated_front_matter = format!("{}layout: doc\n", updated_front_matter);
        }
        
        if !title_regex.is_match(&front_matter) {
            updated_front_matter = format!("{}title: {}\n", updated_front_matter, title_case(&file_stem));
        }
        
        if !permalink_regex.is_match(&front_matter) {
            updated_front_matter = format!("{}permalink: {}\n", updated_front_matter, permalink);
        }
        
        updated_front_matter
    };
    
    // Convert MkDocs-specific syntax to Jekyll/Liquid
    let converted_body = convert_mkdocs_to_jekyll(body);
    
    // Combine front matter and converted body
    let final_content = format!("{}{}", new_front_matter, converted_body);
    
    // Write the converted file
    fs::write(dest_path, final_content)
        .map_err(|e| format!("Failed to write markdown file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_docs/{}", dest_path.strip_prefix(dest_path.parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Converted markdown file from {}", source_path.display()),
    });
    
    Ok(())
}

fn extract_front_matter(content: &str) -> (String, &str) {
    // Check if content starts with ---
    if content.starts_with("---") {
        if let Some(end_index) = content[3..].find("---") {
            let front_matter = &content[..end_index+6];
            let body = &content[end_index+6..];
            return (front_matter.to_string(), body.trim_start());
        }
    }
    
    // No front matter found
    (String::new(), content)
}

fn get_permalink_from_path(source_path: &Path, dest_path: &Path) -> Result<String, String> {
    // Get the path relative to the docs directory
    let source_dir = source_path.parent().unwrap();
    let docs_dir = source_dir.parent().unwrap();
    
    let rel_path = source_path.strip_prefix(docs_dir)
        .map_err(|_| format!("Failed to get relative path for {}", source_path.display()))?;
    
    // Convert to a permalink
    let rel_path_str = rel_path.to_string_lossy();
    let parts: Vec<&str> = rel_path_str.split('/').collect();
    
    // Check if this is an index file
    let file_name = source_path.file_name().unwrap().to_string_lossy();
    if file_name == "index.md" {
        // For index.md, the permalink is the directory path
        if parts.len() <= 2 {
            // Root index.md
            Ok("/".to_string())
        } else {
            // Nested index.md
            Ok(format!("/{}/", parts[1..parts.len()-1].join("/")))
        }
    } else {
        // For other files, use the filename without extension
        let file_stem = source_path.file_stem().unwrap().to_string_lossy();
        
        if parts.len() <= 2 {
            // File in root docs directory
            Ok(format!("/{}/", file_stem))
        } else {
            // Nested file
            let dir_path = parts[1..parts.len()-1].join("/");
            Ok(format!("/{}/{}/", dir_path, file_stem))
        }
    }
}

fn convert_mkdocs_to_jekyll(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Convert MkDocs-style admonitions to Jekyll notes
    // !!! note "Title"
    //     Content
    let admonition_regex = Regex::new(r#"(?m)^!!![ \t]+(note|tip|warning|danger|info)(?:[ \t]+"([^"]+)")?[ \t]*\n((?:[ \t]+.*\n)+)"#).unwrap();
    
    converted = admonition_regex.replace_all(&converted, |caps: &regex::Captures| {
        let admonition_type = &caps[1];
        let title = caps.get(2).map_or("", |m| m.as_str());
        let content = caps[3].trim();
        
        let class = match admonition_type {
            "note" => "info",
            "tip" => "success",
            "warning" => "warning",
            "danger" => "danger",
            "info" => "info",
            _ => "info"
        };
        
        let title_html = if title.is_empty() {
            "".to_string()
        } else {
            format!("<strong>{}</strong><br>\n", title)
        };
        
        format!("<div class=\"alert alert-{}\" markdown=\"1\">\n{}{}  \n</div>\n", 
                class, title_html, content)
    }).to_string();
    
    // Convert MkDocs-style code tabs
    // === "Tab 1"
    //     Content 1
    // === "Tab 2"
    //     Content 2
    let code_tabs_regex = Regex::new(r#"(?m)^===[ \t]+"([^"]+)"[ \t]*\n((?:[ \t]+.*\n)+)"#).unwrap();
    
    if code_tabs_regex.is_match(&converted) {
        // If code tabs are found, wrap them in a tabs container
        let mut tabs_html = String::from("<div class=\"code-tabs\">\n<ul class=\"nav nav-tabs\">\n");
        let mut content_html = String::from("<div class=\"tab-content\">\n");
        
        let mut tab_count = 0;
        
        for caps in code_tabs_regex.captures_iter(&converted.clone()) {
            let tab_name = &caps[1];
            let tab_content = caps[2].trim();
            
            let tab_id = format!("tab-{}", tab_count);
            let active = if tab_count == 0 { " active" } else { "" };
            
            tabs_html.push_str(&format!("<li class=\"nav-item\"><a class=\"nav-link{}\" href=\"#{}\" data-toggle=\"tab\">{}</a></li>\n", 
                                      active, tab_id, tab_name));
            
            content_html.push_str(&format!("<div class=\"tab-pane fade show{}\" id=\"{}\" markdown=\"1\">\n{}\n</div>\n", 
                                         active, tab_id, tab_content));
            
            tab_count += 1;
        }
        
        tabs_html.push_str("</ul>\n");
        content_html.push_str("</div>\n");
        
        // Replace all code tabs with the HTML container
        converted = code_tabs_regex.replace_all(&converted, "").to_string();
        converted = format!("{}\n{}</div>\n\n{}", tabs_html, content_html, converted);
    }
    
    // Convert MkDocs-style links to Jekyll links
    // [Text](path/to/page.md) -> [Text]({{ site.baseurl }}/path/to/page/)
    let links_regex = Regex::new(r#"\[([^\]]+)\]\((?!https?://|mailto:|ftp://|#)([^)]+\.md)(?:#([^)]+))?\)"#).unwrap();
    
    converted = links_regex.replace_all(&converted, |caps: &regex::Captures| {
        let link_text = &caps[1];
        let link_path = &caps[2];
        let link_fragment = caps.get(3).map_or("", |m| m.as_str());
        
        // Convert .md path to Jekyll path
        let jekyll_path = link_path.replace(".md", "/");
        
        // Add fragment if present
        let fragment = if link_fragment.is_empty() {
            "".to_string()
        } else {
            format!("#{}", link_fragment)
        };
        
        format!("[{}]({{ {{ site.baseurl }}}}/{}{})", link_text, jekyll_path, fragment)
    }).to_string();
    
    converted
}

fn create_index_file(
    index_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a basic index page
    let index_content = r#"---
layout: default
title: Home
---

# Welcome to the Documentation

This site contains documentation migrated from a MkDocs site.

## Navigation

{% for section in site.docs %}
- [{{ section.title }}]({{ section.url | relative_url }})
{% endfor %}

## Getting Started

Visit the [documentation section]({{ '/docs/' | relative_url }}) to start exploring.
"#;
    
    fs::write(index_path, index_content)
        .map_err(|e| format!("Failed to write index file: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "index.md".into(),
        description: "Created site index page".into(),
    });
    
    Ok(())
}

fn title_case(s: &str) -> String {
    // Convert snake_case or kebab-case to Title Case
    let words: Vec<&str> = s.split(|c| c == '_' || c == '-').collect();
    words.into_iter()
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