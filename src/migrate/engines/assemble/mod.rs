use std::path::Path;
use std::fs;
use crate::migrate::{
    EngineMigrator, MigrationOptions, MigrationResult,
    create_dir_if_not_exists
};

pub struct AssembleMigrator;

impl AssembleMigrator {
    pub fn new() -> Self {
        AssembleMigrator
    }
}

impl EngineMigrator for AssembleMigrator {
    fn name(&self) -> &'static str {
        "Assemble"
    }
    
    fn description(&self) -> &'static str {
        "Migrates Assemble sites to Rustyll format"
    }
    
    fn detect(&self, source_dir: &Path) -> bool {
        // Check for Assemble-specific files and directories
        source_dir.join("assemblefile.js").exists() || 
        source_dir.join("Gruntfile.js").exists() && source_dir.join("pages").exists() ||
        source_dir.join("package.json").exists() && has_assemble_dependency(source_dir)
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
        
        // Migrate data
        self.migrate_data(source_dir, dest_dir, verbose, &mut result)?;
        
        // Migrate static assets
        self.migrate_static_assets(source_dir, dest_dir, verbose, &mut result)?;
        
        // Generate README files
        self.generate_readme(dest_dir, &mut result)?;
        
        Ok(result)
    }
}

// Helper function to check if package.json contains assemble as a dependency
fn has_assemble_dependency(source_dir: &Path) -> bool {
    let package_json_path = source_dir.join("package.json");
    if !package_json_path.exists() {
        return false;
    }
    
    match fs::read_to_string(&package_json_path) {
        Ok(content) => {
            // Simple check for assemble in the dependencies
            content.contains(r#""assemble":"#) || content.contains(r#""assemble-plugin"#)
        },
        Err(_) => false,
    }
}

// Import implementation details from separate files
mod config;
mod content;
mod layouts;
mod data;
mod static_assets;
mod readme; 