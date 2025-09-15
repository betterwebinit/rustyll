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
        log::info!("Migrating Octopress layouts...");
    }

    // Create destination directories
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;

    // In Octopress, layouts are in source/_layouts
    let layouts_dir = source_dir.join("source/_layouts");
    if layouts_dir.exists() && layouts_dir.is_dir() {
        migrate_layout_files(&layouts_dir, &dest_layouts_dir, result)?;
    } else {
        // Also check for _layouts directly
        let alt_layouts_dir = source_dir.join("_layouts");
        if alt_layouts_dir.exists() && alt_layouts_dir.is_dir() {
            migrate_layout_files(&alt_layouts_dir, &dest_layouts_dir, result)?;
        } else {
            // No layouts found, create default ones
            create_default_layouts(&dest_layouts_dir, result)?;
        }
    }

    Ok(())
}

fn migrate_layout_files(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Process all layout files
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Only process HTML files
            if is_layout_file(file_path) {
                migrate_layout_file(file_path, source_dir, dest_dir, result)?;
            }
        }
    }
    
    // Make sure all essential layouts exist
    ensure_essential_layouts(dest_dir, result)?;
    
    Ok(())
}

fn migrate_layout_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the layout file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
    
    // Convert the layout to Jekyll format
    let converted_content = convert_layout_content(&content);
    
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path
    let dest_path = dest_dir.join(rel_path);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Write the file
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write layout file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted Octopress layout from {}", file_path.display()),
    });
    
    Ok(())
}

fn create_default_layouts(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default.html
    let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} | {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
  <header class="site-header">
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
  </header>

  <main class="site-content">
    <div class="container">
      {{ content }}
    </div>
  </main>

  <footer class="site-footer">
    <div class="container">
      <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
    </div>
  </footer>
</body>
</html>"#;

    let default_path = dest_dir.join("default.html");
    fs::write(&default_path, default_layout)
        .map_err(|e| format!("Failed to create default layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".to_string(),
        description: "Created default layout".to_string(),
    });

    // Create post.html
    let post_layout = r#"---
layout: default
---
<article class="post">
  <header class="post-header">
    <h1 class="post-title">{{ page.title }}</h1>
    <div class="post-meta">
      <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%B %-d, %Y" }}</time>
      {% if page.author %} • {{ page.author }}{% endif %}
    </div>
  </header>

  <div class="post-content">
    {{ content }}
  </div>

  {% if page.categories or page.tags %}
  <footer class="post-footer">
    {% if page.categories %}
    <div class="post-categories">
      <h4>Categories:</h4>
      <ul>
        {% for category in page.categories %}
        <li><a href="{{ site.baseurl }}/categories/{{ category | slugify }}/">{{ category }}</a></li>
        {% endfor %}
      </ul>
    </div>
    {% endif %}

    {% if page.tags %}
    <div class="post-tags">
      <h4>Tags:</h4>
      <ul>
        {% for tag in page.tags %}
        <li><a href="{{ site.baseurl }}/tags/{{ tag | slugify }}/">{{ tag }}</a></li>
        {% endfor %}
      </ul>
    </div>
    {% endif %}
  </footer>
  {% endif %}
</article>"#;

    let post_path = dest_dir.join("post.html");
    fs::write(&post_path, post_layout)
        .map_err(|e| format!("Failed to create post layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/post.html".to_string(),
        description: "Created post layout".to_string(),
    });

    // Create page.html
    let page_layout = r#"---
layout: default
---
<article class="page">
  <header class="page-header">
    <h1 class="page-title">{{ page.title }}</h1>
  </header>

  <div class="page-content">
    {{ content }}
  </div>
</article>"#;

    let page_path = dest_dir.join("page.html");
    fs::write(&page_path, page_layout)
        .map_err(|e| format!("Failed to create page layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/page.html".to_string(),
        description: "Created page layout".to_string(),
    });

    Ok(())
}

