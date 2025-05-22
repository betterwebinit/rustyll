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
        log::info!("Migrating Metalsmith layouts...");
    }
    
    // Metalsmith layouts can be in several locations
    let possible_layout_dirs = [
        source_dir.join("layouts"),
        source_dir.join("templates"),
        source_dir.join("themes"),
        source_dir.join("src").join("layouts"),
        source_dir.join("views"),
    ];
    
    let mut found_layouts = false;
    
    // Create the Jekyll layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;
    
    // Also create Jekyll includes directory for partials
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;
    
    for layout_dir in possible_layout_dirs.iter() {
        if !layout_dir.exists() {
            continue;
        }
        
        found_layouts = true;
        
        // Process all layout files
        let layout_files = WalkDir::new(layout_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file());
        
        for entry in layout_files {
            let relative_path = entry.path().strip_prefix(layout_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            
            let file_content = fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read layout file {}: {}", entry.path().display(), e))?;
            
            // Determine if it's a layout or a partial based on path or content
            let (dest_file, is_partial) = if is_partial_template(entry.path(), &file_content) {
                // Partials go to _includes
                (dest_includes_dir.join(relative_path), true)
            } else {
                // Main layouts go to _layouts
                // Convert file extension to .html if it's not already
                let new_path = if let Some(ext) = relative_path.extension() {
                    if ext != "html" && ext != "htm" {
                        relative_path.with_extension("html")
                    } else {
                        relative_path.to_path_buf()
                    }
                } else {
                    relative_path.with_extension("html")
                };
                
                (dest_layouts_dir.join(new_path), false)
            };
            
            // Ensure parent directory exists
            if let Some(parent) = dest_file.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert template syntax to Liquid
            let converted_content = convert_template_to_liquid(&file_content, is_partial)?;
            
            fs::write(&dest_file, converted_content)
                .map_err(|e| format!("Failed to write layout file {}: {}", dest_file.display(), e))?;
            
            // Add to migration results
            let template_type = if is_partial { "partial" } else { "layout" };
            let rel_path = if is_partial {
                format!("_includes/{}", relative_path.display())
            } else {
                format!("_layouts/{}", relative_path.with_extension("html").display())
            };
            
            result.changes.push(MigrationChange {
                file_path: rel_path,
                change_type: ChangeType::Converted,
                description: format!("Converted Metalsmith {} to Liquid", template_type),
            });
        }
    }
    
    if !found_layouts {
        result.warnings.push("Could not find Metalsmith layouts directory".into());
        
        // Create default layout as fallback
        create_default_layout(dest_layouts_dir.join("default.html"), result)?;
    }
    
    Ok(())
}

fn is_partial_template(path: &Path, content: &str) -> bool {
    // Check filename for common partial indicators
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if filename.starts_with("_") || 
       filename.starts_with("partial") || 
       filename.contains("include") || 
       path.parent().map_or(false, |p| p.file_name().map_or(false, |d| d.to_string_lossy().contains("partial"))) {
        return true;
    }
    
    // Check content for partial-like content (no <html>, short content)
    if !content.contains("<html") && !content.contains("{{> body") && content.len() < 1000 {
        return true;
    }
    
    false
}

fn convert_template_to_liquid(content: &str, is_partial: bool) -> Result<String, String> {
    let mut converted = content.to_string();
    
    // Detect template engine used
    let is_handlebars = content.contains("{{") && !content.contains("{%");
    let is_ejs = content.contains("<%") && content.contains("%>");
    let is_nunjucks = content.contains("{%") || content.contains("{{") && content.contains("}}");
    
    if is_handlebars {
        // Convert Handlebars syntax to Liquid
        converted = converted
            // Handlebars partials
            .replace("{{> ", "{% include ")
            .replace(" }}", " %}")
            
            // Handlebars each loops
            .replace("{{#each ", "{% for item in ")
            .replace("{{/each}}", "{% endfor %}")
            
            // Handlebars if conditions
            .replace("{{#if ", "{% if ")
            .replace("{{else}}", "{% else %}")
            .replace("{{/if}}", "{% endif %}")
            
            // Handle "this" contexts in Handlebars
            .replace("{{this.", "{{ item.")
            .replace("{{./", "{{ item.")
            
            // Regular variables
            .replace("{{", "{{ ")
            .replace("}}", " }}");
        
        // Special handling for body content placeholder
        converted = converted.replace("{{> body }}", "{{ content }}");
        converted = converted.replace("{% include body %}", "{{ content }}");
    } else if is_ejs {
        // Convert EJS syntax to Liquid
        converted = converted
            .replace("<%=", "{{ ")
            .replace("<%", "{% ")
            .replace("%>", " %}")
            .replace("{{ include(", "{% include ")
            .replace(" })", " %}");
    } else if is_nunjucks {
        // Nunjucks is already similar to Liquid, but some adjustments are needed
        converted = converted
            .replace("{% block ", "{% capture ")
            .replace("{% endblock %}", "{% endcapture %}")
            .replace("{{ super() }}", "{{ content }}");
    }
    
    // Add Jekyll front matter if it's a layout and not a partial
    if !is_partial && !converted.starts_with("---") {
        converted = format!("---\n---\n\n{}", converted);
    }
    
    Ok(converted)
}

fn create_default_layout(dest_file: PathBuf, result: &mut MigrationResult) -> Result<(), String> {
    let default_layout = r#"---
layout: default
---
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title }}{% else %}Site Title{% endif %}</title>
    <link rel="stylesheet" href="/assets/css/main.css">
</head>
<body>
    <header>
        <h1>Site Header</h1>
        <nav>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
            </ul>
        </nav>
    </header>
    
    <main>
        {{ content }}
    </main>
    
    <footer>
        <p>&copy; {% raw %}{{ 'now' | date: "%Y" }}{% endraw %} - Site Footer</p>
    </footer>
</body>
</html>
"#;
    
    fs::write(&dest_file, default_layout)
        .map_err(|e| format!("Failed to create default layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_layouts/default.html".into(),
        change_type: ChangeType::Created,
        description: "Created default layout as fallback".into(),
    });
    
    Ok(())
} 