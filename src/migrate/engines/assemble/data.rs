use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use serde_yaml;
use serde_json;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::AssembleMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Assemble, data can be in several places:
        // - data/ directory
        // - src/data/ directory
        // - app/data/ directory
        
        let data_dirs = vec![
            source_dir.join("data"),
            source_dir.join("src").join("data"),
            source_dir.join("app").join("data"),
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
        
        // Also check for any package.json that might contain configuration data
        let package_json = source_dir.join("package.json");
        if package_json.exists() {
            if verbose {
                log::info!("Checking package.json for data: {}", package_json.display());
            }
            
            // Extract data from package.json
            self.extract_data_from_package_json(&package_json, &dest_data_dir, result)?;
            found_data = true;
        }
        
        // If no data was found, create sample data
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

This directory contains data migrated from Assemble:
- JSON data files converted to YAML where appropriate
- Configuration data extracted from package.json or other sources
- Any other data files found in the Assemble project
"#;
        
        write_readme(&dest_data_dir, data_readme)?;
        
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
                        "yml" | "yaml" => {
                            // YAML files can be copied directly, ensuring .yml extension
                            self.copy_yaml_file(file_path, data_dir, dest_data_dir, result)?;
                        },
                        "json" => {
                            // Convert JSON to YAML for better Rustyll compatibility
                            self.convert_json_to_yaml(file_path, data_dir, dest_data_dir, result)?;
                        },
                        "js" => {
                            // JavaScript data files need to be handled specially
                            self.handle_js_data_file(file_path, data_dir, dest_data_dir, result)?;
                        },
                        _ => {
                            // Other files are not typical data files, but we'll copy them anyway
                            self.copy_other_data_file(file_path, data_dir, dest_data_dir, result)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn copy_yaml_file(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Get the relative path from data directory
        let rel_path = file_path.strip_prefix(data_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Ensure .yml extension (convert from .yaml if needed)
        let dest_name = if let Some(file_stem) = file_path.file_stem() {
            format!("{}.yml", file_stem.to_string_lossy())
        } else {
            return Err(format!("Invalid file name: {}", file_path.display()));
        };
        
        // Create subdirectories if needed
        if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                let dest_subdir = dest_data_dir.join(parent);
                create_dir_if_not_exists(&dest_subdir)?;
            }
        }
        
        // Determine the destination path
        let dest_path = if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                dest_data_dir.join(parent).join(&dest_name)
            } else {
                dest_data_dir.join(&dest_name)
            }
        } else {
            dest_data_dir.join(&dest_name)
        };
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        let rel_dest_path = dest_path.strip_prefix(dest_data_dir.parent().unwrap())
            .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
        
        result.changes.push(MigrationChange {
            file_path: rel_dest_path.to_string_lossy().to_string(),
            change_type: ChangeType::Copied,
            description: "YAML data file copied".to_string(),
        });
        
        Ok(())
    }
    
    fn convert_json_to_yaml(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the JSON file
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read JSON file {}: {}", file_path.display(), e))?;
            
        // Parse the JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse JSON file {}: {}", file_path.display(), e))?;
            
        // Convert to YAML
        let yaml_content = serde_yaml::to_string(&json_value)
            .map_err(|e| format!("Failed to convert JSON to YAML: {}", e))?;
        
        // Get the relative path from data directory
        let rel_path = file_path.strip_prefix(data_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Create the destination filename with .yml extension
        let dest_name = if let Some(file_stem) = file_path.file_stem() {
            format!("{}.yml", file_stem.to_string_lossy())
        } else {
            return Err(format!("Invalid file name: {}", file_path.display()));
        };
        
        // Create subdirectories if needed
        if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                let dest_subdir = dest_data_dir.join(parent);
                create_dir_if_not_exists(&dest_subdir)?;
            }
        }
        
        // Determine the destination path
        let dest_path = if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                dest_data_dir.join(parent).join(&dest_name)
            } else {
                dest_data_dir.join(&dest_name)
            }
        } else {
            dest_data_dir.join(&dest_name)
        };
        
        // Write the YAML file
        fs::write(&dest_path, yaml_content)
            .map_err(|e| format!("Failed to write YAML file: {}", e))?;
        
        let rel_dest_path = dest_path.strip_prefix(dest_data_dir.parent().unwrap())
            .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
        
        result.changes.push(MigrationChange {
            file_path: rel_dest_path.to_string_lossy().to_string(),
            change_type: ChangeType::Converted,
            description: "JSON data file converted to YAML".to_string(),
        });
        
        Ok(())
    }
    
    fn handle_js_data_file(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // For JS data files, we'll create a reference directory and add a comment
        // because we can't automatically convert JS module exports to YAML
        
        // Create a reference directory for JS files
        let ref_dir = dest_data_dir.join("js_reference");
        create_dir_if_not_exists(&ref_dir)?;
        
        // Get the file name
        let file_name = file_path.file_name()
            .ok_or_else(|| "Invalid file name".to_string())?;
            
        let dest_path = ref_dir.join(file_name);
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        let rel_dest_path = dest_path.strip_prefix(dest_data_dir.parent().unwrap())
            .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
        
        result.changes.push(MigrationChange {
            file_path: rel_dest_path.to_string_lossy().to_string(),
            change_type: ChangeType::Copied,
            description: "JavaScript data file copied for reference (manual conversion needed)".to_string(),
        });
        
        // Add a warning about JS data files
        result.warnings.push(format!(
            "JavaScript data file '{}' requires manual conversion to YAML or JSON for Rustyll compatibility.",
            file_path.display()
        ));
        
        Ok(())
    }
    
    fn copy_other_data_file(&self, file_path: &Path, data_dir: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // For other file types, we'll just copy them as-is
        
        // Get the relative path from data directory
        let rel_path = file_path.strip_prefix(data_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
        
        // Create subdirectories if needed
        if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                let dest_subdir = dest_data_dir.join(parent);
                create_dir_if_not_exists(&dest_subdir)?;
            }
        }
        
        // Determine the destination path
        let dest_path = if let Some(parent) = rel_path.parent() {
            if !parent.as_os_str().is_empty() {
                dest_data_dir.join(parent).join(rel_path.file_name().unwrap())
            } else {
                dest_data_dir.join(rel_path.file_name().unwrap())
            }
        } else {
            dest_data_dir.join(rel_path.file_name().unwrap())
        };
        
        // Copy the file
        copy_file(file_path, &dest_path)?;
        
        let rel_dest_path = dest_path.strip_prefix(dest_data_dir.parent().unwrap())
            .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
        
        result.changes.push(MigrationChange {
            file_path: rel_dest_path.to_string_lossy().to_string(),
            change_type: ChangeType::Copied,
            description: "Other data file copied".to_string(),
        });
        
        Ok(())
    }
    
    fn extract_data_from_package_json(&self, package_json_path: &Path, dest_data_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read package.json
        let content = fs::read_to_string(package_json_path)
            .map_err(|e| format!("Failed to read package.json: {}", e))?;
            
        // Parse the JSON
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse package.json: {}", e))?;
            
        // Extract site metadata
        let mut site_data = serde_json::Map::new();
        
        // Extract basic package info
        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            site_data.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        
        if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
            site_data.insert("description".to_string(), serde_json::Value::String(desc.to_string()));
        }
        
        if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
            site_data.insert("version".to_string(), serde_json::Value::String(version.to_string()));
        }
        
        if let Some(author) = json.get("author") {
            site_data.insert("author".to_string(), author.clone());
        }
        
        if let Some(homepage) = json.get("homepage").and_then(|v| v.as_str()) {
            site_data.insert("homepage".to_string(), serde_json::Value::String(homepage.to_string()));
        }
        
        // Check for Assemble-specific config
        if let Some(assemble) = json.get("assemble") {
            site_data.insert("assemble".to_string(), assemble.clone());
        }
        
        // Only write the file if we have data
        if !site_data.is_empty() {
            // Convert to YAML
            let yaml_content = serde_yaml::to_string(&serde_json::Value::Object(site_data))
                .map_err(|e| format!("Failed to convert site data to YAML: {}", e))?;
                
            // Write site data file
            let site_data_file = dest_data_dir.join("site.yml");
            fs::write(&site_data_file, yaml_content)
                .map_err(|e| format!("Failed to write site data file: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: "_data/site.yml".to_string(),
                change_type: ChangeType::Created,
                description: "Site data extracted from package.json".to_string(),
            });
        }
        
        Ok(())
    }
} 