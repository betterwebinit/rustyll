use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_islands(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Fresh islands...");
    }

    // In Fresh, islands are in islands/ directory
    let islands_dir = source_dir.join("islands");
    if !islands_dir.exists() || !islands_dir.is_dir() {
        result.warnings.push("No islands directory found.".into());
        return Ok(());
    }

    // Create destination assets/js directory for island scripts
    let js_dir = dest_dir.join("assets/js/islands");
    create_dir_if_not_exists(&js_dir)?;
    
    // Create destination includes directory for island templates
    let includes_dir = dest_dir.join("_includes/islands");
    create_dir_if_not_exists(&includes_dir)?;

    // Migrate islands
    for entry in WalkDir::new(&islands_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip non-TypeScript/JavaScript files
            let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension != "tsx" && extension != "jsx" && extension != "ts" && extension != "js" {
                continue;
            }
            
            // Get the relative path from the islands directory
            let rel_path = file_path.strip_prefix(&islands_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create the island name
            let file_stem = file_path.file_stem().unwrap().to_string_lossy();
            let parent_path = rel_path.parent().unwrap_or(Path::new(""));
            
            // Create the include template
            let include_path = if parent_path.as_os_str().is_empty() {
                includes_dir.join(format!("{}.html", file_stem))
            } else {
                let include_subdir = includes_dir.join(parent_path);
                create_dir_if_not_exists(&include_subdir)?;
                include_subdir.join(format!("{}.html", file_stem))
            };
            
            // Create the static version of the island
            migrate_island_to_include(file_path, &include_path, result)?;
            
            // Create a placeholder JavaScript file for the client-side functionality
            let js_path = if parent_path.as_os_str().is_empty() {
                js_dir.join(format!("{}.js", file_stem))
            } else {
                let js_subdir = js_dir.join(parent_path);
                create_dir_if_not_exists(&js_subdir)?;
                js_subdir.join(format!("{}.js", file_stem))
            };
            
            create_island_script(file_path, &js_path, result)?;
        }
    }
    
    Ok(())
}

fn migrate_island_to_include(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the island file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read island file {}: {}", source_path.display(), e))?;
    
    // Get the island name
    let island_name = source_path.file_stem().unwrap().to_string_lossy();
    
    // Create a Jekyll include with appropriate data-attributes to hydrate with JS
    let include_content = format!(r##"{{{{ comment }}}}
This include was migrated from a Fresh island: {}
Original path: {}
{{{{ endcomment }}}}

<div class="island" 
     data-island-id="{{ include.id | default: '{}' }}" 
     data-island-name="{}" 
     data-island-props="{{ include.props | jsonify }}">
  <!-- Static fallback content -->
  <div class="island-content">
    {{{{ if include.title }}}}
    <h3>{{ include.title }}</h3>
    {{{{ endif }}}}
    
    {{ include.content }}
    
    {{{{ if include.children }}}}
      {{ include.children }}
    {{{{ endif }}}}
  </div>
  
  <!-- Island JavaScript will be injected here -->
  <script>
    // This script tag will load the island JS after page load
    document.addEventListener('DOMContentLoaded', function() {{
      const script = document.createElement('script');
      script.src = "{{ '/assets/js/islands/{}.js' | relative_url }}";
      script.async = true;
      document.body.appendChild(script);
    }});
  </script>
</div>

{{{{ comment }}}}
Usage:
{{{{ include islands/{}.html }}}}
or with parameters:
{{{{ include islands/{}.html id="unique-id" title="Example" content="Some content" props=page.island_props }}}}
{{{{ endcomment }}}}
"##, 
        source_path.file_name().unwrap().to_string_lossy(),
        source_path.display(),
        island_name,
        island_name,
        island_name,
        island_name,
        island_name
    );
    
    // Write the Jekyll include
    fs::write(dest_path, include_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/islands/{}", dest_path.strip_prefix(&dest_path.parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Converted Fresh island to include from {}", source_path.display()),
    });
    
    Ok(())
}

fn create_island_script(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get the island name
    let island_name = source_path.file_stem().unwrap().to_string_lossy();
    
    // Create a JavaScript file that would hydrate the island
    let js_content = format!(r##"/**
 * Island script for {}
 * Migrated from Fresh island: {}
 */

(function() {{
  // Find all instances of this island in the page
  const islands = document.querySelectorAll('[data-island-name="{}"]');
  
  islands.forEach(island => {{
    const id = island.getAttribute('data-island-id');
    const propsJson = island.getAttribute('data-island-props');
    const props = propsJson ? JSON.parse(propsJson) : {{}};
    
    // TODO: Implement actual island functionality here
    console.log(`Hydrating island {} with ID: ${{id}}`, props);
    
    // Example of adding interactivity:
    island.addEventListener('click', function() {{
      console.log(`Island ${{id}} was clicked`);
    }});
  }});
}})();
"##, 
        island_name,
        source_path.display(),
        island_name,
        island_name
    );
    
    // Write the JavaScript file
    fs::write(dest_path, js_content)
        .map_err(|e| format!("Failed to write JavaScript file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("assets/js/islands/{}", dest_path.strip_prefix(&dest_path.parent().unwrap().parent().unwrap().parent().unwrap()).unwrap().display()),
        description: format!("Created JavaScript for island from {}", source_path.display()),
    });
    
    Ok(())
} 