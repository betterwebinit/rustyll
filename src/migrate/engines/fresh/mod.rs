use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult,
    create_dir_if_not_exists
};

pub struct FreshMigrator;

impl FreshMigrator {
    pub fn new() -> Self {
        FreshMigrator
    }
}

impl EngineMigrator for FreshMigrator {
    fn name(&self) -> &'static str {
        "Fresh"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Fresh (Deno-based) sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Fresh-specific files and directories
        source_dir.join("fresh.gen.ts").exists() || 
        source_dir.join("deno.json").exists() && has_fresh_deps(source_dir) ||
        source_dir.join("import_map.json").exists() && has_fresh_imports(source_dir)
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
        
        // Migrate islands (interactive components)
        islands::migrate_islands(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate routes (pages)
        routes::migrate_routes(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate components
        components::migrate_components(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        static_assets::migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate data files
        data::migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README
        readme::generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if deno.json contains Fresh dependencies
fn has_fresh_deps(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("deno.json")) {
        content.contains("\"fresh\"") || content.contains("@fresh")
    } else {
        false
    }
}

// Helper function to check if import_map.json contains Fresh imports
fn has_fresh_imports(source_dir: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(source_dir.join("import_map.json")) {
        content.contains("\"fresh\"") || content.contains("@fresh")
    } else {
        false
    }
}

// Import implementation details from separate files
mod config;
mod routes;
mod components;
mod islands;
mod static_assets;
mod data;
mod readme; 