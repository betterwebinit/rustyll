use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_translations(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Pelican translations...");
    }

    // Check if the site uses translations
    let has_translations = check_for_translations(source_dir);
    
    if !has_translations {
        if verbose {
            log::info!("No translations found in Pelican site.");
        }
        return Ok(());
    }
    
    // Add Jekyll Polyglot gem to Gemfile
    add_polyglot_to_gemfile(dest_dir, result)?;
    
    // Add language configuration to _config.yml
    add_language_config(source_dir, dest_dir, result)?;
    
    // Migrate translated content
    migrate_translated_content(source_dir, dest_dir, result)?;
    
    Ok(())
}

fn check_for_translations(source_dir: &Path) -> bool {
    // Check for translations in content directory structure
    let content_dir = source_dir.join("content");
    if content_dir.exists() {
        // In Pelican, translations are often in language-specific subdirectories
        // or with language suffixes in filenames
        if let Ok(entries) = fs::read_dir(&content_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap().to_string_lossy();
                        if is_language_code(&dir_name) {
                            return true;
                        }
                    }
                }
            }
        }
        
        // Check for files with language suffixes (e.g., post-fr.md)
        for entry in WalkDir::new(&content_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.contains("-fr.") || file_name.contains("-es.") || file_name.contains("-de.") {
                    return true;
                }
            }
        }
    }
    
    // Check if pelicanconf.py has i18n settings
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            if content.contains("I18N_SUBSITES") || content.contains("DEFAULT_LANG") {
                return true;
            }
        }
    }
    
    false
}

fn add_polyglot_to_gemfile(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    let gemfile_path = dest_dir.join("Gemfile");
    if !gemfile_path.exists() {
        return Ok(());
    }
    
    let mut gemfile_content = fs::read_to_string(&gemfile_path)
        .map_err(|e| format!("Failed to read Gemfile: {}", e))?;
    
    // Add Jekyll Polyglot gem if not already present
    if !gemfile_content.contains("jekyll-polyglot") {
        gemfile_content.push_str("\n# Multilingual support\ngem \"jekyll-polyglot\", \"~> 1.5.0\"\n");
        
        fs::write(&gemfile_path, gemfile_content)
            .map_err(|e| format!("Failed to write updated Gemfile: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Modified,
            file_path: "Gemfile".into(),
            description: "Added Jekyll Polyglot gem for multilingual support".into(),
        });
    }
    
    Ok(())
}

