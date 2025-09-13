use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult,
    create_dir_if_not_exists
};

pub struct NanocMigrator;

impl NanocMigrator {
    pub fn new() -> Self {
        NanocMigrator
    }
}

impl EngineMigrator for NanocMigrator {
    fn name(&self) -> &'static str {
        "Nanoc"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Nanoc sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Nanoc-specific files
        source_dir.join("nanoc.yaml").exists() || 
        source_dir.join("Rules").exists() || 
        source_dir.join("Rules.rb").exists() ||
        source_dir.join("config.yaml").exists() && source_dir.join("content").exists()
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
        
        // Migrate configuration
        self.migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate content
        self.migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate layouts
        self.migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data
        self.migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        self.migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Write README files for various directories
        self.write_readme_files(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod data;
mod static_assets;
mod readme; 