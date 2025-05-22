use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::HugoMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let data_source_dir = source_dir.join("data");
        let data_dest_dir = dest_dir.join("_data");
        
        if data_source_dir.exists() && data_source_dir.is_dir() {
            if verbose {
                log::info!("Migrating data files");
            }
            
            create_dir_if_not_exists(&data_dest_dir)?;
            
            // Process all data files
            for entry in WalkDir::new(&data_source_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Get the relative path from data directory
                    let rel_path = file_path.strip_prefix(&data_source_dir)
                        .map_err(|_| "Failed to get relative path".to_string())?;
                    
                    let dest_path = data_dest_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the file
                    copy_file(file_path, &dest_path)?;
                    
                    let file_path_str = format!("_data/{}", rel_path.to_string_lossy());
                    result.changes.push(MigrationChange {
                        file_path: file_path_str,
                        change_type: ChangeType::Converted,
                        description: "Data file migrated".to_string(),
                    });
                }
            }
            
            // Create README for data directory
            let data_readme = r#"# Data Directory

This directory contains data files migrated from Hugo.

## Data Usage

Data files in Rustyll:
- YAML, JSON, and CSV files are supported
- Data is accessible through the `site.data` variable in templates

## Example

If you have a file named `_data/navigation.yml`, you can access it in templates using:

```liquid
{% for item in site.data.navigation %}
  <a href="{{ item.url }}">{{ item.title }}</a>
{% endfor %}
```

## Changes from Hugo

Hugo uses the `site.Data` object, while Rustyll uses `site.data`
"#;
            
            write_readme(&data_dest_dir, data_readme)?;
        }
        
        Ok(())
    }
} 