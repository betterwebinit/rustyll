use std::path::Path;
use std::fs;

use crate::migrate::MigrationResult;

pub fn migrate_data(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Zola data to Jekyll...");
    }

    // Check for various data locations in Zola
    let config_file = source_dir.join("config.toml");
    let data_dir = source_dir.join("data");
    
    // Create Jekyll _data directory
    let jekyll_data_dir = dest_dir.join("_data");
    if !jekyll_data_dir.exists() {
        fs::create_dir_all(&jekyll_data_dir).map_err(|e| {
            format!("Failed to create _data directory: {}", e)
        })?;
    }
    
    // Process config.toml for site data
    if config_file.exists() {
        migrate_config_to_data(&config_file, &jekyll_data_dir, verbose, result)?;
    }
    
    // Process data directory if it exists
    if data_dir.exists() && data_dir.is_dir() {
        migrate_data_directory(&data_dir, &jekyll_data_dir, verbose, result)?;
    }
    
    if verbose {
        println!("Completed Zola data migration");
    }
    
    Ok(())
}

fn migrate_config_to_data(
    config_file: &Path,
    data_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Converting Zola config to Jekyll data...");
    }
    
    // Read the config file
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    // Create a site.yml with the config values
    // In a real implementation, this would parse TOML and convert to YAML
    // Here we'll do a simplified version
    
    let site_data_file = data_dir.join("site.yml");
    
    // Very basic TOML to YAML conversion
    let yaml_content = convert_toml_to_yaml(&config_content);
    
    fs::write(&site_data_file, yaml_content)
        .map_err(|e| format!("Failed to write site data file: {}", e))?;
    
    if verbose {
        println!("Created site data from config at {:?}", site_data_file);
    }
    
    Ok(())
}

fn migrate_data_directory(
    data_dir: &Path,
    jekyll_data_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Zola data directory...");
    }
    
    // Process all files in the data directory
    if let Ok(entries) = fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                // Get the file extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    
                    // Handle different data formats
                    match ext_str.as_str() {
                        "toml" => {
                            if let Some(file_stem) = path.file_stem() {
                                let dest_file = jekyll_data_dir.join(format!(
                                    "{}.yml", file_stem.to_string_lossy()
                                ));
                                
                                if verbose {
                                    println!("Converting {:?} to {:?}", path, dest_file);
                                }
                                
                                // Read the TOML file
                                match fs::read_to_string(&path) {
                                    Ok(content) => {
                                        // Convert TOML to YAML
                                        let yaml_content = convert_toml_to_yaml(&content);
                                        
                                        // Write YAML file
                                        if let Err(e) = fs::write(&dest_file, yaml_content) {
                                            result.errors.push(format!(
                                                "Failed to write data file {:?}: {}", dest_file, e
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        result.errors.push(format!(
                                            "Failed to read data file {:?}: {}", path, e
                                        ));
                                    }
                                }
                            }
                        }
                        "json" => {
                            // Copy JSON files as is (Jekyll supports JSON)
                            if let Some(file_name) = path.file_name() {
                                let dest_file = jekyll_data_dir.join(file_name);
                                
                                if verbose {
                                    println!("Copying JSON data file {:?} to {:?}", path, dest_file);
                                }
                                
                                if let Err(e) = fs::copy(&path, &dest_file) {
                                    result.errors.push(format!(
                                        "Failed to copy data file {:?}: {}", path, e
                                    ));
                                }
                            }
                        }
                        "yaml" | "yml" => {
                            // Copy YAML files as is
                            if let Some(file_name) = path.file_name() {
                                let dest_file = jekyll_data_dir.join(file_name);
                                
                                if verbose {
                                    println!("Copying YAML data file {:?} to {:?}", path, dest_file);
                                }
                                
                                if let Err(e) = fs::copy(&path, &dest_file) {
                                    result.errors.push(format!(
                                        "Failed to copy data file {:?}: {}", path, e
                                    ));
                                }
                            }
                        }
                        _ => {
                            // Skip other file types
                            if verbose {
                                println!("Skipping unsupported data file: {:?}", path);
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // Process subdirectories
                if let Some(dir_name) = path.file_name() {
                    let dest_subdir = jekyll_data_dir.join(dir_name);
                    
                    if !dest_subdir.exists() {
                        if let Err(e) = fs::create_dir_all(&dest_subdir) {
                            result.errors.push(format!(
                                "Failed to create directory {:?}: {}", dest_subdir, e
                            ));
                            continue;
                        }
                    }
                    
                    migrate_data_directory(&path, &dest_subdir, verbose, result)?;
                }
            }
        }
    }
    
    Ok(())
}

fn convert_toml_to_yaml(toml_content: &str) -> String {
    // In a real implementation, this would use proper TOML and YAML libraries
    // For this exercise, we'll do a simplified conversion
    let mut yaml_content = String::new();
    
    for line in toml_content.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with('#') {
            // Copy comments and empty lines as is
            yaml_content.push_str(line);
            yaml_content.push('\n');
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Convert TOML sections to YAML
            // [section] becomes section:
            let section = trimmed.trim_start_matches('[').trim_end_matches(']');
            yaml_content.push_str(&format!("{}:\n", section));
        } else if let Some(pos) = trimmed.find('=') {
            // Convert key = value to key: value
            let key = trimmed[..pos].trim();
            let value = trimmed[pos+1..].trim();
            
            // Handle string values
            if (value.starts_with('"') && value.ends_with('"')) || 
               (value.starts_with('\'') && value.ends_with('\'')) {
                yaml_content.push_str(&format!("  {}: {}\n", key, value));
            } else {
                // For non-string values
                yaml_content.push_str(&format!("  {}: {}\n", key, value));
            }
        } else {
            // Pass through other lines (arrays, etc.)
            yaml_content.push_str(line);
            yaml_content.push('\n');
        }
    }
    
    yaml_content
} 