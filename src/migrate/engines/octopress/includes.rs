use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_includes(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress includes...");
    }

    // Create destination includes directory
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // In Octopress, includes are in source/_includes and source/_includes/custom
    let includes_dir = source_dir.join("source/_includes");
    if includes_dir.exists() && includes_dir.is_dir() {
        migrate_includes_dir(&includes_dir, &dest_includes_dir, result)?;
    }
    
    // Also check for _includes directly
    let alt_includes_dir = source_dir.join("_includes");
    if alt_includes_dir.exists() && alt_includes_dir.is_dir() {
        migrate_includes_dir(&alt_includes_dir, &dest_includes_dir, result)?;
    }

    // Create default includes if needed
    ensure_default_includes(&dest_includes_dir, result)?;

    Ok(())
}

fn migrate_includes_dir(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process all include files
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Process the include file
            migrate_include_file(file_path, source_dir, dest_dir, result)?;
        }
    }
    
    Ok(())
}

fn migrate_include_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the include file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read include file {}: {}", file_path.display(), e))?;
    
    // Convert the include to Jekyll format
    let converted_content = convert_include_content(&content);
    
    // Determine the destination path
    let dest_path = get_destination_path(file_path, source_dir, dest_dir)?;
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    // Get the include file path relative to _includes
    let rel_path = dest_path.strip_prefix(dest_dir)
        .map_err(|_| format!("Failed to get relative path for {}", dest_path.display()))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/{}", rel_path.display()),
        description: format!("Converted Octopress include from {}", file_path.display()),
    });
    
    Ok(())
}

fn get_destination_path(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<PathBuf, String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Check if it's in the custom/ directory - if so, flatten the structure
    let rel_path_str = rel_path.to_string_lossy();
    if rel_path_str.starts_with("custom/") {
        // Remove the "custom/" prefix to flatten the structure
        let new_name = rel_path_str.strip_prefix("custom/").unwrap();
        Ok(dest_dir.join(new_name))
    } else {
        // Use the original relative path
        Ok(dest_dir.join(rel_path))
    }
}

fn convert_include_content(content: &str) -> String {
    // Convert Octopress-specific syntax to Jekyll
    let mut converted = content.to_string();
    
    // Replace Octopress-specific Liquid tags
    
    // Replace {% img %} tags
    let img_regex = regex::Regex::new(r#"\{%\s*img\s+([^%]+)%\}"#).unwrap();
    converted = img_regex.replace_all(&converted, |caps: &regex::Captures| {
        let img_parts: Vec<&str> = caps[1].trim().split_whitespace().collect();
        
        if img_parts.len() >= 2 {
            // Basic format: path alt_text
            let path = img_parts[0];
            let alt = img_parts[1];
            
            format!(r#"<img src="{{ '{}' | relative_url }}" alt="{}">"#, path, alt)
        } else if !img_parts.is_empty() {
            // Just a path
            let path = img_parts[0];
            
            format!(r#"<img src="{{ '{}' | relative_url }}" alt="">"#, path)
        } else {
            // Something went wrong, keep the original
            caps[0].to_string()
        }
    }).to_string();
    
    // Replace {% render_partial %} with {% include %}
    let render_partial_regex = regex::Regex::new(r#"\{%\s*render_partial\s+["']([^"']+)["']\s*%\}"#).unwrap();
    converted = render_partial_regex.replace_all(&converted, "{{% include $1 %}}").to_string();
    
    // Add a comment to indicate conversion
    converted = format!(r#"{{% comment %}}
Converted from Octopress include
{{% endcomment %}}

{}"#, converted);
    
    converted
}

fn ensure_default_includes(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // List of default includes to create if they don't exist
    let default_includes = [
        ("head.html", r#"<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{% if page.title %}{{ page.title }} | {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
<meta name="description" content="{% if page.excerpt %}{{ page.excerpt | strip_html | strip_newlines | truncate: 160 }}{% else %}{{ site.description }}{% endif %}">
<link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
<link rel="canonical" href="{{ page.url | replace:'index.html','' | absolute_url }}">
<link rel="alternate" type="application/rss+xml" title="{{ site.title }}" href="{{ '/feed.xml' | relative_url }}">"#),
        
        ("header.html", r#"<header class="site-header">
  <div class="container">
    <h1 class="site-title"><a href="{{ '/' | relative_url }}">{{ site.title }}</a></h1>
    <nav class="site-nav">
      <ul>
        <li><a href="{{ '/' | relative_url }}">Home</a></li>
        {% for page in site.pages %}
          {% if page.title and page.title != 'Home' %}
            <li><a href="{{ page.url | relative_url }}">{{ page.title }}</a></li>
          {% endif %}
        {% endfor %}
      </ul>
    </nav>
  </div>
</header>"#),
        
        ("footer.html", r#"<footer class="site-footer">
  <div class="container">
    <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
    {% if site.description %}
    <p>{{ site.description }}</p>
    {% endif %}
  </div>
</footer>"#),
    ];
    
    for (filename, content) in default_includes {
        let file_path = dest_dir.join(filename);
        
        if !file_path.exists() {
            fs::write(&file_path, content)
                .map_err(|e| format!("Failed to create default include {}: {}", filename, e))?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_includes/{}", filename),
                description: format!("Created default {} include", filename),
            });
        }
    }
    
    Ok(())
} 