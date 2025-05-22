use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::JekyllMigrator {
    pub(super) fn migrate_includes(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        let includes_source_dir = source_dir.join("_includes");
        let includes_dest_dir = dest_dir.join("_includes");
        
        if includes_source_dir.exists() {
            if verbose {
                log::info!("Migrating includes");
            }
            
            create_dir_if_not_exists(&includes_dest_dir)?;
            
            // Process all include files recursively
            for entry in WalkDir::new(&includes_source_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let rel_path = file_path.strip_prefix(&includes_source_dir)
                        .map_err(|_| "Failed to get relative path".to_string())?;
                    
                    let dest_path = includes_dest_dir.join(rel_path);
                    
                    // Create parent directory if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the file
                    copy_file(file_path, &dest_path)?;
                    
                    let file_path_str = format!("_includes/{}", rel_path.to_string_lossy());
                    result.changes.push(MigrationChange {
                        file_path: file_path_str,
                        change_type: ChangeType::Converted,
                        description: "Include file migrated".to_string(),
                    });
                }
            }
            
            // Create README for includes directory
            let includes_readme = r#"# Includes Directory

This directory contains reusable template fragments migrated from Jekyll.

## Include Usage

In Rustyll, includes work the same way as in Jekyll:
- Files can be included using the `{% include file.html %}` Liquid tag
- Includes can accept parameters: `{% include file.html param="value" %}`
- Includes can be nested

## Example 

```liquid
{% include header.html %}

<main>
  {% include sidebar.html highlighted=true %}
  {{ content }}
</main>

{% include footer.html %}
```
"#;
            
            write_readme(&includes_dest_dir, includes_readme)?;
        }
        
        Ok(())
    }
} 