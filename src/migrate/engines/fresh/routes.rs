use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_routes(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Fresh routes...");
    }

    // In Fresh, routes are in routes/ directory
    let routes_dir = source_dir.join("routes");
    if !routes_dir.exists() || !routes_dir.is_dir() {
        result.warnings.push("No routes directory found.".into());
        return Ok(());
    }

    // Create destination pages directory
    let pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&pages_dir)?;

    // Migrate routes
    for entry in WalkDir::new(&routes_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip non-TypeScript/JavaScript files
            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension != "tsx" && extension != "jsx" && extension != "ts" && extension != "js" {
                continue;
            }
            
            // Get the relative path from the routes directory
            let rel_path = file_path.strip_prefix(&routes_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create the destination path
            // Convert route paths to Jekyll paths
            let dest_path = convert_route_to_page_path(&pages_dir, rel_path)?;
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert the route file to a Jekyll page
            migrate_route_file(file_path, &dest_path, result)?;
        }
    }
    
    Ok(())
}

fn convert_route_to_page_path(pages_dir: &Path, route_path: &Path) -> Result<PathBuf, String> {
    // Extract the route path components
    let file_name = route_path.file_name().unwrap().to_string_lossy();
    let file_stem = route_path.file_stem().unwrap().to_string_lossy();
    let parent_path = route_path.parent().unwrap_or(Path::new(""));
    
    // Handle special route file names in Fresh
    let new_path = if file_stem == "index" {
        if parent_path.as_os_str().is_empty() {
            // Root index.tsx becomes index.html
            pages_dir.join("index.md")
        } else {
            // Nested index.tsx becomes directory/index.html
            pages_dir.join(parent_path).join("index.md")
        }
    } else if file_stem == "_404" {
        // 404 page
        pages_dir.join("404.md")
    } else if file_stem.starts_with('_') {
        // Other special files
        pages_dir.join(format!("{}.md", &file_stem[1..]))
    } else if file_stem.starts_with('[') && file_stem.ends_with(']') {
        // Dynamic route like [id].tsx becomes params-id.md
        let param_name = &file_stem[1..file_stem.len()-1];
        pages_dir.join(parent_path).join(format!("params-{}.md", param_name))
    } else {
        // Regular route
        pages_dir.join(parent_path).join(format!("{}.md", file_stem))
    };
    
    Ok(new_path)
}

fn migrate_route_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the route file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read route file {}: {}", source_path.display(), e))?;
    
    // Extract page title from the content if possible
    let title_regex = Regex::new(r#"<title>(.*?)</title>"#).unwrap();
    let title = title_regex.captures(&content)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| {
            // If no title tag, try to infer from file name
            let file_stem = source_path.file_stem().unwrap().to_string_lossy();
            let parts: Vec<&str> = file_stem.split(|c: char| !c.is_alphanumeric()).collect();
            
            parts.iter()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
        });
    
    // Try to extract the main component content
    let component_regex = Regex::new(r#"export default function.*?\{([\s\S]*?)\}|\(\) => \{([\s\S]*?)\}"#).unwrap();
    let component_content = component_regex.captures(&content)
        .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str().trim().to_string()))
        .unwrap_or_else(|| "Content extracted from Fresh component".to_string());
    
    // Create the Jekyll page with front matter
    let page_content = format!(r#"---
layout: default
title: "{}"
permalink: "{}"
---

# {}

{}

<div class="note">
  <p><strong>Note:</strong> This page was automatically migrated from a Fresh route. 
  The original React/Preact component has been converted to static HTML.
  You may need to adjust the content manually.</p>
</div>
"#, 
        title,
        get_permalink_from_path(dest_path, &dest_path.parent().unwrap().parent().unwrap())?,
        title,
        component_content.replace("return (", "").replace(");", "")
    );
    
    // Write the Jekyll page
    fs::write(dest_path, page_content)
        .map_err(|e| format!("Failed to write page file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_pages/{}", dest_path.strip_prefix(&dest_path.parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Converted Fresh route from {}", source_path.display()),
    });
    
    Ok(())
}

fn get_permalink_from_path(file_path: &Path, pages_root_dir: &Path) -> Result<String, String> {
    let rel_path = file_path.strip_prefix(pages_root_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    let rel_path_string = rel_path.to_string_lossy();
    let parts: Vec<&str> = rel_path_string.split('/').collect();
    let last_part = parts.last().unwrap();
    
    if *last_part == "index.md" {
        // For index files, permalink is the directory path
        if parts.len() == 1 {
            Ok("/".to_string())
        } else {
            Ok(format!("/{}/", parts[..parts.len()-1].join("/")))
        }
    } else {
        // For regular files, permalink is the path without extension
        let file_stem = file_path.file_stem().unwrap().to_string_lossy();
        
        if parts.len() == 1 {
            Ok(format!("/{}/", file_stem))
        } else {
            let dir_path = parts[..parts.len()-1].join("/");
            Ok(format!("/{}/{}/", dir_path, file_stem))
        }
    }
} 