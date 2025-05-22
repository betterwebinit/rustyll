use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

impl super::MiddlemanMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Middleman, layouts are typically in source/layouts/
        let source_layouts = source_dir.join("source").join("layouts");
        
        // Create destination layouts directory
        let dest_layouts = dest_dir.join("_layouts");
        create_dir_if_not_exists(&dest_layouts)?;
        
        let mut found_layouts = false;
        
        if source_layouts.exists() && source_layouts.is_dir() {
            found_layouts = true;
            
            if verbose {
                log::info!("Migrating layouts from {}", source_layouts.display());
            }
            
            // Copy all layout files
            for entry in WalkDir::new(&source_layouts)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = entry.file_name().to_string_lossy();
                    
                    // Only process template files
                    if let Some(extension) = file_path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        
                        if ["erb", "html.erb", "haml", "slim", "html"].contains(&ext.as_ref()) {
                            // Clean the file name/extension for Rustyll
                            let final_file_name = if file_name.ends_with(".html.erb") {
                                file_name.replace(".html.erb", ".html")
                            } else if file_name.ends_with(".erb") {
                                file_name.replace(".erb", ".html")
                            } else if file_name.ends_with(".haml") {
                                file_name.replace(".haml", ".html")
                            } else if file_name.ends_with(".slim") {
                                file_name.replace(".slim", ".html")
                            } else {
                                file_name.to_string()
                            };
                            
                            let dest_path = dest_layouts.join(&final_file_name);
                            
                            // Convert or copy the layout file
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
                                        .replace(" -%>", " %}")
                                        // Convert yield to content
                                        .replace("<%= yield %>", "{{ content }}")
                                        .replace("<%= yield_content :content %>", "{{ content }}")
                                        .replace("<%= yield_content(:content) %>", "{{ content }}"),
                                    
                                    "haml" | "slim" => {
                                        // For Haml/Slim, we would need a more sophisticated converter
                                        // This is a placeholder that marks it for manual conversion
                                        format!(
                                            "---\n---\n<!-- MANUAL CONVERSION NEEDED -->\n<!-- Original {} template from: {} -->\n\n{}",
                                            ext, file_path.display(), content
                                        )
                                    },
                                    
                                    _ => content,
                                };
                                
                                fs::write(&dest_path, converted_content)
                                    .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_layouts/{}", final_file_name),
                                    change_type: ChangeType::Converted,
                                    description: format!("{} template converted to Liquid", ext),
                                });
                                
                                if ext == "haml" || ext == "slim" {
                                    result.warnings.push(format!(
                                        "Layout file _layouts/{} requires manual conversion from {} to Liquid",
                                        final_file_name, ext
                                    ));
                                }
                            } else {
                                // For HTML files, just copy
                                copy_file(file_path, &dest_path)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("_layouts/{}", final_file_name),
                                    change_type: ChangeType::Copied,
                                    description: "Layout file copied".to_string(),
                                });
                            }
                        }
                    }
                }
            }
            
            // Create README for layouts directory
            let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Middleman.

## Layout Format

Layouts in Rustyll:
- Use Liquid template syntax
- Should reference content with `{{ content }}` variable
- Can use includes with `{% include "file.html" %}`
- Can extend other layouts with `layout: other_layout` in front matter

## Changes from Middleman

