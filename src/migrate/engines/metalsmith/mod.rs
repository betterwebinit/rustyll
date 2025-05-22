use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct MetalsmithMigrator;

impl MetalsmithMigrator {
    pub fn new() -> Self {
        MetalsmithMigrator
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
    
    fn migrate_plugins(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        plugins::migrate_plugins(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_collections(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        collections::migrate_collections(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_metadata(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        metadata::migrate_metadata(source_dir, dest_dir, verbose, result)
    }
    
    fn migrate_assets(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        assets::migrate_assets(source_dir, dest_dir, verbose, result)
    }
    
    fn generate_readme(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        readme::generate_readme(dest_dir, result)
    }
}

impl EngineMigrator for MetalsmithMigrator {
    fn name(&self) -> &'static str {
        "Metalsmith"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Metalsmith sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Metalsmith-specific files and directories
        source_dir.join("metalsmith.json").exists() || 
        source_dir.join("metalsmith.js").exists() ||
        (source_dir.join("package.json").exists() && has_metalsmith_deps(source_dir))
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
        
        // Migrate using instance methods
        self.migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_plugins(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_collections(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_metadata(source_dir, dest_dir, verbose, &mut result)?;
        self.migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README
        self.generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if package.json contains Metalsmith dependency
fn has_metalsmith_deps(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("package.json")) {
        content.contains("\"metalsmith\"") || content.contains("'metalsmith'")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod plugins;
mod collections;
mod metadata;
mod assets;
mod readme; 