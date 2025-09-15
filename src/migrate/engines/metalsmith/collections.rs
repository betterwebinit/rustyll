use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_collections(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Metalsmith collections...");
    }
    
    // First try to find explicit collection directories or configuration
    let metalsmith_json = source_dir.join("metalsmith.json");
    let metalsmith_js = source_dir.join("metalsmith.js");
    let mut collection_patterns = Vec::new();
    
    // Try to extract collection patterns from metalsmith.json
    if metalsmith_json.exists() {
        if let Some(patterns) = extract_collection_patterns_from_json(&metalsmith_json)? {
            collection_patterns.extend(patterns);
        }
    } else if metalsmith_js.exists() {
        // Parsing JS is complex, so we'll look for common collection directories
        result.warnings.push("Found metalsmith.js configuration. Manual review needed for collections.".into());
    }
    
    // If no explicit collections found, look for common patterns
    if collection_patterns.is_empty() {
        // Common collection patterns used in Metalsmith
        collection_patterns.push("src/posts/**/*.md".to_string());
        collection_patterns.push("src/blog/**/*.md".to_string());
        collection_patterns.push("content/posts/**/*.md".to_string());
        collection_patterns.push("content/blog/**/*.md".to_string());
        collection_patterns.push("src/articles/**/*.md".to_string());
        collection_patterns.push("src/projects/**/*.md".to_string());
    }
    
    // Create the destination collections directories
    for pattern in &collection_patterns {
        let collection_name = extract_collection_name_from_pattern(pattern);
        let dest_collection_dir = dest_dir.join(format!("_{}", collection_name));
        create_dir_if_not_exists(&dest_collection_dir)?;
    }
    
    // Look for content that matches these patterns
    for pattern in &collection_patterns {
        let parts: Vec<&str> = pattern.split('/').collect();
        if parts.len() < 2 {
            continue;
        }
        
        let base_dir = parts[0]; // Usually 'src' or 'content'
        let collection_name = if parts.len() > 1 { parts[1] } else { "posts" };
        
        let source_base_dir = source_dir.join(base_dir);
        if !source_base_dir.exists() {
            continue;
        }
        
        let collection_dir = source_base_dir.join(collection_name);
        if !collection_dir.exists() {
            continue;
        }
        
        // Create destination collection directory
        let dest_collection_dir = dest_dir.join(format!("_{}", collection_name));
        
        // Process all Markdown and HTML files in this collection
        let collection_files = WalkDir::new(&collection_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                e.path().is_file() && (extension == "md" || extension == "markdown" || extension == "html")
            });
        
        for entry in collection_files {
            let relative_path = entry.path().strip_prefix(&collection_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read file {}: {}", entry.path().display(), e))?;
            
            // For posts, we want to format with Jekyll's date-based filenames
            let dest_file = if collection_name == "posts" || collection_name == "blog" || collection_name == "articles" {
                // Extract date from content or filename
                let date = extract_date(&file_content, entry.path())
                    .unwrap_or_else(|| "2023-01-01".to_string());
                
                let filename = relative_path.file_name().unwrap_or_default().to_string_lossy();
                dest_collection_dir.join(format!("{}-{}", date, filename))
            } else {
                // For other collections, keep the path structure
                dest_collection_dir.join(relative_path)
            };
            
            // Ensure parent directory exists
            if let Some(parent) = dest_file.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert to Jekyll collection format
            let jekyll_content = convert_to_jekyll_collection(&file_content, collection_name)?;
            
            fs::write(&dest_file, jekyll_content)
                .map_err(|e| format!("Failed to write file {}: {}", dest_file.display(), e))?;
            
            // Add to migration results
            result.changes.push(MigrationChange {
                file_path: format!("_{}/{}", collection_name, relative_path.display()),
                change_type: ChangeType::Converted,
                description: format!("Converted Metalsmith collection item in '{}'", collection_name),
            });
        }
        
        // Add collection configuration if needed
        ensure_collection_config(collection_name, dest_dir, result)?;
    }
    
    Ok(())
}

