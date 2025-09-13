use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, create_dir_if_not_exists
};

// Import implementation modules
mod config;
mod content;
mod theme;
mod preprocessors;
mod renderers;
mod search;
mod static_files;
mod readme;

pub struct MdBookMigrator;

impl MdBookMigrator {
    pub fn new() -> Self {
        MdBookMigrator
    }
}

impl EngineMigrator for MdBookMigrator {
    fn name(&self) -> &'static str {
        "MDBook"
    }
    
    fn description(&self) -> &'static str {
        "Migrates MDBook (Rust-based documentation) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for MDBook-specific files and directories
        source_dir.join("book.toml").exists() ||
        source_dir.join("src/SUMMARY.md").exists() ||
        source_dir.join("theme").exists() && source_dir.join("book").exists() ||
        source_dir.join("Cargo.toml").exists() && has_mdbook_dependency(source_dir)
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
        
        // Migrate config (book.toml -> _config.yml)
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate content (SUMMARY.md, *.md)
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate theme
        theme::migrate_theme(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate preprocessors
        preprocessors::migrate_preprocessors(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate renderers
        renderers::migrate_renderers(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate search functionality
        search::migrate_search(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static files
        static_files::migrate_static(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README
        readme::generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if Cargo.toml contains MDBook dependency
fn has_mdbook_dependency(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("Cargo.toml")) {
        content.contains("mdbook")
    } else {
        false
    }
} 