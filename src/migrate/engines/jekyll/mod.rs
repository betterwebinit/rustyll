use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct JekyllMigrator;

impl JekyllMigrator {
    pub fn new() -> Self {
        JekyllMigrator
    }
}

impl EngineMigrator for JekyllMigrator {
    fn name(&self) -> &'static str {
        "Jekyll"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Jekyll sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Jekyll-specific files
        source_dir.join("_config.yml").exists() || 
        source_dir.join("_layouts").exists() || 
        source_dir.join("_includes").exists()
    }
    
    fn migrate(&self, options: &MigrationOptions) -> Result<MigrationResult, String> {
        let source_dir = &options.source_dir;
        let dest_dir = &options.dest_dir;
        let verbose = options.verbose;
        
        // Clean destination directory if requested
        if options.clean && dest_dir.exists() {
            if verbose {
                log::info!("Cleaning destination directory: {}", dest_dir.display());
            }
            fs::remove_dir_all(dest_dir)
                .map_err(|e| format!("Failed to clean destination directory: {}", e))?;
        }
        
        // Create destination directory
        create_dir_if_not_exists(dest_dir)?;
        
        // Initialize the result
        let mut result = MigrationResult {
            engine_name: self.name().to_string(),
            changes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };
        
        // Migrate main configuration file
        self.migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate content directories
        self.migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate layouts
        self.migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate includes
        self.migrate_includes(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data files
        self.migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate assets
        self.migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Write README files for various directories
        self.write_readme_files(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod includes;
mod data;
mod assets;
mod readme; 