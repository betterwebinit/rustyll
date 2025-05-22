use std::path::Path;
use std::fs;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

pub(super) fn migrate_config(source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
    // In Bridgetown, configuration could be in multiple places:
    // - bridgetown.config.yml
    // - config/bridgetown.yml
    // - _config.yml (legacy Jekyll format that Bridgetown also supports)
    
    let possible_config_files = vec![
        source_dir.join("bridgetown.config.yml"),
        source_dir.join("config").join("bridgetown.yml"),
        source_dir.join("_config.yml"),
    ];
    
    let mut found_config = false;
    let mut site_title = "Bridgetown Site".to_string();
    let mut site_description = "A site migrated from Bridgetown".to_string();
    let mut site_url = "".to_string();
    let mut other_config = serde_yaml::Mapping::new();
    
    // Process any found config files
    for config_file in &possible_config_files {
        if config_file.exists() {
            if verbose {
                log::info!("Found Bridgetown configuration file: {}", config_file.display());
            }
            
            // Copy the file for reference
            let file_name = config_file.file_name().unwrap().to_string_lossy();
            let ref_file = dest_dir.join(format!("bridgetown_original_{}", file_name));
            copy_file(config_file, &ref_file)?;
            
            result.changes.push(MigrationChange {
                file_path: format!("bridgetown_original_{}", file_name),
                change_type: ChangeType::Converted,
                description: "Original Bridgetown configuration preserved for reference".to_string(),
            });
            
            found_config = true;
            
            // Try to extract configuration from the file
                            if let Ok(content) = fs::read_to_string(config_file) {
                    if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        // Create a clone of yaml to avoid borrowing issues
                        let yaml_clone = yaml.clone();
                        
                        // Handle title
                        if let Some(title) = yaml_clone.get("title").and_then(|v| v.as_str()) {
                            site_title = title.to_string();
                        }
                        
                        // Handle description
                        if let Some(desc) = yaml_clone.get("description").and_then(|v| v.as_str()) {
                            site_description = desc.to_string();
                        }
                        
                        // Handle URL
                        if let Some(url) = yaml_clone.get("url").and_then(|v| v.as_str()) {
                            site_url = url.to_string();
                        }
                        
                        // Store other configuration options
                        if let Some(yaml_mapping) = yaml.as_mapping() {
                            for (key, value) in yaml_mapping {
                                if let Some(key_str) = key.as_str() {
                                    if key_str != "title" && key_str != "description" && key_str != "url" {
                                        other_config.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                        }
                    }
                }
        }
    }
    
    // Create Rustyll _config.yml
    let mut config_content = String::from("# Configuration for site migrated from Bridgetown\n\n");
    
    // Add basic configuration
    config_content.push_str(&format!("title: \"{}\"\n", site_title));
    config_content.push_str(&format!("description: \"{}\"\n", site_description));
    config_content.push_str(&format!("url: \"{}\"\n", site_url));
    config_content.push_str("\n");
    
    // Add standard Jekyll/Rustyll configuration
    config_content.push_str("# Build settings\n");
    config_content.push_str("markdown: kramdown\n");
    config_content.push_str("highlighter: rouge\n");
    config_content.push_str("\n");
    
    // Add migration note
    config_content.push_str("# Migration note\n");
    config_content.push_str("migration:\n");
    config_content.push_str("  source: bridgetown\n");
    config_content.push_str(&format!("  date: {}\n", chrono::Local::now().format("%Y-%m-%d")));
    config_content.push_str("\n");
    
    // Add collections configuration
    config_content.push_str("# Collections\n");
    config_content.push_str("collections:\n");
    config_content.push_str("  pages:\n");
    config_content.push_str("    output: true\n");
    config_content.push_str("    permalink: /:path/\n");
    config_content.push_str("\n");

    // Add any other configuration from Bridgetown
    if !other_config.is_empty() {
        config_content.push_str("# Additional configuration migrated from Bridgetown\n");
        
        // Convert other config to YAML string
        if let Ok(other_yaml) = serde_yaml::to_string(&serde_yaml::Value::Mapping(other_config)) {
            // Skip the document separator
            if other_yaml.starts_with("---\n") {
                config_content.push_str(&other_yaml[4..]);
            } else {
                config_content.push_str(&other_yaml);
            }
        }
    }
    
    // Write the new config file
    let dest_config = dest_dir.join("_config.yml");
    fs::write(&dest_config, config_content)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_config.yml".to_string(),
        change_type: ChangeType::Created,
        description: "Rustyll configuration created from Bridgetown config".to_string(),
    });
    
    if !found_config {
        result.warnings.push(
            "No Bridgetown configuration files found. A basic config has been created.".to_string()
        );
    } else {
        result.warnings.push(
            "Bridgetown configuration has been migrated to Rustyll format. You may need to manually adjust some settings.".to_string()
        );
    }
    
    Ok(())
} 