use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists
};

mod config;
mod content;
mod layouts;
mod includes;
mod data;
mod static_assets;
mod readme;
mod plugins;

pub struct CobaltMigrator;

impl CobaltMigrator {
    pub fn new() -> Self {
        CobaltMigrator
    }
}

impl EngineMigrator for CobaltMigrator {
    fn name(&self) -> &'static str {
        "Cobalt"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Cobalt.rs sites to Jekyll"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for _cobalt.yml
        let config_path = source_dir.join("_cobalt.yml");
        if config_path.exists() {
            return true;
        }
        
        // Check for Cargo.toml with cobalt dependency
        let cargo_path = source_dir.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_path) {
                if content.contains("cobalt") {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn migrate(&self, options: &MigrationOptions) -> Result<MigrationResult, String> {
        let source_dir = &options.source_dir;
        let dest_dir = &options.dest_dir;
        let verbose = options.verbose;
        
        let mut result = MigrationResult {
            engine_name: self.name().to_string(),
            changes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };
        
        if verbose {
            log::info!("Starting migration from Cobalt to Jekyll...");
        }
        
        // Create directory structure
        self.create_directory_structure(dest_dir, &mut result)?;
        
        // Migrate components by calling module functions directly
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        includes::migrate_includes(source_dir, dest_dir, verbose, &mut result)?;
        plugins::migrate_plugins(source_dir, dest_dir, verbose, &mut result)?;
        static_assets::migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        if verbose {
            log::info!("Migration from Cobalt to Jekyll completed successfully!");
        }
        
        Ok(result)
    }
}

impl CobaltMigrator {
    fn create_directory_structure(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create standard Jekyll directories
        let dirs = [
            "_layouts",
            "_includes",
            "_data",
            "_posts",
            "_sass",
            "assets/css",
            "assets/js",
            "assets/images",
        ];
        
        for dir in &dirs {
            let dir_path = dest_dir.join(dir);
            create_dir_if_not_exists(&dir_path)?;
            
            result.changes.push(MigrationChange {
                file_path: dir.to_string(),
                change_type: ChangeType::Created,
                description: format!("Created directory {}", dir),
            });
        }
        
        Ok(())
    }
}

// Helper function to check if a directory has a typical Cobalt structure
fn has_cobalt_structure(source_dir: &Path) -> bool {
    // Look for typical Cobalt directories/files
    source_dir.join("_layouts").exists() ||
    source_dir.join("_includes").exists() ||
    source_dir.join("posts").exists() ||
    source_dir.join("_posts").exists() ||
    source_dir.join("assets").exists()
}

// Helper function to check if a config file appears to be a Cobalt config
fn is_cobalt_config(source_dir: &Path) -> bool {
    let config_path = source_dir.join("_config.yml");
    if !config_path.exists() {
        return false;
    }
    
    // Check if the config file has Cobalt-specific settings
    if let Ok(content) = fs::read_to_string(&config_path) {
        content.contains("source:") || content.contains("destination:") ||
        content.contains("default:") || content.contains("template_extensions:") ||
        content.contains("ignore:") || content.contains("syntax_highlight:") ||
        content.contains("excerpt_separator:") || content.contains("posts_path:")
    } else {
        false
    }
} 