fn extract_collection_patterns_from_json(
    config_file: &Path,
) -> Result<Option<Vec<String>>, String> {
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read metalsmith.json: {}", e))?;
    
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse metalsmith.json: {}", e))?;
    
    let mut patterns = Vec::new();
    
    // Check for metalsmith-collections plugin
    if let Some(plugins) = config.get("plugins").and_then(|p| p.as_object()) {
        if let Some(collections_config) = plugins.get("metalsmith-collections").and_then(|c| c.as_object()) {
            for (name, config) in collections_config {
                if let Some(pattern) = config.get("pattern").and_then(|p| p.as_str()) {
                    patterns.push(pattern.to_string());
                } else {
                    // Default pattern for this collection
                    patterns.push(format!("src/{}/**/*.md", name));
                }
            }
            
            return Ok(Some(patterns));
        }
    }
    
    Ok(None)
}

fn extract_collection_name_from_pattern(pattern: &str) -> String {
    let parts: Vec<&str> = pattern.split('/').collect();
    if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "posts".to_string()
    }
}

fn extract_date(content: &str, path: &Path) -> Option<String> {
    // Try to extract date from frontmatter
    let date_regex = regex::Regex::new(r"(?m)^date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").ok()?;
    if let Some(cap) = date_regex.captures(content) {
        return Some(cap[1].to_string());
    }
    
    // Try to extract date from filename
    let filename = path.file_name()?.to_string_lossy();
    let filename_date_regex = regex::Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2})").ok()?;
    if let Some(cap) = filename_date_regex.captures(&filename) {
        return Some(cap[1].to_string());
    }
    
    // Try to extract date from path
    let path_str = path.to_string_lossy();
    let path_date_regex = regex::Regex::new(r"([0-9]{4})/([0-9]{2})/([0-9]{2})").ok()?;
    if let Some(cap) = path_date_regex.captures(&path_str) {
        let year = cap.get(1)?.as_str();
        let month = cap.get(2)?.as_str();
        let day = cap.get(3)?.as_str();
        
        return Some(format!("{}-{}-{}", year, month, day));
    }
    
    None
}

fn convert_to_jekyll_collection(content: &str, collection_name: &str) -> Result<String, String> {
    // Check if content already has frontmatter
    let has_frontmatter = content.starts_with("---");
    
    let mut converted = if has_frontmatter {
        content.to_string()
    } else {
        // Add Jekyll frontmatter
        format!("---\nlayout: {}\n---\n\n{}", collection_name.trim_end_matches('s'), content)
    };
    
    // If it has frontmatter but no layout, add a layout field
    if has_frontmatter && !converted.contains("layout:") {
        // Insert layout after the first --- line
        let parts: Vec<&str> = converted.splitn(2, "---").collect();
        if parts.len() > 1 {
            converted = format!("---\nlayout: {}\n{}", collection_name.trim_end_matches('s'), parts[1]);
        }
    }
    
    Ok(converted)
}

fn ensure_collection_config(
    collection_name: &str,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let config_path = dest_dir.join("_config.yml");
    let mut config_content = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read _config.yml: {}", e))?
    } else {
        "# Site settings\ntitle: Migrated Site\n".to_string()
    };
    
    // Check if collections section exists
    if !config_content.contains("collections:") {
        config_content.push_str("\n# Collections configuration\ncollections:\n");
        
        // Add this collection
        if collection_name != "posts" { // posts are handled specially in Jekyll
            config_content.push_str(&format!("  {}:\n    output: true\n", collection_name));
            
            fs::write(&config_path, &config_content)
                .map_err(|e| format!("Failed to update _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".into(),
                change_type: ChangeType::Modified,
                description: format!("Added '{}' collection configuration", collection_name),
            });
        }
    } else if !config_content.contains(&format!("  {}:", collection_name)) && collection_name != "posts" {
        // Find the collections section and add this collection
        let collections_regex = regex::Regex::new(r"(?m)^collections:").ok()
            .ok_or_else(|| "Failed to create regex".to_string())?;
        
        if let Some(pos) = collections_regex.find(&config_content) {
            let insert_pos = pos.end();
            config_content.insert_str(insert_pos, &format!("\n  {}:\n    output: true\n", collection_name));
            
            fs::write(&config_path, &config_content)
                .map_err(|e| format!("Failed to update _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".into(),
                change_type: ChangeType::Modified,
                description: format!("Added '{}' collection configuration", collection_name),
            });
        }
    }
    
    Ok(())
} 