use std::path::Path;
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
        log::info!("Migrating Nikola content...");
    }

    // Create destination directories
    let dest_posts_dir = dest_dir.join("_posts");
    create_dir_if_not_exists(&dest_posts_dir)?;
    
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;

    // In Nikola, content is typically in posts/ and pages/ directories
    let posts_dir = source_dir.join("posts");
    let pages_dir = source_dir.join("pages");
    
    // Process posts if the directory exists
    if posts_dir.exists() && posts_dir.is_dir() {
        migrate_posts(&posts_dir, &dest_posts_dir, result)?;
    }
    
    // Process pages if the directory exists
    if pages_dir.exists() && pages_dir.is_dir() {
        migrate_pages(&pages_dir, &dest_pages_dir, result)?;
    }

    Ok(())
}

fn migrate_posts(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Walk through the posts directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process markdown and rst files
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
    // Walk through the pages directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process markdown and rst files
            if is_content_file(file_path) {
                migrate_page_file(file_path, source_dir, dest_dir, result)?;
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
    // Read the content file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read post file {}: {}", file_path.display(), e))?;
    
    // Extract front matter and content
    let (nikola_metadata, body) = extract_nikola_metadata(&content);
    
    // Create Jekyll front matter
    let jekyll_front_matter = create_jekyll_front_matter(&nikola_metadata, true);
    
    // Convert content
    let converted_content = convert_nikola_content(body);
    
    // Create final content
    let final_content = format!("{}\n{}", jekyll_front_matter, converted_content);
    
    // Determine destination file name with date prefix
    let dest_file_name = create_post_filename(file_path, &nikola_metadata);
    let dest_path = dest_dir.join(dest_file_name);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, final_content)
        .map_err(|e| format!("Failed to write post file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_posts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Nikola post from {}", file_path.display()),
    });
    
    Ok(())
}

fn migrate_page_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the content file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read page file {}: {}", file_path.display(), e))?;
    
    // Extract front matter and content
    let (nikola_metadata, body) = extract_nikola_metadata(&content);
    
    // Create Jekyll front matter
    let jekyll_front_matter = create_jekyll_front_matter(&nikola_metadata, false);
    
    // Convert content
    let converted_content = convert_nikola_content(body);
    
    // Create final content
    let final_content = format!("{}\n{}", jekyll_front_matter, converted_content);
    
    // Determine destination file name
    let dest_file_name = create_page_filename(file_path, &nikola_metadata);
    let dest_path = dest_dir.join(dest_file_name);
    
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
        description: format!("Converted Nikola page from {}", file_path.display()),
    });
    
    Ok(())
}

fn extract_nikola_metadata(content: &str) -> (std::collections::HashMap<String, String>, &str) {
    let mut metadata = std::collections::HashMap::new();
    let mut body_start = 0;
    
    // Check for restructuredtext metadata format
    if content.starts_with(".. ") {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() && lines[i].starts_with(".. ") {
            let line = lines[i].trim_start_matches(".. ");
            if let Some(colon_pos) = line.find(": ") {
                let key = line[0..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 2..].trim().to_string();
                metadata.insert(key, value);
            }
            i += 1;
            
            // Skip any blank lines after the metadata
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
        }
        
        if i > 0 {
            body_start = lines[..i].join("\n").len() + 1; // +1 for the newline
        }
    }
    // Check for markdown metadata format
    else if content.starts_with("<!--") {
        if let Some(end_comment) = content.find("-->") {
            let meta_section = &content[4..end_comment].trim();
            
            for line in meta_section.lines() {
                if let Some(colon_pos) = line.find(": ") {
                    let key = line[0..colon_pos].trim().to_lowercase();
                    let value = line[colon_pos + 2..].trim().to_string();
                    metadata.insert(key, value);
                }
            }
            
            body_start = end_comment + 3;
            
            // Skip any blank lines after the metadata
            let remainder = &content[body_start..];
            if let Some(first_non_blank) = remainder.find(|c: char| !c.is_whitespace()) {
                body_start += first_non_blank;
            }
        }
    }
    
    (metadata, &content[body_start..])
}

