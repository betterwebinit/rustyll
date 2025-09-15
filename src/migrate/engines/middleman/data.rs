use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::MiddlemanMigrator {
    pub(super) fn migrate_data(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Middleman, data is typically in data/ directory
        let source_data = source_dir.join("data");
        
        // Create destination data directory
        let dest_data = dest_dir.join("_data");
        create_dir_if_not_exists(&dest_data)?;
        
        if source_data.exists() && source_data.is_dir() {
            if verbose {
                log::info!("Migrating data from {}", source_data.display());
            }
            
            // Copy all data files
            for entry in WalkDir::new(&source_data)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let relative_path = file_path.strip_prefix(&source_data)
                        .map_err(|e| format!("Failed to get relative path: {}", e))?;
                    
                    let dest_path = dest_data.join(relative_path);
                    
                    // Create parent directories if needed
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Get the file extension
                    let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    
                    // Handle Ruby data files specially
                    if extension == "rb" {
                        // Extract file name without extension
                        let file_stem = file_path.file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("unknown");
                        
                        // Create YAML version of the file
                        let yaml_file_name = format!("{}.yml", file_stem);
                        let yaml_dest_path = dest_data.join(relative_path.parent().unwrap_or_else(|| Path::new("")))
                            .join(&yaml_file_name);
                        
                        // Create parent directories if needed
                        if let Some(parent) = yaml_dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        
                        // Copy the original Ruby file too for reference
                        let rb_dest_path = dest_data.join(relative_path);
                        if let Some(parent) = rb_dest_path.parent() {
                            create_dir_if_not_exists(parent)?;
                        }
                        copy_file(file_path, &rb_dest_path)?;
                        
                        // Create a placeholder YAML file
                        let yaml_content = "# This is a placeholder converted from a Ruby data file.\n# Manual conversion is required.\n\n# Example data:\n# title: Example\n# items:\n#   - name: Item 1\n#     url: /item1\n#   - name: Item 2\n#     url: /item2";
                        
                        fs::write(&yaml_dest_path, yaml_content)
                            .map_err(|e| format!("Failed to write YAML placeholder: {}", e))?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.display()),
                            change_type: ChangeType::Copied,
                            description: "Original Ruby data file copied for reference".to_string(),
                        });
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.parent().unwrap_or_else(|| Path::new("")).join(yaml_file_name).display()),
                            change_type: ChangeType::Created,
                            description: "Placeholder YAML data file created".to_string(),
                        });
                        
                        result.warnings.push(format!(
                            "Ruby data file _data/{} requires manual conversion to YAML/JSON",
                            relative_path.display()
                        ));
                    } else {
                        // For other data files (YAML, JSON), just copy them
                        copy_file(file_path, &dest_path)?;
                        
                        result.changes.push(MigrationChange {
                            file_path: format!("_data/{}", relative_path.display()),
                            change_type: ChangeType::Copied,
                            description: "Data file copied".to_string(),
                        });
                    }
                }
            }
            
            // Create README for data directory
            let data_readme = r#"# Data Directory

This directory contains data files migrated from Middleman.

## Data Format

Data in Rustyll:
- Can be in YAML or JSON format
- Is available in templates via the `site.data` namespace
- For example, data in `_data/navigation.yml` is accessible as `site.data.navigation`

## Changes from Middleman

