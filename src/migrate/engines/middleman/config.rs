use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};
use regex;
use chrono;

impl super::MiddlemanMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for Middleman config.rb file
        let config_rb = source_dir.join("config.rb");
        let mut found_config = false;
        
        if config_rb.exists() {
            if verbose {
                log::info!("Found Middleman configuration file: {}", config_rb.display());
            }
            
            // Copy the config.rb as reference
            let dest_config_ref = dest_dir.join("middleman_config.rb");
            copy_file(&config_rb, &dest_config_ref)?;
            
            result.changes.push(MigrationChange {
                file_path: "middleman_config.rb".to_string(),
                change_type: ChangeType::Copied,
                description: "Original Middleman config preserved for reference".to_string(),
            });
            
            found_config = true;
        }
        
        // Check for Gemfile which might contain middleman config
        let gemfile = source_dir.join("Gemfile");
        if gemfile.exists() {
            if verbose {
                log::info!("Found Gemfile, checking for middleman configuration");
            }
            
            // Copy Gemfile as reference
            let dest_gemfile = dest_dir.join("middleman_Gemfile");
            copy_file(&gemfile, &dest_gemfile)?;
            
            result.changes.push(MigrationChange {
                file_path: "middleman_Gemfile".to_string(),
                change_type: ChangeType::Copied,
                description: "Original Gemfile preserved for reference".to_string(),
            });
            
            // Also check for Gemfile.lock
            let gemfile_lock = source_dir.join("Gemfile.lock");
            if gemfile_lock.exists() {
                let dest_gemfile_lock = dest_dir.join("middleman_Gemfile.lock");
                copy_file(&gemfile_lock, &dest_gemfile_lock)?;
                
                result.changes.push(MigrationChange {
                    file_path: "middleman_Gemfile.lock".to_string(),
                    change_type: ChangeType::Copied,
                    description: "Original Gemfile.lock preserved for reference".to_string(),
                });
            }
        }
        
        // Create a compatible _config.yml for Rustyll
        let dest_config = dest_dir.join("_config.yml");
        
        // Attempt to extract some info from the config.rb if available
        if config_rb.exists() {
            // Read the config.rb file
            let config_content = fs::read_to_string(&config_rb)
                .map_err(|e| format!("Failed to read config.rb: {}", e))?;
            
            // Extract site details from config.rb
            // Look for common Middleman config patterns
            let mut title = "Middleman Site".to_string();
            let mut description = "A site migrated from Middleman".to_string();
            let mut url = "".to_string();
            let mut base_url = "".to_string();
            
            // Extract title - look for patterns like: set :site_title, "My Site"
            if let Some(title_match) = regex::Regex::new(r#"set\s+:site_title\s*,\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = title_match.get(1) {
                    title = m.as_str().to_string();
                }
            } else if let Some(title_match) = regex::Regex::new(r#"config\[:title\]\s*=\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = title_match.get(1) {
                    title = m.as_str().to_string();
                }
            }
            
            // Extract description
            if let Some(desc_match) = regex::Regex::new(r#"set\s+:description\s*,\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = desc_match.get(1) {
                    description = m.as_str().to_string();
                }
            } else if let Some(desc_match) = regex::Regex::new(r#"config\[:description\]\s*=\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = desc_match.get(1) {
                    description = m.as_str().to_string();
                }
            }
            
            // Extract URL
            if let Some(url_match) = regex::Regex::new(r#"set\s+:url\s*,\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = url_match.get(1) {
                    url = m.as_str().to_string();
                }
            } else if let Some(url_match) = regex::Regex::new(r#"config\[:url\]\s*=\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = url_match.get(1) {
                    url = m.as_str().to_string();
                }
            }
            
            // Extract base URL / relative URL root
            if let Some(base_match) = regex::Regex::new(r#"set\s+:base_url\s*,\s*["']([^"']+)["']"#)
                .map_err(|_| "Regex error".to_string())?
                .captures(&config_content) {
                if let Some(m) = base_match.get(1) {
                    base_url = m.as_str().to_string();
                }
            } else if regex::Regex::new(r#"set\s+:relative_links\s*,\s*true"#)
                .map_err(|_| "Regex error".to_string())?
                .is_match(&config_content) {
                // If relative_links is true, baseurl is generally empty
                base_url = "".to_string();
            }
            
            // Create config.yml based on extracted values
            let config_yml = format!(r#"# Site settings
title: "{}"
description: "{}"
url: "{}"
baseurl: "{}"

# Build settings
markdown: kramdown
permalink: pretty

# Middleman Migration
middleman:
  migration_date: {}
  version: unknown

# Collections
collections:
  pages:
    output: true
    permalink: /:path/
"#, title, description, url, base_url, chrono::Local::now().format("%Y-%m-%d"));

            fs::write(&dest_config, config_yml)
                .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Created Rustyll configuration from Middleman config".to_string(),
            });
        }
        
        if found_config {
            result.warnings.push(
                "Middleman Ruby configuration needs manual adaptation to Rustyll's YAML format".to_string()
            );
        } else {
            result.warnings.push(
                "No Middleman config.rb found. Basic _config.yml created for Rustyll".to_string()
            );
        }
        
        Ok(())
    }
} 