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
        log::info!("Migrating Slate layouts...");
    }

    // Create destination layouts directory
    let dest_layouts_dir = dest_dir.join("_layouts");
    create_dir_if_not_exists(&dest_layouts_dir)?;

    // In Slate, layouts are typically in source/layouts
    let source_layouts_dir = source_dir.join("source/layouts");
    if source_layouts_dir.exists() && source_layouts_dir.is_dir() {
        // Process layout files
        for entry in WalkDir::new(&source_layouts_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process layout files (typically .erb or .haml)
                if is_layout_file(file_path) {
                    migrate_layout_file(file_path, &source_layouts_dir, &dest_layouts_dir, result)?;
                }
            }
        }
    } else {
        // If no layouts directory exists, create default Jekyll layouts
        create_default_layouts(&dest_layouts_dir, result)?;
    }

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
    let converted_content = convert_slate_layout(&content);
    
    // Determine destination filename (with .html extension)
    let file_name = file_path.file_name().unwrap().to_string_lossy();
    let dest_name = if file_name == "layout.erb" || file_name == "layout.haml" {
        "default.html".to_string()
    } else {
        // Convert extension to .html
        let stem = file_path.file_stem().unwrap().to_string_lossy();
        format!("{}.html", stem)
    };
    
    // Write to destination
    let dest_path = dest_dir.join(dest_name);
    fs::write(&dest_path, converted_content)
        .map_err(|e| format!("Failed to write layout file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted layout from {}", file_path.display()),
    });
    
    Ok(())
}

fn convert_slate_layout(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Replace ERB/HAML-specific syntax with Liquid
    // <%= yield %> -> {{ content }}
    converted = converted.replace("<%= yield %>", "{{ content }}");
    
    // <%= current_page.data.title %> -> {{ page.title }}
    converted = converted.replace("<%= current_page.data.title %>", "{{ page.title }}");
    
    // Replace other ERB expressions
    let erb_regex = regex::Regex::new(r"<%=\s*(.*?)\s*%>").unwrap();
    converted = erb_regex.replace_all(&converted, |caps: &regex::Captures| {
        let expr = &caps[1];
        
        // Try to convert common patterns
        let expr = expr.replace("current_page.data.", "page.");
        let expr = expr.replace("config.", "site.");
        
        format!("{{ {} }}", expr)
    }).to_string();
    
    // Replace ERB control structures
    converted = converted.replace("<% if ", "{% if ");
    converted = converted.replace("<% else %>", "{% else %}");
    converted = converted.replace("<% end %>", "{% endif %}");
    
    // Add Jekyll front matter and doctype if missing
    if !converted.starts_with("<!DOCTYPE") && !converted.starts_with("<!doctype") {
        converted = format!("<!DOCTYPE html>\n{}", converted);
    }
    
    // Add header comment
    converted = format!("<!-- Converted from Slate layout -->\n{}", converted);
    
    converted
}

fn create_default_layouts(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Default layout
    let default_layout = r##"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta content="IE=edge,chrome=1" http-equiv="X-UA-Compatible">
    <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1">
    <title>{{ page.title }} | {{ site.title }}</title>
    
    <link rel="stylesheet" href="{{ site.baseurl }}/assets/css/style.css">
    <script src="{{ site.baseurl }}/assets/js/script.js"></script>
  </head>

  <body class="{% if page.language_tabs %}has-language-tabs{% endif %}">
    <header>
      <div class="logo">
        <a href="{{ site.baseurl }}/">{{ site.title }}</a>
      </div>
      <nav>
        {% for link in site.data.navigation %}
          <a href="{{ link.url | prepend: site.baseurl }}">{{ link.title }}</a>
        {% endfor %}
      </nav>
    </header>

    <div class="page-wrapper">
      {% if page.language_tabs %}
        <div class="lang-selector">
          {% for lang in page.language_tabs %}
            <a href="#" data-language-name="{{ lang }}">{{ lang }}</a>
          {% endfor %}
        </div>
      {% endif %}

      <div class="content">
        {{ content }}
      </div>

      {% if page.toc_footers %}
        <div class="toc-footer">
          <ul>
            {% for footer in page.toc_footers %}
              <li>{{ footer }}</li>
            {% endfor %}
          </ul>
        </div>
      {% endif %}
    </div>
  </body>
</html>"##;

    let default_path = dest_dir.join("default.html");
    fs::write(&default_path, default_layout)
        .map_err(|e| format!("Failed to create default layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".into(),
        description: "Created default layout".into(),
    });

    // Page layout
    let page_layout = r##"---
layout: default
---
<div class="page-content">
  {{ content }}
</div>"##;

    let page_path = dest_dir.join("page.html");
    fs::write(&page_path, page_layout)
        .map_err(|e| format!("Failed to create page layout: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/page.html".into(),
        description: "Created page layout".into(),
    });

    Ok(())
}

fn is_layout_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "erb" || ext_str == "haml" || ext_str == "html"
    } else {
        false
    }
} 