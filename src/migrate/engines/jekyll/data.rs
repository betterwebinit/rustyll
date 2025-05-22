use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::JekyllMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let data_source_dir = source_dir.join("_data");
        let data_dest_dir = dest_dir.join("_data");
        
        if data_source_dir.exists() {
            if verbose {
                log::info!("Migrating data files");
            }
            
            create_dir_if_not_exists(&data_dest_dir)?;
            
            // Process all data files recursively
            for entry in WalkDir::new(&data_source_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
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

This directory contains data files migrated from Jekyll.

## Data Usage

Data files in Rustyll work the same way as in Jekyll:
- YAML, JSON, and CSV files can store structured data
- Data is accessible in templates through the `site.data` object

## Examples

For a file named `_data/navigation.yml` containing:
```yaml
- title: Home
  url: /
- title: About
  url: /about/
```

In a template, you'd use:
```liquid
<nav>
  <ul>
    {% for item in site.data.navigation %}
      <li><a href="{{ item.url }}">{{ item.title }}</a></li>
    {% endfor %}
  </ul>
</nav>
```

## Changes from Jekyll

Data functionality in Rustyll is compatible with Jekyll, but some advanced
features might require adjustments.
"#;
            
            write_readme(&data_dest_dir, data_readme)?;
        }
        
        Ok(())
    }
} 