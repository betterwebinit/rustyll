use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

// Public module function that can be called from anywhere
pub fn migrate_content(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let migrator = super::ZolaMigrator::new();
    migrator.migrate_content(source_dir, dest_dir, verbose, result)
}

impl super::ZolaMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let content_dir = source_dir.join("content");
        
        if !content_dir.exists() {
            result.warnings.push("No content directory found in Zola site".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating content from Zola to Rustyll format");
        }
        
        // Create destination directories
        let dest_posts_dir = dest_dir.join("_posts");
        let dest_pages_dir = dest_dir.join("_pages");
        
        create_dir_if_not_exists(&dest_posts_dir)?;
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        // Detect blog directories - common patterns in Zola
        let potential_blog_dirs = vec![
            content_dir.join("blog"),
            content_dir.join("posts"),
            content_dir.join("post"),
            content_dir.join("articles"),
            content_dir.join("article"),
            content_dir.join("news"),
        ];
        
        let mut found_blog = false;
        
        // First, check for dedicated blog directories
        for blog_dir in &potential_blog_dirs {
            if blog_dir.exists() && blog_dir.is_dir() {
                found_blog = true;
                
                if verbose {
                    log::info!("Found blog directory: {}", blog_dir.display());
                }
                
                process_blog_directory(blog_dir, &content_dir, &dest_posts_dir, result)?;
            }
        }
        
        // Now process all other content as pages
        process_other_content(&content_dir, &dest_pages_dir, &dest_posts_dir, &potential_blog_dirs, found_blog, result)?;
        
        Ok(())
    }
}

fn process_blog_directory(blog_dir: &Path, content_dir: &Path, dest_posts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Implementation for processing blog directory
    // This would convert Zola blog posts to Jekyll-compatible format
    
    // Process all markdown files in the blog directory
    for entry in WalkDir::new(blog_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Process markdown files
            if let Some(extension) = file_path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                
                if ext == "md" || ext == "markdown" {
                    // Process blog post
                    // For now, simply copy the file
                    let file_name = file_path.file_name().unwrap_or_default();
                    copy_file(file_path, &dest_posts_dir.join(file_name))?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("_posts/{}", file_name.to_string_lossy()),
                        change_type: ChangeType::Copied,
                        description: "Blog post from Zola site".to_string()
                    });
                }
            }
        }
    }
    
    Ok(())
}

fn process_other_content(content_dir: &Path, dest_pages_dir: &Path, dest_posts_dir: &Path, 
                        potential_blog_dirs: &Vec<PathBuf>, found_blog: bool, result: &mut MigrationResult) -> Result<(), String> {
    for entry in WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let entry_path = entry.path();
            
            // Skip if this is in a blog directory we already processed
            if potential_blog_dirs.iter().any(|dir| entry_path.starts_with(dir)) && found_blog {
                continue;
            }
            
            // Only process markdown and HTML files
            if let Some(extension) = entry_path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                
                if ext == "md" || ext == "markdown" || ext == "html" {
                    process_page(entry_path, content_dir, dest_pages_dir, dest_posts_dir, result)?;
                }
            }
        }
    }
    
    Ok(())
}

fn process_page(file_path: &Path, content_dir: &Path, dest_pages_dir: &Path, dest_posts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Get relative path from content directory
    let rel_path = file_path.strip_prefix(content_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Determine if this is a section index or regular page
    let file_name = file_path.file_name().unwrap_or_default();
    let is_index = file_name.to_string_lossy() == "_index.md" || file_name.to_string_lossy() == "index.md";
    
    // For simplicity, we'll just copy the file to the appropriate directory
    let dest_path = if is_index {
        // Convert section indices to pages with section name
        let section = rel_path.parent().unwrap_or_else(|| Path::new(""));
        let section_name = section.to_string_lossy().replace("/", "-");
        
        if section_name.is_empty() {
            dest_pages_dir.join("index.md")
        } else {
            dest_pages_dir.join(format!("{}-index.md", section_name))
        }
    } else {
        // Regular page - use the file name directly
        dest_pages_dir.join(file_name)
    };
    
    // Make sure the parent directory exists
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Copy the file
    copy_file(file_path, &dest_path)?;
    
    // Add to the migration result
    let rel_dest_path = dest_path.strip_prefix(dest_pages_dir.parent().unwrap_or(Path::new("")))
        .unwrap_or(dest_path.as_path())
        .to_string_lossy();
    
    result.changes.push(MigrationChange {
        file_path: rel_dest_path.to_string(),
        change_type: ChangeType::Copied,
        description: if is_index {
            "Section index page from Zola site".to_string()
        } else {
            "Content page from Zola site".to_string()
        }
    });
    
    Ok(())
} 