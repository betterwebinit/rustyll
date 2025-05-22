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
        log::info!("Migrating Jigsaw content...");
    }
    
    let content_dir = source_dir.join("source");
    if !content_dir.exists() {
        result.warnings.push("Could not find Jigsaw content directory (source)".into());
        return Ok(());
    }
    
    // Create destination content directory
    let dest_content_dir = dest_dir.join("_posts");
    create_dir_if_not_exists(&dest_content_dir)?;
    
    // Create destination pages directory
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;
    
    // Process Markdown files in the source directory
    let markdown_files = WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            extension == "md" || extension == "markdown"
        });
    
    for entry in markdown_files {
        let relative_path = entry.path().strip_prefix(&content_dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        let file_content = fs::read_to_string(entry.path())
            .map_err(|e| format!("Failed to read file {}: {}", entry.path().display(), e))?;
        
        // Determine if it's a post or a page based on path/content
        let (dest_file, is_post) = if file_content.contains("date:") || relative_path.starts_with("_posts") {
            // It's a post - create Jekyll-style filename with date
            let date = extract_date_from_content(&file_content).unwrap_or_else(|| "2023-01-01".to_string());
            let filename = relative_path.file_name().unwrap_or_default().to_string_lossy();
            (dest_content_dir.join(format!("{}-{}", date, filename)), true)
        } else {
            // It's a page
            (dest_pages_dir.join(relative_path), false)
        };
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Convert content to Jekyll format
        let jekyll_content = convert_jigsaw_to_jekyll(&file_content)?;
        
        fs::write(&dest_file, jekyll_content)
            .map_err(|e| format!("Failed to write file {}: {}", dest_file.display(), e))?;
        
        // Add to migration results
        let file_type = if is_post { "post" } else { "page" };
        result.changes.push(MigrationChange {
            file_path: format!("{}", dest_file.strip_prefix(dest_dir).unwrap_or(relative_path).display()),
            change_type: ChangeType::Converted,
            description: format!("Converted Jigsaw {} to Jekyll format", file_type),
        });
    }
    
    Ok(())
}

// Extract date from frontmatter or filename
fn extract_date_from_content(content: &str) -> Option<String> {
    // Check for date in frontmatter
    let date_regex = regex::Regex::new(r"date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").ok()?;
    if let Some(cap) = date_regex.captures(content) {
        return Some(cap[1].to_string());
    }
    
    None
}

// Convert Jigsaw markdown to Jekyll format
fn convert_jigsaw_to_jekyll(content: &str) -> Result<String, String> {
    // Identify frontmatter
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    
    // Ensure proper Jekyll frontmatter with triple dashes
    if !content.starts_with("---") {
        lines.insert(0, "---".to_string());
        
        // Check if there's any existing frontmatter to end
        let mut has_frontmatter_end = false;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 && line == "---" {
                has_frontmatter_end = true;
                break;
            }
        }
        
        if !has_frontmatter_end {
            lines.insert(1, "layout: default".to_string());
            lines.insert(2, "---".to_string());
        }
    }
    
    // Replace Jigsaw-specific template syntax with Liquid
    let mut converted = lines.join("\n");
    
    // Replace Blade template syntax with Liquid
    converted = converted.replace("@section", "{% block")
        .replace("@endsection", "{% endblock %}")
        .replace("@yield", "{{ content }}")
        .replace("{{ $", "{{ ")
        .replace("{!! $", "{{ ")
        .replace(" }}", " }}");
    
    Ok(converted)
} 