use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct DocsyMigrator;

impl DocsyMigrator {
    pub fn new() -> Self {
        DocsyMigrator
    }
    
    // Add implementation methods that delegate to module functions
    fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        config::migrate_config(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_content(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        content::migrate_content(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        layouts::migrate_layouts(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_partials(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        partials::migrate_partials(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        data::migrate_data(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        assets::migrate_assets(source_dir, dest_dir, verbose, result)
    }
    
    fn generate_readme(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        readme::generate_readme(dest_dir, result)
    }
}

impl EngineMigrator for DocsyMigrator {
    fn name(&self) -> &'static str {
        "Docsy"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Docsy (Hugo-based documentation theme) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Docsy-specific files and directories
        source_dir.join("config.toml").exists() && 
        (source_dir.join("themes/docsy").exists() || 
         source_dir.join("go.mod").exists() && has_docsy_module(source_dir))
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
        
        // Migrate partials (includes)
        self.migrate_partials(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data files
        self.migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate assets
        self.migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        self.generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if go.mod contains Docsy module
fn has_docsy_module(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("go.mod")) {
        content.contains("github.com/google/docsy")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod partials;
mod data;
mod assets;
mod readme; 