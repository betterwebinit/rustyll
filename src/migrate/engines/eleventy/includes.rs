use std::path::Path;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::EleventyMigrator {
    pub(super) fn migrate_includes(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Eleventy typically uses _includes directory
        let source_includes = source_dir.join("_includes");
        
        // Create destination includes directory
        let dest_includes = dest_dir.join("_includes");
        
        if source_includes.exists() && source_includes.is_dir() {
            if verbose {
                log::info!("Migrating includes from {}", source_includes.display());
            }
            
            // Create _includes directory in destination
            create_dir_if_not_exists(&dest_includes)?;
            
            // Copy all includes
            for entry in WalkDir::new(&source_includes)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let relative_path = file_path.strip_prefix(&source_includes)
                        .map_err(|e| format!("Failed to get relative path: {}", e))?;
                    
                    let dest_path = dest_includes.join(relative_path);
                    
                    // Create parent directories if they don't exist
                    if let Some(parent) = dest_path.parent() {
                        create_dir_if_not_exists(parent)?;
                    }
                    
                    // Copy the include file
                    copy_file(file_path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("_includes/{}", relative_path.display()),
                        change_type: ChangeType::Copied,
                        description: "Include file copied from Eleventy".to_string(),
                    });
                    
                    // Detect non-liquid templates (Eleventy supports multiple template languages)
                    if let Some(extension) = file_path.extension() {
                        let ext_str = extension.to_string_lossy().to_string();
                        if ext_str != "liquid" && ext_str != "html" {
                            result.warnings.push(format!(
                                "Include file _includes/{} has non-standard extension. May need conversion to Liquid.",
                                relative_path.display()
                            ));
                        }
                    }
                }
            }
            
            // Create README for includes directory
            let includes_readme = r#"# Includes Directory

This directory contains include files migrated from Eleventy.

## Include Format

Includes in Rustyll:
- Are processed using the Liquid template engine
- Can be included in templates and content with `{% include "filename.html" %}`
- Can receive variables from the parent template

## Changes from Eleventy

- Eleventy supports multiple template engines (Nunjucks, Handlebars, etc.), while Rustyll uses Liquid
- Some includes may need syntax conversion to work properly
- Nunjucks macros don't have a direct equivalent in Liquid and may need to be restructured
"#;
            
            write_readme(&dest_includes, includes_readme)?;
        } else {
            // Also check alternate locations that Eleventy sites might use
            let alt_includes = [
                source_dir.join("includes"),
                source_dir.join("src/includes"),
                source_dir.join("src/_includes")
            ];
            
            let mut found_alt = false;
            
            for alt_include_dir in alt_includes.iter() {
                if alt_include_dir.exists() && alt_include_dir.is_dir() {
                    found_alt = true;
                    
                    if verbose {
                        log::info!("Found alternative includes directory: {}", alt_include_dir.display());
                    }
                    
                    // Create destination includes directory
                    create_dir_if_not_exists(&dest_includes)?;
                    
                    // Copy alternative includes
                    for entry in WalkDir::new(alt_include_dir)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if entry.file_type().is_file() {
                            let file_path = entry.path();
                            let relative_path = file_path.strip_prefix(alt_include_dir)
                                .map_err(|e| format!("Failed to get relative path: {}", e))?;
                            
                            let dest_path = dest_includes.join(relative_path);
                            
                            // Create parent directories if they don't exist
                            if let Some(parent) = dest_path.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Copy the include file
                            copy_file(file_path, &dest_path)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("_includes/{}", relative_path.display()),
                                change_type: ChangeType::Copied,
                                description: format!("Include file copied from alternative location: {}", alt_include_dir.display()),
                            });
                        }
                    }
                    
                    break;
                }
            }
            
            if !found_alt {
                result.warnings.push(
                    "No _includes directory found. Eleventy includes need to be created manually.".to_string()
                );
            }
        }
        
        Ok(())
    }
} 