use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::HarpMigrator {
    pub(super) fn migrate_partials(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, partials are prefixed with underscore (_partial.*)
        
        // Create destination includes directory
        let dest_includes_dir = dest_dir.join("_includes");
        create_dir_if_not_exists(&dest_includes_dir)?;
        
        // First check if there's a dedicated _partials directory
        let partials_dirs = [
            source_dir.join("_partials"),
            source_dir.join("_includes"),
        ];
        
        let mut found_partials = false;
        let content_dir = source_dir.to_path_buf();
        
        // Check dedicated partials directories first
        for partials_dir in partials_dirs.iter() {
            if partials_dir.exists() && partials_dir.is_dir() {
                found_partials = true;
                
                if verbose {
                    log::info!("Migrating partials from {}", partials_dir.display());
                }
                
                for entry in WalkDir::new(partials_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        self.migrate_partial_file(file_path, partials_dir, &dest_includes_dir, result)?;
                    }
                }
            }
        }
        
        // Then look for _*.* files throughout the content directory
        // Create a clone of content_dir for the WalkDir to use
        let walk_dir = content_dir.clone();
        for entry in WalkDir::new(&walk_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                
                // Check if it's a partial (files starting with underscore, but not _layout or _data)
                if file_name.starts_with("_") && 
                   !file_name.starts_with("_layout.") && 
                   !file_name.starts_with("_data.") {
                    
                    if verbose {
                        log::info!("Migrating partial: {}", file_path.display());
                    }
                    
                    self.migrate_partial_file(file_path, &content_dir, &dest_includes_dir, result)?;
                    found_partials = true;
                }
            }
        }
        
        // Create basic includes if none were found
        if !found_partials {
            if verbose {
                log::info!("No partials found, creating basic includes");
            }
            
            self.create_basic_includes(&dest_includes_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_partial_file(&self, file_path: &Path, source_dir: &Path, dest_includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the partial file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read partial file {}: {}", file_path.display(), e))?;
        
        // Convert the partial based on file type
        let converted_content = match file_path.extension().and_then(|e| e.to_str()) {
            Some("jade") => convert_jade_partial_to_liquid(&content),
            Some("ejs") => convert_ejs_partial_to_liquid(&content),
            _ => content,
        };
        
        // Determine the partial name (remove leading underscore)
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        let partial_name = if file_name.starts_with('_') && file_name.len() > 1 {
            let mut chars = file_name.chars();
            chars.next(); // Skip the underscore
            chars.as_str()
        } else {
            &file_name
        };
        
        // Create partial name with .html extension
        let dest_file_name = if let Some(dot_pos) = partial_name.rfind('.') {
            format!("{}.html", &partial_name[0..dot_pos])
        } else {
            format!("{}.html", partial_name)
        };
        
        // Write the file to destination
        let dest_path = dest_includes_dir.join(&dest_file_name);
        fs::write(&dest_path, converted_content)
            .map_err(|e| format!("Failed to write include file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: format!("_includes/{}", dest_file_name),
            change_type: ChangeType::Converted,
            description: "Partial converted to include".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_includes(&self, dest_includes_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a header include
        let header_content = r#"<header class="site-header" role="banner">
  <div class="container">
    <a class="site-title" href="{{ "/" | relative_url }}">{{ site.title }}</a>
    <nav class="site-nav">
      <ul>
        <li><a href="{{ "/" | relative_url }}">Home</a></li>
        <li><a href="{{ "/about/" | relative_url }}">About</a></li>
        <li><a href="{{ "/blog/" | relative_url }}">Blog</a></li>
      </ul>
    </nav>
  </div>
</header>"#;

        fs::write(dest_includes_dir.join("header.html"), header_content)
            .map_err(|e| format!("Failed to write header include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/header.html".to_string(),
            change_type: ChangeType::Created,
            description: "Header include created".to_string(),
        });
        
        // Create a footer include
        let footer_content = r#"<footer class="site-footer">
  <div class="container">
    <h2 class="footer-heading">{{ site.title }}</h2>
    <p>{{ site.description }}</p>
    <p>&copy; {{ site.time | date: "%Y" }} {{ site.title }}</p>
  </div>
</footer>"#;

        fs::write(dest_includes_dir.join("footer.html"), footer_content)
            .map_err(|e| format!("Failed to write footer include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/footer.html".to_string(),
            change_type: ChangeType::Created,
            description: "Footer include created".to_string(),
        });
        
        // Create a head include
        let head_content = r#"<head>
  <meta charset="utf-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <meta name="description" content="{% if page.excerpt %}{{ page.excerpt | strip_html | strip_newlines | truncate: 160 }}{% else %}{{ site.description }}{% endif %}">
  <link rel="stylesheet" href="{{ "/assets/css/main.css" | relative_url }}">
  <link rel="canonical" href="{{ page.url | replace:'index.html','' | absolute_url }}">
</head>"#;

        fs::write(dest_includes_dir.join("head.html"), head_content)
            .map_err(|e| format!("Failed to write head include: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/head.html".to_string(),
            change_type: ChangeType::Created,
            description: "Head include created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to convert EJS partial to Liquid
fn convert_ejs_partial_to_liquid(content: &str) -> String {
    // Use the same EJS converter as for content
    super::content::convert_ejs_to_liquid(content)
}

// Helper function to convert Jade partial to Liquid
fn convert_jade_partial_to_liquid(content: &str) -> String {
    // Use the same Jade converter as for content
    super::content::convert_jade_to_liquid(content)
} 