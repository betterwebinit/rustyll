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
        println!("Migrating MkDocs data to Jekyll...");
    }

    // In MkDocs, data is mainly in the mkdocs.yml file
    let mkdocs_config = source_dir.join("mkdocs.yml");
    
    // Create Jekyll _data directory
    let jekyll_data_dir = dest_dir.join("_data");
    if !jekyll_data_dir.exists() {
        fs::create_dir_all(&jekyll_data_dir).map_err(|e| {
            format!("Failed to create _data directory: {}", e)
        })?;
    }
    
    // Process mkdocs.yml for site data
    if mkdocs_config.exists() {
        migrate_config_to_data(&mkdocs_config, &jekyll_data_dir, verbose, result)?;
    } else {
        if verbose {
            println!("No mkdocs.yml found, skipping data migration");
        }
    }
    
    // Look for data directories that might contain additional data
    // MkDocs doesn't have a standard data directory, but users might use custom directories
    let possible_data_dirs = [
        source_dir.join("data"),
        source_dir.join("_data"),
    ];
    
    for data_dir in possible_data_dirs.iter() {
        if data_dir.exists() && data_dir.is_dir() {
            copy_data_directory(data_dir, &jekyll_data_dir, verbose, result)?;
        }
    }
    
    if verbose {
        println!("Completed MkDocs data migration");
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
        println!("Converting MkDocs config to Jekyll data...");
    }
    
    // Read the mkdocs.yml file
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read MkDocs config file: {}", e))?;
    
    // Create a site.yml in Jekyll _data directory
    let site_data_file = data_dir.join("site.yml");
    
    // In a real implementation, this would parse YAML and modify as needed
    // For our example, we'll just copy it with a comment
    let jekyll_yaml = format!(
        "# Converted from MkDocs configuration\n\
         # Some settings may need adjustment for Jekyll\n\
         {}", 
        config_content
    );
    
    fs::write(&site_data_file, jekyll_yaml)
        .map_err(|e| format!("Failed to write site data file: {}", e))?;
    
    if verbose {
        println!("Created site data from config at {:?}", site_data_file);
    }
    
    // Extract navigation from mkdocs.yml to create a navigation.yml file
    if config_content.contains("nav:") || config_content.contains("navigation:") {
        let nav_file = data_dir.join("navigation.yml");
        
        // In a real implementation, this would properly extract the navigation section
        // For simplicity, we're doing a naive extraction
        let mut in_nav = false;
        let mut nav_content = String::from("# MkDocs navigation structure\n");
        
        for line in config_content.lines() {
            if line.trim().starts_with("nav:") || line.trim().starts_with("navigation:") {
                in_nav = true;
                nav_content.push_str("pages:\n");
            } else if in_nav {
                if line.trim().is_empty() || line.starts_with(" ") || line.starts_with("\t") {
                    nav_content.push_str(line);
                    nav_content.push('\n');
                } else {
                    // End of navigation section
                    in_nav = false;
                }
            }
        }
        
        fs::write(&nav_file, nav_content)
            .map_err(|e| format!("Failed to write navigation file: {}", e))?;
        
        if verbose {
            println!("Created navigation data at {:?}", nav_file);
        }
    }
    
    Ok(())
}

fn copy_data_directory(
    source_data_dir: &Path,
    target_data_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Copying data directory {:?} to {:?}", source_data_dir, target_data_dir);
    }
    
    if let Ok(entries) = fs::read_dir(source_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let target_path = target_data_dir.join(&file_name);
            
            if path.is_file() {
                // Copy files
                if let Err(e) = fs::copy(&path, &target_path) {
                    result.errors.push(format!(
                        "Failed to copy data file {:?} to {:?}: {}", path, target_path, e
                    ));
                } else if verbose {
                    println!("Copied {:?} to {:?}", path, target_path);
                }
            } else if path.is_dir() {
                // Create and copy directories recursively
                if !target_path.exists() {
                    if let Err(e) = fs::create_dir_all(&target_path) {
                        result.errors.push(format!(
                            "Failed to create directory {:?}: {}", target_path, e
                        ));
                        continue;
                    }
                }
                
                copy_data_directory(&path, &target_path, verbose, result)?;
            }
        }
    }
    
    Ok(())
} 