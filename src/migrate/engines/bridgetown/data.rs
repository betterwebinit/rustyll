use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_data(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Bridgetown data files...");
    }

    // Create destination data directory
    let dest_data_dir = dest_dir.join("_data");
    create_dir_if_not_exists(&dest_data_dir)?;

    // Look for data files in Bridgetown's data directory
    let source_data_dir = source_dir.join("src/_data");
    if !source_data_dir.exists() || !source_data_dir.is_dir() {
        if verbose {
            log::info!("No data directory found in Bridgetown project");
        }
        return Ok(());
    }

    // Copy data files, converting as needed
    for entry in WalkDir::new(&source_data_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the data directory
            let rel_path = file_path.strip_prefix(&source_data_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
            // Create destination path
            let dest_path = dest_data_dir.join(rel_path);
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Copy the data file, converting if needed
            migrate_data_file(file_path, &dest_path, result)?;
        }
    }
    
    Ok(())
}

fn migrate_data_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get file extension
    let extension = source_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    
    // Handle different data file types
    match extension {
        "json" => convert_json_to_yaml(source_path, dest_path, result)?,
        "yml" | "yaml" => copy_file(source_path, dest_path)?,
        "rb" => convert_ruby_to_yaml(source_path, dest_path, result)?,
        _ => copy_file(source_path, dest_path)?,
    }
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_data/{}", dest_path.strip_prefix(dest_path.parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Converted data file from {}", source_path.display()),
    });
    
    Ok(())
}

fn convert_json_to_yaml(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the JSON file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read JSON file {}: {}", source_path.display(), e))?;
    
    // Parse the JSON
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON from {}: {}", source_path.display(), e))?;
    
    // Convert to YAML
    let yaml = serde_yaml::to_string(&parsed)
        .map_err(|e| format!("Failed to convert JSON to YAML: {}", e))?;
    
    // Create a new path with .yml extension
    let dest_yaml_path = dest_path.with_extension("yml");
    
    // Write the YAML file
    fs::write(&dest_yaml_path, yaml)
        .map_err(|e| format!("Failed to write YAML file {}: {}", dest_yaml_path.display(), e))?;
    
    Ok(())
}

fn convert_ruby_to_yaml(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the Ruby file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read Ruby file {}: {}", source_path.display(), e))?;
    
    // Very simple conversion of Ruby hashes to YAML
    // Note: This is a very naive implementation and will only work for simple cases
    let yaml_content = content
        .replace("=>", ":")
        .replace("{", "\n")
        .replace("}", "\n")
        .replace(",", "\n");
    
    // Create a new path with .yml extension
    let dest_yaml_path = dest_path.with_extension("yml");
    
    // Write the YAML file
    fs::write(&dest_yaml_path, yaml_content)
        .map_err(|e| format!("Failed to write YAML file {}: {}", dest_yaml_path.display(), e))?;
    
    result.warnings.push(format!(
        "Ruby data file {} was converted to YAML using a simple conversion. Manual review recommended.",
        source_path.display()
    ));
    
    Ok(())
} 