use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

pub(super) fn migrate_includes(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Cobalt includes...");
    }

    // In Cobalt, includes are typically in _includes directory
    let includes_dir = source_dir.join("_includes");
    if !includes_dir.exists() || !includes_dir.is_dir() {
        result.warnings.push("No _includes directory found.".into());
        return Ok(());
    }

    // Create destination includes directory
    let dest_includes_dir = dest_dir.join("_includes");
    create_dir_if_not_exists(&dest_includes_dir)?;

    // Migrate include files
    for entry in WalkDir::new(&includes_dir)
        .into_iter()
        .filter_map(Result::ok) {
        
        if entry.file_type().is_file() {
            let file_path = entry.path();
            
            // Get the relative path from the includes directory
            let rel_path = file_path.strip_prefix(&includes_dir)
                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
            // Create destination path
            let dest_path = dest_includes_dir.join(rel_path);
            
            // Create parent directory if needed
            if let Some(parent) = dest_path.parent() {
                create_dir_if_not_exists(parent)?;
            }
            
            // Convert the include file
            convert_include_file(file_path, &dest_path, result)?;
        }
    }
    
    Ok(())
}

fn convert_include_file(
    source_path: &Path,
    dest_path: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Read the include file
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read include file {}: {}", source_path.display(), e))?;
    
    // Convert Cobalt include to Jekyll include
    let converted_content = convert_cobalt_include_to_jekyll(&content);
    
    // Write the converted include
    fs::write(dest_path, converted_content)
        .map_err(|e| format!("Failed to write include file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: format!("_includes/{}", dest_path.file_name().unwrap().to_string_lossy()),
        description: format!("Converted include from {}", source_path.display()),
    });
    
    Ok(())
}

fn convert_cobalt_include_to_jekyll(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Cobalt uses similar Liquid syntax to Jekyll, but there may be differences
    
    // Convert any Cobalt-specific includes to Jekyll format
    converted = converted.replace("{% include \"", "{% include ");
    converted = converted.replace(".liquid\" %}", ".html %} ");
    
    // Convert any custom Cobalt filters or tags to Jekyll equivalents
    converted = converted.replace("| date: \"%F\"", "| date: \"%Y-%m-%d\"");
    
    converted
}

fn create_default_includes(
    includes_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a default header include
    let header_include = r#"<header class="site-header">
    <div class="wrapper">
        <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title }}</a>
        <nav class="site-nav">
            <input type="checkbox" id="nav-trigger" class="nav-trigger" />
            <label for="nav-trigger">
                <span class="menu-icon">
                    <svg viewBox="0 0 18 15" width="18px" height="15px">
                        <path d="M18,1.484c0,0.82-0.665,1.484-1.484,1.484H1.484C0.665,2.969,0,2.304,0,1.484l0,0C0,0.665,0.665,0,1.484,0 h15.032C17.335,0,18,0.665,18,1.484L18,1.484z M18,7.516C18,8.335,17.335,9,16.516,9H1.484C0.665,9,0,8.335,0,7.516l0,0 c0-0.82,0.665-1.484,1.484-1.484h15.032C17.335,6.031,18,6.696,18,7.516L18,7.516z M18,13.516C18,14.335,17.335,15,16.516,15H1.484 C0.665,15,0,14.335,0,13.516l0,0c0-0.82,0.665-1.483,1.484-1.483h15.032C17.335,12.031,18,12.695,18,13.516L18,13.516z"/>
                    </svg>
                </span>
            </label>

            <div class="trigger">
                {% for page in site.pages %}
                {% if page.title %}
                <a class="page-link" href="{{ page.url | relative_url }}">{{ page.title }}</a>
                {% endif %}
                {% endfor %}
            </div>
        </nav>
    </div>
</header>"#;
    
    fs::write(includes_dir.join("header.html"), header_include)
        .map_err(|e| format!("Failed to write header include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/header.html".into(),
        description: "Created default header include".into(),
    });
    
    // Create a default footer include
    let footer_include = r#"<footer class="site-footer">
    <div class="wrapper">
        <h2 class="footer-heading">{{ site.title }}</h2>
        <div class="footer-col-wrapper">
            <div class="footer-col">
                <p>{{ site.description }}</p>
            </div>
        </div>
    </div>
</footer>"#;
    
    fs::write(includes_dir.join("footer.html"), footer_include)
        .map_err(|e| format!("Failed to write footer include: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_includes/footer.html".into(),
        description: "Created default footer include".into(),
    });
    
    Ok(())
} 