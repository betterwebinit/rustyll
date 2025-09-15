use std::path::Path;
use std::fs;
use serde_json;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::GitbookMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Look for common GitBook config files
        let possible_config_files = vec![
            source_dir.join("book.json"),
            source_dir.join(".bookrc"),
            source_dir.join("book.yml"),
            source_dir.join("book.yaml"),
        ];
        
        // Initialize with default values
        let mut site_title = String::from("GitBook Site");
        let mut site_description = String::from("A site migrated from GitBook");
        let mut site_author = String::from("");
        let mut site_lang = String::from("en");
        let mut plugins = Vec::new();
        let other_config = serde_yaml::Mapping::new();
        
        let mut found_config = false;
        
        // Process any found GitBook config
        for config_file in &possible_config_files {
            if config_file.exists() {
                found_config = true;
                
                if verbose {
                    log::info!("Found GitBook config: {}", config_file.display());
                }
                
                // Copy the original config for reference
                let file_name = config_file.file_name().unwrap().to_string_lossy();
                let ref_file = dest_dir.join(format!("gitbook_original_{}", file_name));
                copy_file(config_file, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("gitbook_original_{}", file_name),
                    change_type: ChangeType::Converted,
                    description: "Original GitBook configuration preserved for reference".to_string(),
                });
                
                // Try to extract configuration from the file
                if let Ok(content) = fs::read_to_string(config_file) {
                    let extension = config_file.extension().map(|ext| ext.to_string_lossy().to_lowercase());
                    
                    // Process based on file type
                    if extension.as_deref() == Some("json") || config_file.ends_with(".bookrc") {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let json_clone = json.clone();
                            // Extract site title
                            if let Some(title) = json_clone.get("title").and_then(|t| t.as_str()) {
                                site_title = title.to_string();
                            }
                            
                            // Extract site description
                            if let Some(description) = json_clone.get("description").and_then(|d| d.as_str()) {
                                site_description = description.to_string();
                            }
                            
                            // Extract author
                            if let Some(author) = json_clone.get("author").and_then(|a| a.as_str()) {
                                site_author = author.to_string();
                            } else if let Some(authors) = json_clone.get("authors").and_then(|a| a.as_array()) {
                                if !authors.is_empty() {
                                    if let Some(author) = authors[0].as_str() {
                                        site_author = author.to_string();
                                    }
                                }
                            }
                            
                            // Extract language
                            if let Some(lang) = json_clone.get("language").and_then(|l| l.as_str()) {
                                site_lang = lang.to_string();
                            }
                            
                            // Extract plugins
                            if let Some(plug) = json_clone.get("plugins").and_then(|p| p.as_array()) {
                                for plugin in plug {
                                    if let Some(plugin_str) = plugin.as_str() {
                                        plugins.push(plugin_str.to_string());
                                    }
                                }
                            }
                        }
                    } else if extension.as_deref() == Some("yml") || extension.as_deref() == Some("yaml") {
                        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            // Extract site title
                            if let Some(title) = yaml.get("title").and_then(|t| t.as_str()) {
                                site_title = title.to_string();
                            }
                            
                            // Extract site description
                            if let Some(description) = yaml.get("description").and_then(|d| d.as_str()) {
                                site_description = description.to_string();
                            }
                            
                            // Extract author
                            if let Some(author) = yaml.get("author").and_then(|a| a.as_str()) {
                                site_author = author.to_string();
                            }
                            
                            // Extract language
                            if let Some(lang) = yaml.get("language").and_then(|l| l.as_str()) {
                                site_lang = lang.to_string();
                            }
                        }
                    }
                }
            }
        }
        
        // Check for README.md to get title
        let readme_path = source_dir.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = fs::read_to_string(&readme_path) {
                // Try to extract title from first heading
                let mut extracted_title = None;
                if let Some(title_line) = content.lines().find(|line| line.starts_with("# ")) {
                    extracted_title = Some(title_line[2..].trim().to_string());
                }
                
                if let Some(title) = extracted_title {
                    if site_title.is_empty() || site_title == "GitBook Site" {
                        site_title = title;
                    }
                }
                
                // Extract description from first paragraph after title
                let lines: Vec<&str> = content.lines().collect();
                for i in 0..lines.len() {
                    if lines[i].starts_with("# ") && i + 1 < lines.len() && !lines[i+1].is_empty() && !lines[i+1].starts_with("#") {
                        site_description = lines[i+1].trim().to_string();
                        break;
                    }
                }
            }
        }
        
        // Create Rustyll _config.yml
        let mut config_content = String::from("# Configuration for site migrated from GitBook\n\n");
        
        // Add basic configuration
        config_content.push_str(&format!("title: \"{}\"\n", site_title));
        config_content.push_str(&format!("description: \"{}\"\n", site_description));
        config_content.push_str(&format!("author: \"{}\"\n", site_author));
        config_content.push_str(&format!("lang: \"{}\"\n", site_lang));
        config_content.push_str("baseurl: \"\"\n"); // Usually empty for GitBook
        config_content.push_str("\n");
        
        // Add GitHub Pages compatibility
        config_content.push_str("# Build settings\n");
        config_content.push_str("markdown: kramdown\n");
        config_content.push_str("highlighter: rouge\n");
        config_content.push_str("\n");
        
        // Add GitBook plugins as Jekyll plugins where possible
        if !plugins.is_empty() {
            config_content.push_str("# Plugins (converted from GitBook)\n");
            config_content.push_str("plugins:\n");
            
            for plugin in &plugins {
                match plugin.as_str() {
                    "highlight" => config_content.push_str("  - jekyll-highlight\n"),
                    "github" => config_content.push_str("  - jekyll-github-metadata\n"),
                    "search" => config_content.push_str("  - jekyll-search\n"),
                    "lunr" => config_content.push_str("  - jekyll-lunr\n"),
                    "ga" => config_content.push_str("  - jekyll-analytics\n"),
                    _ => result.warnings.push(format!("GitBook plugin '{}' has no direct Jekyll equivalent", plugin)),
                }
            }
            config_content.push_str("\n");
        }
        
        // Add migration note
        config_content.push_str("# Migration note\n");
        config_content.push_str("migration:\n");
        config_content.push_str("  source: gitbook\n");
        config_content.push_str(&format!("  date: {}\n", chrono::Local::now().format("%Y-%m-%d")));
        config_content.push_str("\n");
        
        // Add collections configuration
        config_content.push_str("# Collections\n");
        config_content.push_str("collections:\n");
        config_content.push_str("  docs:\n");
        config_content.push_str("    output: true\n");
        config_content.push_str("    permalink: /:collection/:path/\n");
        config_content.push_str("\n");
        
        // Write the new config file
        let dest_config = dest_dir.join("_config.yml");
        fs::write(&dest_config, config_content)
            .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Rustyll configuration created from GitBook config".to_string(),
        });
        
        if !found_config {
            result.warnings.push(
                "No GitBook configuration files found. A basic config has been created.".to_string()
            );
        }
        
        Ok(())
    }
}

// Helper function to convert serde_json::Value to serde_yaml::Value
fn convert_json_to_yaml(json_value: serde_json::Value) -> serde_yaml::Value {
    match json_value {
        serde_json::Value::Null => serde_yaml::Value::Null,
        serde_json::Value::Bool(b) => serde_yaml::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(f))
            } else {
                serde_yaml::Value::Null
            }
        },
        serde_json::Value::String(s) => serde_yaml::Value::String(s),
        serde_json::Value::Array(arr) => {
            let yaml_arr = arr.into_iter()
                .map(convert_json_to_yaml)
                .collect();
            serde_yaml::Value::Sequence(yaml_arr)
        },
        serde_json::Value::Object(obj) => {
            let mut yaml_map = serde_yaml::Mapping::new();
            for (k, v) in obj {
                yaml_map.insert(
                    serde_yaml::Value::String(k),
                    convert_json_to_yaml(v),
                );
            }
            serde_yaml::Value::Mapping(yaml_map)
        },
    }
} 