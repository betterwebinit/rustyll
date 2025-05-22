use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, copy_file};

impl super::JekyllMigrator {
    pub(super) fn migrate_config(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let source_config = source_dir.join("_config.yml");
        let dest_config = dest_dir.join("_config.yml");
        
        if source_config.exists() {
            if verbose {
                log::info!("Migrating configuration file");
            }
            
            // In a real implementation, we would parse the YAML and convert it
            // For now, just copy the file and add a note
            copy_file(&source_config, &dest_config)?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".to_string(),
                change_type: ChangeType::Converted,
                description: "Configuration file migrated".to_string(),
            });
            
            result.warnings.push(
                "You may need to adjust some configuration settings in _config.yml to be compatible with Rustyll".to_string()
            );
        }
        
        Ok(())
    }
} 