fn add_language_config(source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Detect languages used in the Pelican site
    let languages = detect_languages(source_dir);
    
    if languages.is_empty() {
        return Ok(());
    }
    
    let default_lang = languages.first().unwrap().clone();
    
    // Update _config.yml with language settings
    let config_path = dest_dir.join("_config.yml");
    if !config_path.exists() {
        return Ok(());
    }
    
    let mut config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read _config.yml: {}", e))?;
    
    // Add Polyglot configuration if not present
    if !config_content.contains("languages:") {
        let language_list = languages.iter()
            .map(|lang| format!("\"{}\"", lang))
            .collect::<Vec<_>>()
            .join(", ");
        
        let polyglot_config = format!(r#"
# Multilingual settings (via jekyll-polyglot)
languages: [{}]
default_lang: "{}"
exclude_from_localization: ["assets", "images", "css", "js", "fonts"]
parallel_localization: true
"#, language_list, default_lang);
        
        config_content.push_str(&polyglot_config);
        
        // Add plugins configuration or update existing one
        if !config_content.contains("plugins:") {
            config_content.push_str("\nplugins:\n  - jekyll-polyglot\n");
        } else if !config_content.contains("jekyll-polyglot") {
            // Find plugins: section and add jekyll-polyglot
            if let Some(pos) = config_content.find("plugins:") {
                let insert_pos = pos + "plugins:".len();
                config_content.insert_str(insert_pos, "\n  - jekyll-polyglot");
            }
        }
        
        fs::write(&config_path, config_content)
            .map_err(|e| format!("Failed to write updated _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Modified,
            file_path: "_config.yml".into(),
            description: format!("Added multilingual configuration for languages: {}", language_list),
        });
    }
    
    Ok(())
}

fn detect_languages(source_dir: &Path) -> Vec<String> {
    let mut languages = Vec::new();
    
    // Try to detect languages from pelicanconf.py
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            // Check for DEFAULT_LANG
            let default_lang_regex = regex::Regex::new(r#"DEFAULT_LANG\s*=\s*["']([^"']+)["']"#).unwrap();
            if let Some(captures) = default_lang_regex.captures(&content) {
                let lang = captures[1].to_string();
                if !languages.contains(&lang) {
                    languages.push(lang);
                }
            }
            
            // Check for I18N_SUBSITES
            let subsites_regex = regex::Regex::new(r"I18N_SUBSITES\s*=\s*\{(.*?)\}").unwrap();
            if let Some(captures) = subsites_regex.captures(&content) {
                let subsites_str = &captures[1];
                
                // Extract language codes from subsites
                let lang_regex = regex::Regex::new(r#"["']([a-z]{2})["']"#).unwrap();
                for lang_match in lang_regex.captures_iter(subsites_str) {
                    let lang = lang_match[1].to_string();
                    if !languages.contains(&lang) {
                        languages.push(lang);
                    }
                }
            }
        }
    }
    
    // Check for language directories in content
    let content_dir = source_dir.join("content");
    if content_dir.exists() {
        if let Ok(entries) = fs::read_dir(&content_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap().to_string_lossy();
                        if is_language_code(&dir_name) && !languages.contains(&dir_name.to_string()) {
                            languages.push(dir_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // If no languages detected, default to "en"
    if languages.is_empty() {
        languages.push("en".to_string());
    }
    
    languages
}

fn migrate_translated_content(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Pelican's content directory
    let content_dir = source_dir.join("content");
    if !content_dir.exists() {
        return Ok(());
    }
    
    // Created destination directories if needed
    let dest_posts_dir = dest_dir.join("_posts");
    create_dir_if_not_exists(&dest_posts_dir)?;
    
    let dest_pages_dir = dest_dir.join("_pages");
    create_dir_if_not_exists(&dest_pages_dir)?;
    
    // Collect language directories
    let mut language_dirs = HashMap::new();
    if let Ok(entries) = fs::read_dir(&content_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy();
                    if is_language_code(&dir_name) {
                        language_dirs.insert(dir_name.to_string(), path);
                    }
                }
            }
        }
    }
    
    // Process each language directory
    for (lang, dir) in &language_dirs {
        process_language_directory(lang, dir, &dest_posts_dir, &dest_pages_dir, result)?;
    }
    
    // Process files with language suffixes (for sites not using directory structure)
    process_language_suffixed_files(&content_dir, &dest_posts_dir, &dest_pages_dir, result)?;
    
    Ok(())
}

fn process_language_directory(
    lang: &str,
    dir: &Path,
    dest_posts_dir: &Path,
    dest_pages_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip non-content files
            if !is_content_file(file_path) {
                continue;
            }
            
            // Read the content file
            let content = fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
            
            // Parse front matter and content
            let (mut metadata, body) = extract_pelican_metadata(&content);
            
            // Determine if it's a page or post
            let is_page = is_page_content(file_path, &metadata);
            
            // Add language to front matter
            metadata.insert("lang".to_string(), lang.to_string());
            
            // Build Jekyll front matter
            let yaml_front_matter = build_jekyll_front_matter(&metadata, is_page);
            
            // Combine front matter with converted content
            let jekyll_content = format!("---\n{}---\n\n{}", yaml_front_matter, body);
            
            // Determine destination path
            let dest_path = if is_page {
                let rel_path = file_path.strip_prefix(dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                let mut dest_file = dest_pages_dir.join(rel_path);
                
                // Ensure it has .md extension
                if let Some(ext) = dest_file.extension() {
                    if ext != "md" {
                        let stem = dest_file.file_stem().unwrap().to_string_lossy();
                        dest_file = dest_file.with_file_name(format!("{}.md", stem));
                    }
                } else {
                    let filename = dest_file.file_name().unwrap().to_string_lossy();
                    dest_file = dest_file.with_file_name(format!("{}.md", filename));
                }
                
                dest_file
            } else {
                // For posts, ensure Jekyll naming convention
                let file_name = file_path.file_name().unwrap().to_string_lossy();
                
                // Extract or generate date
                let date_prefix = if let Some(date) = metadata.get("date") {
                    // Try to parse date from metadata
                    if let Some(parsed_date) = parse_pelican_date(date) {
                        parsed_date.format("%Y-%m-%d-").to_string()
                    } else {
                        chrono::Local::now().format("%Y-%m-%d-").to_string()
                    }
                } else {
                    chrono::Local::now().format("%Y-%m-%d-").to_string()
                };
                
                // Use slug if available, otherwise use file stem
                let slug = if let Some(slug_val) = metadata.get("slug") {
                    slug_val.clone()
                } else {
                    file_path.file_stem().unwrap().to_string_lossy().to_string()
                };
                
                let post_name = format!("{}{}.md", date_prefix, slug);
                dest_posts_dir.join(post_name)
            };
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Write the file
            fs::write(&dest_path, jekyll_content)
                .map_err(|e| format!("Failed to write content file {}: {}", dest_path.display(), e))?;
            
            // Add to changes
            let rel_dest_path = if is_page {
                format!("_pages/{}", dest_path.file_name().unwrap().to_string_lossy())
            } else {
                format!("_posts/{}", dest_path.file_name().unwrap().to_string_lossy())
            };
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Converted,
                file_path: rel_dest_path,
                description: format!("Converted Pelican {} from {} (language: {})", 
                                    if is_page { "page" } else { "post" }, 
                                    file_path.display(), 
                                    lang),
            });
        }
    }
    
    Ok(())
}

