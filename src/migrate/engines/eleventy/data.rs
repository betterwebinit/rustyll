use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::EleventyMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Eleventy uses _data directory for global data
        let source_data = source_dir.join("_data");
        
        // Create destination data directory
        let dest_data = dest_dir.join("_data");
        
        if source_data.exists() && source_data.is_dir() {
            if verbose {
                log::info!("Migrating data from {}", source_data.display());
            }
            
            // Create _data directory in destination
            create_dir_if_not_exists(&dest_data)?;
            
            // Copy all data files
            for entry in WalkDir::new(&source_data)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let relative_path = file_path.strip_prefix(&source_data)
                        .map_err(|e| format!("Failed to get relative path: {}", e))?;
                    
                    // Get the file extension
                    let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    
                    // Handle JavaScript data files specially
                    if extension == "js" || extension == "cjs" {
                        // Extract file name without extension
                        let file_stem = file_path.file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("unknown");
                        
                        // Create a JSON version of the file name
                        let json_file_name = format!("{}.json", file_stem);
                        let dest_path = dest_data.join(relative_path.parent().unwrap_or_else(|| Path::new("")))
                            .join(&json_file_name);
                        
                        // Create parent directories if they don't exist
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the original JS file too for reference
                        let js_dest_path = dest_data.join(relative_path);
                        if let Some(parent) = js_dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        copy_file(file_path, &js_dest_path)?;
                        
                        // Create a placeholder JSON file
                        let json_content = "{\n  \"_notice\": \"This is a placeholder converted from a JavaScript data file. Manual conversion is required.\"\n}";
                        fs::write(&dest_path, json_content)
                            .map_err(|e| format!("Failed to write JSON placeholder: {}", e))?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.display()),
                            change_type: ChangeType::Copied,
                            description: "Original JavaScript data file copied for reference".to_string(),
                        });
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.parent().unwrap_or_else(|| Path::new("")).join(json_file_name).display()),
                            change_type: ChangeType::Created,
                            description: "Placeholder JSON data file created".to_string(),
                        });
                        
                        result.warnings.push(format!(
                            "JavaScript data file _data/{} requires manual conversion to JSON",
                            relative_path.display()
                        ));
                    } else {
                        // For other data files (JSON, YAML, etc.), just copy them directly
                        let dest_path = dest_data.join(relative_path);
                        
                        // Create parent directories if they don't exist
                        if let Some(parent) = dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the data file
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.display()),
                            change_type: ChangeType::Copied,
                            description: "Data file copied from Eleventy".to_string(),
                        });
                    }
                }
            }
            
            // Create README for data directory
            let data_readme = r#"# Data Directory

This directory contains data files migrated from Eleventy.

## Data Format

Data in Rustyll:
- Can be in JSON or YAML format
- Is available in templates via the `site.data` variable
- For example, data in `_data/navigation.yml` is accessible as `site.data.navigation`

## Changes from Eleventy

- Eleventy supports JavaScript data files which need to be converted to JSON/YAML
- Eleventy can have computed data which may need manual adaptation
- JavaScript functions in data need to be reimplemented or removed
"#;
            
            write_readme(&dest_data, data_readme)?;
        } else {
            // Check alternative locations
            let alt_data_dirs = [
                source_dir.join("src/_data"),
                source_dir.join("data")
            ];
            
            let mut found_alt = false;
            
            for alt_data_dir in alt_data_dirs.iter() {
                if alt_data_dir.exists() && alt_data_dir.is_dir() {
                    found_alt = true;
                    
                    if verbose {
                        log::info!("Found alternative data directory: {}", alt_data_dir.display());
                    }
                    
                    // Create destination data directory
                    create_dir_if_not_exists(&dest_data)?;
                    
                    // Copy all data files
                    for entry in WalkDir::new(alt_data_dir)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if entry.file_type().is_file() {
                            let file_path = entry.path();
                            let relative_path = file_path.strip_prefix(alt_data_dir)
                                .map_err(|e| format!("Failed to get relative path: {}", e))?;
                            
                            let dest_path = dest_data.join(relative_path);
                            
                            // Create parent directories if they don't exist
                            if let Some(parent) = dest_path.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Copy the data file
                            copy_file(file_path, &dest_path)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("_data/{}", relative_path.display()),
                                change_type: ChangeType::Copied,
                                description: format!("Data file copied from alternative location: {}", alt_data_dir.display()),
                            });
                        }
                    }
                    
                    break;
                }
            }
            
            if !found_alt {
                result.warnings.push(
                    "No _data directory found. Eleventy data files need to be created manually.".to_string()
                );
            }
        }
        
        Ok(())
    }
} 