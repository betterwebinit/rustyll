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
        log::info!("Migrating Bridgetown layouts...");
    }

    // Create destination layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;

    // In Bridgetown, layouts are typically in src/_layouts directory
    let layouts_dir = source_dir.join("src/_layouts");
    if !layouts_dir.exists() || !layouts_dir.is_dir() {
        result.warnings.push("No src/_layouts directory found.".into());
        return Ok(());
    }

    // Migrate layout files
    for entry in WalkDir::new(&layouts_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
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
    
    // Check if we need to create a default layout
    if !dest_layouts_dir.join("default.html").exists() {
        create_default_layout(&dest_layouts_dir, result)?;
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
    
    // Convert Bridgetown layout to Jekyll layout
    let converted_content = convert_bridgetown_layout_to_jekyll(&content);
    
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

fn convert_bridgetown_layout_to_jekyll(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Bridgetown uses <%= yield %> for rendering content, Jekyll uses {{ content }}
    converted = converted.replace("<%= yield %>", "{{ content }}");
    
    // Bridgetown uses <%= site.data.something %>, Jekyll uses {{ site.data.something }}
    let erb_expression_regex = regex::Regex::new(r"<%=\s*(.*?)\s*%>").unwrap();
    converted = erb_expression_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{ {} }}", &caps[1])
    }).to_string();
    
    // Bridgetown uses <% if condition %>...<%end%>, Jekyll uses {% if condition %}...{% endif %}
    let erb_conditional_regex = regex::Regex::new(r"<%\s*if\s+(.*?)\s*%>").unwrap();
    converted = erb_conditional_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% if {} %}}", &caps[1])
    }).to_string();
    
    converted = converted.replace("<% else %>", "{% else %}");
    converted = converted.replace("<% end %>", "{% endif %}");
    
    // Bridgetown may use more Ruby-like syntax for loops
    let erb_loop_regex = regex::Regex::new(r"<%\s*(.+?)\.each\s+do\s+\|\s*(.+?)\s*\|\s*%>").unwrap();
    converted = erb_loop_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% for {} in {} %}}", &caps[2], &caps[1])
    }).to_string();
    
    converted = converted.replace("<% end %>", "{% endfor %}");
    
    // Handle assets helper
    converted = converted.replace("src=\"<%= asset_path", "src=\"{{ '/assets");
    converted = converted.replace("href=\"<%= asset_path", "href=\"{{ '/assets");
    converted = converted.replace("%>\"", "' | relative_url }}\"");
    
    converted
}

fn create_default_layout(
    layouts_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a default layout
    let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
    <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
    <header class="site-header">
        <div class="wrapper">
            <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title }}</a>
            <nav class="site-nav">
                <a href="{{ '/about/' | relative_url }}">About</a>
                <a href="{{ '/blog/' | relative_url }}">Blog</a>
            </nav>
        </div>
    </header>

    <main class="site-content">
        <div class="wrapper">
            {{ content }}
        </div>
    </main>

    <footer class="site-footer">
        <div class="wrapper">
            <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
        </div>
    </footer>
</body>
</html>"#;

    fs::write(layouts_dir.join("default.html"), default_layout)
        .map_err(|e| format!("Failed to write default layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".into(),
        description: "Created default layout for Bridgetown site".into(),
    });

    Ok(())
} 