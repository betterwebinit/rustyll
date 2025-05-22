use std::path::Path;
use std::fs;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::NanocMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for nanoc.yaml or config.yaml
        let config_files = vec![
            source_dir.join("nanoc.yaml"),
            source_dir.join("config.yaml"),
        ];
        
        let mut found_config = false;
        
        for config_file in config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Found Nanoc configuration file: {}", config_file.display());
                }
                
                // Keep a copy of the original config for reference
                let ref_file = dest_dir.join("nanoc_original.yaml");
                copy_file(&config_file, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: "nanoc_original.yaml".to_string(),
                    change_type: ChangeType::Copied,
                    description: "Original Nanoc configuration preserved for reference".to_string(),
                });
                
                // Read the config file
                let content = fs::read_to_string(&config_file)
                    .map_err(|e| format!("Failed to read config file: {}", e))?;
                
                // Try to parse YAML
                if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    // Extract relevant information
                    let site_title = yaml.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Nanoc Site");
                    
                    let site_description = yaml.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("A site migrated from Nanoc");
                    
                    let base_url = yaml.get("base_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    // Create Rustyll _config.yml
                    let config_content = format!(r#"# Configuration for migrated Nanoc site

title: "{}"
description: "{}"
url: "{}"

# Build settings
markdown: kramdown
highlighter: rouge

# Migration note
migration:
  source: nanoc
  date: {}

# Collections
collections:
  pages:
    output: true
    permalink: /:path/
"#, site_title, site_description, base_url, chrono::Local::now().format("%Y-%m-%d"));
                    
                    // Write the new config file
                    let dest_config = dest_dir.join("_config.yml");
                    fs::write(&dest_config, config_content)
                        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
                    
                    result.changes.push(MigrationChange {
                        file_path: "_config.yml".to_string(),
                        change_type: ChangeType::Created,
                        description: "Rustyll configuration created from Nanoc config".to_string(),
                    });
                    
                    found_config = true;
                    break;
                } else {
                    result.warnings.push(format!("Failed to parse configuration file: {}", config_file.display()));
                }
            }
        }
        
        // Check for Rules file, which contains routing rules
        let rules_files = vec![
            source_dir.join("Rules"),
            source_dir.join("Rules.rb"),
        ];
        
        for rules_file in rules_files {
            if rules_file.exists() {
                if verbose {
                    log::info!("Found Nanoc Rules file: {}", rules_file.display());
                }
                
                // Keep a copy for reference
                let ref_file = dest_dir.join("nanoc_rules.rb");
                copy_file(&rules_file, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: "nanoc_rules.rb".to_string(),
                    change_type: ChangeType::Copied,
                    description: "Nanoc Rules file preserved for reference".to_string(),
                });
                
                result.warnings.push(
                    "Nanoc Rules file contains routing information that needs manual conversion to Rustyll".to_string()
                );
                
                break;
            }
        }
        
        if !found_config {
            // Create a basic config file
            let basic_config = r#"# Basic configuration for migrated Nanoc site
title: "Migrated Nanoc Site"
description: "A site migrated from Nanoc to Rustyll"
url: ""

# Build settings
markdown: kramdown
permalink: pretty

# Migration note
migration:
  source: nanoc
  date: 2023-01-01
"#;
            
            let dest_config = dest_dir.join("_config.yml");
            fs::write(&dest_config, basic_config)
                .map_err(|e| format!("Failed to write basic _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Basic Rustyll configuration created".to_string(),
            });
            
            result.warnings.push(
                "Could not find or parse Nanoc configuration. A basic config has been created.".to_string()
            );
        }
        
        Ok(())
    }
} 