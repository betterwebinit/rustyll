use std::path::{Path, PathBuf};
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
        log::info!("Migrating Bridgetown components...");
    }

    // Create destination includes directory for components
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // In Bridgetown, components are typically in src/_components directory
    let components_dir = source_dir.join("src/_components");
    if !components_dir.exists() || !components_dir.is_dir() {
        result.warnings.push("No src/_components directory found.".into());
        return Ok(());
    }

    // Migrate component files
    for entry in WalkDir::new(&components_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the components directory
            let rel_path = file_path.strip_prefix(&components_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create destination path
            let dest_path = dest_includes_dir.join(rel_path);
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert the component file
            convert_component_file(file_path, &dest_path, result)?;
        }
    }
    
    Ok(())
}

fn convert_component_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the component file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read component file {}: {}", source_path.display(), e))?;
    
    // Convert Bridgetown component to Jekyll include
    let converted_content = convert_bridgetown_component_to_jekyll(&content, source_path);
    
    // Write the converted include
    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted component from {}", source_path.display()),
    });
    
    Ok(())
}

fn convert_bridgetown_component_to_jekyll(content: &str, source_path: &Path) -> String {
    let mut converted = content.to_string();
    
    // Extract component front matter if it exists
    let has_front_matter = content.starts_with("---");
    
    if has_front_matter {
        // Remove Bridgetown component-specific front matter
        if let Some(end_index) = content.find("---\n") {
            if let Some(second_end) = content[end_index + 4..].find("---\n") {
                let front_matter = &content[..end_index + 4 + second_end + 4];
                let front_matter_lines: Vec<&str> = front_matter.lines().collect();
                
                // Extract any useful information from front matter
                let mut include_comment = String::from("{% comment %}\nConverted from Bridgetown component\n");
                
                for line in front_matter_lines {
                    if line.contains("name:") || line.contains("description:") {
                        include_comment.push_str(&format!("{}\n", line.trim()));
                    }
                }
                
                include_comment.push_str("{% endcomment %}\n\n");
                
                // Replace the front matter with a comment
                converted = converted.replace(front_matter, &include_comment);
            }
        }
    } else {
        // Add a comment indicating this is a converted component
        converted = format!("{{% comment %}}\nConverted from Bridgetown component: {}\n{{% endcomment %}}\n\n{}", 
            source_path.file_name().unwrap().to_string_lossy(),
            converted);
    }
    
    // Bridgetown uses <%= yield %> for rendering content, Jekyll uses {{ content }}
    converted = converted.replace("<%= yield %>", "{{ include.content }}");
    
    // Bridgetown uses <%= component.PROP %> for props, Jekyll uses {{ include.PROP }}
    let component_props_regex = regex::Regex::new(r"<%=\s*component\.(\w+)\s*%>").unwrap();
    converted = component_props_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{ include.{} }}", &caps[1])
    }).to_string();
    
    // Convert Bridgetown-style Ruby expressions
    let erb_expression_regex = regex::Regex::new(r"<%=\s*(.*?)\s*%>").unwrap();
    converted = erb_expression_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{ {} }}", &caps[1].replace("site.", "site."))
    }).to_string();
    
    // Convert Ruby conditionals to Liquid
    let erb_conditional_regex = regex::Regex::new(r"<%\s*if\s+(.*?)\s*%>").unwrap();
    converted = erb_conditional_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% if {} %}}", &caps[1].replace("component.", "include."))
    }).to_string();
    
    converted = converted.replace("<% else %>", "{% else %}");
    converted = converted.replace("<% end %>", "{% endif %}");
    
    // Convert each loops
    let erb_loop_regex = regex::Regex::new(r"<%\s*(.+?)\.each\s+do\s+\|\s*(.+?)\s*\|\s*%>").unwrap();
    converted = erb_loop_regex.replace_all(&converted, |caps: &regex::Captures| {
        let collection = &caps[1].replace("component.", "include.");
        format!("{{% for {} in {} %}}", &caps[2], collection)
    }).to_string();
    
    converted = converted.replace("<% end %>", "{% endfor %}");
    
    // Add usage example at the bottom
    let file_name = source_path.file_name().unwrap().to_string_lossy();
    converted.push_str(&format!(
        "\n{{% comment %}}\nUsage:\n{{{{ include {} param1=\"value1\" param2=\"value2\" content=\"Some content\" }}}}\n{{% endcomment %}}\n",
        file_name
    ));
    
    converted
} 