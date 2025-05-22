use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_data(source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
    if verbose {
        log::info!("Migrating data files from Cobalt to Rustyll format");
    }
    
    // Create the _data directory in the destination
    let dest_data_dir = dest_dir.join("_data");
    create_dir_if_not_exists(&dest_data_dir)?;
    
    // Cobalt sites often have data files in the _data directory
    let potential_data_dirs = vec![
        source_dir.join("_data"),
        source_dir.join("data"),
    ];
    
    let mut found_data = false;
    
    // Process each potential data directory
    for data_dir in &potential_data_dirs {
        if data_dir.exists() && data_dir.is_dir() {
            if verbose {
                log::info!("Found data directory: {}", data_dir.display());
            }
            
            // Walk the data directory
            for entry in WalkDir::new(data_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    // Filter for data files
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        
                        if ext == "json" || ext == "yaml" || ext == "yml" || ext == "toml" {
                            // Get the relative path from the data directory
                            let rel_path = file_path.strip_prefix(data_dir)
                                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                            
                            // Create destination path
                            let dest_path = dest_data_dir.join(rel_path);
                            
                            // Create parent directories if needed
                            if let Some(parent) = dest_path.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Copy the file
                            fs::copy(file_path, &dest_path)
                                .map_err(|e| format!("Failed to copy data file: {}", e))?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("_data/{}", rel_path.display()),
                                change_type: ChangeType::Copied,
                                description: format!("Data file copied from Cobalt site").to_string(),
                            });
                            
                            found_data = true;
                        }
                    }
                }
            }
        }
    }
    
    // If no data files were found, create a sample data file
    if !found_data {
        if verbose {
            log::info!("No data files found, creating sample data");
        }
        
        // Create a basic site data file
        let site_data = r#"---
title: Cobalt Site
description: A site migrated from Cobalt to Rustyll
navigation:
  - text: Home
    url: /
  - text: About
    url: /about
  - text: Blog
    url: /blog
---"#;
        
        fs::write(dest_data_dir.join("site.yml"), site_data)
            .map_err(|e| format!("Failed to write sample data file: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_data/site.yml".to_string(),
            change_type: ChangeType::Created,
            description: "Sample site data created".to_string(),
        });
    }
    
    // Create a README for the data directory
    let readme_content = r#"# Data Directory

This directory contains data files migrated from your Cobalt site. These files can be accessed in templates using the `site.data` namespace.

## Usage

In your templates, you can access this data using Liquid syntax:

```liquid
{% for item in site.data.site.navigation %}
  <a href="{{ item.url }}">{{ item.text }}</a>
{% endfor %}
```

## Data Format

The data files in this directory are in YAML or JSON format. If you need to add more data:

1. Add new YAML or JSON files to this directory
2. Access them using `site.data.filename`
3. Nested directories can be accessed using dot notation

For example, a file at `_data/authors/info.yml` would be accessible as `site.data.authors.info`.
"#;
    
    fs::write(dest_data_dir.join("README.md"), readme_content)
        .map_err(|e| format!("Failed to write README file: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_data/README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Data directory README created".to_string(),
    });
    
    Ok(())
} 