use std::path::Path;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists
};
use walkdir::WalkDir;

mod config;
mod content;
mod plugins;
mod themes;
mod static_files;
mod translations;

pub struct PelicanMigrator;

impl PelicanMigrator {
    pub fn new() -> Self {
        PelicanMigrator
    }
}

impl EngineMigrator for PelicanMigrator {
    fn name(&self) -> &'static str {
        "Pelican"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Pelican sites to Jekyll"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for pelicanconf.py
        let config_path = source_dir.join("pelicanconf.py");
        if config_path.exists() {
            return true;
        }
        
        // Check for content directory with typical Pelican structure
        let content_dir = source_dir.join("content");
        if content_dir.exists() && content_dir.is_dir() {
            // Check for typical Pelican content files
            for entry in WalkDir::new(&content_dir).max_depth(2).into_iter().filter_map(Result::ok) {
                if entry.path().extension().map_or(false, |ext| ext == "md" || ext == "rst") {
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
            log::info!("Starting migration from Pelican to Jekyll...");
        }
        
        // Create directory structure
        self.create_directory_structure(dest_dir, &mut result)?;
        
        // Migrate components
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        plugins::migrate_plugins(source_dir, dest_dir, verbose, &mut result)?;
        themes::migrate_themes(source_dir, dest_dir, verbose, &mut result)?;
        static_files::migrate_static_files(source_dir, dest_dir, verbose, &mut result)?;
        translations::migrate_translations(source_dir, dest_dir, verbose, &mut result)?;
        
        if verbose {
            log::info!("Migration from Pelican to Jekyll completed successfully!");
        }
        
        Ok(result)
    }
}

impl PelicanMigrator {
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