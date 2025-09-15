use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_data(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Nikola data...");
    }

    // Create destination data directory
    let dest_data_dir = dest_dir.join("_data");
    create_dir_if_not_exists(&dest_data_dir)?;

    // In Nikola, data can be in different places, we'll check a few common locations
    
    // First check for a dedicated data directory
    let data_dir = source_dir.join("data");
    if data_dir.exists() && data_dir.is_dir() {
        copy_data_files(&data_dir, &dest_data_dir, result)?;
    }
    
    // Also check the conf.py file which contains various configuration
    let conf_file = source_dir.join("conf.py");
    if conf_file.exists() {
        extract_conf_data(&conf_file, &dest_data_dir, result)?;
    }
    
    // Create some default data files if none were found
    if !has_data_files(&dest_data_dir) {
        create_default_data_files(&dest_data_dir, result)?;
    }

    Ok(())
}

fn copy_data_files(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Walk through the data directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process data files (json, yaml, etc.)
            if is_data_file(file_path) {
                migrate_data_file(file_path, source_dir, dest_dir, result)?;
            }
        }
    }
    
    Ok(())
}

fn migrate_data_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path with YAML extension
    let mut dest_path = dest_dir.join(rel_path);
    if let Some(ext) = dest_path.extension() {
        // If it's already YAML or YML, keep it as is
        if ext.to_string_lossy().to_lowercase() != "yml" && ext.to_string_lossy().to_lowercase() != "yaml" {
            // Otherwise convert to YAML
            let stem = dest_path.file_stem().unwrap();
            dest_path = dest_path.with_file_name(format!("{}.yml", stem.to_string_lossy()));
        }
    } else {
        // No extension, add .yml
        dest_path = dest_path.with_extension("yml");
    }
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Read the data file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read data file {}: {}", file_path.display(), e))?;
    
    // Convert to Jekyll format if needed
    let converted_content = convert_to_yaml_if_needed(file_path, &content);
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write data file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_data/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted data file from {}", file_path.display()),
    });
    
    Ok(())
}

fn extract_conf_data(
    conf_file: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the conf.py file
    let content = fs::read_to_string(conf_file)
        .map_err(|e| format!("Failed to read conf.py: {}", e))?;
    
    // Extract navigation information
    let navigation = extract_navigation_from_conf(&content);
    
    // Create navigation.yml
    let nav_path = dest_dir.join("navigation.yml");
    fs::write(&nav_path, navigation)
        .map_err(|e| format!("Failed to write navigation.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/navigation.yml".to_string(),
        description: "Created navigation data from conf.py".to_string(),
    });
    
    // Extract and migrate authors information if available
    if let Some(authors) = extract_authors_from_conf(&content) {
        let authors_path = dest_dir.join("authors.yml");
        fs::write(&authors_path, authors)
            .map_err(|e| format!("Failed to write authors.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "_data/authors.yml".to_string(),
            description: "Created authors data from conf.py".to_string(),
        });
    }
    
    Ok(())
}

fn has_data_files(data_dir: &Path) -> bool {
    // Check if there are any YML files in the data directory
    if !data_dir.exists() {
        return false;
    }
    
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().extension().map_or(false, |ext| 
                    ext.to_string_lossy().to_lowercase() == "yml" ||
                    ext.to_string_lossy().to_lowercase() == "yaml") {
                    return true;
                }
            }
        }
    }
    
    false
}

fn create_default_data_files(
    data_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default navigation.yml
    let navigation_content = r#"# Site navigation links
- title: Home
  url: /
- title: About
  url: /about/
- title: Blog
  url: /blog/
- title: Contact
  url: /contact/
"#;

    let nav_path = data_dir.join("navigation.yml");
    fs::write(&nav_path, navigation_content)
        .map_err(|e| format!("Failed to create navigation.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/navigation.yml".to_string(),
        description: "Created default navigation data".to_string(),
    });

    // Create default social.yml
    let social_content = r#"# Social profiles
github: username
twitter: username
facebook: username
instagram: username
linkedin: username
"#;

    let social_path = data_dir.join("social.yml");
    fs::write(&social_path, social_content)
        .map_err(|e| format!("Failed to create social.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/social.yml".to_string(),
        description: "Created default social data".to_string(),
    });

    Ok(())
}

fn is_data_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "json" | "yaml" | "yml" | "csv" | "toml" | "xml")
    } else {
        false
    }
}

fn convert_to_yaml_if_needed(file_path: &Path, content: &str) -> String {
    // Check the file extension to determine conversion
    if let Some(ext) = file_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        
        if ext_str == "json" {
            // Try to convert JSON to YAML
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                if let Ok(yaml) = serde_yaml::to_string(&json_value) {
                    return yaml;
                }
            }
        } else if ext_str == "toml" {
            // For TOML, we would need to parse and convert to YAML
            // This is simplified and may not work for all TOML files
            return "# Converted from TOML\n".to_string() + content;
        }
    }
    
    // If no conversion needed or possible, return the original content
    content.to_string()
}

fn extract_navigation_from_conf(content: &str) -> String {
    // Look for NAVIGATION_LINKS in conf.py
    let mut navigation_yml = String::from("# Navigation links extracted from Nikola config\n");
    
    // Simple regex-based extraction (this is a simplification)
    let nav_regex = regex::Regex::new(r#"NAVIGATION_LINKS\s*=\s*\{[^}]*\}"#).unwrap();
    
    if let Some(nav_match) = nav_regex.find(content) {
        // Found navigation links section
        let nav_section = &content[nav_match.start()..nav_match.end()];
        
        // Extract link entries - this is a simplification
        let link_regex = regex::Regex::new(r#"["']([^"']+)["']\s*:\s*["']([^"']+)["']"#).unwrap();
        
        for cap in link_regex.captures_iter(nav_section) {
            let title = &cap[1];
            let url = &cap[2];
            
            navigation_yml.push_str(&format!("- title: {}\n  url: {}\n", title, url));
        }
    } else {
        // No navigation found, create a default one
        navigation_yml.push_str(r#"- title: Home
  url: /
- title: About
  url: /about/
- title: Blog
  url: /blog/
"#);
    }
    
    navigation_yml
}

fn extract_authors_from_conf(content: &str) -> Option<String> {
    // Look for BLOG_AUTHORS in conf.py
    let mut authors_yml = String::from("# Authors extracted from Nikola config\n");
    let mut found_authors = false;
    
    // Simple regex-based extraction (this is a simplification)
    let authors_regex = regex::Regex::new(r#"BLOG_AUTHORS\s*=\s*\{[^}]*\}"#).unwrap();
    
    if let Some(authors_match) = authors_regex.find(content) {
        // Found authors section
        let authors_section = &content[authors_match.start()..authors_match.end()];
        
        // Extract author entries - this is a simplification
        let author_regex = regex::Regex::new(r#"["']([^"']+)["']\s*:\s*["']([^"']+)["']"#).unwrap();
        
        for cap in author_regex.captures_iter(authors_section) {
            let id = &cap[1];
            let name = &cap[2];
            
            authors_yml.push_str(&format!("{}:\n  name: {}\n", id, name));
            found_authors = true;
        }
    }
    
    if found_authors {
        Some(authors_yml)
    } else {
        None
    }
} 