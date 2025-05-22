use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::HugoMigrator {
    pub(super) fn migrate_partials(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Hugo, partials are in themes/theme-name/layouts/partials or layouts/partials
        let mut partials_dirs = vec![
            source_dir.join("layouts").join("partials"),
        ];
        
        // Check for theme partials
        let themes_dir = source_dir.join("themes");
        if themes_dir.exists() && themes_dir.is_dir() {
            for theme_entry in fs::read_dir(&themes_dir)
                .map_err(|e| format!("Failed to read themes directory: {}", e))? {
                
                if let Ok(theme_entry) = theme_entry {
                    let theme_path = theme_entry.path();
                    if theme_path.is_dir() {
                        partials_dirs.push(theme_path.join("layouts").join("partials"));
                    }
                }
            }
        }
        
        let includes_dest_dir = dest_dir.join("_includes");
        create_dir_if_not_exists(&includes_dest_dir)?;
        
        // Keep track of processed includes to avoid duplicates
        let mut processed_includes = std::collections::HashSet::new();
        
        for partials_dir in partials_dirs {
            if partials_dir.exists() && partials_dir.is_dir() {
                if verbose {
                    log::info!("Migrating partials from {}", partials_dir.display());
                }
                
                // Process all partial files
                for entry in WalkDir::new(&partials_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        // Get a simplified name for the include
                        let file_name = file_path.file_name()
                            .ok_or_else(|| "Invalid file name".to_string())?
                            .to_string_lossy()
                            .to_string();
                        
                        // Skip if already processed
                        if processed_includes.contains(&file_name) {
                            continue;
                        }
                        
                        let dest_path = includes_dest_dir.join(&file_name);
                        
                        // Read the template
                        let content = fs::read_to_string(file_path)
                            .map_err(|e| format!("Failed to read partial template {}: {}", file_path.display(), e))?;
                        
                        // Convert Go template to Liquid
                        // We're using the convert_hugo_template function from layouts.rs through the trait
                        let converted_content = self.convert_hugo_template(&content);
                        
                        // Write the converted template
                        fs::write(&dest_path, converted_content)
                            .map_err(|e| format!("Failed to write converted include: {}", e))?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_includes/{}", file_name),
                            change_type: ChangeType::Converted,
                            description: "Partial template converted to include".to_string(),
                        });
                        
                        // Mark as processed
                        processed_includes.insert(file_name);
                    }
                }
            }
        }
        
        // Create README for includes directory if any partials were found
        if !processed_includes.is_empty() {
            // Create README for includes directory
            let includes_readme = r#"# Includes Directory

This directory contains reusable template fragments migrated from Hugo's partials.

## Include Usage

In Rustyll:
- Files can be included using the `{% include "filename.html" %}` Liquid tag
- Includes can accept parameters using variables

## Changes from Hugo

- Hugo partials have been converted to Liquid includes
- Go template syntax has been converted to Liquid syntax
- `{{ .Title }}` is now `{{ page.title }}`
- `{{ .Site.Title }}` is now `{{ site.title }}`
- Context is handled differently - in Hugo you pass context with '.'

Example conversion:
- Hugo: `{{ partial "header.html" . }}`
- Rustyll: `{% include "header.html" %}`
"#;
            
            write_readme(&includes_dest_dir, includes_readme)?;
        } else {
            result.warnings.push(
                "No Hugo partials found to migrate.".to_string()
            );
        }
        
        Ok(())
    }
} 