- Middleman uses Ruby/YAML/JSON files for data, while Rustyll primarily uses YAML/JSON
- Ruby data files need to be manually converted to YAML or JSON
- Data access is different: `data.file_name` in Middleman vs `site.data.file_name` in Rustyll
"#;
            
            write_readme(&dest_data, data_readme)?;
        } else {
            // Check for alternative locations
            let alt_data_dirs = [
                source_dir.join("source").join("data"),
                source_dir.join("source").join("_data"),
            ];
            
            let mut found_alt = false;
            
            for alt_data_dir in alt_data_dirs.iter() {
                if alt_data_dir.exists() && alt_data_dir.is_dir() {
                    found_alt = true;
                    
                    if verbose {
                        log::info!("Found alternative data directory: {}", alt_data_dir.display());
                    }
                    
                    // Process data files (similar logic to above)
                    for entry in WalkDir::new(alt_data_dir)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if entry.file_type().is_file() {
                            let file_path = entry.path();
                            let relative_path = file_path.strip_prefix(alt_data_dir)
                                .map_err(|e| format!("Failed to get relative path: {}", e))?;
                            
                            let dest_path = dest_data.join(relative_path);
                            
                            // Create parent directories if needed
                            if let Some(parent) = dest_path.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Special handling for Ruby files
                            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                            if extension == "rb" {
                                // Same Ruby-to-YAML conversion logic as above
                                let file_stem = file_path.file_stem()
                                    .and_then(|stem| stem.to_str())
                                    .unwrap_or("unknown");
                                
                                let yaml_file_name = format!("{}.yml", file_stem);
                                let yaml_dest_path = dest_data.join(relative_path.parent().unwrap_or_else(|| Path::new("")))
                                    .join(&yaml_file_name);
                                
                                if let Some(parent) = yaml_dest_path.parent() {
                                    create_dir_if_not_exists(parent)?;
                                }
                                
                                copy_file(file_path, &dest_path)?;
                                
                                let yaml_content = "# This is a placeholder converted from a Ruby data file.\n# Manual conversion is required.\n\n# Example data:\n# title: Example\n# items:\n#   - name: Item 1\n#     url: /item1\n#   - name: Item 2\n#     url: /item2";
                                
                                fs::write(&yaml_dest_path, yaml_content)
                                    .map_err(|e| format!("Failed to write YAML placeholder: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_data/{}", relative_path.display()),
                                    change_type: ChangeType::Copied,
                                    description: "Original Ruby data file copied for reference".to_string(),
                                });
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_data/{}", relative_path.parent().unwrap_or_else(|| Path::new("")).join(yaml_file_name).display()),
                                    change_type: ChangeType::Created,
                                    description: "Placeholder YAML data file created".to_string(),
                                });
                                
                                result.warnings.push(format!(
                                    "Ruby data file _data/{} requires manual conversion to YAML/JSON",
                                    relative_path.display()
                                ));
                            } else {
                                // For regular data files, just copy
                                copy_file(file_path, &dest_path)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_data/{}", relative_path.display()),
                                    change_type: ChangeType::Copied,
                                    description: "Data file copied".to_string(),
                                });
                            }
                        }
                    }
                    
                    // Create README for data directory (same as above)
                    let data_readme = r#"# Data Directory

This directory contains data files migrated from Middleman.

## Data Format

Data in Rustyll:
- Can be in YAML or JSON format
- Is available in templates via the `site.data` namespace
- For example, data in `_data/navigation.yml` is accessible as `site.data.navigation`

## Changes from Middleman

- Middleman uses Ruby/YAML/JSON files for data, while Rustyll primarily uses YAML/JSON
- Ruby data files need to be manually converted to YAML or JSON
- Data access is different: `data.file_name` in Middleman vs `site.data.file_name` in Rustyll
"#;
                    
                    write_readme(&dest_data, data_readme)?;
                    
                    break;
                }
            }
            
            if !found_alt {
                result.warnings.push(
                    "No data directory found. Middleman data files need to be created manually.".to_string()
                );
                
                // Create an empty data directory with README anyway
                let data_readme = r#"# Data Directory

This directory can contain data files for your Rustyll site.

## Data Format

Data in Rustyll:
- Can be in YAML or JSON format
- Is available in templates via the `site.data` namespace
- For example, data in `_data/navigation.yml` is accessible as `site.data.navigation`

## Example Data File

Create a file named `navigation.yml` with content like:

```yaml
main:
  - title: Home
    url: /
  - title: About
    url: /about/
  - title: Blog
    url: /blog/
```

Then in your templates, you can use:

```liquid
{% for item in site.data.navigation.main %}
  <a href="{{ item.url }}">{{ item.title }}</a>
{% endfor %}
```
"#;
                
                write_readme(&dest_data, data_readme)?;
            }
        }
        
        Ok(())
    }
} 