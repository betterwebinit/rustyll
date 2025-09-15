use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_partials(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy partials...");
    }

    // Create includes directory
    let includes_dir = dest_dir.join("_includes");
    fs::create_dir_all(&includes_dir)
        .map_err(|e| format!("Failed to create includes directory: {}", e))?;

    // Look for partials in theme or user override directories
    let theme_partials_dir = source_dir.join("themes/docsy/layouts/partials");
    let user_partials_dir = source_dir.join("layouts/partials");

    // User partials first (overrides)
    if user_partials_dir.exists() {
        migrate_partial_files(&user_partials_dir, &includes_dir, verbose, result)?;
    }

    // Then theme partials
    if theme_partials_dir.exists() {
        migrate_partial_files(&theme_partials_dir, &includes_dir, verbose, result)?;
    } else {
        // Create default partials if none found
        create_default_partials(&includes_dir, result)?;
    }

    Ok(())
}

fn migrate_partial_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if extension == "html" {
                let relative_path = path.strip_prefix(source_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_path = dest_dir.join(relative_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                migrate_partial_file(path, &dest_path, verbose, result)?;
            }
        }
    }

    Ok(())
}

fn migrate_partial_file(
    source_path: &Path,
    dest_path: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read partial file: {}", e))?;

    // Convert Hugo/Docsy go templates to Jekyll/Liquid syntax
    let converted_content = convert_docsy_partial(&content)?;

    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write partial file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("_includes/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted partial from {}", source_path.display()),
    });

    Ok(())
}

fn convert_docsy_partial(content: &str) -> Result<String, String> {
    // Convert Hugo/Docsy template syntax to Jekyll/Liquid syntax
    let mut converted = content.to_string();
    
    // Basic variable replacements
    converted = converted.replace("{{ .Site.Title }}", "{{ site.title }}");
    converted = converted.replace("{{ .Site.Params.description }}", "{{ site.description }}");
    converted = converted.replace("{{ .Site.BaseURL }}", "{{ site.baseurl }}");
    
    // Convert context variable references (common in partials)
    converted = converted.replace("{{ .Title }}", "{{ include.title | default: page.title }}");
    converted = converted.replace("{{ .Content }}", "{{ include.content | default: content }}");
    converted = converted.replace("{{ .Description }}", "{{ include.description | default: page.description }}");
    
    // Convert if statements
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
    
    // Convert nested partials
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
    // Similar logic as in layouts.rs but tailored to partials
    let result = match condition {
        ".Title" => "include.title | default: page.title",
        ".Description" => "include.description | default: page.description",
        ".IsHome" => "page.url == '/'",
        ".Site.Params.ui.navbar_logo" => "site.logo",
        ".Site.Params.github_repo" => "site.github_repo",
        _ => &condition.replace(".", ""),
    };
    result.to_string()
}

fn convert_collection(collection: &str) -> (String, String) {
    // Similar logic as in layouts.rs but tailored to partials
    match collection {
        ".Site.Menus.main" => ("item".to_string(), "site.data.menu.main".to_string()),
        ".Site.Params.links.developer" => ("link".to_string(), "site.data.links.developer".to_string()),
        ".Site.Params.links.user" => ("link".to_string(), "site.data.links.user".to_string()),
        _ => ("item".to_string(), "collection".to_string()),
    }
}

fn create_default_partials(
    includes_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a default header partial
    let header_content = r#"<nav class="js-navbar-scroll navbar navbar-expand navbar-dark flex-column flex-md-row td-navbar">
  <a class="navbar-brand" href="{{ site.baseurl }}/">
    <span class="navbar-logo">{% if site.logo %}<img src="{{ site.logo | relative_url }}" alt="{{ site.title }}">{% else %}{{ site.title }}{% endif %}</span>
  </a>
  <div class="td-navbar-nav-scroll ml-md-auto" id="main_navbar">
    <ul class="navbar-nav mt-2 mt-lg-0">
      {% for link in site.data.menu.main %}
      <li class="nav-item mr-4 mb-2 mb-lg-0">
        <a class="nav-link" href="{{ link.url | relative_url }}">{{ link.name }}</a>
      </li>
      {% endfor %}
      {% if site.github_repo %}
      <li class="nav-item mr-4 mb-2 mb-lg-0">
        <a class="nav-link" href="{{ site.github_repo }}" target="_blank"><span>GitHub</span></a>
      </li>
      {% endif %}
    </ul>
  </div>
</nav>"#;

    fs::write(includes_dir.join("header.html"), header_content)
        .map_err(|e| format!("Failed to write header partial: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/header.html".into(),
        description: "Created default header partial".into(),
    });

    // Create a default footer partial
    let footer_content = r#"<footer class="td-footer row d-print-none">
  <div class="container-fluid">
    <div class="row">
      <div class="col-6 col-sm-4 text-xs-center order-sm-2">
        {% if site.links %}
        <ul class="td-footer-links">
          {% for link in site.links %}
          <li><a href="{{ link.url }}">{{ link.name }}</a></li>
          {% endfor %}
        </ul>
        {% endif %}
      </div>
      <div class="col-6 col-sm-4 text-right text-xs-center order-sm-3">
        <ul class="td-footer-social">
          {% if site.twitter %}
          <li><a href="{{ site.twitter }}" target="_blank"><i class="fab fa-twitter"></i></a></li>
          {% endif %}
          {% if site.github_repo %}
          <li><a href="{{ site.github_repo }}" target="_blank"><i class="fab fa-github"></i></a></li>
          {% endif %}
        </ul>
      </div>
      <div class="col-12 col-sm-4 text-center py-2 order-sm-1">
        <small>&copy; {{ 'now' | date: "%Y" }} {{ site.author }} {% if site.privacy_policy %}<a href="{{ site.privacy_policy }}">Privacy Policy</a>{% endif %}</small>
      </div>
    </div>
  </div>
</footer>"#;

    fs::write(includes_dir.join("footer.html"), footer_content)
        .map_err(|e| format!("Failed to write footer partial: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/footer.html".into(),
        description: "Created default footer partial".into(),
    });

    // Create a default sidebar partial
    let sidebar_content = r#"<div class="td-sidebar-nav">
  <nav class="collapse td-sidebar-nav__section" id="td-sidebar-nav">
    {% for section in site.data.sidebar %}
      <ul class="td-sidebar-nav__section-title">
        <li class="td-sidebar-nav__section-title">
          <a href="{% if section.url %}{{ section.url | relative_url }}{% else %}#{% endif %}" class="align-left pl-0 td-sidebar-link td-sidebar-link__section">{{ section.title }}</a>
        </li>
        {% if section.links %}
          <ul>
            {% for entry in section.links %}
              <li class="collapse show" id="{{ entry.title | slugify }}">
                <a class="td-sidebar-link td-sidebar-link__page" href="{{ entry.url | relative_url }}">{{ entry.title }}</a>
              </li>
            {% endfor %}
          </ul>
        {% endif %}
      </ul>
    {% endfor %}
  </nav>
</div>"#;

    fs::write(includes_dir.join("sidebar.html"), sidebar_content)
        .map_err(|e| format!("Failed to write sidebar partial: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/sidebar.html".into(),
        description: "Created default sidebar partial".into(),
    });

    Ok(())
} 