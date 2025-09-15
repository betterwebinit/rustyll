use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::NanocMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Nanoc, data can be in several places:
        // - data/ directory
        // - lib/ directory (may contain data or helpers)
        
        let data_dirs = vec![
            source_dir.join("data"),
            source_dir.join("lib"),
        ];
        
        // Create destination data directory
        let dest_data_dir = dest_dir.join("_data");
        create_dir_if_not_exists(&dest_data_dir)?;
        
        let mut found_data = false;
        
        // Process data directories
        for data_dir in data_dirs {
            if data_dir.exists() && data_dir.is_dir() {
                if verbose {
                    log::info!("Migrating data from {}", data_dir.display());
                }
                
                found_data = true;
                self.process_data_directory(&data_dir, &dest_data_dir, result)?;
            }
        }
        
        // Also check for config.yaml that might contain data
        let config_files = vec![
            source_dir.join("nanoc.yaml"),
            source_dir.join("config.yaml"),
        ];
        
        for config_file in config_files {
            if config_file.exists() {
                if verbose {
                    log::info!("Checking configuration file for data: {}", config_file.display());
                }
                
                // Try to extract data sections from config
                self.extract_data_from_config(&config_file, &dest_data_dir, result)?;
                found_data = true;
                break;
            }
        }
        
        if !found_data {
            if verbose {
                log::info!("No data sources found. Creating sample data file.");
            }
            
            // Create a sample navigation data file
            let sample_data = r#"# Sample navigation data
main_nav:
  - title: Home
    url: /
  - title: About
    url: /about/
  - title: Blog
    url: /blog/
"#;
            
            let sample_data_file = dest_data_dir.join("navigation.yml");
            fs::write(&sample_data_file, sample_data)
                .map_err(|e| format!("Failed to write sample data file: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: "_data/navigation.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Sample navigation data created".to_string(),
            });
        }
        
        // Create README for data directory
        let data_readme = r#"# Data Directory

This directory contains data files that can be used in your Rustyll site.

## Data Usage

Data files in Rustyll:
- Can be YAML (.yml) or JSON (.json) format
- Are accessible in templates via the `site.data` variable

Example:
```liquid
{% for item in site.data.navigation.main_nav %}
  <a href="{{ item.url }}">{{ item.title }}</a>
{% endfor %}
```

## Migrated Data

This directory contains data migrated from Nanoc:
- Configuration data extracted from nanoc.yaml/config.yaml
- Data files from the data/ directory (if present)
- Helper data from the lib/ directory (if present)
"#;
        
        crate::migrate::write_readme(&dest_data_dir, data_readme)?;
        
        Ok(())
    }
    
    fn process_data_directory(&self, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        for entry in WalkDir::new(data_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Check if this is a data file we can handle
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    match ext.as_ref() {
                        "yml" | "yaml" | "json" => {
                            // These can be copied directly
                            self.copy_data_file(file_path, data_dir, dest_data_dir, result)?;
                        },
                        "rb" => {
                            // Ruby files might contain data or helpers
                            self.process_ruby_data_file(file_path, data_dir, dest_data_dir, result)?;
                        },
                        _ => {
                            // Other files are not data files, so we'll ignore them
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn copy_data_file(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Get the relative path from data directory
        let rel_path = file_path.strip_prefix(data_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Determine the destination file path, ensuring .yml extension
        let dest_name = if let Some(ext) = file_path.extension() {
            if ext == "yaml" {
                format!("{}.yml", file_path.file_stem().unwrap().to_string_lossy())
            } else {
                rel_path.file_name().unwrap().to_string_lossy().to_string()
            }
        } else {
            format!("{}.yml", rel_path.file_name().unwrap().to_string_lossy())
        };
        
        let dest_path = dest_data_dir.join(&dest_name);
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_data/{}", dest_name),
            change_type: ChangeType::Copied,
            description: "Data file copied".to_string(),
        });
        
        Ok(())
    }
    
    fn process_ruby_data_file(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Ruby files are complex, we'll just copy them for reference
        // and add a warning about manual conversion
        
        // Get the file name
        let file_name = file_path.file_name()
            .ok_or_else(|| "Invalid file name".to_string())?
            .to_string_lossy();
            
        // Create a reference directory
        let ref_dir = dest_data_dir.join("ruby_reference");
        create_dir_if_not_exists(&ref_dir)?;
        
        let dest_path = ref_dir.join(&*file_name);
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        result.changes.push(MigrationChange {
            file_path: format!("_data/ruby_reference/{}", file_name),
            change_type: ChangeType::Copied,
            description: "Ruby helper/data file copied for reference".to_string(),
        });
        
        // Add a warning
        result.warnings.push(
            format!("Ruby file {} needs manual conversion to YAML/JSON data or Liquid helpers", file_name)
        );
        
        Ok(())
    }
    
    fn extract_data_from_config(&self, config_file: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the config file
        let content = fs::read_to_string(config_file)
            .map_err(|e| format!("Failed to read config file {}: {}", config_file.display(), e))?;
            
        // Try to parse YAML
        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            // Look for data sections
            let data_sections = vec![
                "data_sources", "collections", "taxonomies", "navigation", 
                "menu", "site", "metadata", "data"
            ];
            
            for section in data_sections {
                if let Some(data) = yaml.get(section) {
                    // Extract this section to a separate file
                    let data_filename = format!("{}.yml", section);
                    let data_path = dest_data_dir.join(&data_filename);
                    
                    // Serialize the data to YAML
                    let yaml_content = serde_yaml::to_string(data)
                        .map_err(|e| format!("Failed to serialize {} data: {}", section, e))?;
                        
                    fs::write(&data_path, yaml_content)
                        .map_err(|e| format!("Failed to write data file {}: {}", data_filename, e))?;
                        
                    result.changes.push(MigrationChange {
                        file_path: format!("_data/{}", data_filename),
                        change_type: ChangeType::Created,
                        description: format!("Data extracted from config: {}", section).to_string(),
                    });
                }
            }
        }
        
        Ok(())
    }
} 