use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_themes(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Pelican themes...");
    }

    // Create destination directories
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;
    
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // In Pelican, themes can be in a theme directory or in the project itself
    let theme_dirs = find_theme_directories(source_dir);
    
    if theme_dirs.is_empty() {
        result.warnings.push("No theme directories found in Pelican source.".into());
        return Ok(());
    }

    // Process each theme directory
    for theme_dir in theme_dirs {
        // Process templates
        let templates_dir = theme_dir.join("templates");
        if templates_dir.exists() {
            migrate_templates(&templates_dir, &dest_layouts_dir, &dest_includes_dir, result)?;
        }
        
        // Process static files
        let static_dir = theme_dir.join("static");
        if static_dir.exists() {
            // These will be handled by the static_files module
            if verbose {
                log::info!("Found static directory in theme: {}", static_dir.display());
            }
        }
    }

    Ok(())
}

fn find_theme_directories(source_dir: &Path) -> Vec<PathBuf> {
    let mut theme_dirs = Vec::new();
    
    // Check for theme directory in the source root
    let theme_dir = source_dir.join("theme");
    if theme_dir.exists() && theme_dir.is_dir() {
        theme_dirs.push(theme_dir);
    }
    
    // Check for themes directory in the source root
    let themes_dir = source_dir.join("themes");
    if themes_dir.exists() && themes_dir.is_dir() {
        // Add each subdirectory in themes/
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        theme_dirs.push(path);
                    }
                }
            }
        }
    }
    
    // Check if pelicanconf.py specifies a theme
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(config_content) = fs::read_to_string(&pelicanconf_path) {
            // Look for THEME = "theme_name" in the config
            let theme_regex = regex::Regex::new(r#"THEME\s*=\s*["']([^"']+)["']"#).unwrap();
            if let Some(captures) = theme_regex.captures(&config_content) {
                let theme_name = &captures[1];
                
                // Check if it's an absolute path
                let theme_path = PathBuf::from(theme_name);
                if theme_path.is_absolute() && theme_path.exists() {
                    theme_dirs.push(theme_path);
                } else {
                    // Check if it's in the themes directory
                    let in_themes_dir = themes_dir.join(theme_name);
                    if in_themes_dir.exists() {
                        theme_dirs.push(in_themes_dir);
                    }
                }
            }
        }
    }
    
    theme_dirs
}

fn migrate_templates(
    templates_dir: &Path,
    dest_layouts_dir: &Path,
    dest_includes_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(templates_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Skip non-template files
            if !is_template_file(file_path) {
                continue;
            }
            
            // Check if it's a layout or include
            let rel_path = file_path.strip_prefix(templates_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            let filename = file_path.file_name().unwrap().to_string_lossy();
            let is_layout = is_layout_template(&filename, rel_path);
            
            // Process the template file
            let content = fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read template file {}: {}", file_path.display(), e))?;
            
            let converted_content = convert_jinja2_to_liquid(&content);
            
            // Determine destination
            let dest_path = if is_layout {
                let mut layout_name = filename.to_string();
                
                // Add .html extension if needed
                if !layout_name.ends_with(".html") {
                    layout_name = format!("{}.html", layout_name);
                }
                
                dest_layouts_dir.join(layout_name)
            } else {
                // It's an include, preserve directory structure
                dest_includes_dir.join(rel_path)
            };
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Write the converted template
            fs::write(&dest_path, converted_content)
                .map_err(|e| format!("Failed to write template file {}: {}", dest_path.display(), e))?;
            
            // Add to changes
            let rel_dest_path = if is_layout {
                format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy())
            } else {
                format!("_includes/{}", rel_path.display())
            };
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Converted,
                file_path: rel_dest_path,
                description: format!("Converted Pelican template from {}", file_path.display()),
            });
        }
    }
    
    // Create default layouts if they don't exist
    ensure_default_layouts(dest_layouts_dir, result)?;
    
    Ok(())
}

fn is_template_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "html" | "jinja" | "jinja2")
    } else {
        false
    }
}

fn is_layout_template(filename: &str, rel_path: &Path) -> bool {
    // Common layout names in Pelican
    let layout_names = [
        "base.html", 
        "index.html", 
        "article.html", 
        "page.html", 
        "category.html", 
        "tag.html", 
        "archives.html"
    ];
    
    layout_names.contains(&filename) || 
        rel_path.to_string_lossy().starts_with("base") ||
        !rel_path.to_string_lossy().contains('/')  // Files in the root templates directory are often layouts
}