fn process_language_suffixed_files(
    content_dir: &Path,
    dest_posts_dir: &Path,
    dest_pages_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process files with language suffixes (e.g., post-fr.md)
    for entry in WalkDir::new(content_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            
            // Check if filename has language suffix
            if let Some((lang, base_name)) = extract_language_suffix(&file_name) {
                // Skip non-content files
                if !is_content_file(file_path) {
                    continue;
                }
                
                // Read the content file
                let content = fs::read_to_string(file_path)
                    .map_err(|e| format!("Failed to read content file {}: {}", file_path.display(), e))?;
                
                // Parse front matter and content
                let (mut metadata, body) = extract_pelican_metadata(&content);
                
                // Determine if it's a page or post
                let is_page = is_page_content(file_path, &metadata);
                
                // Add language to front matter
                metadata.insert("lang".to_string(), lang.to_string());
                
                // Build Jekyll front matter
                let yaml_front_matter = build_jekyll_front_matter(&metadata, is_page);
                
                // Combine front matter with converted content
                let jekyll_content = format!("---\n{}---\n\n{}", yaml_front_matter, body);
                
                // Determine destination path
                let dest_path = if is_page {
                    dest_pages_dir.join(format!("{}.md", base_name))
                } else {
                    // For posts, ensure Jekyll naming convention
                    // Extract or generate date
                    let date_prefix = if let Some(date) = metadata.get("date") {
                        // Try to parse date from metadata
                        if let Some(parsed_date) = parse_pelican_date(date) {
                            parsed_date.format("%Y-%m-%d-").to_string()
                        } else {
                            chrono::Local::now().format("%Y-%m-%d-").to_string()
                        }
                    } else {
                        chrono::Local::now().format("%Y-%m-%d-").to_string()
                    };
                    
                    // Use slug if available, otherwise use base name
                    let slug = if let Some(slug_val) = metadata.get("slug") {
                        slug_val.clone()
                    } else {
                        base_name.to_string()
                    };
                    
                    let post_name = format!("{}{}.md", date_prefix, slug);
                    dest_posts_dir.join(post_name)
                };
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Write the file
                fs::write(&dest_path, jekyll_content)
                    .map_err(|e| format!("Failed to write content file {}: {}", dest_path.display(), e))?;
                
                // Add to changes
                let rel_dest_path = if is_page {
                    format!("_pages/{}", dest_path.file_name().unwrap().to_string_lossy())
                } else {
                    format!("_posts/{}", dest_path.file_name().unwrap().to_string_lossy())
                };
                
                result.changes.push(MigrationChange {
                    change_type: ChangeType::Converted,
                    file_path: rel_dest_path,
                    description: format!("Converted Pelican {} from {} (language: {})", 
                                        if is_page { "page" } else { "post" }, 
                                        file_path.display(), 
                                        lang),
                });
            }
        }
    }
    
    Ok(())
}

// Helper function to check if a string is a language code
fn is_language_code(code: &str) -> bool {
    let language_codes = [
        "en", "fr", "es", "de", "it", "pt", "ru", "ja", "zh", "ko",
        "nl", "tr", "ar", "fa", "pl", "uk", "hi", "bn", "sv", "el"
    ];
    
    language_codes.contains(&code)
}

// Helper function to check if a file is a content file
fn is_content_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "md" | "markdown" | "rst" | "html")
    } else {
        false
    }
}

// Helper function to determine if content is a page or post
fn is_page_content(path: &Path, metadata: &HashMap<String, String>) -> bool {
    // Check metadata for page indicator
    if let Some(status) = metadata.get("status") {
        if status.to_lowercase() == "page" {
            return true;
        }
    }
    
    // Check path - in Pelican, pages are often in a 'pages' directory
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("/pages/") || path_str.contains("\\pages\\")
}

