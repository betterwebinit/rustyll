use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult,
    create_dir_if_not_exists
};

pub struct OctopressMigrator;

impl OctopressMigrator {
    pub fn new() -> Self {
        OctopressMigrator
    }
}

impl EngineMigrator for OctopressMigrator {
    fn name(&self) -> &'static str {
        "Octopress"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Octopress (Jekyll-based) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Octopress-specific files and directories
        source_dir.join("Rakefile").exists() && has_octopress_rakefile(source_dir) ||
        source_dir.join("_config.yml").exists() && has_octopress_config(source_dir) ||
        source_dir.join("source/_posts").exists() && source_dir.join("source/_includes").exists()
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
        
        // Migrate layouts
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate includes
        includes::migrate_includes(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate plugins
        plugins::migrate_plugins(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate styles
        styles::migrate_styles(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate assets
        assets::migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        readme::generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if Rakefile contains Octopress-specific tasks
fn has_octopress_rakefile(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("Rakefile")) {
        content.contains("octopress") || content.contains("jekyll")
    } else {
        false
    }
}

// Helper function to check if _config.yml contains Octopress-specific settings
fn has_octopress_config(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("_config.yml")) {
        content.contains("octopress") || content.contains("jekyll_plugins")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod includes;
mod plugins;
mod styles;
mod assets;
mod readme; 