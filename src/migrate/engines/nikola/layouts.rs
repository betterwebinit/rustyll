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
        log::info!("Migrating Nikola layouts...");
    }

    // Create destination layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;
    
    // Create destination includes directory
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // In Nikola, templates are typically in templates/ directory
    let templates_dir = source_dir.join("templates");
    if templates_dir.exists() && templates_dir.is_dir() {
        migrate_templates(&templates_dir, &dest_layouts_dir, &dest_includes_dir, result)?;
    } else {
        // Template directory doesn't exist, create default layouts
        create_default_layouts(&dest_layouts_dir, &dest_includes_dir, result)?;
    }

    Ok(())
}

fn migrate_templates(
    templates_dir: &Path,
    dest_layouts_dir: &Path,
    dest_includes_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process all template files
    for entry in WalkDir::new(templates_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process template files
            if is_template_file(file_path) {
                // Determine if it's a layout or include
                let filename = file_path.file_name().unwrap().to_string_lossy();
                let is_layout = is_layout_template(&filename);
                
                if is_layout {
                    migrate_layout_file(file_path, templates_dir, dest_layouts_dir, result)?;
                } else {
                    migrate_include_file(file_path, templates_dir, dest_includes_dir, result)?;
                }
            }
        }
    }
    
    // Make sure we have all essential layouts
    ensure_essential_layouts(dest_layouts_dir, result)?;
    
    Ok(())
}

fn migrate_layout_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the template file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read template file {}: {}", file_path.display(), e))?;
    
    // Convert the template to Jekyll/Liquid format
    let converted_content = convert_template_to_liquid(&content);
    
    // Determine destination file name
    let filename = file_path.file_name().unwrap().to_string_lossy();
    let dest_filename = if filename.ends_with(".tmpl") || filename.ends_with(".html") {
        // Standardize on .html extension
        if let Some(name) = filename.split('.').next() {
            format!("{}.html", name)
        } else {
            "default.html".to_string()
        }
    } else {
        // Add .html extension if missing
        format!("{}.html", filename)
    };
    
    let dest_path = dest_dir.join(dest_filename);
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write layout file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Nikola layout from {}", file_path.display()),
    });
    
    Ok(())
}

fn migrate_include_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the template file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read template file {}: {}", file_path.display(), e))?;
    
    // Convert the template to Jekyll/Liquid format
    let converted_content = convert_template_to_liquid(&content);
    
    // Determine destination file name (keep relative structure)
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Ensure it has a .html extension
    let mut dest_path = dest_dir.join(rel_path);
    if let Some(ext) = dest_path.extension() {
        if ext != "html" {
            let stem = dest_path.file_stem().unwrap();
            dest_path = dest_path.with_file_name(format!("{}.html", stem.to_string_lossy()));
        }
    } else {
        dest_path = dest_path.with_extension("html");
    }
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Nikola include from {}", file_path.display()),
    });
    
    Ok(())
}

fn create_default_layouts(
    layouts_dir: &Path,
    includes_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default layout
    let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} | {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
  {% include header.html %}
  
  <main>
    <div class="container">
      {{ content }}
    </div>
  </main>
  
  {% include footer.html %}
</body>
</html>"#;

    let default_path = layouts_dir.join("default.html");
    fs::write(&default_path, default_layout)
        .map_err(|e| format!("Failed to create default layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".to_string(),
        description: "Created default layout".to_string(),
    });

    // Create post layout
    let post_layout = r#"---
layout: default
---
<article class="post">
  <h1>{{ page.title }}</h1>
  
  <div class="post-meta">
    <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%B %-d, %Y" }}</time>
    {% if page.author %} • {{ page.author }}{% endif %}
  </div>
  
  <div class="post-content">
    {{ content }}
  </div>
  
  {% if page.tags.size > 0 %}
  <div class="post-tags">
    <h4>Tags:</h4>
    <ul>
      {% for tag in page.tags %}
      <li><a href="{{ site.baseurl }}/tags/{{ tag | slugify }}/">{{ tag }}</a></li>
      {% endfor %}
    </ul>
  </div>
  {% endif %}
</article>"#;

    let post_path = layouts_dir.join("post.html");
    fs::write(&post_path, post_layout)
        .map_err(|e| format!("Failed to create post layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/post.html".to_string(),
        description: "Created post layout".to_string(),
    });

    // Create page layout
    let page_layout = r#"---
layout: default
---
<article class="page">
  <h1>{{ page.title }}</h1>
  
  <div class="page-content">
    {{ content }}
  </div>
</article>"#;

    let page_path = layouts_dir.join("page.html");
    fs::write(&page_path, page_layout)
        .map_err(|e| format!("Failed to create page layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/page.html".to_string(),
        description: "Created page layout".to_string(),
    });

    // Create header include
    let header_include = r#"<header>
  <div class="container">
    <h1><a href="{{ '/' | relative_url }}">{{ site.title }}</a></h1>
    <nav>
      <ul>
        <li><a href="{{ '/' | relative_url }}">Home</a></li>
        <li><a href="{{ '/about/' | relative_url }}">About</a></li>
      </ul>
    </nav>
  </div>
</header>"#;

    let header_path = includes_dir.join("header.html");
    fs::write(&header_path, header_include)
        .map_err(|e| format!("Failed to create header include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/header.html".to_string(),
        description: "Created header include".to_string(),
    });

    // Create footer include
    let footer_include = r#"<footer>
  <div class="container">
    <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
  </div>
</footer>"#;

    let footer_path = includes_dir.join("footer.html");
    fs::write(&footer_path, footer_include)
        .map_err(|e| format!("Failed to create footer include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/footer.html".to_string(),
        description: "Created footer include".to_string(),
    });

    Ok(())
}

