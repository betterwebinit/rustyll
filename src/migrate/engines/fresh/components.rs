use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_components(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Fresh components...");
    }

    // In Fresh, components are in components/ directory
    let components_dir = source_dir.join("components");
    if !components_dir.exists() || !components_dir.is_dir() {
        result.warnings.push("No components directory found.".into());
        return Ok(());
    }

    // Create destination includes directory
    let includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&includes_dir)?;

    // Migrate components to Jekyll includes
    for entry in WalkDir::new(&components_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip non-TypeScript/JavaScript files
            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension != "tsx" && extension != "jsx" && extension != "ts" && extension != "js" {
                continue;
            }
            
            // Get the relative path from the components directory
            let rel_path = file_path.strip_prefix(&components_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create the destination path
            let file_stem = file_path.file_stem().unwrap().to_string_lossy();
            let parent_path = rel_path.parent().unwrap_or(Path::new(""));
            
            let dest_path = if parent_path.as_os_str().is_empty() {
                includes_dir.join(format!("{}.html", file_stem))
            } else {
                let component_dir = includes_dir.join(parent_path);
                create_dir_if_not_exists(&component_dir)?;
                component_dir.join(format!("{}.html", file_stem))
            };
            
            // Convert the component file to a Jekyll include
            migrate_component_file(file_path, &dest_path, result)?;
        }
    }
    
    Ok(())
}

fn migrate_component_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the component file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read component file {}: {}", source_path.display(), e))?;
    
    // Extract props from the component if possible
    let props_pattern = r#"(?:interface\s+(\w+)Props|type\s+(\w+)Props|Props\s*=|props\s*:)"#;
    let has_props = regex::Regex::new(props_pattern).unwrap().is_match(&content);
    
    // Create a Jekyll include
    let include_content = format!(r##"{{{{ comment }}}}
This include was migrated from a Fresh component: {}
Original path: {}
{{{{ endcomment }}}}

<div class="component">
  <!-- TODO: Implement this component in HTML/Liquid -->
  <!-- The original component likely had the following structure: -->
  
  {{{{ if include.title }}}}
  <h2>{{ include.title }}</h2>
  {{{{ endif }}}}
  
  <div class="component-content">
    {{ include.content }}
    {{{{ if include.children }}}}
      {{ include.children }}
    {{{{ endif }}}}
  </div>
</div>

{{{{ comment }}}}
Usage:
{{{{ include {} }}}}
or with parameters:
{{{{ include {} title="Example" content="Some content" }}}}
{{{{ endcomment }}}}
"##, 
        source_path.file_name().unwrap().to_string_lossy(),
        source_path.display(),
        dest_path.file_name().unwrap().to_string_lossy(),
        dest_path.file_name().unwrap().to_string_lossy()
    );
    
    // Write the Jekyll include
    fs::write(dest_path, include_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/{}", dest_path.strip_prefix(&dest_path.parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Converted Fresh component from {}", source_path.display()),
    });
    
    Ok(())
} 