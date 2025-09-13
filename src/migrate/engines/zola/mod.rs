use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, create_dir_if_not_exists
};

pub struct ZolaMigrator;

impl ZolaMigrator {
    pub fn new() -> Self {
        ZolaMigrator
    }
}

impl EngineMigrator for ZolaMigrator {
    fn name(&self) -> &'static str {
        "Zola"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Zola sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Zola-specific files
        source_dir.join("config.toml").exists() && 
        source_dir.join("templates").exists() && 
        source_dir.join("content").exists()
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
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate content (Markdown files)
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate templates (Tera templates to Liquid templates)
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static files
        static_assets::migrate_static(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate sass/scss files
        sass::migrate_sass(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data files
        data::migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README file
        readme::generate_readme(dest_dir, &mut result)?;
        
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
mod sass; 