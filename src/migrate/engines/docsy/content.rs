use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy content...");
    }

    // Create content directories
    let content_dir = dest_dir.join("content");
    fs::create_dir_all(&content_dir)
        .map_err(|e| format!("Failed to create content directory: {}", e))?;

    // Migrate content from Docsy content directory
    let source_content_dir = source_dir.join("content");
    if source_content_dir.exists() {
        migrate_markdown_files(&source_content_dir, &content_dir, verbose, result)?;
    }

    // Also check for content in /docs directory (alternative structure)
    let docs_dir = source_dir.join("docs");
    if docs_dir.exists() {
        migrate_markdown_files(&docs_dir, &content_dir, verbose, result)?;
    }

    Ok(())
}

fn migrate_markdown_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "md" || extension == "markdown" {
                let relative_path = path.strip_prefix(source_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = dest_dir.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                migrate_markdown_file(path, &dest_path, verbose, result)?;
            }
        }
    }

    Ok(())
}

fn migrate_markdown_file(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read markdown file: {}", e))?;

    // Convert Hugo/Docsy-specific markdown features to Jekyll/Liquid
    let converted_content = convert_docsy_markdown(&content)?;

    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write markdown file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("content/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted markdown file from {}", source_path.display()),
    });

    Ok(())
}

fn convert_docsy_markdown(content: &str) -> Result<String, String> {
    let mut converted = String::new();
    let mut in_frontmatter = false;
    let mut has_frontmatter = false;
    
    // Split the content by lines to process
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    // Check if there's front matter
    if lines.get(0) == Some(&"---") || lines.get(0) == Some(&"+++") {
        has_frontmatter = true;
        in_frontmatter = true;
        
        // Add the front matter delimiter
        converted.push_str("---\n");
        i += 1;
        
        let delimiter = if lines.get(0) == Some(&"---") { "---" } else { "+++" };
        
        // Process front matter content
        while i < lines.len() && lines[i] != delimiter {
            // Convert TOML/YAML keys if needed
            let line = convert_frontmatter_line(lines[i]);
            converted.push_str(&line);
            converted.push('\n');
            i += 1;
        }
        
        // Add layout if not present
        if !content.contains("layout:") {
            converted.push_str("layout: page\n");
        }
        
        // Close front matter
        converted.push_str("---\n\n");
        
        // Skip the closing delimiter
        if i < lines.len() {
            i += 1;
        }
        
        in_frontmatter = false;
    } else {
        // If no front matter is present, add a default one
        converted.push_str("---\nlayout: page\n---\n\n");
    }
    
    // Process the rest of the content
    while i < lines.len() {
        let line = convert_content_line(lines[i]);
        converted.push_str(&line);
        converted.push('\n');
        i += 1;
    }
    
    Ok(converted)
}

fn convert_frontmatter_line(line: &str) -> String {
    // Convert Hugo/Docsy front matter syntax to Jekyll
    if line.starts_with("title =") {
        line.replace("title =", "title:")
    } else if line.starts_with("description =") {
        line.replace("description =", "description:")
    } else if line.starts_with("date =") {
        line.replace("date =", "date:")
    } else if line.starts_with("weight =") {
        line.replace("weight =", "weight:")
    } else if line.starts_with("draft =") {
        line.replace("draft =", "published:")
           .replace("true", "false")
           .replace("false", "true")
    } else {
        line.to_string()
    }
}

fn convert_content_line(line: &str) -> String {
    // Convert Hugo/Docsy specific shortcodes to Liquid format
    let mut converted = line.to_string();
    
    // Convert Hugo shortcodes: {{< shortcode >}} -> {% include shortcode.html %}
    if converted.contains("{{<") && converted.contains(">}}") {
        converted = converted.replace("{{<", "{%").replace(">}}", "%}");
        
        // Extract shortcode name and replace with include
        let shortcode_pattern = "{%";
        if let Some(start_index) = converted.find(shortcode_pattern) {
            if let Some(end_index) = converted[start_index..].find("%}") {
                let shortcode_content = &converted[start_index + shortcode_pattern.len()..start_index + end_index];
                let shortcode_parts: Vec<&str> = shortcode_content.trim().split_whitespace().collect();
                
                if !shortcode_parts.is_empty() {
                    let shortcode_name = shortcode_parts[0];
                    let shortcode_include = format!("{{% include {}.html ", shortcode_name);
                    
                    converted = converted.replace(
                        &format!("{}{}{}", shortcode_pattern, shortcode_content, "%}"),
                        &format!("{}{}{}", shortcode_include, shortcode_content.trim()[shortcode_name.len()..].trim(), " %}")
                    );
                }
            }
        }
    }
    
    // Convert other shortcodes or syntax as needed
    
    converted
} 