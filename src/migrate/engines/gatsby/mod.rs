use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};

pub struct GatsbyMigrator;

impl GatsbyMigrator {
    pub fn new() -> Self {
        GatsbyMigrator
    }
}

impl EngineMigrator for GatsbyMigrator {
    fn name(&self) -> &'static str {
        "Gatsby"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Gatsby (React-based) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        source_dir.join("gatsby-config.js").exists() ||
        source_dir.join("package.json").exists() && has_gatsby_dependency(source_dir)
    }
    
    fn migrate(&self, options: &MigrationOptions) -> Result<MigrationResult, String> {
        // For now, just return a placeholder result
        let mut result = MigrationResult {
            engine_name: self.name().to_string(),
            changes: Vec::new(),
            warnings: vec!["Gatsby migration is not yet fully implemented".to_string()],
            errors: Vec::new(),
        };
        
        Ok(result)
    }
}

// Helper function to check if package.json contains Gatsby dependency
fn has_gatsby_dependency(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("package.json")) {
        content.contains("\"gatsby\"")
    } else {
        false
    }
} 