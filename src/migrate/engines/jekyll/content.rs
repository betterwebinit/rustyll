use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::JekyllMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Process posts
        let posts_source_dir = source_dir.join("_posts");
        let posts_dest_dir = dest_dir.join("_posts");
        
        if posts_source_dir.exists() {
            if verbose {
                log::info!("Migrating posts");
            }
            
            create_dir_if_not_exists(&posts_dest_dir)?;
            
            // Iterate through all posts
            for entry in fs::read_dir(posts_source_dir)
                .map_err(|e| format!("Failed to read posts directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    let file_name = path.file_name()
                        .ok_or_else(|| "Invalid file name".to_string())?
                        .to_string_lossy()
                        .to_string();
                    
                    let dest_path = posts_dest_dir.join(&file_name);
                    
                    // Copy post file
                    copy_file(&path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("_posts/{}", file_name),
                        change_type: ChangeType::Converted,
                        description: "Post file migrated".to_string(),
                    });
                }
            }
            
            // Create README for posts directory
            let posts_readme = r#"# Posts Directory

This directory contains all your blog posts migrated from Jekyll.

## Post Format

Posts in Rustyll follow a similar format to Jekyll:
- Files are named with the format `YYYY-MM-DD-title.md`
- Front matter is at the top of the file, enclosed in `---`
- Content follows the front matter

## Changes from Jekyll

- Some front matter variables might have different names in Rustyll
- Liquid tag syntax might be slightly different
"#;
            
            write_readme(&posts_dest_dir, posts_readme)?;
        }
        
        // Process pages
        let pages = vec![
            source_dir.join("index.md"),
            source_dir.join("index.html"),
            source_dir.join("about.md"),
            source_dir.join("about.html"),
        ];
        
        for page in pages {
            if page.exists() {
                let file_name = page.file_name()
                    .ok_or_else(|| "Invalid file name".to_string())?
                    .to_string_lossy()
                    .to_string();
                
                let dest_path = dest_dir.join(&file_name);
                
                if verbose {
                    log::info!("Migrating page: {}", file_name);
                }
                
                // Copy page file
                copy_file(&page, &dest_path)?;
                
                result.changes.push(MigrationChange {
                    file_path: file_name,
                    change_type: ChangeType::Converted,
                    description: "Page file migrated".to_string(),
                });
            }
        }
        
        // Process collections (other than posts)
        for entry in fs::read_dir(source_dir)
            .map_err(|e| format!("Failed to read source directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name()
                    .ok_or_else(|| "Invalid directory name".to_string())?
                    .to_string_lossy();
                
                // Check if this is a collection (starts with _)
                if dir_name.starts_with('_') && dir_name != "_posts" && dir_name != "_layouts" 
                   && dir_name != "_includes" && dir_name != "_data" && dir_name != "_site" 
                   && dir_name != "_sass" && dir_name != "_drafts" {
                    
                    let dest_collection_dir = dest_dir.join(&*dir_name);
                    
                    if verbose {
                        log::info!("Migrating collection: {}", dir_name);
                    }
                    
                    create_dir_if_not_exists(&dest_collection_dir)?;
                    
                    // Copy all files in the collection
                    for file_entry in WalkDir::new(&path)
                        .min_depth(1)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if file_entry.file_type().is_file() {
                            let file_path = file_entry.path();
                            let rel_path = file_path.strip_prefix(&path)
                                .map_err(|_| "Failed to get relative path".to_string())?;
                            
                            let dest_file_path = dest_collection_dir.join(rel_path);
                            
                            // Create parent directory if needed
                            if let Some(parent) = dest_file_path.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Copy the file
                            copy_file(file_path, &dest_file_path)?;
                            
                            let file_path_str = format!("{}/{}", dir_name, rel_path.to_string_lossy());
                            result.changes.push(MigrationChange {
                                file_path: file_path_str,
                                change_type: ChangeType::Converted,
                                description: "Collection file migrated".to_string(),
                            });
                        }
                    }
                    
                    // Create README for the collection
                    let collection_readme = format!(r#"# {} Collection

This directory contains files migrated from the Jekyll {} collection.

## Collection Usage

Collections in Rustyll:
- Can be accessed through the `site.{}` variable in templates
- Files can have front matter and content
- Can be used to organize content that doesn't fit into posts or pages
"#, dir_name[1..].to_string(), dir_name, dir_name[1..].to_string());
                    
                    write_readme(&dest_collection_dir, &collection_readme)?;
                }
            }
        }
        
        Ok(())
    }
} 