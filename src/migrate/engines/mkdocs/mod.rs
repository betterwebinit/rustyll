use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists
};

mod config;
mod content;
mod layouts;
mod assets;
mod data;
mod static_files;
mod static_assets;
mod readme;

pub struct MkdocsMigrator;

impl MkdocsMigrator {
    pub fn new() -> Self {
        MkdocsMigrator
    }
}

impl EngineMigrator for MkdocsMigrator {
    fn name(&self) -> &'static str {
        "MkDocs"
    }
    
    fn description(&self) -> &'static str {
        "Migrates MkDocs sites to Jekyll"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for mkdocs.yml at root
        let config_path = source_dir.join("mkdocs.yml");
        if config_path.exists() {
            return true;
        }
        
        // Check for docs directory with an index.md
        let docs_index = source_dir.join("docs").join("index.md");
        if docs_index.exists() {
            return true;
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
            log::info!("Starting migration from MkDocs to Jekyll...");
        }
        
        // Execute migration steps in sequence
        self.create_directory_structure(dest_dir, &mut result)?;
        
        // Call the module functions directly
        content::migrate_content(source_dir, dest_dir, verbose, &mut result)?;
        layouts::migrate_layouts(source_dir, dest_dir, verbose, &mut result)?;
        config::migrate_config(source_dir, dest_dir, verbose, &mut result)?;
        assets::migrate_assets(source_dir, dest_dir, verbose, &mut result)?;
        static_files::migrate_static(source_dir, dest_dir, verbose, &mut result)?;
        data::migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate a README
        readme::generate_readme(dest_dir, &mut result)?;
        
        if verbose {
            log::info!("Migration from MkDocs to Jekyll completed successfully!");
        }
        
        Ok(result)
    }
}

impl MkdocsMigrator {
    fn create_directory_structure(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create standard Jekyll directories
        let dirs = [
            "_layouts",
            "_includes",
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