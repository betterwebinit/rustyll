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
        log::info!("Migrating Slate content...");
    }

    // Create destination directories
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;

    // In Slate, content is typically in source/index.html.md
    let index_path = source_dir.join("source/index.html.md");
    if index_path.exists() {
        migrate_index_file(&index_path, dest_dir, result)?;
    } else {
        // Try alternative paths
        let alt_paths = [
            source_dir.join("source/index.md"),
            source_dir.join("index.html.md"),
            source_dir.join("index.md"),
        ];
        
        let mut found = false;
        for path in alt_paths {
            if path.exists() {
                migrate_index_file(&path, dest_dir, result)?;
                found = true;
                break;
            }
        }
        
        if !found {
            result.warnings.push("No main index file found in Slate source.".into());
        }
    }

    // Look for additional content files in source directory
    let source_content_dir = source_dir.join("source");
    if source_content_dir.exists() && source_content_dir.is_dir() {
        for entry in WalkDir::new(&source_content_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip the main index file as it's already handled
                if file_path == index_path {
                    continue;
                }
                
                // Process markdown files
                if is_markdown_file(file_path) {
                    migrate_content_file(file_path, &source_content_dir, &dest_pages_dir, result)?;
                }
            }
        }
    }

    Ok(())
}

fn migrate_index_file(
    file_path: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the index file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read index file {}: {}", file_path.display(), e))?;
    
    // Process the content
    let processed_content = process_slate_content(&content);
    
    // Write to the destination index file
    let dest_index_path = dest_dir.join("index.md");
    fs::write(&dest_index_path, processed_content)
        .map_err(|e| format!("Failed to write index file: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: "index.md".into(),
        description: format!("Converted main index file from {}", file_path.display()),
    });
    
    Ok(())
}

fn migrate_content_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the content file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
    
    // Process the content
    let processed_content = process_slate_content(&content);
    
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path
    let mut dest_path = dest_dir.join(rel_path);
    
    // Ensure it has .md extension
    if let Some(ext) = dest_path.extension() {
        if ext != "md" {
            let stem = dest_path.file_stem().unwrap().to_string_lossy();
            dest_path = dest_path.with_file_name(format!("{}.md", stem));
        }
    } else {
        let filename = dest_path.file_name().unwrap().to_string_lossy();
        dest_path = dest_path.with_file_name(format!("{}.md", filename));
    }
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, processed_content)
        .map_err(|e| format!("Failed to write content file {}: {}", dest_path.display(), e))?;
    
    // Add to changes
    let rel_dest_path = format!("_pages/{}", dest_path.file_name().unwrap().to_string_lossy());
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: rel_dest_path,
        description: format!("Converted content from {}", file_path.display()),
    });
    
    Ok(())
}

fn process_slate_content(content: &str) -> String {
    let mut processed = content.to_string();
    
    // Extract YAML front matter if present
    let has_front_matter = content.starts_with("---");
    let mut front_matter = String::from("---\n");
    let mut body = content.to_string();
    
    if has_front_matter {
        if let Some(end_index) = content.find("---\n") {
            if let Some(second_end) = content[end_index + 4..].find("---\n") {
                let original_front_matter = &content[..end_index + 4 + second_end + 4];
                
                // Extract and adjust front matter
                let front_matter_lines: Vec<&str> = original_front_matter.lines().collect();
                
                front_matter = "---\n".to_string();
                front_matter.push_str("layout: page\n");
                
                for line in front_matter_lines {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed != "---" {
                        // Add the line with adjustments if needed
                        if trimmed.starts_with("title:") || 
                           trimmed.starts_with("language_tabs:") || 
                           trimmed.starts_with("toc_footers:") {
                            front_matter.push_str(&format!("{}\n", trimmed));
                        }
                    }
                }
                
                front_matter.push_str("---\n\n");
                
                // Update body to exclude front matter
                body = content[end_index + 4 + second_end + 4..].to_string();
            }
        }
    } else {
        // If no front matter exists, add a minimal one
        front_matter = "---\nlayout: page\ntitle: Slate Documentation\n---\n\n".to_string();
    }
    
    // Combine adjusted front matter with body
    processed = format!("{}{}", front_matter, body);
    
    // Process code blocks
    let code_block_regex = regex::Regex::new(r"```([a-z]+)\n([\s\S]*?)\n```").unwrap();
    processed = code_block_regex.replace_all(&processed, |caps: &regex::Captures| {
        let language = &caps[1];
        let code = &caps[2];
        
        format!("```{}\n{}\n```", language, code)
    }).to_string();
    
    // Handle Slate-specific syntax (if any)
    // Slate uses standard Markdown for the most part
    
    processed
}

fn is_markdown_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "md" || ext_str == "markdown" || path.to_string_lossy().ends_with(".html.md")
    } else {
        false
    }
} 