// Helper function to extract Pelican metadata from content
fn extract_pelican_metadata(content: &str) -> (HashMap<String, String>, &str) {
    let mut metadata = HashMap::new();
    let mut body_start = 0;
    
    // Pelican supports multiple metadata formats
    // Try the reStructuredText metadata format first
    if content.starts_with(':') {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() && lines[i].starts_with(':') {
            let line = lines[i].trim();
            if let Some(colon_pos) = line.find(": ") {
                let key = line[1..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 2..].trim().to_string();
                metadata.insert(key, value);
            }
            i += 1;
        }
        body_start = lines[..i].join("\n").len() + 1; // +1 for the newline
    } 
    // Try Markdown metadata format (YAML front matter)
    else if content.starts_with("---") {
        if let Some(end_pos) = content[3..].find("---") {
            let front_matter = &content[3..end_pos + 3];
            for line in front_matter.lines() {
                if let Some(colon_pos) = line.find(": ") {
                    let key = line[..colon_pos].trim().to_lowercase();
                    let value = line[colon_pos + 2..].trim().to_string();
                    metadata.insert(key, value);
                }
            }
            body_start = end_pos + 6; // Skip past the second ---
        }
    }
    // Try Markdown metadata format (MultiMarkdown)
    else {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut in_metadata = true;
        
        while i < lines.len() && in_metadata {
            let line = lines[i].trim();
            
            if line.is_empty() {
                in_metadata = false;
            } else if let Some(colon_pos) = line.find(": ") {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 2..].trim().to_string();
                metadata.insert(key, value);
            } else {
                in_metadata = false;
            }
            
            i += 1;
        }
        
        if i > 0 {
            body_start = lines[..i].join("\n").len() + 1;
        }
    }
    
    (metadata, &content[body_start..])
}

// Helper function to parse Pelican date format
fn parse_pelican_date(date_str: &str) -> Option<chrono::NaiveDate> {
    // Try various date formats that Pelican supports
    let formats = [
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
    ];
    
    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return Some(date);
        }
    }
    
    // Try to extract just the date part if there's a time component
    if let Some(space_pos) = date_str.find(' ') {
        let date_part = &date_str[..space_pos];
        return parse_pelican_date(date_part);
    }
    
    None
}

// Helper function to build Jekyll front matter from metadata
fn build_jekyll_front_matter(metadata: &HashMap<String, String>, is_page: bool) -> String {
    let mut front_matter = String::new();
    
    // Required fields
    if let Some(title) = metadata.get("title") {
        front_matter.push_str(&format!("title: \"{}\"\n", escape_yaml_string(title)));
    }
    
    // Date handling
    if let Some(date) = metadata.get("date") {
        front_matter.push_str(&format!("date: {}\n", date));
    }
    
    // Language
    if let Some(lang) = metadata.get("lang") {
        front_matter.push_str(&format!("lang: {}\n", lang));
    }
    
    // Layout
    front_matter.push_str(&format!("layout: \"{}\"\n", if is_page { "page" } else { "post" }));
    
    // Handle other common metadata
    if let Some(author) = metadata.get("author") {
        front_matter.push_str(&format!("author: \"{}\"\n", escape_yaml_string(author)));
    }
    
    if let Some(category) = metadata.get("category") {
        front_matter.push_str(&format!("category: \"{}\"\n", escape_yaml_string(category)));
    }
    
    if let Some(tags) = metadata.get("tags") {
        // Pelican tags are comma-separated
        let tag_list: Vec<&str> = tags.split(',').map(|t| t.trim()).collect();
        front_matter.push_str("tags:\n");
        for tag in tag_list {
            front_matter.push_str(&format!("  - \"{}\"\n", escape_yaml_string(tag)));
        }
    }
    
    if is_page {
        // For pages, we need a permalink
        if let Some(slug) = metadata.get("slug") {
            front_matter.push_str(&format!("permalink: /{}/\n", slug));
        }
    }
    
    // Add any other metadata as custom variables
    for (key, value) in metadata {
        if !["title", "date", "author", "category", "tags", "slug", "lang", "layout"].contains(&key.as_str()) {
            front_matter.push_str(&format!("{}: \"{}\"\n", key, escape_yaml_string(value)));
        }
    }
    
    front_matter
}

// Helper function to escape special characters in YAML strings
fn escape_yaml_string(s: &str) -> String {
    s.replace("\"", "\\\"").replace("\n", "\\n")
}

// Helper function to extract language suffix from filename
fn extract_language_suffix(filename: &str) -> Option<(String, String)> {
    let re = regex::Regex::new(r"^(.+)-([a-z]{2})\.([^.]+)$").unwrap();
    
    if let Some(captures) = re.captures(filename) {
        let base_name = captures[1].to_string();
        let lang = captures[2].to_string();
        
        if is_language_code(&lang) {
            return Some((lang, base_name));
        }
    }
    
    None
} 