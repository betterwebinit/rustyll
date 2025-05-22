use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_plugins(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress plugins...");
    }

    // Look for plugins directory in the source
    let plugins_dir = source_dir.join("plugins");
    if !plugins_dir.exists() {
        // No plugins directory, check for _plugins
        let alt_plugins_dir = source_dir.join("_plugins");
        if !alt_plugins_dir.exists() {
            // No plugins directory found
            return Ok(());
        }
    }

    // Create destination _plugins directory
    let dest_plugins_dir = dest_dir.join("_plugins");
    create_dir_if_not_exists(&dest_plugins_dir)?;

    // Migrate plugins from plugins or _plugins directory
    let source_plugins_dir = if plugins_dir.exists() {
        plugins_dir
    } else {
        source_dir.join("_plugins")
    };

    // Process plugin files
    for entry in WalkDir::new(&source_plugins_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process Ruby files
            if let Some(ext) = file_path.extension() {
                if ext == "rb" {
                    migrate_plugin_file(file_path, &source_plugins_dir, &dest_plugins_dir, result)?;
                }
            }
        }
    }

    // Add plugins to _config.yml
    update_config_with_plugins(dest_dir, result)?;

    Ok(())
}

fn migrate_plugin_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path
    let dest_path = dest_dir.join(rel_path);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Read and process the plugin file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read plugin file {}: {}", file_path.display(), e))?;
    
    // Convert the plugin file, or just copy if no conversion needed
    let converted_content = convert_plugin_file(&content);
    
    // Write to destination
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write plugin file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_plugins/{}", rel_path.display()),
        description: format!("Converted plugin from {}", file_path.display()),
    });
    
    Ok(())
}

fn convert_plugin_file(content: &str) -> String {
    // Most Octopress plugins work as-is in Jekyll, so we just check for specific issues
    
    let mut converted = content.to_string();
    
    // Replace Octopress-specific code with Jekyll equivalents if needed
    
    // Replace require 'octopress' if present
    converted = converted.replace("require 'octopress'", "# Original required 'octopress'");
    
    // Replace other Octopress-specific APIs if needed
    
    converted
}

fn update_config_with_plugins(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Path to Jekyll config file
    let config_path = dest_dir.join("_config.yml");
    if !config_path.exists() {
        return Ok(());
    }
    
    // Read the current config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read _config.yml: {}", e))?;
    
    // Check if plugins section already exists
    if config_content.contains("plugins:") || config_content.contains("gems:") {
        // Already has plugins section
        return Ok(());
    }
    
    // Add standard Jekyll plugins section
    let mut new_config = config_content;
    new_config.push_str("\n# Added plugins\nplugins:\n");
    new_config.push_str("  - jekyll-feed\n");
    new_config.push_str("  - jekyll-sitemap\n");
    new_config.push_str("  - jekyll-paginate\n");
    
    // Write the updated config
    fs::write(&config_path, new_config)
        .map_err(|e| format!("Failed to update _config.yml with plugins: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Modified,
        file_path: "_config.yml".to_string(),
        description: "Added Jekyll plugins configuration to _config.yml".to_string(),
    });
    
    Ok(())
} 