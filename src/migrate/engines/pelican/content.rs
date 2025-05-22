use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use chrono::NaiveDate;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Pelican content...");
    }

    // Create destination directories
    let dest_posts_dir = dest_dir.join("_posts");
    create_dir_if_not_exists(&dest_posts_dir)?;
    
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;

    // Pelican content is typically in content/ directory
    let content_dir = source_dir.join("content");
    if !content_dir.exists() {
        result.warnings.push("No content directory found in Pelican source.".into());
        return Ok(());
    }

    // Process content directory
    for entry in WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process supported content file types
            if is_content_file(file_path) {
                migrate_content_file(file_path, &content_dir, &dest_posts_dir, &dest_pages_dir, result)?;
            }
        }
    }

    Ok(())
}

fn migrate_content_file(
    file_path: &Path,
    content_dir: &Path,
    dest_posts_dir: &Path,
    dest_pages_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the content file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
    
    // Parse the Pelican metadata and content
    let (metadata, body) = extract_pelican_metadata(&content);
    
    // Determine if it's a page or a post based on path or metadata
    let is_page = is_page_content(file_path, &metadata);
    
    // Build Jekyll front matter
    let mut jekyll_front_matter = String::from("---\n");
    
    // Required fields
    if let Some(title) = metadata.get("title") {
        jekyll_front_matter.push_str(&format!("title: \"{}\"\n", escape_yaml_string(title)));
    } else {
        // Use filename as title if not specified
        let title = file_path.file_stem().unwrap().to_string_lossy();
        jekyll_front_matter.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&title)));
    }
    
    // Date handling
    let (date_str, file_prefix) = if let Some(date) = metadata.get("date") {
        // Try to parse the date
        if let Some(parsed_date) = parse_pelican_date(date) {
            (parsed_date.format("%Y-%m-%d").to_string(), parsed_date.format("%Y-%m-%d-").to_string())
        } else {
            // Default to current date if we can't parse
            let today = chrono::Local::now().naive_local().date();
            (today.format("%Y-%m-%d").to_string(), today.format("%Y-%m-%d-").to_string())
        }
    } else {
        // Default to current date if not specified
        let today = chrono::Local::now().naive_local().date();
        (today.format("%Y-%m-%d").to_string(), today.format("%Y-%m-%d-").to_string())
    };
    
    jekyll_front_matter.push_str(&format!("date: {}\n", date_str));
    
    // Handle other common metadata
    if let Some(author) = metadata.get("author") {
        jekyll_front_matter.push_str(&format!("author: \"{}\"\n", escape_yaml_string(author)));
    }
    
    if let Some(category) = metadata.get("category") {
        jekyll_front_matter.push_str(&format!("category: \"{}\"\n", escape_yaml_string(category)));
    }
    
    if let Some(tags) = metadata.get("tags") {
        // Pelican tags are comma-separated
        let tag_list: Vec<&str> = tags.split(',').map(|t| t.trim()).collect();
        jekyll_front_matter.push_str("tags:\n");
        for tag in tag_list {
            jekyll_front_matter.push_str(&format!("  - \"{}\"\n", escape_yaml_string(tag)));
        }
    }
    
    // Special handling for Jekyll-specific fields
    jekyll_front_matter.push_str(&format!("layout: \"{}\"\n", if is_page { "page" } else { "post" }));
    
    if is_page {
        // For pages, we need a permalink
        if let Some(slug) = metadata.get("slug") {
            jekyll_front_matter.push_str(&format!("permalink: /{}/\n", slug));
        } else {
            // Derive permalink from filename
            let filename = file_path.file_stem().unwrap().to_string_lossy();
            jekyll_front_matter.push_str(&format!("permalink: /{}/\n", filename));
        }
    }
    
    // Add any other metadata as custom variables
    for (key, value) in &metadata {
        if !["title", "date", "author", "category", "tags", "slug"].contains(&key.as_str()) {
            jekyll_front_matter.push_str(&format!("{}: \"{}\"\n", key, escape_yaml_string(value)));
        }
    }
    
    jekyll_front_matter.push_str("---\n\n");
    
    // Combine front matter with converted content
    let converted_content = format!("{}{}", jekyll_front_matter, convert_pelican_content(body));
    
    // Determine destination file
    let dest_file = if is_page {
        // Pages go to _pages directory with their original name
        let filename = file_path.file_name().unwrap().to_string_lossy();
        let mut page_path = PathBuf::from(filename.to_string());
        
        // Make sure it has the right extension
        if let Some(old_ext) = page_path.extension() {
            if old_ext != "md" {
                let stem = page_path.file_stem().unwrap().to_string_lossy();
                page_path = PathBuf::from(format!("{}.md", stem));
            }
        } else {
            page_path = PathBuf::from(format!("{}.md", page_path.to_string_lossy()));
        }
        
        dest_pages_dir.join(page_path)
    } else {
        // Posts go to _posts directory with date prefix
        let filename = file_path.file_name().unwrap().to_string_lossy();
        let mut post_name = String::new();
        
        // Check if the filename already has a date prefix
        let date_prefix_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}-").unwrap();
        if date_prefix_regex.is_match(&filename) {
            post_name = filename.to_string();
        } else if let Some(slug) = metadata.get("slug") {
            post_name = format!("{}{}.md", file_prefix, slug);
        } else {
            // Use the file stem as the slug
            let stem = file_path.file_stem().unwrap().to_string_lossy();
            post_name = format!("{}{}.md", file_prefix, stem);
        }
        
        // Make sure it has .md extension
        if !post_name.ends_with(".md") {
            let stem = post_name.trim_end_matches(|c| c != '.');
            post_name = format!("{}.md", stem.trim_end_matches('.'));
        }
        
        dest_posts_dir.join(post_name)
    };
    
    // Create parent directory if needed
    if let Some(parent) = dest_file.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_file, converted_content)
        .map_err(|e| format!("Failed to write content file {}: {}", dest_file.display(), e))?;
    
    // Add to changes
    let rel_path = if is_page {
        format!("_pages/{}", dest_file.file_name().unwrap().to_string_lossy())
    } else {
        format!("_posts/{}", dest_file.file_name().unwrap().to_string_lossy())
    };
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: rel_path,
        description: format!("Converted Pelican {} from {}", if is_page { "page" } else { "post" }, file_path.display()),
    });
    
    Ok(())
}

