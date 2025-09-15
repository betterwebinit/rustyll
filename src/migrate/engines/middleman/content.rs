use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::MiddlemanMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Middleman, content is typically in the source/ directory
        let source_content_dir = source_dir.join("source");
        
        if !source_content_dir.exists() || !source_content_dir.is_dir() {
            result.warnings.push("Could not find standard Middleman source directory.".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating content from {}", source_content_dir.display());
        }
        
        // Create posts destination directory
        let posts_dest_dir = dest_dir.join("_posts");
        create_dir_if_not_exists(&posts_dest_dir)?;
        
        // Check for various blog post locations in Middleman projects
        let potential_blog_dirs = vec![
            source_content_dir.join("blog"),
            source_content_dir.join("posts"),
            source_content_dir.join("articles"),
        ];
        
        let mut found_posts = false;
        
        // Process blog posts
        for blog_dir in &potential_blog_dirs {
            if blog_dir.exists() && blog_dir.is_dir() {
                found_posts = true;
                
                if verbose {
                    log::info!("Found blog posts in {}", blog_dir.display());
                }
                
                // Process all Markdown/HTML files in the blog directory
                for entry in WalkDir::new(blog_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        let file_name = entry.file_name().to_string_lossy();
                        
                        // Only process content files
                        if let Some(extension) = file_path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            
                            if ["md", "markdown", "html", "erb", "html.erb", "md.erb"].contains(&ext.as_ref()) {
                                // Check if file is a blog post (has date pattern or is in yyyy/mm/dd structure)
                                let relative_path = file_path.strip_prefix(blog_dir)
                                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                                
                                // Handle Middleman posts which can be in various formats
                                let dest_file_name = if file_name.starts_with(|c: char| c.is_ascii_digit()) 
                                                    && file_name.chars().nth(10) == Some('-') {
                                    // Already has a date pattern like YYYY-MM-DD-title.md
                                    file_name.to_string()
                                } else {
                                    // Check if in directory structure like /2022/01/15/post.md
                                    let parent_components: Vec<_> = relative_path.parent()
                                        .map(|p| p.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect())
                                        .unwrap_or_default();
                                    
                                    if parent_components.len() >= 3 
                                       && parent_components[0].chars().all(|c| c.is_ascii_digit())
                                       && parent_components[1].chars().all(|c| c.is_ascii_digit())
                                       && parent_components[2].chars().all(|c| c.is_ascii_digit()) {
                                        // Extract date from directory structure
                                        format!("{}-{}-{}-{}", 
                                            parent_components[0], 
                                            parent_components[1], 
                                            parent_components[2], 
                                            file_name)
                                    } else {
                                        // Use placeholder date
                                        format!("2023-01-01-{}", file_name)
                                    }
                                };
                                
                                // Clean the extension to a standard format
                                let final_file_name = if dest_file_name.ends_with(".html.erb") {
                                    dest_file_name.replace(".html.erb", ".html")
                                } else if dest_file_name.ends_with(".md.erb") {
                                    dest_file_name.replace(".md.erb", ".md")
                                } else {
                                    dest_file_name
                                };
                                
                                let dest_path = posts_dest_dir.join(&final_file_name);
                                
                                // Copy post file, possibly with ERB transformation
                                // This is simplified - a real implementation would convert ERB to Liquid
                                if ext == "erb" || ext == "html.erb" || ext == "md.erb" {
                                    // For ERB files, we need to convert ERB tags to Liquid
                                    let content = fs::read_to_string(file_path)
                                        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                                    
                                    // Very basic ERB to Liquid conversion - would need to be more sophisticated
                                    let converted_content = content
                                        .replace("<%= ", "{{ ")
                                        .replace("<% ", "{% ")
                                        .replace(" %>", " %}")
                                        .replace(" -%>", " %}");
                                    
                                    fs::write(&dest_path, converted_content)
                                        .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                    
                                    result.changes.push(MigrationChange {
                                        file_path: format!("_posts/{}", final_file_name),
                                        change_type: ChangeType::Converted,
                                        description: "ERB template converted to Liquid".to_string(),
                                    });
                                } else {
                                    // For Markdown and HTML files, just copy
                                    copy_file(file_path, &dest_path)?;
                                    
                                    result.changes.push(MigrationChange {
                                        file_path: format!("_posts/{}", final_file_name),
                                        change_type: ChangeType::Converted,
                                        description: "Post converted from Middleman format".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Process regular pages (typically in source/ root)
        for entry in WalkDir::new(&source_content_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let file_name = entry.file_name().to_string_lossy();
                
                // Skip files we've already processed and non-content files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if ["md", "markdown", "html", "erb", "html.erb", "md.erb"].contains(&ext.as_ref()) {
                        // Don't process files that are in layouts, partials, etc.
                        if file_name.starts_with('_') {
                            continue;
                        }
                        
                        // Clean the extension to a standard format
                        let final_file_name = if file_name.ends_with(".html.erb") {
                            file_name.replace(".html.erb", ".html")
                        } else if file_name.ends_with(".md.erb") {
                            file_name.replace(".md.erb", ".md")
                        } else {
                            file_name.to_string()
                        };
                        
                        let dest_path = dest_dir.join(&final_file_name);
                        
                        if verbose {
                            log::info!("Migrating page: {}", final_file_name);
                        }
                        
                        // Copy or convert the page file
                        if ext == "erb" || ext == "html.erb" || ext == "md.erb" {
                            // For ERB files, we need to convert ERB tags to Liquid
                            let content = fs::read_to_string(file_path)
                                .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                            
                            // Very basic ERB to Liquid conversion
                            let converted_content = content
                                .replace("<%= ", "{{ ")
                                .replace("<% ", "{% ")
                                .replace(" %>", " %}")
                                .replace(" -%>", " %}");
                            
                            fs::write(&dest_path, converted_content)
                                .map_err(|e| format!("Failed to write converted file: {}", e))?;
                            
                            result.changes.push(MigrationChange {
                                file_path: final_file_name.to_string(),
                                change_type: ChangeType::Converted,
                                description: "ERB template converted to Liquid".to_string(),
                            });
                        } else {
                            // For Markdown and HTML files, just copy
                            copy_file(file_path, &dest_path)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: final_file_name.to_string(),
                                change_type: ChangeType::Converted,
                                description: "Page converted from Middleman format".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Create README for posts directory
        let posts_readme = r#"# Posts Directory

This directory contains blog posts migrated from Middleman.

## Post Format

Posts in Rustyll:
- Should have filenames with the format `YYYY-MM-DD-title.md`
- Front matter should be at the top of the file, enclosed in `---`
- Content follows the front matter

## Changes from Middleman

- Middleman often uses ERB templates, which have been converted to Liquid
- Some Ruby code in templates might need manual adjustment
- Front matter should now be in YAML format
"#;
        
        write_readme(&posts_dest_dir, posts_readme)?;
        
        if !found_posts {
            result.warnings.push(
                "Could not find a clear blog posts directory in the Middleman site. Posts may need to be manually identified.".to_string()
            );
        }
        
        Ok(())
    }
} 