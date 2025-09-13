use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists
};

mod config;
mod content;
mod layouts;
mod components;
mod plugins;
mod assets;

pub struct BridgetownMigrator;

impl BridgetownMigrator {
    pub fn new() -> Self {
        BridgetownMigrator
    }
}

impl EngineMigrator for BridgetownMigrator {
    fn name(&self) -> &'static str {
        "Bridgetown"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Bridgetown sites to Jekyll"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for bridgetown.config.rb
        let config_path = source_dir.join("bridgetown.config.rb");
        if config_path.exists() {
            return true;
        }
        
        // Check for Gemfile with bridgetown gem
        let gemfile_path = source_dir.join("Gemfile");
        if gemfile_path.exists() {
            if let Ok(content) = fs::read_to_string(&gemfile_path) {
                if content.contains("bridgetown") {
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
            log::info!("Starting migration from Bridgetown to Jekyll...");
        }
        
        // Create directories
        self.create_directory_structure(dest_dir, &mut result)?;
        
        // Delegate to module functions
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        components::migrate_components(source_dir, dest_dir, verbose, &mut result)?;
        plugins::migrate_plugins(source_dir, dest_dir, verbose, &mut result)?;
        assets::migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        if verbose {
            log::info!("Migration from Bridgetown to Jekyll completed successfully!");
        }
        
        Ok(result)
    }
}

impl BridgetownMigrator {
    fn create_directory_structure(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create standard Jekyll directories
        let dirs = [
            "_layouts",
            "_includes",
            "_data",
            "_posts",
            "_pages",
            "_plugins",
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