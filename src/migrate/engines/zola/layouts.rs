use std::path::Path;

use crate::migrate::MigrationResult;

pub fn migrate_layouts(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Zola layouts to Jekyll...");
    }

    // Look for templates in the themes directory and root templates directory
    let templates_dir = source_dir.join("templates");
    let themes_dir = source_dir.join("themes");

    // Check if either directory exists
    if !templates_dir.exists() && !themes_dir.exists() {
        if verbose {
            println!("No templates directory found. Skipping layouts migration.");
        }
        return Ok(());
    }

    // Create the _layouts directory in Jekyll
    let layouts_dir = dest_dir.join("_layouts");
    if !layouts_dir.exists() {
        std::fs::create_dir_all(&layouts_dir).map_err(|e| {
            format!("Failed to create _layouts directory: {}", e)
        })?;
    }

    // Process templates from the root templates directory
    if templates_dir.exists() {
        process_templates_dir(&templates_dir, &layouts_dir, verbose, result)?;
    }

    // Process templates from themes directories if they exist
    if themes_dir.exists() {
        // Find theme directories
        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let theme_path = entry.path();
                if theme_path.is_dir() {
                    let theme_templates = theme_path.join("templates");
                    if theme_templates.exists() {
                        process_templates_dir(&theme_templates, &layouts_dir, verbose, result)?;
                    }
                }
            }
        }
    }

    if verbose {
        println!("Completed Zola layouts migration");
    }

    Ok(())
}

fn process_templates_dir(
    templates_dir: &Path,
    layouts_dir: &Path, 
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(templates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    // Process HTML, JINJA templates
                    if ext == "html" || ext == "j2" || ext == "jinja" || ext == "tera" {
                        if verbose {
                            println!("Processing template: {:?}", path);
                        }
                        
                        // Get the filename
                        if let Some(file_name) = path.file_name() {
                            let dest_file = layouts_dir.join(file_name);
                            
                            // Copy and convert the template
                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    // Convert Zola/Tera syntax to Liquid
                                    let converted = convert_zola_to_liquid(&content);
                                    
                                    // Write the converted template
                                    if let Err(e) = std::fs::write(&dest_file, converted) {
                                        result.errors.push(format!(
                                            "Failed to write template {:?}: {}", dest_file, e
                                        ));
                                    } else if verbose {
                                        println!("Converted {:?} to {:?}", path, dest_file);
                                    }
                                }
                                Err(e) => {
                                    result.errors.push(format!(
                                        "Failed to read template {:?}: {}", path, e
                                    ));
                                }
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // Process subdirectories recursively
                let dir_name = path.file_name().unwrap_or_default();
                let dest_subdir = layouts_dir.join(dir_name);
                
                if !dest_subdir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&dest_subdir) {
                        result.errors.push(format!(
                            "Failed to create directory {:?}: {}", dest_subdir, e
                        ));
                        continue;
                    }
                }
                
                process_templates_dir(&path, &dest_subdir, verbose, result)?;
            }
        }
    }
    
    Ok(())
}

fn convert_zola_to_liquid(content: &str) -> String {
    // This is a simplified conversion from Zola's Tera templates to Liquid
    // In a real implementation, this would be more comprehensive
    let mut converted = content.to_string();
    
    // Replace Zola/Tera syntax with Liquid syntax
    // - Replace {% for x in y %} format (already compatible)
    // - Replace {% if x %} format (already compatible)
    // - Replace {{ x }} format (already compatible)
    
    // Replace Zola-specific macros and functions
    converted = converted.replace("{{ get_url", "{{ site.baseurl }}");
    converted = converted.replace("{{ config.base_url", "{{ site.url }}{{ site.baseurl ");
    
    // Basic replacement for Zola sections
    converted = converted.replace("{% block content %}", "{{ content }}");
    converted = converted.replace("{% endblock content %}", "");
    
    converted
} 