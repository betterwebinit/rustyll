use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_layouts(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy layouts...");
    }

    // Create layouts directory
    let layouts_dir = dest_dir.join("_layouts");
    fs::create_dir_all(&layouts_dir)
        .map_err(|e| format!("Failed to create layouts directory: {}", e))?;

    // Check for layouts in themes/docsy/layouts or layouts directories
    let theme_layouts_dir = source_dir.join("themes/docsy/layouts");
    let layouts_source_dir = source_dir.join("layouts");

    // First check user layouts directory (these override theme layouts)
    if layouts_source_dir.exists() {
        migrate_layout_files(&layouts_source_dir, &layouts_dir, verbose, result)?;
    }

    // Then check theme layouts
    if theme_layouts_dir.exists() {
        migrate_layout_files(&theme_layouts_dir, &layouts_dir, verbose, result)?;
    } else {
        // Create default layouts if no layouts directory found
        create_default_layouts(&layouts_dir, result)?;
    }

    Ok(())
}

fn migrate_layout_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Only process direct HTML files in the layouts directory and _default subdirectory
    // (Hugo has a complex layout system, but we'll simplify for Jekyll)
    
    // First, check the _default directory which contains the base layouts
    let default_dir = source_dir.join("_default");
    if default_dir.exists() {
        for entry in fs::read_dir(&default_dir).map_err(|e| format!("Failed to read _default directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                if extension == "html" {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let dest_file = match file_name.as_str() {
                        "baseof.html" => "default.html",
                        "list.html" => "list.html",
                        "single.html" => "page.html",
                        _ => &file_name,
                    };
                    
                    migrate_layout_file(&path, &dest_dir.join(dest_file), verbose, result)?;
                }
            }
        }
    }
    
    // Then, check the root layouts directory for specific page types
    for entry in fs::read_dir(source_dir).map_err(|e| format!("Failed to read layouts directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "html" {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                migrate_layout_file(&path, &dest_dir.join(&file_name), verbose, result)?;
            }
        }
    }
    
    Ok(())
}

fn migrate_layout_file(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read layout file: {}", e))?;

    // Convert Hugo/Docsy go templates to Jekyll/Liquid syntax
    let converted_content = convert_docsy_layout(&content)?;

    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write layout file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("_layouts/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted layout from {}", source_path.display()),
    });

    Ok(())
}

fn convert_docsy_layout(content: &str) -> Result<String, String> {
    // Convert Hugo/Docsy Go template syntax to Jekyll/Liquid syntax
    let mut converted = content.to_string();
    
    // Basic Go template to Liquid conversions
    converted = converted.replace("{{ .Title }}", "{{ page.title }}");
    converted = converted.replace("{{ .Content }}", "{{ content }}");
    converted = converted.replace("{{ .Description }}", "{{ page.description }}");
    converted = converted.replace("{{ .Site.Title }}", "{{ site.title }}");
    converted = converted.replace("{{ .Site.Params.description }}", "{{ site.description }}");
    
    // Convert if statements
    // Go: {{ if condition }}content{{ end }}
    // Liquid: {% if condition %}content{% endif %}
    let mut i = 0;
    while let Some(start_pos) = converted[i..].find("{{ if ") {
        let start_pos = i + start_pos;
        if let Some(end_if) = converted[start_pos..].find(" }}") {
            let end_if = start_pos + end_if;
            let condition = &converted[start_pos + 6..end_if];
            
            // Convert the condition to Liquid syntax
            let liquid_condition = convert_condition(condition);
            
            // Replace with Liquid if statement
            converted = format!(
                "{}{{{{ if {} }}}}{}",
                &converted[..start_pos],
                liquid_condition,
                &converted[end_if + 3..]
            );
            
            // Find the corresponding end tag and replace it
            if let Some(end_tag_pos) = converted[start_pos..].find("{{ end }}") {
                let end_tag_pos = start_pos + end_tag_pos;
                converted = format!(
                    "{}{{{{ endif }}}}{}",
                    &converted[..end_tag_pos],
                    &converted[end_tag_pos + 9..]
                );
            }
        }
        
        i = start_pos + 1;
    }
    
    // Convert range statements
    // Go: {{ range .Pages }}content{{ end }}
    // Liquid: {% for page in pages %}content{% endfor %}
    i = 0;
    while let Some(start_pos) = converted[i..].find("{{ range ") {
        let start_pos = i + start_pos;
        if let Some(end_range) = converted[start_pos..].find(" }}") {
            let end_range = start_pos + end_range;
            let collection = &converted[start_pos + 9..end_range];
            
            // Convert the collection to Liquid syntax
            let (item_name, liquid_collection) = convert_collection(collection);
            
            // Replace with Liquid for loop
            converted = format!(
                "{}{{{{ for {} in {} }}}}{}",
                &converted[..start_pos],
                item_name,
                liquid_collection,
                &converted[end_range + 3..]
            );
            
            // Find the corresponding end tag and replace it
            if let Some(end_tag_pos) = converted[start_pos..].find("{{ end }}") {
                let end_tag_pos = start_pos + end_tag_pos;
                converted = format!(
                    "{}{{{{ endfor }}}}{}",
                    &converted[..end_tag_pos],
                    &converted[end_tag_pos + 9..]
                );
            }
        }
        
        i = start_pos + 1;
    }
    
    // Convert partial includes
    // Go: {{ partial "header.html" . }}
    // Liquid: {% include header.html %}
    i = 0;
    while let Some(start_pos) = converted[i..].find("{{ partial ") {
        let start_pos = i + start_pos;
        if let Some(end_partial) = converted[start_pos..].find(" }}") {
            let end_partial = start_pos + end_partial;
            let partial_content = &converted[start_pos + 11..end_partial];
            
            // Extract the partial name (quoted string)
            if let Some(start_quote) = partial_content.find('"') {
                if let Some(end_quote) = partial_content[start_quote + 1..].find('"') {
                    let partial_name = &partial_content[start_quote + 1..start_quote + 1 + end_quote];
                    
                    // Replace with Liquid include
                    converted = format!(
                        "{}{{{{ include {} }}}}{}",
                        &converted[..start_pos],
                        partial_name,
                        &converted[end_partial + 3..]
                    );
                }
            }
        }
        
        i = start_pos + 1;
    }
    
    Ok(converted)
}

