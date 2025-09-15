use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::MiddlemanMigrator {
    pub(super) fn migrate_partials(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Middleman, partials are typically in source/partials/ or can be prefixed with _
        let source_partials_dir = source_dir.join("source").join("partials");
        
        // Create destination includes directory
        let dest_includes = dest_dir.join("_includes");
        create_dir_if_not_exists(&dest_includes)?;
        
        let mut found_partials = false;
        
        // Check if there's a dedicated partials directory
        if source_partials_dir.exists() && source_partials_dir.is_dir() {
            found_partials = true;
            
            if verbose {
                log::info!("Migrating partials from {}", source_partials_dir.display());
            }
            
            // Copy all partial files
            for entry in WalkDir::new(&source_partials_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = entry.file_name().to_string_lossy();
                    
                    // Only process template files
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        
                        if ["erb", "html.erb", "haml", "slim", "html"].contains(&ext.as_ref()) {
                            // Determine the destination file name
                            let final_file_name = if file_name.starts_with('_') {
                                // Remove the leading underscore (Middleman convention)
                                let name_without_underscore = file_name.strip_prefix('_').unwrap_or(&file_name);
                                
                                if name_without_underscore.ends_with(".html.erb") {
                                    name_without_underscore.replace(".html.erb", ".html")
                                } else if name_without_underscore.ends_with(".erb") {
                                    name_without_underscore.replace(".erb", ".html")
                                } else if name_without_underscore.ends_with(".haml") {
                                    name_without_underscore.replace(".haml", ".html")
                                } else if name_without_underscore.ends_with(".slim") {
                                    name_without_underscore.replace(".slim", ".html")
                                } else {
                                    name_without_underscore.to_string()
                                }
                            } else {
                                if file_name.ends_with(".html.erb") {
                                    file_name.replace(".html.erb", ".html")
                                } else if file_name.ends_with(".erb") {
                                    file_name.replace(".erb", ".html")
                                } else if file_name.ends_with(".haml") {
                                    file_name.replace(".haml", ".html")
                                } else if file_name.ends_with(".slim") {
                                    file_name.replace(".slim", ".html")
                                } else {
                                    file_name.to_string()
                                }
                            };
                            
                            let dest_path = dest_includes.join(&final_file_name);
                            
                            // Convert or copy the partial file
                            if ["erb", "html.erb", "haml", "slim"].contains(&ext.as_ref()) {
                                // For template files, we need to convert to Liquid
                                let content = fs::read_to_string(file_path)
                                    .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                                
                                // Apply basic conversion based on template type
                                let converted_content = match ext.as_ref() {
                                    "erb" | "html.erb" => content
                                        .replace("<%= ", "{{ ")
                                        .replace("<% ", "{% ")
                                        .replace(" %>", " %}")
                                        .replace(" -%>", " %}"),
                                    
                                    "haml" | "slim" => {
                                        // For Haml/Slim, we would need a more sophisticated converter
                                        format!(
                                            "<!-- MANUAL CONVERSION NEEDED -->\n<!-- Original {} template from: {} -->\n\n{}",
                                            ext, file_path.display(), content
                                        )
                                    },
                                    
                                    _ => content,
                                };
                                
                                fs::write(&dest_path, converted_content)
                                    .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_includes/{}", final_file_name),
                                    change_type: ChangeType::Converted,
                                    description: format!("Partial converted from {} to Liquid", ext),
                                });
                                
                                if ext == "haml" || ext == "slim" {
                                    result.warnings.push(format!(
                                        "Partial file _includes/{} requires manual conversion from {} to Liquid",
                                        final_file_name, ext
                                    ));
                                }
                            } else {
                                // For HTML files, just copy
                                copy_file(file_path, &dest_path)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_includes/{}", final_file_name),
                                    change_type: ChangeType::Copied,
                                    description: "Partial file copied".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Also look for files prefixed with underscore in the source directory
        let source_content_dir = source_dir.join("source");
        if source_content_dir.exists() && source_content_dir.is_dir() {
            // Recursively find all files starting with underscore
            for entry in WalkDir::new(&source_content_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = entry.file_name().to_string_lossy();
                    
                    // Skip paths within the partials directory (if it exists)
                    if source_partials_dir.exists() && file_path.starts_with(&source_partials_dir) {
                        continue;
                    }
                    
                    // Only process underscore files that look like templates
                    if file_name.starts_with('_') && file_path.extension().is_some() {
                        let ext = file_path.extension().unwrap().to_string_lossy().to_lowercase();
                        
                        if ["erb", "html.erb", "haml", "slim", "html"].contains(&ext.as_ref()) {
                            found_partials = true;
                            
                            if verbose {
                                log::info!("Found partial file: {}", file_path.display());
                            }
                            
                            // Use the same conversion logic as above
                            let name_without_underscore = file_name.strip_prefix('_').unwrap_or(&file_name);
                            
                            let final_file_name = if name_without_underscore.ends_with(".html.erb") {
                                name_without_underscore.replace(".html.erb", ".html")
                            } else if name_without_underscore.ends_with(".erb") {
                                name_without_underscore.replace(".erb", ".html")
                            } else if name_without_underscore.ends_with(".haml") {
                                name_without_underscore.replace(".haml", ".html")
                            } else if name_without_underscore.ends_with(".slim") {
                                name_without_underscore.replace(".slim", ".html")
                            } else {
                                name_without_underscore.to_string()
                            };
                            
                            let dest_path = dest_includes.join(&final_file_name);
                            
                            // Convert or copy the partial file (same logic as above)
                            if ["erb", "html.erb", "haml", "slim"].contains(&ext.as_ref()) {
                                let content = fs::read_to_string(file_path)
                                    .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                                
                                let converted_content = match ext.as_ref() {
                                    "erb" | "html.erb" => content
                                        .replace("<%= ", "{{ ")
                                        .replace("<% ", "{% ")
                                        .replace(" %>", " %}")
                                        .replace(" -%>", " %}"),
                                    
                                    "haml" | "slim" => {
                                        format!(
                                            "<!-- MANUAL CONVERSION NEEDED -->\n<!-- Original {} template from: {} -->\n\n{}",
                                            ext, file_path.display(), content
                                        )
                                    },
                                    
                                    _ => content,
                                };
                                
                                fs::write(&dest_path, converted_content)
                                    .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_includes/{}", final_file_name),
                                    change_type: ChangeType::Converted,
                                    description: format!("Partial converted from {} to Liquid", ext),
                                });
                                
                                if ext == "haml" || ext == "slim" {
                                    result.warnings.push(format!(
                                        "Partial file _includes/{} requires manual conversion from {} to Liquid",
                                        final_file_name, ext
                                    ));
                                }
                            } else {
                                copy_file(file_path, &dest_path)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_includes/{}", final_file_name),
                                    change_type: ChangeType::Copied,
                                    description: "Partial file copied".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Create README for includes directory
        let includes_readme = r#"# Includes Directory

This directory contains partial templates migrated from Middleman.

## Include Format

Includes in Rustyll:
- Use Liquid template syntax
- Can be included in templates with `{% include "filename.html" %}`
- Can receive variables from the parent template

## Changes from Middleman

- Middleman partials are prefixed with underscore, but in Rustyll this is not required
- Partials in Middleman are used with `partial` or `render` helpers; in Rustyll use `{% include "file.html" %}`
- ERB/Haml/Slim syntax has been converted to Liquid where possible
"#;
        
        write_readme(&dest_includes, includes_readme)?;
        
        if !found_partials {
            result.warnings.push(
                "No partials found in the Middleman site. Includes may need to be created manually.".to_string()
            );
        }
        
        Ok(())
    }
} 