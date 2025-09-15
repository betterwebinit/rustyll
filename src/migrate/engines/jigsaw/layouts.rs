use std::path::Path;
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
        log::info!("Migrating Jigsaw layouts...");
    }
    
    let layouts_dir = source_dir.join("source").join("_layouts");
    if !layouts_dir.exists() {
        // Check alternative location
        let alt_layouts_dir = source_dir.join("source").join("_templates");
        if !alt_layouts_dir.exists() {
            result.warnings.push("Could not find Jigsaw layouts directory".into());
            return Ok(());
        }
    }
    
    // Create destination layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;
    
    // Process all layout files (may be .blade.php, .php, .html, etc.)
    let layout_dirs = [
        source_dir.join("source").join("_layouts"),
        source_dir.join("source").join("_templates"),
        source_dir.join("source").join("_components"),
    ];
    
    for layout_dir in layout_dirs.iter() {
        if !layout_dir.exists() {
            continue;
        }
        
        let layout_files = WalkDir::new(layout_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file());
        
        for entry in layout_files {
            let relative_path = entry.path().strip_prefix(layout_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            // Determine the destination filename (convert .blade.php to .html)
            let filename = relative_path.file_name().unwrap_or_default().to_string_lossy();
            let new_filename = if filename.ends_with(".blade.php") {
                filename.replace(".blade.php", ".html")
            } else if filename.ends_with(".php") {
                filename.replace(".php", ".html")
            } else {
                filename.to_string()
            };
            
            let dest_path = dest_layouts_dir.join(
                relative_path.with_file_name(&new_filename)
            );
            
            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Read the layout file content
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read layout file {}: {}", entry.path().display(), e))?;
            
            // Convert Blade syntax to Liquid
            let converted_content = convert_blade_to_liquid(&file_content)?;
            
            // Write converted file
            fs::write(&dest_path, converted_content)
                .map_err(|e| format!("Failed to write layout file {}: {}", dest_path.display(), e))?;
            
            // Add to migration results
            result.changes.push(MigrationChange {
                file_path: format!("_layouts/{}", relative_path.with_file_name(&new_filename).display()),
                change_type: ChangeType::Converted,
                description: "Converted Blade layout to Liquid".into(),
            });
        }
    }
    
    // Also process master layout if it exists
    let master_layout = source_dir.join("source").join("_layouts").join("master.blade.php");
    if master_layout.exists() {
        let file_content = fs::read_to_string(&master_layout)
            .map_err(|e| format!("Failed to read master layout: {}", e))?;
        
        let converted_content = convert_blade_to_liquid(&file_content)?;
        
        // Write as default layout
        let default_layout = dest_layouts_dir.join("default.html");
        fs::write(&default_layout, converted_content)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
        
        // Add to migration results
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".into(),
            change_type: ChangeType::Converted,
            description: "Converted master layout to Jekyll default layout".into(),
        });
    }
    
    Ok(())
}

fn convert_blade_to_liquid(content: &str) -> Result<String, String> {
    let mut converted = content.to_string();
    
    // Replace Blade directives with Liquid tags
    converted = converted
        // Blade sections and yields
        .replace("@section", "{% block")
        .replace("@endsection", "{% endblock %}")
        .replace("@yield", "{{ content }}")
        
        // Blade conditionals
        .replace("@if", "{% if")
        .replace("@else", "{% else")
        .replace("@elseif", "{% elsif")
        .replace("@endif", "{% endif %}")
        
        // Blade loops
        .replace("@foreach", "{% for")
        .replace("@endforeach", "{% endfor %}")
        .replace("@for", "{% for")
        .replace("@endfor", "{% endfor %}")
        .replace("@while", "{% while")
        .replace("@endwhile", "{% endwhile %}")
        
        // Blade includes
        .replace("@include", "{% include")
        .replace("@extends", "{% extends")
        
        // Blade variables
        .replace("{{ $", "{{ ")
        .replace("{!! $", "{{ ")
        .replace(" }}", " }}");
    
    // Add Jekyll front matter if not present
    if !converted.starts_with("---") {
        converted = format!("---\n---\n{}", converted);
    }
    
    Ok(converted)
} 