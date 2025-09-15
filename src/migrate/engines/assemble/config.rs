use std::path::Path;
use std::fs;
use serde_json;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::AssembleMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, configuration could be in multiple places:
        // - assemblefile.js
        // - Gruntfile.js
        // - package.json
        // - .assemblerc or .assemblerc.json or .assemblerc.yml
        
        let possible_config_files = vec![
            source_dir.join("assemblefile.js"),
            source_dir.join("Gruntfile.js"),
            source_dir.join("package.json"),
            source_dir.join(".assemblerc"),
            source_dir.join(".assemblerc.json"),
            source_dir.join(".assemblerc.yml"),
            source_dir.join(".assemblerc.yaml"),
        ];
        
        let mut found_config = false;
        let mut site_title = String::from("Assemble Site");
        let mut site_description = String::from("A site migrated from Assemble");
        
        // Copy any found config files for reference
        for config_file in &possible_config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Found Assemble configuration file: {}", config_file.display());
                }
                
                // Copy the file for reference
                let file_name = config_file.file_name().unwrap().to_string_lossy();
                let ref_file = dest_dir.join(format!("assemble_original_{}", file_name));
                copy_file(config_file, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("assemble_original_{}", file_name),
                    change_type: ChangeType::Copied,
                    description: "Original Assemble configuration preserved for reference".to_string(),
                });
                
                found_config = true;
                
                // Try to extract some metadata from the file
                if config_file.extension().map_or("", |e| e.to_str().unwrap_or("")) == "json" {
                    if let Ok(content) = fs::read_to_string(config_file) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Check package.json for name and description
                            let json_clone = json.clone();
                            if let Some(name) = json_clone.get("name").and_then(|v| v.as_str()) {
                                site_title = name.to_string();
                            }
                            
                            if let Some(desc) = json_clone.get("description").and_then(|v| v.as_str()) {
                                site_description = desc.to_string();
                            }
                            
                            // Check for assemble-specific config
                            if let Some(assemble_config) = json_clone.get("assemble") {
                                if let Some(options) = assemble_config.get("options") {
                                    if let Some(title) = options.get("title").and_then(|v| v.as_str()) {
                                        site_title = title.to_string();
                                    }
                                    
                                    if let Some(desc) = options.get("description").and_then(|v| v.as_str()) {
                                        site_description = desc.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create Rustyll _config.yml
        let config_content = format!(r#"# Configuration for site migrated from Assemble

title: "{}"
description: "{}"
url: ""

# Build settings
markdown: kramdown
highlighter: rouge

# Migration note
migration:
  source: assemble
  date: {}

# Collections
collections:
  pages:
    output: true
    permalink: /:path/
"#, site_title, site_description, chrono::Local::now().format("%Y-%m-%d"));
        
        // Write the new config file
        let dest_config = dest_dir.join("_config.yml");
        fs::write(&dest_config, config_content)
            .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Rustyll configuration created".to_string(),
        });
        
        if !found_config {
            result.warnings.push(
                "No Assemble configuration files found. A basic config has been created.".to_string()
            );
        } else {
            result.warnings.push(
                "Assemble configuration has been partially migrated. You may need to manually adjust some settings.".to_string()
            );
        }
        
        Ok(())
    }
} 