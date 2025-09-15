use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::JekyllMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let layouts_source_dir = source_dir.join("_layouts");
        let layouts_dest_dir = dest_dir.join("_layouts");
        
        if layouts_source_dir.exists() {
            if verbose {
                log::info!("Migrating layouts");
            }
            
            create_dir_if_not_exists(&layouts_dest_dir)?;
            
            // Iterate through all layout files
            for entry in fs::read_dir(&layouts_source_dir)
                .map_err(|e| format!("Failed to read layouts directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    let file_name = path.file_name()
                        .ok_or_else(|| "Invalid file name".to_string())?
                        .to_string_lossy()
                        .to_string();
                    
                    let dest_path = layouts_dest_dir.join(&file_name);
                    
                    // Copy layout file
                    copy_file(&path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("_layouts/{}", file_name),
                        change_type: ChangeType::Converted,
                        description: "Layout file migrated".to_string(),
                    });
                }
            }
            
            // Create README for layouts directory
            let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Jekyll.

## Layout Usage

Layouts in Rustyll:
- Define the HTML structure of pages
- Can be specified in front matter using `layout: template_name`
- Can inherit from other layouts

Example usage in a content file:
```yaml
---
layout: default
title: My Page
---

Content goes here...
```

## Changes from Jekyll

The layout system in Rustyll is compatible with Jekyll, but some specific features
or extensions might need adjustments.
"#;
            
            write_readme(&layouts_dest_dir, layouts_readme)?;
        }
        
        Ok(())
    }
} 