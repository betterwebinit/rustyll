use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress content...");
    }

    // Create destination directories
    let dest_posts_dir = dest_dir.join("_posts");
    create_dir_if_not_exists(&dest_posts_dir)?;
    
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;

    // In Octopress, content is typically in source/_posts and source directory
    let posts_dir = source_dir.join("source/_posts");
    if posts_dir.exists() && posts_dir.is_dir() {
        migrate_posts(&posts_dir, &dest_posts_dir, result)?;
    }
    
    // Also check for posts in _posts directly
    let alt_posts_dir = source_dir.join("_posts");
    if alt_posts_dir.exists() && alt_posts_dir.is_dir() {
        migrate_posts(&alt_posts_dir, &dest_posts_dir, result)?;
    }
    
    // Migrate pages from source directory
    migrate_pages(source_dir, &dest_pages_dir, result)?;

    // Create an about page if none exists
    create_default_pages(dest_dir, result)?;
    
    Ok(())
}

fn migrate_posts(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process all post files
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process Markdown and HTML files
            if is_content_file(file_path) {
                migrate_post_file(file_path, source_dir, dest_dir, result)?;
            }
        }
    }
    
    Ok(())
}

fn migrate_pages(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // In Octopress, pages can be in different locations
    let page_sources = [
        source_dir.join("source"),
        source_dir.join("source/pages"),
        source_dir.join("pages"),
    ];
    
    for page_source in page_sources.iter() {
        if page_source.exists() && page_source.is_dir() {
            // Process all page files
            for entry in WalkDir::new(page_source)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Only process Markdown and HTML files that are not posts
                    if is_content_file(file_path) && !is_post_file(file_path) {
                        migrate_page_file(file_path, page_source, dest_dir, result)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn migrate_post_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the post file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read post file {}: {}", file_path.display(), e))?;
    
    // Extract YAML front matter and content
    let (front_matter, body) = extract_front_matter(&content);
    
    // Convert content to Jekyll format if needed
    let converted_body = convert_content(body);
    
    // Create final content with front matter
    let final_content = if !front_matter.is_empty() {
        format!("---\n{}---\n\n{}", front_matter, converted_body)
    } else {
        // If no front matter was found, create a minimal one
        let filename = file_path.file_stem().unwrap().to_string_lossy();
        let today = chrono::Local::now().naive_local().date();
        
        format!("---\nlayout: post\ntitle: \"{}\"\ndate: {}\n---\n\n{}", 
                filename, today.format("%Y-%m-%d"), converted_body)
    };
    
    // Determine destination file name
    let dest_filename = create_post_filename(file_path);
    let dest_path = dest_dir.join(dest_filename);
    
    // Write the file
    fs::write(&dest_path, final_content)
        .map_err(|e| format!("Failed to write post file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_posts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Octopress post from {}", file_path.display()),
    });
    
    Ok(())
}

fn migrate_page_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Skip index files, as they're handled separately
    if file_path.file_name().unwrap().to_string_lossy().starts_with("index.") {
        return Ok(());
    }
    
    // Read the page file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read page file {}: {}", file_path.display(), e))?;
    
    // Extract YAML front matter and content
    let (front_matter, body) = extract_front_matter(&content);
    
    // Convert content to Jekyll format if needed
    let converted_body = convert_content(body);
    
    // Create final content with front matter
    let final_content = if !front_matter.is_empty() {
        let updated_front_matter = ensure_layout_in_front_matter(&front_matter, "page");
        format!("---\n{}---\n\n{}", updated_front_matter, converted_body)
    } else {
        // If no front matter was found, create a minimal one
        let filename = file_path.file_stem().unwrap().to_string_lossy();
        
        format!("---\nlayout: page\ntitle: \"{}\"\n---\n\n{}", 
                filename, converted_body)
    };
    
    // Determine destination file name
    let dest_filename = create_page_filename(file_path);
    let dest_path = dest_dir.join(dest_filename);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, final_content)
        .map_err(|e| format!("Failed to write page file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_pages/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Octopress page from {}", file_path.display()),
    });
    
    Ok(())
}

fn create_default_pages(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let pages_dir = dest_dir.join("_pages");
    let about_path = pages_dir.join("about.md");
    
    // Only create about page if it doesn't exist
    if !about_path.exists() {
        let about_content = r#"---
layout: page
title: "About"
permalink: /about/
---

This is the base Jekyll site converted from an Octopress site.

You can find out more info about customizing your Jekyll theme, as well as basic Jekyll usage documentation at [jekyllrb.com](https://jekyllrb.com/)

You can find the source code for Jekyll at GitHub:
[jekyll](https://github.com/jekyll/jekyll)
"#;

        fs::write(&about_path, about_content)
            .map_err(|e| format!("Failed to create about page: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "_pages/about.md".to_string(),
            description: "Created default about page".to_string(),
        });
    }
    
    Ok(())
}

fn extract_front_matter(content: &str) -> (String, &str) {
    // Check if the content starts with "---" for YAML front matter
    if content.starts_with("---") {
        if let Some(end_marker) = content[3..].find("---") {
            let front_matter = &content[3..end_marker + 3].trim();
            let body_start = 3 + end_marker + 3;
            
            // Skip any whitespace after the front matter
            let body = content[body_start..].trim_start();
            
            return (front_matter.to_string(), body);
        }
    }
    
    // No front matter found
    ("".to_string(), content)
}

fn ensure_layout_in_front_matter(front_matter: &str, layout_type: &str) -> String {
    if front_matter.contains("layout:") {
        // Layout already specified, return as is
        front_matter.to_string()
    } else {
        // Add layout to front matter
        format!("layout: {}\n{}", layout_type, front_matter)
    }
}

fn convert_content(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Replace Octopress-specific Liquid tags
    // For example, {% img ... %} -> ![alt text](/path/to/image)
    let img_regex = regex::Regex::new(r#"\{%\s*img\s+([^%]+)%\}"#).unwrap();
    converted = img_regex.replace_all(&converted, |caps: &regex::Captures| {
        let img_parts: Vec<&str> = caps[1].trim().split_whitespace().collect();
        
        if img_parts.len() >= 2 {
            // Basic format: path alt_text
            let path = img_parts[0];
            let alt = img_parts[1];
            
            format!("![{}]({})", alt, path)
        } else if !img_parts.is_empty() {
            // Just a path
            let path = img_parts[0];
            
            format!("![]({})", path)
        } else {
            // Something went wrong, keep the original
            caps[0].to_string()
        }
    }).to_string();
    
    // Replace other Octopress-specific constructs if needed
    
    converted
}

fn create_post_filename(file_path: &Path) -> String {
    // Octopress post filenames are typically already in Jekyll format
    // (YYYY-MM-DD-title.md)
    file_path.file_name().unwrap().to_string_lossy().to_string()
}

fn create_page_filename(file_path: &Path) -> String {
    // Just use the original filename for pages
    file_path.file_name().unwrap().to_string_lossy().to_string()
}

fn is_content_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "md" | "markdown" | "html" | "htm")
    } else {
        false
    }
}

fn is_post_file(path: &Path) -> bool {
    // Check if the file is in a _posts directory
    path.to_string_lossy().contains("_posts/") ||
    
    // Or if it has the YYYY-MM-DD-title.md format
    regex::Regex::new(r"^\d{4}-\d{2}-\d{2}-.*$")
        .unwrap()
        .is_match(&path.file_name().unwrap().to_string_lossy())
} 