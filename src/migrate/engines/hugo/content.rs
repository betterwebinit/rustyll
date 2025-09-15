use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use chrono::NaiveDate;
use yaml_front_matter::{YamlFrontMatter, Document};
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, write_readme};

impl super::HugoMigrator {
    pub(super) fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let content_source_dir = source_dir.join("content");
        
        if !content_source_dir.exists() {
            result.warnings.push("Hugo content directory not found. No content to migrate.".to_string());
            return Ok(());
        }
        
        if verbose {
            log::info!("Migrating content from Hugo to Rustyll format");
        }
        
        // Create _posts directory for blog posts
        let posts_dest_dir = dest_dir.join("_posts");
        create_dir_if_not_exists(&posts_dest_dir)?;
        
        // Create collections directory for pages
        let collections_dir = dest_dir.join("_pages");
        create_dir_if_not_exists(&collections_dir)?;
        
        // Process posts (typically in content/posts or content/post or content/blog)
        let possible_post_dirs = vec![
            content_source_dir.join("posts"),
            content_source_dir.join("post"),
            content_source_dir.join("blog"),
            content_source_dir.join("articles"),
        ];
        
        let mut processed_post_dirs = Vec::new();
        
        for post_dir in possible_post_dirs {
            if post_dir.exists() && post_dir.is_dir() {
                processed_post_dirs.push(post_dir.clone());
                
                if verbose {
                    log::info!("Migrating posts from {}", post_dir.display());
                }
                
                // Migrate all posts
                for entry in WalkDir::new(&post_dir)
                    .min_depth(1)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        // Only process markdown files
                        if let Some(extension) = file_path.extension() {
                            let ext = extension.to_string_lossy();
                            if ext == "md" || ext == "markdown" {
                                self.process_post_file(file_path, &post_dir, &posts_dest_dir, verbose, result)?;
                            }
                        }
                    }
                }
            }
        }
        
        // Create README for posts directory
        let posts_readme = r#"# Posts Directory

This directory contains blog posts migrated from Hugo.

## Post Format

Posts in Rustyll:
- Should have filenames with the format `YYYY-MM-DD-title.md`
- Front matter should be at the top of the file, enclosed in `---`
- Content follows the front matter

## Changes from Hugo

- Some front matter variables have different names in Rustyll:
  - `date` is preserved
  - `lastmod` becomes `last_modified_at`
  - `draft` becomes `published: false`
  - `tags` and `categories` are preserved
- Hugo shortcodes have been converted to Liquid tags where possible
- For complex shortcodes, manual review may be needed
"#;
        
        write_readme(&posts_dest_dir, posts_readme)?;
        
        // Create README for pages directory
        let pages_readme = r#"# Pages Collection

This directory contains pages migrated from Hugo.

## Page Format

Pages in Rustyll:
- Should have front matter at the top of the file, enclosed in `---`
- Content follows the front matter
- Pages use the collection approach in Rustyll

## Changes from Hugo