fn ensure_essential_layouts(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Make sure the essential layouts exist: default, post, and page
    let essential_layouts = [
        ("default.html", false), // true if exists
        ("post.html", false),
        ("page.html", false),
    ];
    
    let mut missing_layouts = essential_layouts.to_vec();
    
    // Check if the layouts exist
    if let Ok(entries) = fs::read_dir(dest_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let filename = entry.file_name().to_string_lossy().to_string();
                
                for i in 0..missing_layouts.len() {
                    if missing_layouts[i].0 == filename {
                        // Mark as existing
                        missing_layouts[i].1 = true;
                        break;
                    }
                }
            }
        }
    }
    
    // Create any missing layouts
    for (layout_name, exists) in missing_layouts {
        if !exists {
            match layout_name {
                "default.html" => {
                    let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} | {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <link rel="stylesheet" href="{{ '/assets/css/main.css' | relative_url }}">
</head>
<body>
  <header class="site-header">
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
  </header>

  <main class="site-content">
    <div class="container">
      {{ content }}
    </div>
  </main>

  <footer class="site-footer">
    <div class="container">
      <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
    </div>
  </footer>
</body>
</html>"#;

                    let default_path = dest_dir.join("default.html");
                    fs::write(&default_path, default_layout)
                        .map_err(|e| format!("Failed to create default layout: {}", e))?;
                    
                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Created,
                        file_path: "_layouts/default.html".to_string(),
                        description: "Created default layout".to_string(),
                    });
                },
                "post.html" => {
                    let post_layout = r#"---
layout: default
---
<article class="post">
  <header class="post-header">
    <h1 class="post-title">{{ page.title }}</h1>
    <div class="post-meta">
      <time datetime="{{ page.date | date_to_xmlschema }}">{{ page.date | date: "%B %-d, %Y" }}</time>
      {% if page.author %} • {{ page.author }}{% endif %}
    </div>
  </header>

  <div class="post-content">
    {{ content }}
  </div>

  {% if page.categories or page.tags %}
  <footer class="post-footer">
    {% if page.categories %}
    <div class="post-categories">
      <h4>Categories:</h4>
      <ul>
        {% for category in page.categories %}
        <li><a href="{{ site.baseurl }}/categories/{{ category | slugify }}/">{{ category }}</a></li>
        {% endfor %}
      </ul>
    </div>
    {% endif %}

    {% if page.tags %}
    <div class="post-tags">
      <h4>Tags:</h4>
      <ul>
        {% for tag in page.tags %}
        <li><a href="{{ site.baseurl }}/tags/{{ tag | slugify }}/">{{ tag }}</a></li>
        {% endfor %}
      </ul>
    </div>
    {% endif %}
  </footer>
  {% endif %}
</article>"#;

                    let post_path = dest_dir.join("post.html");
                    fs::write(&post_path, post_layout)
                        .map_err(|e| format!("Failed to create post layout: {}", e))?;
                    
                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Created,
                        file_path: "_layouts/post.html".to_string(),
                        description: "Created post layout".to_string(),
                    });
                },
                "page.html" => {
                    let page_layout = r#"---
layout: default
---
<article class="page">
  <header class="page-header">
    <h1 class="page-title">{{ page.title }}</h1>
  </header>

  <div class="page-content">
    {{ content }}
  </div>
</article>"#;

                    let page_path = dest_dir.join("page.html");
                    fs::write(&page_path, page_layout)
                        .map_err(|e| format!("Failed to create page layout: {}", e))?;
                    
                    result.changes.push(MigrationChange {
                        change_type: ChangeType::Created,
                        file_path: "_layouts/page.html".to_string(),
                        description: "Created page layout".to_string(),
                    });
                },
                _ => {}
            }
        }
    }
    
    Ok(())
}

fn convert_layout_content(content: &str) -> String {
    // Convert Octopress-specific syntax to Jekyll
    let mut converted = content.to_string();
    
    // Replace Octopress-specific includes and constructs
    
    // Replace {% include custom/head.html %} with {% include head.html %}
    let custom_include_regex = regex::Regex::new(r#"\{%\s*include\s+custom/([^%]+)%\}"#).unwrap();
    converted = custom_include_regex.replace_all(&converted, "{{% include $1 %}}").to_string();
    
    // Replace {% include_array %}
    let include_array_regex = regex::Regex::new(r#"\{%\s*include_array\s+([^%]+)%\}"#).unwrap();
    converted = include_array_regex.replace_all(&converted, |caps: &regex::Captures| {
        // Get the array name
        let array_name = caps[1].trim();
        
        // Create a for loop to include each item
        format!(r#"{{% for item in site.data.{} %}}
  {{% include {{{{ item }}}} %}}
{{% endfor %}}"#, array_name)
    }).to_string();
    
    // Replace other Octopress-specific constructs
    
    // Add a comment to indicate conversion
    converted = format!(r#"{{% comment %}}
Converted from Octopress layout
{{% endcomment %}}

{}"#, converted);
    
    converted
}

fn is_layout_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "html" | "htm")
    } else {
        false
    }
} 