fn create_jekyll_front_matter(
    metadata: &std::collections::HashMap<String, String>,
    is_post: bool,
) -> String {
    let mut front_matter = String::from("---\n");
    
    // Set layout
    front_matter.push_str(&format!("layout: {}\n", if is_post { "post" } else { "page" }));
    
    // Add title
    if let Some(title) = metadata.get("title") {
        front_matter.push_str(&format!("title: \"{}\"\n", escape_yaml_string(title)));
    }
    
    // Add date for posts
    if is_post {
        if let Some(date) = metadata.get("date") {
            front_matter.push_str(&format!("date: {}\n", date));
        } else {
            // Use current date if not specified
            let today = chrono::Local::now().naive_local().date();
            front_matter.push_str(&format!("date: {}\n", today.format("%Y-%m-%d")));
        }
    }
    
    // Add author
    if let Some(author) = metadata.get("author") {
        front_matter.push_str(&format!("author: \"{}\"\n", escape_yaml_string(author)));
    }
    
    // Add tags and categories
    if let Some(tags) = metadata.get("tags") {
        front_matter.push_str("tags:\n");
        for tag in tags.split(',') {
            front_matter.push_str(&format!("  - \"{}\"\n", escape_yaml_string(tag.trim())));
        }
    }
    
    if let Some(categories) = metadata.get("category") {
        front_matter.push_str("categories:\n");
        for category in categories.split(',') {
            front_matter.push_str(&format!("  - \"{}\"\n", escape_yaml_string(category.trim())));
        }
    }
    
    // Add permalink for pages
    if !is_post {
        if let Some(slug) = metadata.get("slug") {
            front_matter.push_str(&format!("permalink: /{}/\n", slug));
        }
    }
    
    // Add any other metadata
    for (key, value) in metadata {
        if !["title", "date", "author", "tags", "category", "slug", "layout"].contains(&key.as_str()) {
            front_matter.push_str(&format!("{}: \"{}\"\n", key, escape_yaml_string(value)));
        }
    }
    
    front_matter.push_str("---");
    front_matter
}

fn convert_nikola_content(content: &str) -> String {
    // Convert Nikola-specific syntax to Jekyll/Liquid
    let mut converted = content.to_string();
    
    // Replace Nikola shortcodes with Liquid tags
    converted = converted.replace("{{% raw %}}", "{% raw %}");
    converted = converted.replace("{{% endraw %}}", "{% endraw %}");
    
    // Other conversions as needed
    
    converted
}

fn create_post_filename(
    original_path: &Path,
    metadata: &std::collections::HashMap<String, String>,
) -> String {
    // Extract date from metadata or filename
    let date_str = if let Some(date) = metadata.get("date") {
        // Parse the date to ensure consistent format
        if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            parsed_date.format("%Y-%m-%d").to_string()
        } else {
            // Fallback to current date
            chrono::Local::now().naive_local().date().format("%Y-%m-%d").to_string()
        }
    } else {
        // Try to extract date from filename (Nikola often uses format like 2021-01-01-post-title.md)
        let filename = original_path.file_name().unwrap().to_string_lossy();
        if let Some(captures) = regex::Regex::new(r"^(\d{4}-\d{2}-\d{2})-").unwrap().captures(&filename) {
            captures[1].to_string()
        } else {
            // Fallback to current date
            chrono::Local::now().naive_local().date().format("%Y-%m-%d").to_string()
        }
    };
    
    // Extract slug from metadata or filename
    let slug = if let Some(slug_value) = metadata.get("slug") {
        slug_value.clone()
    } else {
        // Remove date prefix if present and extension
        let filename = original_path.file_stem().unwrap().to_string_lossy();
        if let Some(no_date) = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}-(.+)$").unwrap().captures(&filename) {
            no_date[1].to_string()
        } else {
            filename.to_string()
        }
    };
    
    // Combine date and slug
    format!("{}-{}.md", date_str, slug)
}

fn create_page_filename(
    original_path: &Path,
    metadata: &std::collections::HashMap<String, String>,
) -> String {
    // Extract slug from metadata or filename
    let slug = if let Some(slug_value) = metadata.get("slug") {
        slug_value.clone()
    } else {
        original_path.file_stem().unwrap().to_string_lossy().to_string()
    };
    
    // Add .md extension
    format!("{}.md", slug)
}

fn is_content_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "md" | "markdown" | "rst" | "txt")
    } else {
        false
    }
}

fn escape_yaml_string(s: &str) -> String {
    s.replace("\"", "\\\"").replace("\n", "\\n")
} 