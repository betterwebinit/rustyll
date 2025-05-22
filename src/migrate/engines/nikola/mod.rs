use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct NikolaMigrator;

impl NikolaMigrator {
    pub fn new() -> Self {
        NikolaMigrator
    }
}

impl EngineMigrator for NikolaMigrator {
    fn name(&self) -> &'static str {
        "Nikola"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Nikola (Python-based) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Nikola-specific files and directories
        source_dir.join("conf.py").exists() || 
        source_dir.join("nikola").exists() ||
        (source_dir.join("posts").exists() && source_dir.join("pages").exists()) ||
        source_dir.join("requirements.txt").exists() && has_nikola_requirement(source_dir)
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
        
        // Migrate content (posts and pages)
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate layouts (templates)
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data
        data::migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        static_assets::migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        readme::generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if requirements.txt contains Nikola
fn has_nikola_requirement(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("requirements.txt")) {
        content.contains("nikola")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod data;
mod static_assets;
mod readme; 