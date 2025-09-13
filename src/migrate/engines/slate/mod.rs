use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult,
    create_dir_if_not_exists
};

pub struct SlateMigrator;

impl SlateMigrator {
    pub fn new() -> Self {
        SlateMigrator
    }
}

impl EngineMigrator for SlateMigrator {
    fn name(&self) -> &'static str {
        "Slate"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Slate (API documentation generator) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Slate-specific files and directories
        source_dir.join("Gemfile").exists() && has_slate_gem(source_dir) ||
        source_dir.join("source/index.html.md").exists() ||
        source_dir.join("source/layouts/layout.erb").exists() ||
        source_dir.join("source/stylesheets/_variables.scss").exists()
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
        
        // Migrate content (API documentation)
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate layouts
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate includes
        includes::migrate_includes(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate styles
        styles::migrate_styles(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate JavaScript
        javascript::migrate_javascript(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate images and other assets
        assets::migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        readme::generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if Gemfile contains Slate gem
fn has_slate_gem(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("Gemfile")) {
        content.contains("slate") || content.contains("middleman")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod includes;
mod styles;
mod javascript;
mod assets;
mod readme; 