// Helper function to extract Pelican metadata from content
fn extract_pelican_metadata(content: &str) -> (std::collections::HashMap<String, String>, &str) {
    let mut metadata = std::collections::HashMap::new();
    let mut body_start = 0;
    
    // Pelican supports multiple metadata formats
    // Try the reStructuredText metadata format first
    if content.starts_with(':') {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() && lines[i].starts_with(':') {
            let line = lines[i].trim();
            if let Some(colon_pos) = line.find(": ") {
                let key = line[1..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 2..].trim().to_string();
                metadata.insert(key, value);
            }
            i += 1;
        }
        body_start = lines[..i].join("\n").len() + 1; // +1 for the newline
    } 
    // Try Markdown metadata format (YAML front matter)
    else if content.starts_with("---") {
        if let Some(end_pos) = content[3..].find("---") {
            let front_matter = &content[3..end_pos + 3];
            for line in front_matter.lines() {
                if let Some(colon_pos) = line.find(": ") {
                    let key = line[..colon_pos].trim().to_lowercase();
                    let value = line[colon_pos + 2..].trim().to_string();
                    metadata.insert(key, value);
                }
            }
            body_start = end_pos + 6; // Skip past the second ---
        }
    }
    // Try Markdown metadata format (MultiMarkdown)
    else {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut in_metadata = true;
        
        while i < lines.len() && in_metadata {
            let line = lines[i].trim();
            
            if line.is_empty() {
                in_metadata = false;
            } else if let Some(colon_pos) = line.find(": ") {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 2..].trim().to_string();
                metadata.insert(key, value);
            } else {
                in_metadata = false;
            }
            
            i += 1;
        }
        
        if i > 0 {
            body_start = lines[..i].join("\n").len() + 1;
        }
    }
    
    (metadata, &content[body_start..])
}

// Helper function to convert Pelican content to Jekyll format
fn convert_pelican_content(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Convert Pelican-specific syntax to Jekyll/Liquid
    
    // Convert {filename} and {static} directives
    let filename_regex = Regex::new(r"\{filename\}([^}]+)").unwrap();
    converted = filename_regex.replace_all(&converted, |caps: &regex::Captures| {
        let path = &caps[1];
        if path.ends_with(".md") || path.ends_with(".rst") {
            // Convert to Jekyll post_url or page
            format!("{{ {{ site.baseurl }}}}{{{{ link {} }}}}", path.replace(".md", ".html"))
        } else {
            // Probably a static file
            format!("{{ {{ site.baseurl }}}}/assets/{}", path)
        }
    }).to_string();
    
    let static_regex = Regex::new(r"\{static\}([^}]+)").unwrap();
    converted = static_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{ {{ site.baseurl }}}}/assets/{}", &caps[1])
    }).to_string();
    
    // Convert other Pelican-specific syntax as needed
    
    converted
}

// Helper function to check if a file is a content file
fn is_content_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "md" | "markdown" | "rst" | "html")
    } else {
        false
    }
}

// Helper function to determine if content is a page or post
fn is_page_content(path: &Path, metadata: &std::collections::HashMap<String, String>) -> bool {
    // Check metadata for page indicator
    if let Some(status) = metadata.get("status") {
        if status.to_lowercase() == "page" {
            return true;
        }
    }
    
    // Check path - in Pelican, pages are often in a 'pages' directory
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("/pages/") || path_str.contains("\\pages\\")
}

// Helper function to parse Pelican date format
fn parse_pelican_date(date_str: &str) -> Option<NaiveDate> {
    // Try various date formats that Pelican supports
    let formats = [
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
    ];
    
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(date);
        }
    }
    
    // Try to extract just the date part if there's a time component
    if let Some(space_pos) = date_str.find(' ') {
        let date_part = &date_str[..space_pos];
        return parse_pelican_date(date_part);
    }
    
    None
}

// Helper function to escape special characters in YAML strings
fn escape_yaml_string(s: &str) -> String {
    s.replace("\"", "\\\"").replace("\n", "\\n")
} 