fn ensure_essential_layouts(
    layouts_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // List of essential layouts and their content if missing
    let essential_layouts = [
        ("default.html", None),  // No default content here, we'll check if it exists
        ("post.html", Some(r#"---
layout: default
---
<article class="post">
  <h1>{{ page.title }}</h1>
  
  <div class="post-meta">
    <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%B %-d, %Y" }}</time>
    {% if page.author %} • {{ page.author }}{% endif %}
  </div>
  
  <div class="post-content">
    {{ content }}
  </div>
  
  {% if page.tags.size > 0 %}
  <div class="post-tags">
    <h4>Tags:</h4>
    <ul>
      {% for tag in page.tags %}
      <li><a href="{{ site.baseurl }}/tags/{{ tag | slugify }}/">{{ tag }}</a></li>
      {% endfor %}
    </ul>
  </div>
  {% endif %}
</article>"#)),
        ("page.html", Some(r#"---
layout: default
---
<article class="page">
  <h1>{{ page.title }}</h1>
  
  <div class="page-content">
    {{ content }}
  </div>
</article>"#)),
    ];
    
    for (layout_name, default_content) in essential_layouts {
        let layout_path = layouts_dir.join(layout_name);
        
        if !layout_path.exists() {
            if let Some(content) = default_content {
                fs::write(&layout_path, content)
                    .map_err(|e| format!("Failed to create {} layout: {}", layout_name, e))?;
                
                result.changes.push(MigrationChange {
                    change_type: ChangeType::Created,
                    file_path: format!("_layouts/{}", layout_name),
                    description: format!("Created {} layout", layout_name),
                });
            } else {
                // If default.html is missing but we don't have content for it,
                // that's a problem as it's the base layout
                return Err(format!("Essential layout {} is missing and couldn't be created", layout_name));
            }
        }
    }
    
    Ok(())
}

fn is_template_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "tmpl" | "html" | "jinja" | "j2" | "mako")
    } else {
        false
    }
}

fn is_layout_template(filename: &str) -> bool {
    // Common layout filenames in Nikola
    let layout_files = [
        "base.tmpl", "base.html", 
        "post.tmpl", "post.html", 
        "page.tmpl", "page.html", 
        "index.tmpl", "index.html",
        "tag.tmpl", "tag.html",
        "author.tmpl", "author.html",
        "archive.tmpl", "archive.html",
        "gallery.tmpl", "gallery.html"
    ];
    
    layout_files.contains(&filename)
}

fn convert_template_to_liquid(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Convert Jinja2/Mako/Nunjucks template syntax to Liquid
    
    // Convert variable syntax: {{ variable }} is the same in Liquid
    
    // Convert for loops
    let for_regex = regex::Regex::new(r"\{%\s*for\s+(\w+)\s+in\s+([^%]+)\s*%\}").unwrap();
    converted = for_regex.replace_all(&converted, "{{% for $1 in $2 %}}").to_string();
    
    // Convert if statements
    let if_regex = regex::Regex::new(r"\{%\s*if\s+([^%]+)\s*%\}").unwrap();
    converted = if_regex.replace_all(&converted, "{{% if $1 %}}").to_string();
    
    // Convert Nikola-specific template variables to Jekyll equivalents
    converted = converted.replace("{{ blog_title }}", "{{ site.title }}");
    converted = converted.replace("{{ blog_description }}", "{{ site.description }}");
    converted = converted.replace("{{ blog_url }}", "{{ site.url }}");
    converted = converted.replace("{{ post.title }}", "{{ page.title }}");
    converted = converted.replace("{{ post.text }}", "{{ content }}");
    converted = converted.replace("{{ post.date }}", "{{ page.date | date: '%B %-d, %Y' }}");
    
    // Convert URLs and links
    converted = converted.replace("{{ permalink }}", "{{ page.url | absolute_url }}");
    
    // Add Jekyll header comment
    converted = format!("{{% comment %}}\nConverted from Nikola template\n{{% endcomment %}}\n\n{}", converted);
    
    converted
} 