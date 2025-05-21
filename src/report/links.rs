use std::path::Path;
use std::fs;
use std::collections::HashSet;
use regex::Regex;

pub async fn check_broken_links(source_dir: &Path, verbose: bool) -> Result<Vec<String>, String> {
    if verbose {
        log::info!("Checking for broken links in {}...", source_dir.display());
    }
    
    let mut broken_links = Vec::new();
    let mut all_files = HashSet::new();
    
    // First collect all files to check against relative links
    collect_all_files(source_dir, &mut all_files)?;
    
    // Then scan for broken links
    scan_directory_for_broken_links(source_dir, source_dir, &all_files, &mut broken_links, verbose)?;
    
    if verbose {
        log::info!("Broken links check completed, found {} broken links", broken_links.len());
    }
    
    Ok(broken_links)
}

fn collect_all_files(dir: &Path, all_files: &mut HashSet<String>) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            collect_all_files(&path, all_files)?;
        } else if path.is_file() {
            if let Ok(relative_path) = path.strip_prefix(dir) {
                all_files.insert(relative_path.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(())
}

fn scan_directory_for_broken_links(
    base_dir: &Path,
    dir: &Path,
    all_files: &HashSet<String>,
    broken_links: &mut Vec<String>,
    verbose: bool
) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_broken_links(base_dir, &path, all_files, broken_links, verbose)?;
        } else if path.is_file() {
            // Check if it's an HTML file
            if let Some(extension) = path.extension() {
                if extension == "html" {
                    check_file_for_broken_links(base_dir, &path, all_files, broken_links, verbose)?;
                }
            }
        }
    }
    
    Ok(())
}

fn check_file_for_broken_links(
    base_dir: &Path,
    file_path: &Path,
    all_files: &HashSet<String>,
    broken_links: &mut Vec<String>,
    verbose: bool
) -> Result<(), String> {
    if verbose {
        log::info!("Checking links in {}", file_path.display());
    }
    
    // Read the HTML file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    // Create a regex to find href and src attributes
    let href_regex = Regex::new(r#"(href|src)=["']([^"']+)["']"#)
        .map_err(|e| format!("Failed to create regex: {}", e))?;
    
    for cap in href_regex.captures_iter(&content) {
        if let Some(url) = cap.get(2) {
            let url = url.as_str();
            
            // Skip external URLs, anchors, and javascript
            if url.starts_with("http://") || url.starts_with("https://") || 
               url.starts_with("#") || url.starts_with("mailto:") ||
               url.starts_with("tel:") || url.starts_with("javascript:") {
                continue;
            }
            
            // Check if the relative URL exists
            let mut url_path = url.to_string();
            
            // Remove query parameters and fragments
            if let Some(pos) = url_path.find('?') {
                url_path.truncate(pos);
            }
            if let Some(pos) = url_path.find('#') {
                url_path.truncate(pos);
            }
            
            // Check if the file exists
            if !url_path.is_empty() && !all_files.contains(&url_path) {
                broken_links.push(format!("Broken link in {}: {}", file_path.display(), url));
            }
        }
    }
    
    Ok(())
} 