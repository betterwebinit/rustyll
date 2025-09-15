use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use serde_json;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, write_readme};

impl super::HarpMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, data is typically stored in _data.json files throughout the project
        // This function extracts all _data.json files and converts them to YAML in _data/
        
        // Determine content directory
        let content_dir = if source_dir.join("public").exists() && source_dir.join("public").is_dir() {
            source_dir.join("public")
        } else {
            source_dir.to_path_buf()
        };
        
        // Create destination data directory
        let dest_data_dir = dest_dir.join("_data");
        create_dir_if_not_exists(&dest_data_dir)?;
        
        // Find all _data.json files in the project
        let mut found_data = false;
        
        for entry in WalkDir::new(&content_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                
                if file_name == "_data.json" {
                    if verbose {
                        log::info!("Migrating data: {}", file_path.display());
                    }
                    
                    self.process_data_file(file_path, &content_dir, &dest_data_dir, result)?;
                    found_data = true;
                }
            }
        }
        
        // Also check for harp.json and _harp.json for global data
        for global_data_file in &[source_dir.join("harp.json"), source_dir.join("_harp.json")] {
            if global_data_file.exists() {
                if verbose {
                    log::info!("Migrating global data from: {}", global_data_file.display());
                }
                
                self.process_global_data_file(global_data_file, &dest_data_dir, result)?;
                found_data = true;
            }
        }
        
        // Create sample data if none was found
        if !found_data {
            if verbose {
                log::info!("No data files found. Creating sample data.");
            }
            
            self.create_sample_data(&dest_data_dir, result)?;
        }
        
        // Create README for the data directory
        write_data_readme(&dest_data_dir, result)?;
        
        Ok(())
    }
    
    fn process_data_file(&self, file_path: &Path, source_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the JSON data file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read data file {}: {}", file_path.display(), e))?;
            
        // Parse the JSON
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse data file {}: {}", file_path.display(), e))?;
            
        // Determine the data file name based on its location
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
        let parent_dir = rel_path.parent().unwrap_or_else(|| Path::new(""));
        
        // Create a name for the data file based on its location
        let data_file_name = if parent_dir.as_os_str().is_empty() {
            "site.yml".to_string()
        } else {
            // Use the directory name as the data file name
            let dir_name = parent_dir.components().last().unwrap()
                .as_os_str().to_string_lossy();
                
            format!("{}.yml", dir_name)
        };
        
        // Convert JSON to YAML
        let yaml_content = match serde_yaml::to_string(&json_data) {
            Ok(yaml) => yaml,
            Err(e) => return Err(format!("Failed to convert JSON to YAML: {}", e)),
        };
        
        // Write the YAML file
        let dest_path = dest_data_dir.join(&data_file_name);
        fs::write(&dest_path, yaml_content)
            .map_err(|e| format!("Failed to write data file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_data/{}", data_file_name),
            change_type: ChangeType::Converted,
            description: "JSON data converted to YAML".to_string(),
        });
        
        Ok(())
    }
    
    fn process_global_data_file(&self, file_path: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the global config file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read global data file {}: {}", file_path.display(), e))?;
            
        // Parse the JSON
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse global data file {}: {}", file_path.display(), e))?;
            
        // Extract the globals section if it exists
        if let Some(globals) = json_data.get("globals") {
            // Convert globals to YAML
            let yaml_content = match serde_yaml::to_string(globals) {
                Ok(yaml) => yaml,
                Err(e) => return Err(format!("Failed to convert globals to YAML: {}", e)),
            };
            
            // Write the globals file
            let dest_path = dest_data_dir.join("globals.yml");
            fs::write(&dest_path, yaml_content)
                .map_err(|e| format!("Failed to write globals data file: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: "_data/globals.yml".to_string(),
                change_type: ChangeType::Converted,
                description: "Global data extracted from Harp config".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn create_sample_data(&self, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a sample site data file
        let site_data = r#"# Site metadata
title: "Sample Harp Site"
description: "A site migrated from Harp to Rustyll"
author: "Harp User"

# Navigation
navigation:
  - title: Home
    url: /
  - title: About
    url: /about/
  - title: Blog
    url: /blog/
"#;
        
        let site_data_file = dest_data_dir.join("site.yml");
        fs::write(&site_data_file, site_data)
            .map_err(|e| format!("Failed to write sample data file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_data/site.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Sample site data created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to write a README for the data directory
fn write_data_readme(data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    let readme_content = r#"# Data Directory

This directory contains data files that can be used in your Rustyll site.

## Data Files

Data files in Rustyll:
- Are YAML files with `.yml` extension
- Can be accessed in templates via the `site.data` namespace
- Are organized by filename (e.g., `site.yml` becomes `site.data.site`)

## Migrated Data

The data in this directory was migrated from Harp:
- `_data.json` files were converted to YAML
- The directory structure was flattened (directory name became the data file name)
- Global data from `harp.json` or `_harp.json` was extracted to `globals.yml`

## Example Usage

To use data in your templates:

```liquid
{% for item in site.data.site.navigation %}
  <a href="{{ item.url }}">{{ item.title }}</a>
{% endfor %}
```
"#;

    write_readme(data_dir, readme_content)?;
    
    result.changes.push(MigrationChange {
        file_path: "_data/README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Data directory README created".to_string(),
    });
    
    Ok(())
} 