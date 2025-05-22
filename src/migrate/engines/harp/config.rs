use std::path::Path;
use std::fs;
use serde_json;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::HarpMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, configuration is typically in harp.json or _harp.json
        let possible_config_files = vec![
            source_dir.join("harp.json"),
            source_dir.join("_harp.json"),
            source_dir.join("_data.json"),
        ];
        
        let mut found_config = false;
        let mut site_title = "Harp Site".to_string();
        let mut site_description = "A site migrated from Harp".to_string();
        let mut other_config = serde_yaml::Mapping::new();
        
        // Process any found config files
        for config_file in &possible_config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Found Harp configuration file: {}", config_file.display());
                }
                
                // Copy the file for reference
                let file_name = config_file.file_name().unwrap().to_string_lossy();
                let ref_file = dest_dir.join(format!("harp_original_{}", file_name));
                copy_file(config_file, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("harp_original_{}", file_name),
                    change_type: ChangeType::Converted,
                    description: "Original Harp configuration preserved for reference".to_string(),
                });
                
                found_config = true;
                
                // Try to extract configuration from the JSON file
                if let Ok(content) = fs::read_to_string(config_file) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        // Extract globals if they exist
                        if let Some(globals_obj) = json_value.get("globals").and_then(|g| g.as_object()) {
                            // Create a copy of the globals map to avoid borrowing issues
                            let globals_clone = globals_obj.clone();
                            
                            // Extract site title
                            if let Some(title) = globals_clone.clone().get("title").and_then(|t| t.as_str()) {
                                site_title = title.to_string();
                            }
                            
                            // Extract site description
                            if let Some(description) = globals_clone.clone().get("description").and_then(|d| d.as_str()) {
                                site_description = description.to_string();
                            }
                            
                            // Now iterate over the clone for additional settings
                            for (key, value) in globals_clone {
                                if key != "title" && key != "description" {
                                    other_config.insert(
                                        serde_yaml::Value::String(key.clone()),
                                        to_yaml_value(value.clone()),
                                    );
                                }
                            }
                        }
                        
                        // Extract other top-level configuration
                        if let Some(obj) = json_value.as_object() {
                            let json_obj = obj.clone();
                            for (key, value) in json_obj {
                                if key != "globals" {
                                    other_config.insert(
                                        serde_yaml::Value::String(key.clone()),
                                        to_yaml_value(value.clone()),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create Rustyll _config.yml
        let mut config_content = String::from("# Configuration for site migrated from Harp\n\n");
        
        // Add basic configuration
        config_content.push_str(&format!("title: \"{}\"\n", site_title));
        config_content.push_str(&format!("description: \"{}\"\n", site_description));
        config_content.push_str("baseurl: \"\"\n"); // Harp doesn't typically have this
        config_content.push_str("\n");
        
        // Add standard Jekyll/Rustyll configuration
        config_content.push_str("# Build settings\n");
        config_content.push_str("markdown: kramdown\n");
        config_content.push_str("permalink: pretty\n");
        config_content.push_str("\n");
        
        // Add migration note
        config_content.push_str("# Migration note\n");
        config_content.push_str("migration:\n");
        config_content.push_str("  source: harp\n");
        config_content.push_str(&format!("  date: {}\n", chrono::Local::now().format("%Y-%m-%d")));
        config_content.push_str("\n");
        
        // Add collections configuration
        config_content.push_str("# Collections\n");
        config_content.push_str("collections:\n");
        config_content.push_str("  pages:\n");
        config_content.push_str("    output: true\n");
        config_content.push_str("    permalink: /:path/\n");
        config_content.push_str("\n");
        
        // Add any other configuration from Harp
        if !other_config.is_empty() {
            config_content.push_str("# Additional configuration migrated from Harp\n");
            config_content.push_str("harp_config:\n");
            
            // Convert other config to YAML string and indent
            if let Ok(other_yaml) = serde_yaml::to_string(&serde_yaml::Value::Mapping(other_config)) {
                // Indent each line with two spaces
                for line in other_yaml.lines() {
                    if !line.trim().is_empty() && !line.starts_with("---") {
                        config_content.push_str(&format!("  {}\n", line));
                    }
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
            description: "Rustyll configuration created from Harp config".to_string(),
        });
        
        if !found_config {
            result.warnings.push(
                "No Harp configuration files found. A basic config has been created.".to_string()
            );
        }
        
        Ok(())
    }
}

// Helper function to convert serde_json::Value to serde_yaml::Value
fn to_yaml_value(json_value: serde_json::Value) -> serde_yaml::Value {
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
                .map(to_yaml_value)
                .collect();
            serde_yaml::Value::Sequence(yaml_arr)
        },
        serde_json::Value::Object(obj) => {
            let mut yaml_map = serde_yaml::Mapping::new();
            for (k, v) in obj {
                yaml_map.insert(
                    serde_yaml::Value::String(k),
                    to_yaml_value(v),
                );
            }
            serde_yaml::Value::Mapping(yaml_map)
        },
    }
} 