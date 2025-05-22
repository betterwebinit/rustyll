use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_collections(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Jigsaw collections...");
    }
    
    // In Jigsaw, collections can be configured in config.php
    // and typically are stored in source/_posts, source/_docs, etc.
    let possible_collections = [
        "posts", "docs", "people", "projects", "products", "events"
    ];
    
    for collection_name in possible_collections.iter() {
        let collection_dir = source_dir.join("source").join(format!("_{}", collection_name));
        if !collection_dir.exists() {
            continue;
        }
        
        // Create destination collection directory
        let dest_collection_dir = dest_dir.join(format!("_{}", collection_name));
        create_dir_if_not_exists(&dest_collection_dir)?;
        
        // Process collection files
        let collection_files = WalkDir::new(&collection_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file());
        
        for entry in collection_files {
            let relative_path = entry.path().strip_prefix(&collection_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            let dest_file = dest_collection_dir.join(relative_path);
            
            // Ensure parent directory exists
            if let Some(parent) = dest_file.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Read and convert file content
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read collection file {}: {}", entry.path().display(), e))?;
            
            // Convert to Jekyll format
            let jekyll_content = convert_collection_item(&file_content, collection_name)?;
            
            fs::write(&dest_file, jekyll_content)
                .map_err(|e| format!("Failed to write collection file {}: {}", dest_file.display(), e))?;
            
            // Add to migration results
            result.changes.push(MigrationChange {
                file_path: format!("_{}/{}", collection_name, relative_path.display()),
                change_type: ChangeType::Converted,
                description: format!("Converted Jigsaw collection item in '{}'", collection_name),
            });
        }
        
        // Create collection configuration in _config.yml if needed
        generate_collection_config(collection_name, dest_dir, result)?;
    }
    
    // Also check for custom collections defined in PHP files
    detect_custom_collections(source_dir, dest_dir, verbose, result)?;
    
    Ok(())
}

fn convert_collection_item(content: &str, collection_name: &str) -> Result<String, String> {
    let mut converted = content.to_string();
    
    // Ensure proper frontmatter
    if !converted.starts_with("---") {
        converted = format!("---\nlayout: {}\n---\n{}", collection_name.trim_end_matches('s'), converted);
    } else {
        // Check if layout is defined, if not add it
        let layout_regex = regex::Regex::new(r"(?m)^layout:").ok()
            .ok_or_else(|| "Failed to create regex".to_string())?;
        
        if !layout_regex.is_match(&converted) {
            // Insert layout after the first ---
            let parts: Vec<&str> = converted.splitn(2, "---").collect();
            if parts.len() >= 2 {
                converted = format!("---\nlayout: {}{}", 
                    collection_name.trim_end_matches('s'), 
                    parts[1].to_string());
            }
        }
    }
    
    // Convert Blade/Jigsaw template directives to Liquid
    converted = converted
        .replace("@section", "{% block")
        .replace("@endsection", "{% endblock %}")
        .replace("@if", "{% if")
        .replace("@else", "{% else")
        .replace("@elseif", "{% elsif")
        .replace("@endif", "{% endif %}")
        .replace("@foreach", "{% for")
        .replace("@endforeach", "{% endfor %}")
        .replace("@include", "{% include")
        .replace("{{ $", "{{ ")
        .replace("{!! $", "{{ ")
        .replace(" }}", " }}");
    
    Ok(converted)
}

fn generate_collection_config(collection_name: &str, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Add this collection to _config.yml
    let config_path = dest_dir.join("_config.yml");
    let mut config_content = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read _config.yml: {}", e))?
    } else {
        "# Site settings\ntitle: Migrated Site\n".to_string()
    };
    
    // Check if collections section exists
    if !config_content.contains("collections:") {
        config_content.push_str("\ncollections:\n");
    }
    
    // Check if this collection is already configured
    let collection_regex = regex::Regex::new(&format!(r"(?m)^  {}:", collection_name)).ok()
        .ok_or_else(|| "Failed to create regex".to_string())?;
    
    if !collection_regex.is_match(&config_content) {
        // Find the collections section and add this collection
        let collections_regex = regex::Regex::new(r"(?m)^collections:").ok()
            .ok_or_else(|| "Failed to create regex".to_string())?;
        
        if let Some(pos) = collections_regex.find(&config_content) {
            let insert_pos = pos.end();
            let output_setting = if collection_name == "posts" {
                format!("\n  {}:\n    output: true\n    permalink: /{}/{{}}:output_ext", 
                        collection_name, collection_name)
            } else {
                format!("\n  {}:\n    output: true", collection_name)
            };
            
            config_content.insert_str(insert_pos, &output_setting);
            
            fs::write(&config_path, &config_content)
                .map_err(|e| format!("Failed to update _config.yml: {}", e))?;
            
            result.changes.push(MigrationChange {
                file_path: "_config.yml".into(),
                change_type: ChangeType::Modified,
                description: format!("Added '{}' collection configuration", collection_name),
            });
        }
    }
    
    Ok(())
}

fn detect_custom_collections(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Look for PHP files that might define collections
    let php_files = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path.is_file() && path.extension().map_or(false, |ext| ext == "php")
        });
    
    for entry in php_files {
        let content = fs::read_to_string(entry.path())
            .map_err(|e| format!("Failed to read PHP file {}: {}", entry.path().display(), e))?;
        
        // Simple heuristic: look for strings that might define collections
        if content.contains("addCollection") || content.contains("->collection") {
            result.warnings.push(format!(
                "Possible custom collection definition found in {}. Manual review needed.",
                entry.path().display()
            ));
        }
    }
    
    Ok(())
} 