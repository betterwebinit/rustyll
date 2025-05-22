use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::EleventyMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for the various config files Eleventy uses
        let config_files = vec![
            source_dir.join(".eleventy.js"),
            source_dir.join("eleventy.config.js"),
            source_dir.join(".eleventy.cjs"),
        ];
        
        let mut found_config = false;
        
        for config_file in config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Found Eleventy configuration file: {}", config_file.display());
                }
                
                // Copy the 11ty config as reference
                let file_name = config_file.file_name()
                    .ok_or_else(|| "Invalid file name".to_string())?
                    .to_string_lossy();
                
                let dest_config_ref = dest_dir.join(format!("eleventy_{}", file_name));
                copy_file(&config_file, &dest_config_ref)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("eleventy_{}", file_name),
                    change_type: ChangeType::Copied,
                    description: "Original Eleventy config preserved for reference".to_string(),
                });
                
                found_config = true;
            }
        }
        
        // Check for package.json which might contain eleventy config
        let package_json = source_dir.join("package.json");
        if package_json.exists() {
            if verbose {
                log::info!("Found package.json, checking for eleventy configuration");
            }
            
            // Copy package.json as reference
            let dest_package_json = dest_dir.join("eleventy_package.json");
            copy_file(&package_json, &dest_package_json)?;
            
            result.changes.push(MigrationChange {
                file_path: "eleventy_package.json".to_string(),
                change_type: ChangeType::Copied,
                description: "Original package.json preserved for reference".to_string(),
            });
        }
        
        // Create a compatible _config.yml for Rustyll
        let dest_config = dest_dir.join("_config.yml");
        
        let config_content = r#"# Configuration for migrated Eleventy site

title: Migrated Eleventy Site
description: A site migrated from Eleventy to Rustyll
url: ""  # Update with your site's URL
baseurl: ""

# Build settings
markdown: kramdown
highlighter: rouge

# Migration note
migration:
  source: eleventy
  date: 2023-01-01

# Collections
collections:
  posts:
    output: true
    permalink: /posts/:title/
"#;
        
        fs::write(&dest_config, config_content)
            .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Rustyll configuration created".to_string(),
        });
        
        if found_config {
            result.warnings.push(
                "Eleventy JavaScript configuration needs manual adaptation to Rustyll's YAML format".to_string()
            );
        } else {
            result.warnings.push(
                "No Eleventy config found. Basic _config.yml created for Rustyll".to_string()
            );
        }
        
        Ok(())
    }
} 