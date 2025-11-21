use std::path::Path;
use std::fs;
use serde_yaml;
use toml;
use serde_json;
use serde::Deserialize;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file, create_dir_if_not_exists};

#[derive(Debug, Deserialize)]
struct HugoConfig {
    title: Option<String>,
    #[serde(rename = "baseURL")]
    base_url: Option<String>,
    #[serde(rename = "languageCode")]
    language_code: Option<String>,
    theme: Option<String>,
    params: Option<serde_json::Value>,
    menu: Option<serde_json::Value>,
    taxonomies: Option<serde_json::Value>,
}

impl super::HugoMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Hugo config can be in several formats and locations
        let config_files = vec![
            (source_dir.join("config.toml"), "toml"),
            (source_dir.join("config.yaml"), "yaml"),
            (source_dir.join("config.json"), "json"),
        ];
        
        let mut hugo_config: Option<HugoConfig> = None;
        let mut config_format = "";
        let mut original_config_path = None;
        
        // Try to find and parse a config file
        for (config_file, format) in config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Found Hugo configuration file: {}", config_file.display());
                }
                
                // Read the config file
                let content = fs::read_to_string(&config_file)
                    .map_err(|e| format!("Failed to read config file: {}", e))?;
                
                // Parse based on format
                match format {
                    "toml" => {
                        match toml::from_str::<HugoConfig>(&content) {
                            Ok(config) => {
                                hugo_config = Some(config);
                                config_format = "toml";
                                original_config_path = Some(config_file);
                                break;
                            },
                            Err(e) => {
                                result.warnings.push(format!("Failed to parse TOML config: {}", e));
                            }
                        }
                    },
                    "yaml" => {
                        match serde_yaml::from_str::<HugoConfig>(&content) {
                            Ok(config) => {
                                hugo_config = Some(config);
                                config_format = "yaml";
                                original_config_path = Some(config_file);
                                break;
                            },
                            Err(e) => {
                                result.warnings.push(format!("Failed to parse YAML config: {}", e));
                            }
                        }
                    },
                    "json" => {
                        match serde_json::from_str::<HugoConfig>(&content) {
                            Ok(config) => {
                                hugo_config = Some(config);
                                config_format = "json";
                                original_config_path = Some(config_file);
                                break;
                            },
                            Err(e) => {
                                result.warnings.push(format!("Failed to parse JSON config: {}", e));
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // If we found a config file, convert it
        if let Some(config) = hugo_config {
            let dest_config = dest_dir.join("_config.yml");
            
            // Keep a copy of the original for reference
            if let Some(orig_path) = original_config_path {
                let ext = orig_path.extension().unwrap_or_default();
                let ref_file = dest_dir.join(format!("hugo_original.{}", ext.to_string_lossy()));
                
                copy_file(&orig_path, &ref_file)?;
                
                result.changes.push(MigrationChange {
                    file_path: format!("hugo_original.{}", ext.to_string_lossy()),
                    change_type: ChangeType::Copied,
                    description: "Original Hugo config preserved for reference".to_string(),
                });
            }
            
            // Create a new compatible _config.yml
            let site_title = config.title.as_deref().unwrap_or("Hugo Site");
            let site_url = config.base_url.as_deref().unwrap_or("");
            let site_language = config.language_code.as_deref().unwrap_or("en");
            let site_theme = config.theme.as_deref().unwrap_or("default");
            
            // Create YAML config
            let rustyll_config = format!(r#"# Configuration migrated from Hugo ({})

# Site settings
title: "{}"
baseurl: "{}"
language: "{}"
original_theme: "{}"

# Build settings
markdown: kramdown
permalink: /:categories/:year/:month/:day/:title/

# Migrated from Hugo
migrated_from: hugo
build_date: {}

# Collections
collections:
  pages:
    output: true
    permalink: /:path/
"#, 
                config_format, 
                site_title, 
                site_url, 
                site_language, 
                site_theme,
                chrono::Local::now().format("%Y-%m-%d")
            );
            
            // Write the new config
            fs::write(&dest_config, rustyll_config)
                .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Converted,
                description: "Configuration converted from Hugo format".to_string(),
            });
            
            // If there are params, save them separately
            if let Some(params) = config.params {
                let data_dir = dest_dir.join("_data");
                create_dir_if_not_exists(&data_dir)?;
                
                let params_file = data_dir.join("params.yml");
                let params_yaml = serde_yaml::to_string(&params)
                    .map_err(|e| format!("Failed to convert params to YAML: {}", e))?;
                
                fs::write(&params_file, params_yaml)
                    .map_err(|e| format!("Failed to write params.yml: {}", e))?;
                
                result.changes.push(MigrationChange {
                    file_path: "_data/params.yml".to_string(),
                    change_type: ChangeType::Converted,
                    description: "Hugo site parameters converted and saved as data file".to_string(),
                });
            }
            
            // If there's menu configuration, save it separately
            if let Some(menu) = config.menu {
                let data_dir = dest_dir.join("_data");
                create_dir_if_not_exists(&data_dir)?;
                
                let menu_file = data_dir.join("menu.yml");
                let menu_yaml = serde_yaml::to_string(&menu)
                    .map_err(|e| format!("Failed to convert menu to YAML: {}", e))?;
                
                fs::write(&menu_file, menu_yaml)
                    .map_err(|e| format!("Failed to write menu.yml: {}", e))?;
                
                result.changes.push(MigrationChange {
                    file_path: "_data/menu.yml".to_string(),
                    change_type: ChangeType::Converted,
                    description: "Hugo menu configuration converted and saved as data file".to_string(),
                });
            }
        } else {
            // No config found, create a basic one
            let dest_config = dest_dir.join("_config.yml");
            let basic_config = r#"# Basic configuration for migrated Hugo site
title: "Migrated Hugo Site"
description: "A site migrated from Hugo to Rustyll"
baseurl: ""

# Build settings
markdown: kramdown
permalink: /:categories/:year/:month/:day/:title/

# Migrated from Hugo
migrated_from: hugo
build_date: 2023-01-01
"#;
            
            fs::write(&dest_config, basic_config)
                .map_err(|e| format!("Failed to write basic _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Basic configuration file created for Rustyll".to_string(),
            });
            
            result.warnings.push(
                "Could not find Hugo configuration. A basic config has been created.".to_string()
            );
        }
        
        // Check for config directory structure
        let config_dir = source_dir.join("config");
        if config_dir.exists() && config_dir.is_dir() {
            if verbose {
                log::info!("Found Hugo config directory structure");
            }
            
            // Copy the config directory for reference
            let dest_config_dir = dest_dir.join("_hugo_config");
            create_dir_if_not_exists(&dest_config_dir)?;
            
            // This is a simplified approach - in a real implementation we would
            // parse each file and convert them appropriately
            copy_directory(&config_dir, &dest_config_dir, result)?;
            
            result.warnings.push(
                "Hugo uses a structured config directory. These files have been copied to _hugo_config/ for reference, but may need manual conversion.".to_string()
            );
        }
        
        Ok(())
    }
}

// Helper function to copy a directory
fn copy_directory(source: &Path, dest: &Path, result: &mut MigrationResult) -> Result<(), String> {
    create_dir_if_not_exists(dest)?;
    
    for entry in fs::read_dir(source).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let dest_path = dest.join(path.file_name().unwrap());
        
        if path.is_dir() {
            copy_directory(&path, &dest_path, result)?;
        } else {
            copy_file(&path, &dest_path)?;
            
            let relative_path = dest_path.strip_prefix(dest.parent().unwrap())
                .map_err(|_| "Failed to get relative path".to_string())?;
            
            result.changes.push(MigrationChange {
                file_path: relative_path.to_string_lossy().to_string(),
                change_type: ChangeType::Copied,
                description: "Hugo config file copied for reference".to_string(),
            });
        }
    }
    
    Ok(())
} 