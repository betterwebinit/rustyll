use log::{warn, info};

use crate::config::Config;
use crate::utils::error::{BoxResult, RustyllError};
use crate::utils::fs;

/// Validate the configuration
pub fn validate_config(config: &Config) -> BoxResult<()> {
    // Validate source directory
    validate_source_directory(&config)?;
    
    // Validate destination directory
    validate_destination_directory(&config)?;
    
    // Validate layouts directory
    validate_layouts_directory(&config)?;
    
    // Validate collections
    validate_collections(&config)?;
    
    Ok(())
}

/// Validate the source directory
fn validate_source_directory(config: &Config) -> BoxResult<()> {
    let source = &config.source;
    
    if !source.exists() {
        return Err(RustyllError::Config(format!(
            "Source directory does not exist: {}", source.display()
        )).into());
    }
    
    if !source.is_dir() {
        return Err(RustyllError::Config(format!(
            "Source path is not a directory: {}", source.display()
        )).into());
    }
    
    // Check if source is readable
    if !fs::is_directory(source) {
        return Err(RustyllError::Config(format!(
            "Source directory is not accessible: {}", source.display()
        )).into());
    }
    
    info!("Source directory: {}", source.display());
    Ok(())
}

/// Validate the destination directory
fn validate_destination_directory(config: &Config) -> BoxResult<()> {
    let destination = &config.destination;
    
    // Create the destination directory if it doesn't exist
    if !destination.exists() {
        info!("Creating destination directory: {}", destination.display());
        fs::create_directory(destination)?;
    } else if !destination.is_dir() {
        return Err(RustyllError::Config(format!(
            "Destination path exists but is not a directory: {}", destination.display()
        )).into());
    }
    
    // Check if destination is writable
    if !fs::is_directory(destination) {
        return Err(RustyllError::Config(format!(
            "Destination directory is not accessible: {}", destination.display()
        )).into());
    }
    
    info!("Destination directory: {}", destination.display());
    Ok(())
}

/// Validate the layouts directory
fn validate_layouts_directory(config: &Config) -> BoxResult<()> {
    let layouts_dir = &config.layouts_dir;
    
    // If the layouts directory doesn't exist, warn but don't fail
    if !layouts_dir.exists() {
        warn!("Layouts directory does not exist: {}", layouts_dir.display());
        return Ok(());
    }
    
    if !layouts_dir.is_dir() {
        warn!("Layouts path exists but is not a directory: {}", layouts_dir.display());
        return Ok(());
    }
    
    info!("Layouts directory: {}", layouts_dir.display());
    Ok(())
}

/// Validate collections configuration
fn validate_collections(config: &Config) -> BoxResult<()> {
    for (name, collection) in &config.collections.items {
        if let Some(permalink) = &collection.permalink {
            // Basic permalink pattern validation
            if !permalink.contains(':') && !permalink.contains('{') {
                warn!("Collection '{}' has a possible invalid permalink pattern: {}", name, permalink);
            }
        }
        
        // Check if sort_by is valid (basic check)
        if collection.sort_by != "date" && collection.sort_by != "title" && collection.sort_by != "name" {
            warn!("Collection '{}' has an unusual sort_by value: {}", name, collection.sort_by);
        }
    }
    
    Ok(())
} 