use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::NanocMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Nanoc, content is typically in the content/ directory
        let content_dir = source_dir.join("content");
        
        if !content_dir.exists() || !content_dir.is_dir() {
            result.warnings.push("Could not find Nanoc content directory.".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating content from Nanoc to Rustyll format");
        }
        
        // Create destination directories
        let dest_pages_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&dest_pages_dir)?;
        
        let dest_posts_dir = dest_dir.join("_posts");
        create_dir_if_not_exists(&dest_posts_dir)?;
        
        // Process content files
        for entry in WalkDir::new(&content_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip non-content files
                if is_content_file(file_path) {
                    self.migrate_content_file(file_path, &content_dir, dest_dir, &dest_pages_dir, &dest_posts_dir, verbose, result)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn migrate_content_file(&self, file_path: &Path, content_dir: &Path, dest_dir: &Path, 
                          dest_pages_dir: &Path, dest_posts_dir: &Path, 
                          verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Determine if this is a blog post or regular page
        let rel_path = file_path.strip_prefix(content_dir)
            .map_err(|_| "Failed to get relative path".to_string())?;
            
        let is_blog_post = rel_path.to_string_lossy().contains("/blog/") || 
                          rel_path.to_string_lossy().contains("/posts/") ||
                          rel_path.to_string_lossy().contains("/articles/");
                          
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
            
        // Extract metadata and content
        let (metadata, content_text) = extract_nanoc_metadata(&content);
        
        if is_blog_post {
            // Process as blog post
            self.process_blog_post(file_path, &content, &metadata, content_text, dest_posts_dir, result)?;
        } else {
            // Process as regular page
            self.process_page(file_path, rel_path, &content, &metadata, content_text, dest_pages_dir, result)?;
        }
        
        Ok(())
    }
    
    fn process_blog_post(&self, file_path: &Path, original_content: &str, metadata: &Vec<(String, String)>, 
                        content_text: &str, dest_posts_dir: &Path, 
                        result: &mut MigrationResult) -> Result<(), String> {
        // Extract title and date from metadata
        let title = metadata.iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                file_path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            
        // Try to find date
        let date = metadata.iter()
            .find(|(k, _)| k == "created_at" || k == "date")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "2023-01-01".to_string());
            
        // Format date for filename (YYYY-MM-DD)
        let date_for_filename = if date.len() >= 10 {
            date[0..10].to_string()
        } else {
            "2023-01-01".to_string()
        };
        
        // Create post filename
        let file_stem = file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy();
            
        let file_extension = file_path.extension()
            .unwrap_or_default()
            .to_string_lossy();
            
        let dest_filename = format!("{}-{}.{}", date_for_filename, file_stem, file_extension);
        let dest_path = dest_posts_dir.join(&dest_filename);
        
        // Create front matter
        let mut front_matter = String::from("---\n");
        front_matter.push_str(&format!("title: \"{}\"\n", title));
        front_matter.push_str(&format!("date: {}\n", date));
        front_matter.push_str("layout: post\n");
        
        // Add any other metadata
        for (key, value) in metadata {
            if key != "title" && key != "created_at" && key != "date" {
                front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        front_matter.push_str("---\n\n");
        
        // Write the converted file
        let final_content = format!("{}{}", front_matter, content_text);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write post file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", dest_filename),
            change_type: ChangeType::Converted,
            description: "Blog post converted from Nanoc format".to_string(),
        });
        
        Ok(())
    }
    
    fn process_page(&self, file_path: &Path, rel_path: &Path, original_content: &str, 
                  metadata: &Vec<(String, String)>, content_text: &str, 
                  dest_pages_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Extract title from metadata
        let title = metadata.iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                file_path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            
        // Create a suitable filename for the page
        let file_stem = file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy();
            
        let file_extension = file_path.extension()
            .unwrap_or_default()
            .to_string_lossy();
            
        // Handle index files specially
        let (dest_filename, permalink) = if file_stem == "index" {
            let parent_path = rel_path.parent().unwrap_or(Path::new(""));
            let parent_str = parent_path.to_string_lossy().to_string();
            
            if parent_str.is_empty() {
                // This is the root index
                (format!("index.{}", file_extension), "/".to_string())
            } else {
                // This is a section index
                (format!("{}-index.{}", parent_str.replace("/", "-"), file_extension),
                 format!("/{}/", parent_str))
            }
        } else {
            // Regular page
            let path_str = rel_path.with_extension("").to_string_lossy().to_string();
            (format!("{}.{}", path_str.replace("/", "-"), file_extension),
             format!("/{}/", path_str))
        };
        
        let dest_path = dest_pages_dir.join(&dest_filename);
        
        // Create front matter
        let mut front_matter = String::from("---\n");
        front_matter.push_str(&format!("title: \"{}\"\n", title));
        front_matter.push_str("layout: page\n");
        front_matter.push_str(&format!("permalink: {}\n", permalink));
        
        // Add any other metadata
        for (key, value) in metadata {
            if key != "title" {
                front_matter.push_str(&format!("{}: {}\n", key, value));
            }
        }
        front_matter.push_str("---\n\n");
        
        // Write the converted file
        let final_content = format!("{}{}", front_matter, content_text);
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write page file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_pages/{}", dest_filename),
            change_type: ChangeType::Converted,
            description: "Page converted from Nanoc format".to_string(),
        });
        
        Ok(())
    }
}

// Helper functions

fn is_content_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        ext == "md" || ext == "markdown" || ext == "html" || ext == "haml" || ext == "erb"
    } else {
        false
    }
}

fn extract_nanoc_metadata(content: &str) -> (Vec<(String, String)>, &str) {
    let mut metadata = Vec::new();
    
    // Nanoc metadata can be in YAML or attribute format
    
    // Check for YAML front matter
    if content.starts_with("---\n") {
        if let Some(end_index) = content.find("\n---\n") {
            let yaml_content = &content[4..end_index];
            let content_text = &content[end_index + 5..];
            
            // Parse YAML
            for line in yaml_content.lines() {
                if let Some(colon_index) = line.find(':') {
                    let key = line[0..colon_index].trim().to_string();
                    let value = line[colon_index + 1..].trim().to_string();
                    metadata.push((key, value));
                }
            }
            
            return (metadata, content_text);
        }
    }
    
    // Check for Nanoc attribute format
    let mut content_start = 0;
    let re = Regex::new(r"(?m)^([a-zA-Z_]+):\s*(.+)$").unwrap();
    
    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() && line_num > 0 {
            // First empty line marks the end of attributes
            content_start = content.find(line).unwrap() + line.len() + 1;
            break;
        }
        
        if let Some(caps) = re.captures(line) {
            if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                metadata.push((key.as_str().to_string(), value.as_str().to_string()));
            }
        }
    }
    
    let content_text = if content_start > 0 {
        &content[content_start..]
    } else {
        content
    };
    
    (metadata, content_text)
} 