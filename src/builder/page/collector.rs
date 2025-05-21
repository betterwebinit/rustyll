use std::fs;
use std::path::Path;

use log::{debug, error};
use walkdir::WalkDir;

use crate::directory::DirectoryStructure;
use crate::front_matter::FrontMatter;
use crate::builder::types::BoxResult;
use crate::builder::page::model::Page;
use crate::builder::page::utils::determine_output_path;

/// Collect all pages from the site content directory
pub fn collect_pages(dirs: &DirectoryStructure) -> BoxResult<Vec<Page>> {
    debug!("Collecting pages...");
    let mut pages = Vec::new();
    
    // Create source path iterator with excluded paths
    let walker = WalkDir::new(&dirs.source)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_excluded_path(e.path(), dirs));
    
    // Process each file
    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() {
            // Skip files in _site, _posts, _drafts, _includes, _layouts, etc.
            if is_excluded_path(path, dirs) {
                continue;
            }
            
            // Get relative path
            let relative_path = path.strip_prefix(&dirs.source).unwrap_or(path).to_path_buf();
            
            // Determine if this file should be processed or just copied
            let (process, content, front_matter) = if is_processable_file(path) {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        // Extract front matter
                        match crate::front_matter::extract_front_matter(&content) {
                            Ok((front_matter, content)) => {
                                (true, content, front_matter)
                            },
                            Err(e) => {
                                error!("Error extracting front matter from {}: {}", path.display(), e);
                                // Just copy the file if front matter extraction fails
                                (false, content, FrontMatter::default())
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error reading file {}: {}", path.display(), e);
                        continue;
                    }
                }
            } else {
                // Don't process binary or non-text files
                (false, String::new(), FrontMatter::default())
            };
            
            // Determine output path
            let output_path = determine_output_path(path, &relative_path, &front_matter, dirs);
            
            // Create URL
            let url = if let Some(output) = &output_path {
                let url_path = output.strip_prefix(&dirs.destination).unwrap_or(output);
                let url_str = format!("/{}", url_path.to_string_lossy());
                Some(url_str)
            } else {
                None
            };
            
            // Create page
            let page = Page {
                path: path.to_path_buf(),
                relative_path,
                output_path,
                url,
                date: front_matter.get_date(),
                content,
                front_matter,
                process,
            };
            
            pages.push(page);
        }
    }
    
    debug!("Collected {} pages", pages.len());
    
    Ok(pages)
}

/// Check if a path should be excluded from processing
fn is_excluded_path(path: &Path, dirs: &DirectoryStructure) -> bool {
    // Skip files in _site, _posts, _drafts, _includes, _layouts, etc.
    let excluded_dirs = [
        &dirs.destination,
        &dirs.posts_dir,
        &dirs.drafts_dir,
        &dirs.includes_dir,
        &dirs.layouts_dir,
        &dirs.data_dir,
    ];
    
    for dir in &excluded_dirs {
        if path.starts_with(dir) {
            return true;
        }
    }
    
    // Skip files/directories that start with underscore or dot
    if let Some(file_name) = path.file_name() {
        let name = file_name.to_string_lossy();
        if name.starts_with('_') || name.starts_with('.') {
            return true;
        }
    }
    
    false
}

/// Check if a file should be processed with liquid/markdown
fn is_processable_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        
        // Process HTML, Markdown, XML, and other text-based files
        return match ext_str.as_ref() {
            "html" | "htm" | "md" | "markdown" | "xml" | "txt" | "yml" | "yaml" | "json" => true,
            _ => false,
        };
    }
    
    false
} 