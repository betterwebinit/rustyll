use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_metadata(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Metalsmith metadata...");
    }
    
    // In Metalsmith, metadata can come from several sources:
    // 1. Front matter in content files
    // 2. JSON files in a metadata directory
    // 3. Configuration in metalsmith.json
    
    // Check for metadata directory
    let metadata_dirs = [
        source_dir.join("metadata"),
        source_dir.join("src").join("metadata"),
        source_dir.join("data"),
        source_dir.join("src").join("data"),
    ];
    
    let mut found_metadata = false;
    
    // Create destination data directory
    let dest_data_dir = dest_dir.join("_data");
    create_dir_if_not_exists(&dest_data_dir)?;
    
    // Process metadata directories
    for metadata_dir in metadata_dirs.iter() {
        if !metadata_dir.exists() {
            continue;
        }
        
        found_metadata = true;
        
        // Process all JSON, YAML, and TOML files in the metadata directory
        let metadata_files = WalkDir::new(metadata_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                e.path().is_file() && (extension == "json" || extension == "yaml" || extension == "yml" || extension == "toml")
            });
        
        for entry in metadata_files {
            let relative_path = entry.path().strip_prefix(metadata_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read file {}: {}", entry.path().display(), e))?;
            
            // Convert to Jekyll _data format if needed
            let (dest_file, converted_content) = convert_metadata_file(entry.path(), &file_content)?;
            let dest_path = dest_data_dir.join(dest_file);
            
            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            fs::write(&dest_path, &converted_content)
                .map_err(|e| format!("Failed to write file {}: {}", dest_path.display(), e))?;
            
            // Add to migration results
            result.changes.push(MigrationChange {
                file_path: format!("_data/{}", dest_path.file_name().unwrap_or_default().to_string_lossy()),
                change_type: ChangeType::Converted,
                description: "Converted Metalsmith metadata to Jekyll data file".into(),
            });
        }
    }
    
    // Also check for metalsmith.json for global metadata
    let metalsmith_json = source_dir.join("metalsmith.json");
    if metalsmith_json.exists() {
        extract_global_metadata(&metalsmith_json, &dest_data_dir, result)?;
        found_metadata = true;
    }
    
    if !found_metadata {
        result.warnings.push("No Metalsmith metadata directories or files found".into());
    }
    
    // Create a global site.yml file with some defaults
    create_site_data_file(&dest_data_dir, result)?;
    
    Ok(())
}

fn convert_metadata_file(file_path: &Path, content: &str) -> Result<(PathBuf, String), String> {
    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
    let extension = file_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    
    // Determine the destination file name and format
    let dest_file = if extension == "json" {
        // Convert JSON to YAML for Jekyll _data
        let parsed_json: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| format!("Failed to parse JSON file {}: {}", file_path.display(), e))?;
        
        let yaml_content = serde_yaml::to_string(&parsed_json)
            .map_err(|e| format!("Failed to convert JSON to YAML: {}", e))?;
        
        // Replace .json with .yml
        let new_name = file_name.replace(".json", ".yml");
        (PathBuf::from(new_name), yaml_content)
    } else if extension == "toml" {
        // Convert TOML to YAML for Jekyll _data
        let parsed_toml: toml::Value = toml::from_str(content)
            .map_err(|e| format!("Failed to parse TOML file {}: {}", file_path.display(), e))?;
        
        // Convert TOML to JSON as intermediate step
        let json_value = convert_toml_to_json(parsed_toml);
        
        let yaml_content = serde_yaml::to_string(&json_value)
            .map_err(|e| format!("Failed to convert TOML to YAML: {}", e))?;
        
        // Replace .toml with .yml
        let new_name = file_name.replace(".toml", ".yml");
        (PathBuf::from(new_name), yaml_content)
    } else {
        // For YAML files, keep as is
        (PathBuf::from(file_name.to_string()), content.to_string())
    };
    
    Ok(dest_file)
}

fn extract_global_metadata(
    config_file: &Path,
    dest_data_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read metalsmith.json: {}", e))?;
    
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse metalsmith.json: {}", e))?;
    
    if let Some(metadata) = config.get("metadata").and_then(|m| m.as_object()) {
        let yaml_content = serde_yaml::to_string(&metadata)
            .map_err(|e| format!("Failed to convert metadata to YAML: {}", e))?;
        
        let dest_file = dest_data_dir.join("site.yml");
        fs::write(&dest_file, yaml_content)
            .map_err(|e| format!("Failed to write site.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_data/site.yml".into(),
            change_type: ChangeType::Created,
            description: "Created site data file from Metalsmith metadata".into(),
        });
    }
    
    Ok(())
}

fn create_site_data_file(
    dest_data_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Check if site.yml already exists
    let site_yml = dest_data_dir.join("site.yml");
    if site_yml.exists() {
        return Ok(());
    }
    
    // Create a basic site.yml with defaults
    let site_yml_content = r#"# Site data extracted from Metalsmith
title: "Migrated Metalsmith Site"
description: "A site migrated from Metalsmith to Jekyll"
author: 
  name: "Site Author"
  email: ""
url: ""
"#;
    
    fs::write(&site_yml, site_yml_content)
        .map_err(|e| format!("Failed to write site.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_data/site.yml".into(),
        change_type: ChangeType::Created,
        description: "Created default site data file".into(),
    });
    
    Ok(())
}

// Helper function to convert TOML to JSON
fn convert_toml_to_json(toml_value: toml::Value) -> serde_json::Value {
    match toml_value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
        toml::Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            let json_array = arr.into_iter()
                .map(convert_toml_to_json)
                .collect();
            serde_json::Value::Array(json_array)
        },
        toml::Value::Table(table) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in table {
                json_obj.insert(k, convert_toml_to_json(v));
            }
            serde_json::Value::Object(json_obj)
        },
    }
} 