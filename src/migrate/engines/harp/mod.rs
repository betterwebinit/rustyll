use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult, MigrationChange, ChangeType,
    create_dir_if_not_exists, copy_file, write_readme
};
use walkdir::WalkDir;

pub struct HarpMigrator;

impl HarpMigrator {
    pub fn new() -> Self {
        HarpMigrator
    }
}

impl EngineMigrator for HarpMigrator {
    fn name(&self) -> &'static str {
        "Harp"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Harp sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Harp-specific files and directories
        source_dir.join("harp.json").exists() || 
        source_dir.join("_harp.json").exists() ||
        source_dir.join("_data.json").exists() ||
        (source_dir.join("public").exists() && has_harp_structure(source_dir))
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
        
        // Migrate partials (Harp's version of includes)
        self.migrate_partials(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data
        self.migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        self.migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        self.generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if a directory has a typical Harp structure
fn has_harp_structure(source_dir: &Path) -> bool {
    // Look for typical Harp directories/files
    let has_jade = fs::read_dir(source_dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| {
                    let path = entry.path();
                    path.is_file() && path.extension().map_or(false, |ext| ext == "jade" || ext == "ejs")
                })
        })
        .unwrap_or(false);

    let has_data_json = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .any(|entry| {
            let path = entry.path();
            path.is_file() && path.file_name().map(|name| name.to_string_lossy() == "_data.json").unwrap_or(false)
        });

    has_jade || has_data_json ||
    source_dir.join("_layout.jade").exists() ||
    source_dir.join("_layout.ejs").exists()
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod partials;
mod data;
mod static_assets;
mod readme; 