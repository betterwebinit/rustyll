use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file, write_readme};

pub(super) fn migrate_templates(source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
    let templates_source_dir = source_dir.join("templates");
    
    if !templates_source_dir.exists() {
        result.warnings.push("No templates directory found in Zola site".to_string());
        return Ok(());
    }
    
    if verbose {
        log::info!("Migrating templates from Zola to Rustyll format");
    }
    
    // Zola templates need to be converted from Tera to Liquid
    // For this example, we'll create layouts and includes directories
    
    // Process layouts (base templates)
    let layouts_dest_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&layouts_dest_dir)?;
    
    // Common base templates in Zola
    let base_templates = vec![
        templates_source_dir.join("base.html"),
        templates_source_dir.join("index.html"),
        templates_source_dir.join("page.html"),
        templates_source_dir.join("section.html"),
    ];
    
    for base_template in &base_templates {
        if base_template.exists() {
            let file_name = base_template.file_name()
                .ok_or_else(|| "Invalid file name".to_string())?
                .to_string_lossy()
                .to_string();
            
            let dest_path = layouts_dest_dir.join(&file_name);
            
            // Copy the file (in a real implementation, we'd convert Tera templates to Liquid)
            copy_file(&base_template, &dest_path)?;
            
            result.changes.push(MigrationChange {
                file_path: format!("_layouts/{}", file_name),
                change_type: ChangeType::Converted,
                description: "Layout template converted from Zola format".to_string(),
            });
            
            result.warnings.push(
                format!("Template {} uses Tera syntax which needs manual conversion to Liquid", file_name)
            );
        }
    }
    
    // Process macros and partials
    let includes_dirs = vec![
        templates_source_dir.join("macros"),
        templates_source_dir.join("partials"),
        templates_source_dir.join("includes"),
        templates_source_dir.join("shortcodes"),
    ];
    
    let includes_dest_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&includes_dest_dir)?;
    
    for includes_dir in &includes_dirs {
        if includes_dir.exists() && includes_dir.is_dir() {
            for entry in WalkDir::new(includes_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let file_name = file_path.file_name()
                        .ok_or_else(|| "Invalid file name".to_string())?
                        .to_string_lossy()
                        .to_string();
                    
                    let dest_path = includes_dest_dir.join(&file_name);
                    
                    // Copy the file (in a real implementation, we'd convert Tera to Liquid)
                    copy_file(file_path, &dest_path)?;
                    
                    result.changes.push(MigrationChange {
                        file_path: format!("_includes/{}", file_name),
                        change_type: ChangeType::Converted,
                        description: "Include template converted from Zola format".to_string(),
                    });
                    
                    result.warnings.push(
                        format!("Template {} uses Tera syntax which needs manual conversion to Liquid", file_name)
                    );
                }
            }
        }
    }
    
    // Process other templates
    for entry in fs::read_dir(&templates_source_dir)
        .map_err(|e| format!("Failed to read templates directory: {}", e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        // Skip directories we've already processed
        if path.is_dir() && includes_dirs.iter().any(|dir| dir == &path) {
            continue;
        }
        
        // Skip base templates we've already processed
        if path.is_file() && base_templates.iter().any(|template| template == &path) {
            continue;
        }
        
        if path.is_file() {
            let file_name = path.file_name()
                .ok_or_else(|| "Invalid file name".to_string())?
                .to_string_lossy()
                .to_string();
            
            // Determine if this is likely a layout or include
            let dest_dir = if file_name.contains("page") || file_name.contains("base") || file_name.contains("section") {
                &layouts_dest_dir
            } else {
                &includes_dest_dir
            };
            
            let dest_path = dest_dir.join(&file_name);
            
            // Copy the file
            copy_file(&path, &dest_path)?;
            
            let rel_path = if dest_dir == &layouts_dest_dir {
                format!("_layouts/{}", file_name)
            } else {
                format!("_includes/{}", file_name)
            };
            
            result.changes.push(MigrationChange {
                file_path: rel_path,
                change_type: ChangeType::Converted,
                description: "Template converted from Zola format".to_string(),
            });
            
            result.warnings.push(
                format!("Template {} uses Tera syntax which needs manual conversion to Liquid", file_name)
            );
        }
    }
    
    // Create README for layouts directory
    let layouts_readme = r#"# Layouts Directory

This directory contains layout templates migrated from Zola.

## Layout Format

Layouts in Rustyll:
- Use Liquid templating instead of Zola's Tera templates
- Files are typically HTML with Liquid templating
- Layouts can inherit from other layouts using the `layout` front matter property

## Manual Conversion Needed

Zola templates use Tera syntax which is very different from Liquid.
You'll need to manually convert the templates:

1. Replace `{% block content %}{% endblock %}` with `{{ content }}`
2. Replace `{% extends "base.html" %}` with front matter `layout: base`
3. Replace Tera functions and filters with Liquid equivalents
"#;
    
    write_readme(&layouts_dest_dir, layouts_readme)?;
    
    // Create README for includes directory
    let includes_readme = r#"# Includes Directory

This directory contains reusable template fragments migrated from Zola's macros and partials.

## Include Usage

In Rustyll:
- Files can be included using the `{% include file.html %}` Liquid tag
- Includes can accept parameters: `{% include file.html param="value" %}`

## Manual Conversion Needed

Zola templates use Tera syntax which is different from Liquid.
You'll need to manually convert:

1. Replace Tera macros with Liquid includes
2. Convert Tera filters to Liquid filters
3. Update variable access syntax

Example conversion:
- Zola: `{% include "partials/header.html" %}`
- Rustyll: `{% include header.html %}`
"#;
    
    write_readme(&includes_dest_dir, includes_readme)?;
    
    result.warnings.push(
        "Zola uses Tera templates while Rustyll uses Liquid. All templates need manual conversion.".to_string()
    );
    
    Ok(())
} 