fn convert_condition(condition: &str) -> String {
    // Convert Go template conditions to Liquid syntax
    // This is a simplified version, might need expansion for complex conditions
    match condition {
        ".Title" => "page.title",
        ".Description" => "page.description",
        ".IsHome" => "page.url == '/'",
        _ => condition,
    }.to_string()
}

fn convert_collection(collection: &str) -> (String, String) {
    // Convert Go template collections to Liquid syntax
    // Returns a tuple of (item_name, collection)
    match collection {
        ".Pages" => ("page".to_string(), "site.pages".to_string()),
        ".Site.Pages" => ("page".to_string(), "site.pages".to_string()),
        ".Site.RegularPages" => ("page".to_string(), "site.pages".to_string()),
        _ => ("item".to_string(), "collection".to_string()),
    }
}

fn create_default_layouts(
    layouts_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default layout
    let default_layout = r#"<!DOCTYPE html>
<html lang="{{ site.lang | default: "en-US" }}">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} | {% endif %}{{ site.title }}</title>
  <meta name="description" content="{{ page.description | default: site.description }}">
  <link rel="stylesheet" href="{{ "/assets/css/main.css" | relative_url }}">
</head>
<body>
  <div class="td-container">
    <header>
      {% include header.html %}
    </header>
    
    <main>
      <div class="td-content">
        {{ content }}
      </div>
    </main>
    
    <footer>
      {% include footer.html %}
    </footer>
  </div>
</body>
</html>"#;

    fs::write(layouts_dir.join("default.html"), default_layout)
        .map_err(|e| format!("Failed to write default layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/default.html".into(),
        description: "Created default layout".into(),
    });

    // Create page layout
    let page_layout = r#"---
layout: default
---
<div class="td-page">
  <div class="td-page-header">
    <h1 class="title">{{ page.title }}</h1>
    {% if page.description %}
    <div class="lead">{{ page.description }}</div>
    {% endif %}
  </div>
  
  <div class="td-page-content">
    {{ content }}
  </div>
</div>"#;

    fs::write(layouts_dir.join("page.html"), page_layout)
        .map_err(|e| format!("Failed to write page layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/page.html".into(),
        description: "Created page layout".into(),
    });

    // Create list layout
    let list_layout = r#"---
layout: default
---
<div class="td-list">
  <div class="td-list-header">
    <h1 class="title">{{ page.title }}</h1>
    {% if page.description %}
    <div class="lead">{{ page.description }}</div>
    {% endif %}
  </div>
  
  <div class="td-list-content">
    {{ content }}
    
    <div class="section-index">
      {% assign pages = site.pages | where_exp: "page", "page.url contains page.dir" | sort: "weight" %}
      {% for child in pages %}
        {% if child.url != page.url %}
        <div class="entry">
          <h5>
            <a href="{{ child.url | relative_url }}">{{ child.title }}</a>
          </h5>
          <p>{{ child.description }}</p>
        </div>
        {% endif %}
      {% endfor %}
    </div>
  </div>
</div>"#;

    fs::write(layouts_dir.join("list.html"), list_layout)
        .map_err(|e| format!("Failed to write list layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/list.html".into(),
        description: "Created list layout".into(),
    });

    // Create home layout
    let home_layout = r#"---
layout: default
---
<div class="td-home">
  <section class="td-hero">
    <div class="td-hero-content">
      <h1>{{ page.title }}</h1>
      {% if page.description %}
      <div class="lead">{{ page.description }}</div>
      {% endif %}
      
      {% if page.action_buttons %}
      <div class="action-buttons">
        {% for button in page.action_buttons %}
        <a href="{{ button.url | relative_url }}" class="btn">{{ button.title }}</a>
        {% endfor %}
      </div>
      {% endif %}
    </div>
  </section>
  
  <div class="td-home-content">
    {{ content }}
  </div>
</div>"#;

    fs::write(layouts_dir.join("home.html"), home_layout)
        .map_err(|e| format!("Failed to write home layout: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_layouts/home.html".into(),
        description: "Created home layout".into(),
    });

    Ok(())
} 