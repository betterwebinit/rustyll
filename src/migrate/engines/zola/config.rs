use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

// Public module function that can be called from anywhere
pub fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let migrator = super::ZolaMigrator::new();
    migrator.migrate_config(source_dir, dest_dir, verbose, result)
}

impl super::ZolaMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let source_config = source_dir.join("config.toml");
        
        if source_config.exists() {
            if verbose {
                log::info!("Migrating Zola configuration file");
            }
            
            // Copy the Zola config as reference
            let dest_config_ref = dest_dir.join("zola_config.toml");
            copy_file(&source_config, &dest_config_ref)?;
            
            // Create a compatible _config.yml for Rustyll
            let dest_config = dest_dir.join("_config.yml");
            
            // In a real implementation, we'd convert TOML to YAML
            // For now, just create a basic config.yml with reference to the original
            let config_content = r#"# Converted from Zola config.toml
# Original config is saved as zola_config.toml for reference

title: Migrated Zola Site
description: A site migrated from Zola to Rustyll
url: ""  # Update with your site's URL
baseurl: ""

# Build settings
markdown: kramdown
highlighter: rouge

# Migration note
migration:
  source: zola
  date: 2023-01-01
"#;
            
            fs::write(&dest_config, config_content)
                .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "zola_config.toml".to_string(),
                change_type: ChangeType::Copied,
                description: "Original Zola config preserved for reference".to_string(),
            });
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Rustyll configuration created".to_string(),
            });
            
            result.warnings.push(
                "Manual review of configuration needed. Zola and Rustyll use different configuration formats and options.".to_string()
            );
        } else {
            result.warnings.push(
                "No Zola config.toml found. Basic _config.yml created.".to_string()
            );
            
            // Create a minimal config.yml
            let dest_config = dest_dir.join("_config.yml");
            let config_content = r#"# Basic configuration for migrated Zola site
title: Migrated Zola Site
description: A site migrated from Zola to Rustyll
url: ""  # Update with your site's URL
baseurl: ""

# Build settings
markdown: kramdown
highlighter: rouge
"#;
            
            fs::write(&dest_config, config_content)
                .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Basic Rustyll configuration created".to_string(),
            });
        }
        
        Ok(())
    }
} 