- Middleman uses ERB/Haml/Slim templates, which have been converted to Liquid where possible
- Ruby helpers need to be replaced with Liquid filters or plugins
- The `yield` statement has been replaced with `{{ content }}`
"#;
            
            write_readme(&dest_layouts, layouts_readme)?;
            
        } else {
            // Check for alternative layout locations
            let alt_layouts = [
                source_dir.join("layouts"),
                source_dir.join("source/_layouts"),
            ];
            
            for alt_layout_dir in alt_layouts.iter() {
                if alt_layout_dir.exists() && alt_layout_dir.is_dir() {
                    found_layouts = true;
                    
                    if verbose {
                        log::info!("Found alternative layouts directory: {}", alt_layout_dir.display());
                    }
                    
                    // Process layout files (similar logic to above)
                    for entry in WalkDir::new(alt_layout_dir)
                        .into_iter()
                        .filter_map(Result::ok) {
                        
                        if entry.file_type().is_file() {
                            let file_path = entry.path();
                            let file_name = entry.file_name().to_string_lossy();
                            
                            // Only process template files
                            if let Some(extension) = file_path.extension() {
                                let ext = extension.to_string_lossy().to_lowercase();
                                
                                if ["erb", "html.erb", "haml", "slim", "html"].contains(&ext.as_ref()) {
                                    // Clean the file name for Rustyll
                                    let final_file_name = if file_name.ends_with(".html.erb") {
                                        file_name.replace(".html.erb", ".html")
                                    } else if file_name.ends_with(".erb") {
                                        file_name.replace(".erb", ".html")
                                    } else if file_name.ends_with(".haml") {
                                        file_name.replace(".haml", ".html")
                                    } else if file_name.ends_with(".slim") {
                                        file_name.replace(".slim", ".html")
                                    } else {
                                        file_name.to_string()
                                    };
                                    
                                    let dest_path = dest_layouts.join(&final_file_name);
                                    
                                    // Perform the same conversion as above
                                    if ["erb", "html.erb", "haml", "slim"].contains(&ext.as_ref()) {
                                        // Similar conversion code as above
                                        let content = fs::read_to_string(file_path)
                                            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
                                        
                                        let converted_content = match ext.as_ref() {
                                            "erb" | "html.erb" => content
                                                .replace("<%= ", "{{ ")
                                                .replace("<% ", "{% ")
                                                .replace(" %>", " %}")
                                                .replace(" -%>", " %}")
                                                .replace("<%= yield %>", "{{ content }}")
                                                .replace("<%= yield_content :content %>", "{{ content }}")
                                                .replace("<%= yield_content(:content) %>", "{{ content }}"),
                                            
                                            "haml" | "slim" => {
                                                format!(
                                                    "---\n---\n<!-- MANUAL CONVERSION NEEDED -->\n<!-- Original {} template from: {} -->\n\n{}",
                                                    ext, file_path.display(), content
                                                )
                                            },
                                            
                                            _ => content,
                                        };
                                        
                                        fs::write(&dest_path, converted_content)
                                            .map_err(|e| format!("Failed to write converted file: {}", e))?;
                                        
                                        result.changes.push(MigrationChange {
                                            file_path: format!("_layouts/{}", final_file_name),
                                            change_type: ChangeType::Converted,
                                            description: format!("{} template converted to Liquid", ext),
                                        });
                                        
                                        if ext == "haml" || ext == "slim" {
                                            result.warnings.push(format!(
                                                "Layout file _layouts/{} requires manual conversion from {} to Liquid",
                                                final_file_name, ext
                                            ));
                                        }
                                    } else {
                                        // For HTML files, just copy
                                        copy_file(file_path, &dest_path)?;
                                        
                                        result.changes.push(MigrationChange {
                                            file_path: format!("_layouts/{}", final_file_name),
                                            change_type: ChangeType::Copied,
                                            description: "Layout file copied".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    
                    // Create README for layouts directory (same as above)
                    let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Middleman.

## Layout Format

Layouts in Rustyll:
- Use Liquid template syntax
- Should reference content with `{{ content }}` variable
- Can use includes with `{% include "file.html" %}`
- Can extend other layouts with `layout: other_layout` in front matter

## Changes from Middleman

- Middleman uses ERB/Haml/Slim templates, which have been converted to Liquid where possible
- Ruby helpers need to be replaced with Liquid filters or plugins
- The `yield` statement has been replaced with `{{ content }}`
"#;
                    
                    write_readme(&dest_layouts, layouts_readme)?;
                    
                    break;
                }
            }
        }
        
        if !found_layouts {
            result.warnings.push(
                "No layouts directory found. Middleman layouts need to be created manually.".to_string()
            );
        }
        
        Ok(())
    }
} 