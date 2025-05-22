use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub fn migrate_sass(source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
    // Look for sass files in Zola site
    let sass_dir = source_dir.join("sass");
    let scss_dir = source_dir.join("scss");
    
    let dest_sass_dir = dest_dir.join("_sass");
    create_dir_if_not_exists(&dest_sass_dir)?;
    
    // Process sass directory if it exists
    if sass_dir.exists() {
        if verbose {
            log::info!("Migrating Sass files from Zola");
        }
        
        copy_sass_files(&sass_dir, &dest_sass_dir, result)?;
    }
    
    // Process scss directory if it exists
    if scss_dir.exists() {
        if verbose {
            log::info!("Migrating SCSS files from Zola");
        }
        
        copy_sass_files(&scss_dir, &dest_sass_dir, result)?;
    }
    
    Ok(())
}

fn copy_sass_files(source_dir: &Path, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    for entry in fs::read_dir(source_dir).map_err(|e| format!("Failed to read sass directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let dest_path = dest_dir.join(file_name);
            
            copy_file(&path, &dest_path)?;
            
            result.changes.push(MigrationChange {
                file_path: format!("_sass/{}", file_name.to_string_lossy()),
                change_type: ChangeType::Copied,
                description: "Sass file migrated from Zola".to_string(),
            });
        } else if path.is_dir() {
            // Create subdirectory in destination
            let dir_name = path.file_name().unwrap();
            let dest_subdir = dest_dir.join(dir_name);
            create_dir_if_not_exists(&dest_subdir)?;
            
            // Recursively copy files from subdirectory
            copy_sass_files(&path, &dest_subdir, result)?;
        }
    }
    
    Ok(())
} 