use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_layouts(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Cobalt layouts...");
    }

    // In Cobalt, layouts are typically in _layouts directory
    let layouts_dir = source_dir.join("_layouts");
    if !layouts_dir.exists() || !layouts_dir.is_dir() {
        result.warnings.push("No _layouts directory found.".into());
        return Ok(());
    }

    // Create destination layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;

    // Migrate layout files
    for entry in WalkDir::new(&layouts_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Process only markup files
            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "liquid" || extension == "html" || extension == "md" || extension == "markdown" {
                // Get the relative path from the layouts directory
                let rel_path = file_path.strip_prefix(&layouts_dir)
                    .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                
                // Create destination path
                let dest_path = dest_layouts_dir.join(rel_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    create_dir_if_not_exists(parent)?;
                }
                
                // Convert the layout file
                convert_layout_file(file_path, &dest_path, result)?;
            }
        }
    }
    
    Ok(())
}

fn convert_layout_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the layout file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read layout file {}: {}", source_path.display(), e))?;
    
    // Convert Cobalt layout to Jekyll layout
    let converted_content = convert_cobalt_layout_to_jekyll(&content);
    
    // Write the converted layout
    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write layout file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted layout from {}", source_path.display()),
    });
    
    Ok(())
}

fn convert_cobalt_layout_to_jekyll(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Cobalt uses {% include "file.liquid" %}, Jekyll uses {% include file.html %}
    converted = converted.replace("{% include \"", "{% include ");
    converted = converted.replace(".liquid\" %}", ".html %} ");
    
    // Cobalt uses {{ path.to.variable }}, which is similar to Jekyll's {{ page.path.to.variable }}
    // But we need to check for the context and might need to convert
    
    // Handle Cobalt-specific tags/filters
    converted = converted.replace("{{ page.higher_pages | default: empty }}", "{{ page.ancestors }}");
    converted = converted.replace("{{ page.higher_pages }}", "{{ page.ancestors }}");
    
    // Cobalt permalink format
    let permalink_regex = regex::Regex::new(r"permalink:\s*(.+)").unwrap();
    converted = permalink_regex.replace_all(&converted, |caps: &regex::Captures| {
        let path = &caps[1];
        if path.starts_with('/') {
            format!("permalink: {}", path.trim())
        } else {
            format!("permalink: /{}", path.trim())
        }
    }).to_string();
    
    converted
} 