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
        log::info!("Migrating Metalsmith content...");
    }
    
    // In Metalsmith, content is typically in src directory
    let content_dirs = [
        source_dir.join("src"),
        source_dir.join("content"),
        source_dir.join("pages"),
    ];
    
    let mut found_content = false;
    
    for content_dir in content_dirs.iter() {
        if !content_dir.exists() {
            continue;
        }
        
        found_content = true;
        
        // Create destination content directories
        let dest_posts_dir = dest_dir.join("_posts");
        let dest_pages_dir = dest_dir.join("_pages");
        
        create_dir_if_not_exists(&dest_posts_dir)?;
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        // Process markdown files
        let markdown_files = WalkDir::new(content_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                e.path().is_file() && (extension == "md" || extension == "markdown" || extension == "html")
            });
        
        for entry in markdown_files {
            let relative_path = entry.path().strip_prefix(content_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read file {}: {}", entry.path().display(), e))?;
            
            // Determine if it's a post or a page by checking for date-based path or frontmatter
            let (dest_file, is_post) = if is_post_content(&file_content, relative_path) {
                // If we have a post, create Jekyll-style dated filename
                let date = extract_date_from_content(&file_content, relative_path)
                    .unwrap_or_else(|| "2023-01-01".to_string());
                
                let filename = relative_path.file_name().unwrap_or_default().to_string_lossy();
                
                (dest_posts_dir.join(format!("{}-{}", date, filename)), true)
            } else {
                // For pages, keep the original path structure
                (dest_pages_dir.join(relative_path), false)
            };
            
            // Ensure parent directory exists
            if let Some(parent) = dest_file.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert content to Jekyll format
            let jekyll_content = convert_to_jekyll_format(&file_content, is_post)?;
            
            fs::write(&dest_file, jekyll_content)
                .map_err(|e| format!("Failed to write file {}: {}", dest_file.display(), e))?;
            
            // Add to migration results
            let content_type = if is_post { "post" } else { "page" };
            result.changes.push(MigrationChange {
                file_path: format!("{}", dest_file.strip_prefix(dest_dir).unwrap_or(relative_path).display()),
                change_type: ChangeType::Converted,
                description: format!("Converted Metalsmith {} to Jekyll format", content_type),
            });
        }
    }
    
    if !found_content {
        result.warnings.push("Could not find Metalsmith content directory (src, content, or pages)".into());
    }
    
    Ok(())
}

fn is_post_content(content: &str, path: &Path) -> bool {
    // Check if content has a date field in frontmatter
    if content.contains("date:") {
        return true;
    }
    
    // Check if path contains blog, posts, or articles directories
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("/blog/") || path_str.contains("/posts/") || path_str.contains("/articles/") {
        return true;
    }
    
    // Check for date-based directories in path (YYYY/MM/DD)
    let date_regex = regex::Regex::new(r"(?:/|\A)(\d{4})(?:/(\d{2}))?(?:/(\d{2}))?(?:/|\z)").ok();
    if let Some(regex) = date_regex {
        if regex.is_match(&path_str) {
            return true;
        }
    }
    
    false
}

fn extract_date_from_content(content: &str, path: &Path) -> Option<String> {
    // Try to extract date from frontmatter
    let date_regex = regex::Regex::new(r"(?m)^date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").ok()?;
    if let Some(cap) = date_regex.captures(content) {
        return Some(cap[1].to_string());
    }
    
    // Try to extract date from path (common in blogs)
    let path_str = path.to_string_lossy();
    let path_date_regex = regex::Regex::new(r"(?:/|\A)(\d{4})(?:/(\d{2}))?(?:/(\d{2}))?(?:/|\z)").ok()?;
    
    if let Some(cap) = path_date_regex.captures(&path_str) {
        let year = cap.get(1)?.as_str();
        let month = cap.get(2).map_or("01", |m| m.as_str());
        let day = cap.get(3).map_or("01", |d| d.as_str());
        
        return Some(format!("{}-{}-{}", year, month, day));
    }
    
    None
}

fn convert_to_jekyll_format(content: &str, is_post: bool) -> Result<String, String> {
    // Check if content already has frontmatter
    let has_frontmatter = content.starts_with("---");
    
    let mut converted = if has_frontmatter {
        content.to_string()
    } else {
        // Add Jekyll frontmatter
        let layout = if is_post { "post" } else { "page" };
        format!("---\nlayout: {}\n---\n\n{}", layout, content)
    };
    
    // If it has frontmatter but no layout, add a layout field
    if has_frontmatter && !content.contains("layout:") {
        let layout = if is_post { "post" } else { "page" };
        
        // Insert layout after the first --- line
        let parts: Vec<&str> = content.splitn(2, "---").collect();
        if parts.len() > 1 {
            converted = format!("---\nlayout: {}\n{}", layout, parts[1]);
        }
    }
    
    // Convert any Metalsmith/Handlebars template syntax to Liquid
    converted = converted
        .replace("{{", "{%")
        .replace("}}", "%}")
        .replace("{%#", "{%")
        .replace("{%> ", "{% include ")
        .replace("{%if", "{% if")
        .replace("{%each", "{% for")
        .replace("{%/each", "{% endfor")
        .replace("{%/if", "{% endif");
    
    Ok(converted)
} 