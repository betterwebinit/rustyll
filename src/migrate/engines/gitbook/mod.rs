use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct GitbookMigrator;

impl GitbookMigrator {
    pub fn new() -> Self {
        GitbookMigrator
    }
}

impl EngineMigrator for GitbookMigrator {
    fn name(&self) -> &'static str {
        "GitBook"
    }
    
    fn description(&self) -> &'static str {
        "Migrates GitBook sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for GitBook-specific files and directories
        source_dir.join("book.json").exists() || 
        source_dir.join(".bookignore").exists() ||
        source_dir.join("SUMMARY.md").exists() && source_dir.join("README.md").exists() ||
        source_dir.join("_book").exists() ||
        has_gitbook_structure(source_dir)
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
        
        // Migrate includes
        self.migrate_includes(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate styles and plugins
        self.migrate_styles_and_plugins(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        self.migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        self.generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if a directory has a typical GitBook structure
fn has_gitbook_structure(source_dir: &Path) -> bool {
    // Look for typical GitBook structure: SUMMARY.md + content directories
    if !source_dir.join("SUMMARY.md").exists() {
        return false;
    }
    
    // Check if there are markdown files besides README.md and SUMMARY.md
    let mut has_content = false;
    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && 
               path.extension().map_or(false, |ext| ext == "md") && 
               path.file_name().map_or(false, |name| name != "README.md" && name != "SUMMARY.md") {
                has_content = true;
                break;
            }
        }
    }
    
    // Look for gitbook-specific directories
    let gitbook_dirs = vec!["styles", "plugins", "_layouts"];
    for dir in gitbook_dirs {
        if source_dir.join(dir).exists() {
            return true;
        }
    }
    
    has_content
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod includes;
mod styles_and_plugins;
mod static_assets;
mod readme; 