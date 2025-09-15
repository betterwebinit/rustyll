use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::EleventyMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Eleventy, content can be in various directories
        // Common patterns include src/, content/, or at the root
        let potential_content_dirs = vec![
            source_dir.join("src"),
            source_dir.join("content"),
            source_dir.join("posts"),
            source_dir.join("blog"),
            source_dir.to_path_buf(), // Root directory
        ];
        
        // Common post directories in Eleventy
        let post_dirs = vec![
            "posts",
            "blog",
            "articles",
        ];
        
        // Track if we found any posts
        let mut found_posts = false;
        
        // Create posts destination directory
        let posts_dest_dir = dest_dir.join("_posts");
        create_dir_if_not_exists(&posts_dest_dir)?;
        
        // Check all potential content directories for posts
        for content_dir in &potential_content_dirs {
            if !content_dir.exists() || !content_dir.is_dir() {
                continue;
            }
            
            // Look for post directories
            for post_dir_name in &post_dirs {
                let post_dir = content_dir.join(post_dir_name);
                if post_dir.exists() && post_dir.is_dir() {
                    found_posts = true;
                    
                    if verbose {
                        log::info!("Migrating posts from {}", post_dir.display());
                    }
                    
                    // Process all markdown files in the post directory
                    for entry in WalkDir::new(&post_dir)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if entry.file_type().is_file() {
                            let file_path = entry.path();
                            
                            // Only process markdown and HTML files
                            if let Some(extension) = file_path.extension() {
                                if extension == "md" || extension == "markdown" || extension == "html" {
                                    // Get the file name
                                    let file_name = file_path.file_name()
                                        .ok_or_else(|| "Invalid file name".to_string())?
                                        .to_string_lossy()
                                        .to_string();
                                    
                                    // 11ty posts don't always have dates in filenames
                                    // We should extract date from front matter, but for this example,
                                    // we'll use a placeholder date if needed
                                    let dest_file_name = if file_name.starts_with(|c: char| c.is_ascii_digit()) 
                                                        && file_name.chars().nth(10) == Some('-') {
                                        // Filename already has a date pattern
                                        file_name
                                    } else {
                                        // Add placeholder date
                                        format!("2023-01-01-{}", file_name)
                                    };
                                    
                                    let dest_path = posts_dest_dir.join(&dest_file_name);
                                    
                                    // Copy and potentially transform the file
                                    copy_file(file_path, &dest_path)?;
                                    
                                    result.changes.push(MigrationChange {
                                        file_path: format!("_posts/{}", dest_file_name),
                                        change_type: ChangeType::Converted,
                                        description: "Post converted from Eleventy format".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            // Look for root-level markdown files (likely pages)
            for entry in fs::read_dir(content_dir)
                .map_err(|e| format!("Failed to read directory {}: {}", content_dir.display(), e))? {
                
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "md" || extension == "markdown" || extension == "html" {
                            let file_name = path.file_name()
                                .ok_or_else(|| "Invalid file name".to_string())?
                                .to_string_lossy()
                                .to_string();
                            
                            // Skip files in post directories we've already processed
                            let is_in_post_dir = post_dirs.iter().any(|dir| {
                                let dir_path = content_dir.join(dir);
                                path.starts_with(&dir_path)
                            });
                            
                            if is_in_post_dir {
                                continue;
                            }
                            
                            // Common pages like index, about, etc.
                            let dest_path = dest_dir.join(&file_name);
                            
                            if verbose {
                                log::info!("Migrating page: {}", file_name);
                            }
                            
                            // Copy page file
                            copy_file(&path, &dest_path)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: file_name,
                                change_type: ChangeType::Converted,
                                description: "Page file migrated from Eleventy".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Create README for posts directory
        let posts_readme = r#"# Posts Directory

This directory contains blog posts migrated from Eleventy.

## Post Format

Posts in Rustyll:
- Should have filenames with the format `YYYY-MM-DD-title.md`
- Front matter should be at the top of the file, enclosed in `---`
- Content follows the front matter

## Changes from Eleventy

- Eleventy uses various template engines, while Rustyll primarily uses Liquid
- Some templating features may need to be adjusted for compatibility
- Front matter variables might have different meanings or usage
"#;
        
        write_readme(&posts_dest_dir, posts_readme)?;
        
        if !found_posts {
            result.warnings.push(
                "Could not find a clear posts directory in the Eleventy site. Posts may need to be manually identified.".to_string()
            );
        }
        
        // Look for collections (Eleventy often uses custom collections)
        let collections_dir = source_dir.join("_data/collections");
        if collections_dir.exists() && collections_dir.is_dir() {
            if verbose {
                log::info!("Found collections directory in Eleventy site");
            }
            
            result.warnings.push(
                "Eleventy collections were found. Consider manually reviewing their migration to Rustyll collections.".to_string()
            );
        }
        
        Ok(())
    }
} 