- Hugo's content directory structure is converted to collection-based organization
- `_index.md` files are converted to regular pages with appropriate permalinks
- Front matter variables are adjusted to be compatible with Rustyll
- Shortcodes are converted to Liquid tags where possible
"#;
        
        write_readme(&collections_dir, pages_readme)?;
        
        // Process other content (pages, etc.)
        self.process_content_directory(&content_source_dir, dest_dir, &collections_dir, &processed_post_dirs, verbose, result)?;
        
        result.warnings.push(
            "Hugo uses shortcodes which were converted to Liquid tags where possible, but complex shortcodes may need manual review.".to_string()
        );
        
        Ok(())
    }
    
    fn process_post_file(&self, file_path: &Path, post_dir: &Path, posts_dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Read the file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read post file {}: {}", file_path.display(), e))?;
        
        // Parse front matter
        let doc = YamlFrontMatter::parse::<serde_yaml::Value>(&content)
            .map_err(|e| format!("Failed to parse front matter in {}: {}", file_path.display(), e))?;
        
        // Extract and convert front matter 
        let mut new_front_matter = serde_yaml::mapping::Mapping::new();
        
        // Process key front matter fields
        if let Some(title) = doc.metadata.get("title") {
            new_front_matter.insert(serde_yaml::Value::String("title".to_string()), title.clone());
        }
        
        // Handle date
        let mut post_date = String::from("2023-01-01");
        if let Some(date) = doc.metadata.get("date") {
            if let Some(date_str) = date.as_str() {
                // Try to parse date in various formats
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    post_date = parsed_date.format("%Y-%m-%d").to_string();
                    new_front_matter.insert(serde_yaml::Value::String("date".to_string()), date.clone());
                } else if date_str.len() >= 10 {
                    // Just take the first 10 chars as a fallback
                    post_date = date_str[..10].to_string();
                    new_front_matter.insert(serde_yaml::Value::String("date".to_string()), date.clone());
                }
            }
        }
        
        // Handle last modified
        if let Some(lastmod) = doc.metadata.get("lastmod") {
            new_front_matter.insert(serde_yaml::Value::String("last_modified_at".to_string()), lastmod.clone());
        }
        
        // Handle draft status
        if let Some(draft) = doc.metadata.get("draft") {
            if let Some(is_draft) = draft.as_bool() {
                if is_draft {
                    new_front_matter.insert(
                        serde_yaml::Value::String("published".to_string()),
                        serde_yaml::Value::Bool(false)
                    );
                }
            }
        }
        
        // Handle tags and categories
        if let Some(tags) = doc.metadata.get("tags") {
            new_front_matter.insert(serde_yaml::Value::String("tags".to_string()), tags.clone());
        }
        
        if let Some(categories) = doc.metadata.get("categories") {
            new_front_matter.insert(serde_yaml::Value::String("categories".to_string()), categories.clone());
        }
        
        // Handle author
        if let Some(author) = doc.metadata.get("author") {
            new_front_matter.insert(serde_yaml::Value::String("author".to_string()), author.clone());
        }
        
        // Handle layout
        if let Some(layout) = doc.metadata.get("layout") {
            new_front_matter.insert(serde_yaml::Value::String("layout".to_string()), layout.clone());
        } else {
            // Default to post layout
            new_front_matter.insert(
                serde_yaml::Value::String("layout".to_string()),
                serde_yaml::Value::String("post".to_string())
            );
        }
        
        // Create post filename with date prefix
        let base_name = file_path.file_stem()
            .ok_or_else(|| "Invalid file name".to_string())?
            .to_string_lossy();
        
        let extension = file_path.extension()
            .unwrap_or_default()
            .to_string_lossy();
        
        // Check if filename already has date pattern
        let dest_file_name = if base_name.starts_with(|c: char| c.is_ascii_digit()) && base_name.chars().nth(10) == Some('-') {
            // Filename already has a date pattern
            format!("{}.{}", base_name, extension)
        } else {
            // Add date prefix
            format!("{}-{}.{}", post_date, base_name, extension)
        };
        
        let dest_path = posts_dest_dir.join(&dest_file_name);
        
        // Convert shortcodes in content
        let converted_content = self.convert_shortcodes(&doc.content);
        
        // Serialize front matter to YAML
        let front_matter_yaml = serde_yaml::to_string(&new_front_matter)
            .map_err(|e| format!("Failed to serialize front matter: {}", e))?;
        
        // Create final post content
        let final_content = format!("---\n{}---\n\n{}", front_matter_yaml, converted_content);
        
        // Write converted post
        fs::write(&dest_path, final_content)
            .map_err(|e| format!("Failed to write converted post: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_posts/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "Post converted from Hugo format".to_string(),
        });
        
        Ok(())
    }
    
    fn process_content_directory(&self, content_dir: &Path, dest_dir: &Path, collections_dir: &Path, 
                               processed_post_dirs: &Vec<PathBuf>, verbose: bool, 
                               result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(content_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Skip files in already processed post directories
                if processed_post_dirs.iter().any(|dir| file_path.starts_with(dir)) {
                    continue;
                }
                
                // Only process markdown and HTML files
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy();
                    if ext == "md" || ext == "markdown" || ext == "html" {
                        // Read the file
                        let content = fs::read_to_string(file_path)
                            .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
                        
                        // Get the relative path from content directory
                        let rel_path = file_path.strip_prefix(content_dir)
                            .map_err(|_| "Failed to get relative path".to_string())?;
                        
                        let file_name = rel_path.file_name()
                            .ok_or_else(|| "Invalid file name".to_string())?
                            .to_string_lossy();
                        
                        let is_index = file_name == "index.md" || file_name == "_index.md";
                        let is_page = true;
                        
                        // Determine destination and handle special files
                        let (dest_path, permalink) = if is_index {
                            // Handle index files
                            let section = rel_path.parent().unwrap_or(Path::new(""));
                            let section_str = section.to_string_lossy();
                            
                            // Convert to a page in the collection
                            let page_name = if section_str.is_empty() {
                                "index.md".to_string()
                            } else {
                                format!("{}-index.md", section_str.replace("/", "-"))
                            };
                            
                            let permalink = if section_str.is_empty() {
                                "/".to_string()
                            } else {
                                format!("/{}/", section_str)
                            };
                            
                            (collections_dir.join(&page_name), permalink)
                        } else {
                            // Regular content file
                            let parent = rel_path.parent().unwrap_or(Path::new(""));
                            let file_stem = file_path.file_stem()
                                .unwrap_or_default()
                                .to_string_lossy();
                            
                            // Destination in collections
                            let page_name = if parent.to_string_lossy().is_empty() {
                                format!("{}.{}", file_stem, extension.to_string_lossy())
                            } else {
                                format!("{}-{}.{}", 
                                    parent.to_string_lossy().replace("/", "-"),
                                    file_stem,
                                    extension.to_string_lossy())
                            };
                            
                            // Calculate permalink
                            let permalink = if parent.to_string_lossy().is_empty() {
                                format!("/{}/", file_stem)
                            } else {
                                format!("/{}/{}/", parent.to_string_lossy(), file_stem)
                            };
                            
                            (collections_dir.join(page_name), permalink)
                        };
                        
                        // Try to parse front matter
                        let doc = match YamlFrontMatter::parse::<serde_yaml::Value>(&content) {
                            Ok(doc) => doc,
                            Err(_) => {
                                // No valid front matter, continue with empty metadata
                                Document {
                                    metadata: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                                    content: content.clone()
                                }
                            }
                        };
                        
                        // Convert front matter
                        let mut new_front_matter = serde_yaml::mapping::Mapping::new();
                        
                        // Get title from front matter or filename
                        let file_stem_str = file_path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                            
                        let title = doc.metadata.get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or(&file_stem_str);
                        
                        new_front_matter.insert(
                            serde_yaml::Value::String("title".to_string()),
                            serde_yaml::Value::String(title.to_string())
                        );
                        
                        // Set permalink
                        new_front_matter.insert(
                            serde_yaml::Value::String("permalink".to_string()),
                            serde_yaml::Value::String(permalink)
                        );
                        
                        // Set layout
                        if let Some(layout) = doc.metadata.get("layout") {
                            new_front_matter.insert(serde_yaml::Value::String("layout".to_string()), layout.clone());
                        } else {
                            // Default to page layout
                            new_front_matter.insert(
                                serde_yaml::Value::String("layout".to_string()),
                                serde_yaml::Value::String("page".to_string())
                            );
                        }
                        
                        // Copy other metadata
                        for (key, value) in doc.metadata.as_mapping().unwrap_or(&serde_yaml::Mapping::new()) {
                            let key_str = key.as_str().unwrap_or("");
                            
                            // Skip already processed keys
                            if key_str != "title" && key_str != "layout" {
                                new_front_matter.insert(key.clone(), value.clone());
                            }
                        }
                        
                        // Convert shortcodes
                        let converted_content = self.convert_shortcodes(&doc.content);
                        
                        // Serialize front matter to YAML
                        let front_matter_yaml = serde_yaml::to_string(&new_front_matter)
                            .map_err(|e| format!("Failed to serialize front matter: {}", e))?;
                        
                        // Create final content
                        let final_content = format!("---\n{}---\n\n{}", front_matter_yaml, converted_content);
                        
                        // Create parent directory if needed
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Write the file
                        fs::write(&dest_path, final_content)
                            .map_err(|e| format!("Failed to write converted file: {}", e))?;
                        
                        let rel_dest_path = dest_path.strip_prefix(dest_dir)
                            .unwrap_or(dest_path.as_path())
                            .to_string_lossy();
                            
                        result.changes.push(MigrationChange {
                            file_path: rel_dest_path.to_string(),
                            change_type: ChangeType::Converted,
                            description: format!("Content file converted from Hugo {} format", 
                                               if is_index { "index" } else { "page" }).to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn convert_shortcodes(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Convert Hugo shortcodes to Liquid tags
        
        // {{< shortcode param1 param2 >}} or {{% shortcode param1 param2 %}}
        let shortcode_regex = Regex::new(r"\{\{[<%](.*?)[%>]\}\}").unwrap();
        
        result = shortcode_regex.replace_all(&result, |caps: &regex::Captures| {
            let shortcode_content = caps.get(1).map_or("", |m| m.as_str()).trim();
            
            if shortcode_content.starts_with("figure ") {
                // Convert figure shortcode to Liquid
                let figure_parts: Vec<&str> = shortcode_content.split_whitespace().collect();
                if figure_parts.len() >= 2 {
                    let src = figure_parts[1].trim_matches('"').trim_matches('\'');
                    let alt = figure_parts.iter()
                        .find(|&&s| s.starts_with("alt="))
                        .map(|&s| s.trim_start_matches("alt=").trim_matches('"').trim_matches('\''))
                        .unwrap_or("");
                    let caption = figure_parts.iter()
                        .find(|&&s| s.starts_with("caption="))
                        .map(|&s| s.trim_start_matches("caption=").trim_matches('"').trim_matches('\''))
                        .unwrap_or("");
                    
                    return format!(r#"<figure>
  <img src="{}" alt="{}">
  <figcaption>{}</figcaption>
</figure>"#, src, alt, caption);
                }
            } else if shortcode_content.starts_with("highlight ") {
                // Convert highlight shortcode to Liquid
                let highlight_parts: Vec<&str> = shortcode_content.split_whitespace().collect();
                if highlight_parts.len() >= 2 {
                    let lang = highlight_parts[1].trim_matches('"').trim_matches('\'');
                    return format!("{{% highlight {} %}}", lang);
                }
            } else if shortcode_content == "/ highlight" {
                return "{% endhighlight %}".to_string();
            } else if shortcode_content.starts_with("youtube ") {
                // Convert youtube shortcode
                let youtube_id = shortcode_content.split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .trim_matches('"')
                    .trim_matches('\'');
                
                return format!(r#"<div class="video-container">
  <iframe width="700" height="394" src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe>
</div>"#, youtube_id);
            } else if shortcode_content.starts_with("include ") {
                // Convert include shortcode to Liquid include
                let file = shortcode_content.split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .trim_matches('"')
                    .trim_matches('\'');
                
                return format!("{{% include {} %}}", file);
            } else if shortcode_content == "< /shortcode >" || shortcode_content == "% /shortcode %" {
                // Closing shortcode
                return "{% endwhatever %}".to_string();
            }
            
            // For unknown shortcodes, keep them but warn in HTML comment
            format!("<!-- TODO: Convert Hugo shortcode: {} -->\n{}", shortcode_content, caps[0].to_string())
        }).to_string();
        
        // Return the converted content
        result
    }
} 