fn convert_jinja2_to_liquid(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Convert Jinja2 variable syntax to Liquid
    // {{ var }} is the same, no change needed
    
    // Convert Jinja2 comment syntax
    let comment_regex = regex::Regex::new(r"\{#\s*(.*?)\s*#\}").unwrap();
    converted = comment_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% comment %}}{}{{%  endcomment %}}", &caps[1])
    }).to_string();
    
    // Convert Jinja2 block statements
    let block_regex = regex::Regex::new(r"\{%\s*block\s+(\w+)\s*%\}").unwrap();
    converted = block_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% block {} %}}", &caps[1])
    }).to_string();
    
    let endblock_regex = regex::Regex::new(r"\{%\s*endblock\s*%\}").unwrap();
    converted = endblock_regex.replace_all(&converted, "{% endblock %}").to_string();
    
    // Convert Jinja2 extends
    let extends_regex = regex::Regex::new(r#"\{%\s*extends\s+["']([^"']+)["']"#).unwrap();
    converted = extends_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% extends '{}' ", &caps[1])
    }).to_string();
    
    // Convert Jinja2 includes
    let include_regex = regex::Regex::new(r#"\{%\s*include\s+["']([^"']+)["']"#).unwrap();
    converted = include_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% include '{}' ", &caps[1])
    }).to_string();
    
    // Convert Jinja2 if statements
    let if_regex = regex::Regex::new(r"\{%\s*if\s+(.*?)\s*%\}").unwrap();
    converted = if_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% if {} %}}", convert_jinja2_expression(&caps[1]))
    }).to_string();
    
    let elif_regex = regex::Regex::new(r"\{%\s*elif\s+(.*?)\s*%\}").unwrap();
    converted = elif_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% elsif {} %}}", convert_jinja2_expression(&caps[1]))
    }).to_string();
    
    // Convert Jinja2 for loops
    let for_regex = regex::Regex::new(r"\{%\s*for\s+(.*?)\s+in\s+(.*?)\s*%\}").unwrap();
    converted = for_regex.replace_all(&converted, |caps: &regex::Captures| {
        format!("{{% for {} in {} %}}", &caps[1], convert_jinja2_expression(&caps[2]))
    }).to_string();
    
    // Replace Pelican-specific variables with Jekyll equivalents
    converted = converted.replace("{{ SITEURL }}", "{{ site.url }}{{ site.baseurl }}");
    converted = converted.replace("{{ SITENAME }}", "{{ site.title }}");
    converted = converted.replace("{{ SITESUBTITLE }}", "{{ site.description }}");
    
    // Article/page variables
    converted = converted.replace("{{ article.title }}", "{{ page.title }}");
    converted = converted.replace("{{ article.content }}", "{{ content }}");
    converted = converted.replace("{{ page.content }}", "{{ content }}");
    
    // Convert article metadata
    converted = converted.replace("{{ article.date }}", "{{ page.date | date: '%B %d, %Y' }}");
    converted = converted.replace("{{ article.author }}", "{{ page.author }}");
    converted = converted.replace("{{ article.category }}", "{{ page.category }}");
    converted = converted.replace("{{ article.tags }}", "{% for tag in page.tags %}{{ tag }}{% unless forloop.last %}, {% endunless %}{% endfor %}");
    
    // Add a Jekyll comment at the top to indicate conversion
    converted = format!("{{% comment %}}\nConverted from Pelican Jinja2 template\n{{% endcomment %}}\n\n{}", converted);
    
    converted
}

fn convert_jinja2_expression(expr: &str) -> String {
    let mut converted = expr.to_string();
    
    // Replace Jinja2-specific operators with Liquid equivalents
    converted = converted.replace(" and ", " and ");  // Same in Liquid
    converted = converted.replace(" or ", " or ");    // Same in Liquid
    converted = converted.replace(" not ", " != ");   // Different in Liquid
    
    // Convert Jinja-style filters (e.g., var|filter)
    // Liquid uses the same filter syntax
    
    converted
}

fn ensure_default_layouts(
    dest_layouts_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Define the required Jekyll layouts
    let layouts = [
        ("default.html", r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <link rel="stylesheet" href="{{ site.baseurl }}/assets/css/style.css">
</head>
<body>
  <header>
    <div class="container">
      <h1><a href="{{ site.baseurl }}/">{{ site.title }}</a></h1>
      {% if site.description %}<p>{{ site.description }}</p>{% endif %}
    </div>
  </header>
  
  <main class="container">
    {{ content }}
  </main>
  
  <footer>
    <div class="container">
      <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
    </div>
  </footer>
</body>
</html>"#),
        
        ("post.html", r#"---
layout: default
---
<article class="post">
  <header>
    <h1>{{ page.title }}</h1>
    <p class="post-meta">
      <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%b %-d, %Y" }}</time>
      {% if page.author %} • {{ page.author }}{% endif %}
      {% if page.category %} • Category: <a href="{{ site.baseurl }}/category/{{ page.category | slugify }}/">{{ page.category }}</a>{% endif %}
    </p>
  </header>

  <div class="post-content">
    {{ content }}
  </div>

  {% if page.tags.size > 0 %}
  <footer class="post-tags">
    <p>Tags: 
      {% for tag in page.tags %}
        <a href="{{ site.baseurl }}/tag/{{ tag | slugify }}/">{{ tag }}</a>{% unless forloop.last %}, {% endunless %}
      {% endfor %}
    </p>
  </footer>
  {% endif %}
</article>"#),
        
        ("page.html", r#"---
layout: default
---
<article class="page">
  <header>
    <h1>{{ page.title }}</h1>
  </header>

  <div class="page-content">
    {{ content }}
  </div>
</article>"#),
    ];
    
    for (layout_name, content) in layouts {
        let layout_path = dest_layouts_dir.join(layout_name);
        
        // Only create if it doesn't exist already
        if !layout_path.exists() {
            fs::write(&layout_path, content)
                .map_err(|e| format!("Failed to create default layout {}: {}", layout_name, e))?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_layouts/{}", layout_name),
                description: format!("Created default Jekyll {} layout", layout_name),
            });
        }
    }
    
    Ok(())
} 