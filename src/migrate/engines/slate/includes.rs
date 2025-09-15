use std::path::Path;
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
        log::info!("Migrating Slate includes...");
    }

    // Create destination includes directory
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // In Slate, includes/partials are typically in source/includes
    let source_includes_dir = source_dir.join("source/includes");
    if source_includes_dir.exists() && source_includes_dir.is_dir() {
        // Process include files
        for entry in WalkDir::new(&source_includes_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process include files (.md, .erb, etc.)
                migrate_include_file(file_path, &source_includes_dir, &dest_includes_dir, result)?;
            }
        }
    } else {
        // If no includes directory exists, create some default includes
        create_default_includes(&dest_includes_dir, result)?;
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
    let converted_content = convert_slate_include(&content, file_path);
    
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path with .html extension
    let mut dest_path = dest_dir.join(rel_path);
    
    // Ensure it has .html extension for Jekyll includes
    let new_filename = if let Some(ext) = dest_path.extension() {
        let stem = dest_path.file_stem().unwrap().to_string_lossy();
        format!("{}.html", stem)
    } else {
        let filename = dest_path.file_name().unwrap().to_string_lossy();
        format!("{}.html", filename)
    };
    
    dest_path = dest_path.with_file_name(new_filename);
    
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
        description: format!("Converted include from {}", file_path.display()),
    });
    
    Ok(())
}

fn convert_slate_include(content: &str, file_path: &Path) -> String {
    let mut converted = content.to_string();
    
    // Check file extension
    if let Some(ext) = file_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        
        if ext_str == "md" || ext_str == "markdown" {
            // For markdown, keep the content mostly as-is
            // But add a comment at the top for traceability
            converted = format!("{{% comment %}}\nConverted from Slate include: {}\n{{% endcomment %}}\n\n{}", 
                file_path.file_name().unwrap().to_string_lossy(),
                converted);
        } else if ext_str == "erb" || ext_str == "html" {
            // Replace ERB/Middleman-specific syntax with Liquid
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
            
            // Add header comment
            converted = format!("<!-- Converted from Slate include: {} -->\n{}", 
                file_path.file_name().unwrap().to_string_lossy(),
                converted);
        }
    }
    
    converted
}

fn create_default_includes(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a header include
    let header_include = r#"<header>
  <div class="logo">
    <a href="{{ site.baseurl }}/">{{ site.title }}</a>
  </div>
  <nav>
    {% for link in site.data.navigation %}
      <a href="{{ link.url | prepend: site.baseurl }}">{{ link.title }}</a>
    {% endfor %}
  </nav>
</header>"#;

    let header_path = dest_dir.join("header.html");
    fs::write(&header_path, header_include)
        .map_err(|e| format!("Failed to create header include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/header.html".into(),
        description: "Created header include".into(),
    });

    // Create a footer include
    let footer_include = r#"<footer>
  <div class="footer-content">
    <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}{% if site.author %} - {{ site.author }}{% endif %}</p>
  </div>
</footer>"#;

    let footer_path = dest_dir.join("footer.html");
    fs::write(&footer_path, footer_include)
        .map_err(|e| format!("Failed to create footer include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/footer.html".into(),
        description: "Created footer include".into(),
    });

    // Create a table of contents include
    let toc_include = r#"<div class="toc">
  {% if page.toc_footers %}
    <ul class="toc-footer">
      {% for footer in page.toc_footers %}
        <li>{{ footer }}</li>
      {% endfor %}
    </ul>
  {% endif %}
</div>"#;

    let toc_path = dest_dir.join("toc.html");
    fs::write(&toc_path, toc_include)
        .map_err(|e| format!("Failed to create toc include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/toc.html".into(),
        description: "Created table of contents include".into(),
    });

    Ok(())
} 