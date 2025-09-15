use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::HarpMigrator {
    pub(super) fn migrate_layouts(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In Harp, layouts can be in either _layout directory or any other *.jade/*.ejs files
        
        // Create destination layouts directory
        let dest_layouts_dir = dest_dir.join("_layouts");
        create_dir_if_not_exists(&dest_layouts_dir)?;
        
        // Check various possible layout locations
        let layout_dirs = [
            source_dir.join("_layout"),
            source_dir.join("_layouts"),
        ];
        
        let mut found_layouts = false;
        let content_dir = source_dir.to_path_buf();
        
        // First check explicit layout directories
        for layout_dir in layout_dirs.iter() {
            if layout_dir.exists() && layout_dir.is_dir() {
                found_layouts = true;
                
                if verbose {
                    log::info!("Migrating layouts from {}", layout_dir.display());
                }
                
                // Process all layout files
                for entry in WalkDir::new(layout_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        self.migrate_layout_file(file_path, layout_dir, &dest_layouts_dir, result)?;
                    }
                }
            }
        }
        
        // Then check for implicit layout files in the content directory
        // Create a clone of content_dir for the WalkDir to use
        let walk_dir = content_dir.clone();
        for entry in WalkDir::new(&walk_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                
                // Check if this is a layout file (look for layout.* files)
                if file_name.starts_with("layout.") || 
                   (file_name.starts_with("_layout.")) {
                    found_layouts = true;
                    
                    if verbose {
                        log::info!("Found layout file: {}", file_path.display());
                    }
                    
                    self.migrate_layout_file(file_path, &content_dir, &dest_layouts_dir, result)?;
                }
            }
        }
        
        // Create basic layouts if none were found
        if !found_layouts {
            if verbose {
                log::info!("No layouts found, creating basic layouts");
            }
            
            self.create_basic_layouts(&dest_layouts_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_layout_file(&self, file_path: &Path, source_dir: &Path, dest_layouts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Read the layout file
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read layout file {}: {}", file_path.display(), e))?;
        
        // Convert the layout based on file type
        let converted_content = match file_path.extension().and_then(|e| e.to_str()) {
            Some("jade") => convert_jade_layout_to_liquid(&content),
            Some("ejs") => convert_ejs_layout_to_liquid(&content),
            _ => convert_html_layout_to_liquid(&content),
        };
        
        // Determine the layout name
        // If it's in a subdirectory, use the directory name for the layout name
        let rel_path = file_path.strip_prefix(source_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
        let layout_name = if rel_path.components().count() > 1 {
            // For layouts in subdirectories, use the directory name
            let parent = rel_path.parent().unwrap();
            if parent.components().count() > 0 {
                let dir_name = parent.components().last().unwrap()
                    .as_os_str().to_string_lossy();
                if dir_name.starts_with('_') || dir_name == "layouts" {
                    "default".to_string()
                } else {
                    dir_name.to_string()
                }
            } else {
                "default".to_string()
            }
        } else {
            // For root layouts, use "default"
            "default".to_string()
        };
        
        // Create destination files for common layouts
        let dest_layouts = vec![
            (format!("{}.html", layout_name), "Main layout"),
            ("post.html".to_string(), "Blog post layout"),
            ("page.html".to_string(), "Regular page layout"),
            ("home.html".to_string(), "Home page layout"),
        ];
        
        for (layout_file, description) in dest_layouts {
            let mut layout_content = converted_content.clone();
            
            // Customize the layout based on its type
            if layout_file == "post.html" {
                layout_content = layout_content.replace("{{ yield }}", r#"<article class="post">
  <header class="post-header">
    <h1 class="post-title">{{ page.title }}</h1>
    <p class="post-meta">{{ page.date | date: "%b %-d, %Y" }}</p>
  </header>

  <div class="post-content">
    {{ content }}
  </div>
</article>"#);
            } else if layout_file == "page.html" {
                layout_content = layout_content.replace("{{ yield }}", r#"<article class="page">
  <header class="page-header">
    <h1 class="page-title">{{ page.title }}</h1>
  </header>

  <div class="page-content">
    {{ content }}
  </div>
</article>"#);
            } else if layout_file == "home.html" {
                layout_content = layout_content.replace("{{ yield }}", r#"<div class="home">
  <h1 class="page-heading">{{ page.title }}</h1>
  
  {{ content }}
  
  <h2 class="post-list-heading">Posts</h2>
  <ul class="post-list">
    {% for post in site.posts %}
    <li>
      <span class="post-meta">{{ post.date | date: "%b %-d, %Y" }}</span>
      <h3>
        <a class="post-link" href="{{ post.url | relative_url }}">{{ post.title }}</a>
      </h3>
    </li>
    {% endfor %}
  </ul>
</div>"#);
            } else {
                // Default layout
                layout_content = layout_content.replace("{{ yield }}", "{{ content }}");
            }
            
            // Write the layout file
            let dest_path = dest_layouts_dir.join(&layout_file);
            fs::write(&dest_path, layout_content)
                .map_err(|e| format!("Failed to write layout file: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: format!("_layouts/{}", layout_file),
                change_type: ChangeType::Converted,
                description: format!("{} migrated from Harp", description),
            });
        }
        
        Ok(())
    }
    
    fn create_basic_layouts(&self, dest_layouts_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a basic default layout
        let default_layout = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% if page.title %}{{ page.title }} - {{ site.title }}{% else %}{{ site.title }}{% endif %}</title>
  <link rel="stylesheet" href="{{ "/assets/css/main.css" | relative_url }}">
</head>
<body>
  <header class="site-header">
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
  </header>

  <main class="content">
    <div class="container">
      {{ content }}
    </div>
  </main>

  <footer class="site-footer">
    <div class="container">
      <p>&copy; {{ site.time | date: '%Y' }} {{ site.title }}</p>
    </div>
  </footer>

  <script src="{{ "/assets/js/main.js" | relative_url }}"></script>
</body>
</html>"#;

        fs::write(dest_layouts_dir.join("default.html"), default_layout)
            .map_err(|e| format!("Failed to write default layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/default.html".to_string(),
            change_type: ChangeType::Created,
            description: "Default layout created".to_string(),
        });
        
        // Create a post layout
        let post_layout = r#"---
layout: default
---
<article class="post">
  <header class="post-header">
    <h1 class="post-title">{{ page.title }}</h1>
    <p class="post-meta">{{ page.date | date: "%b %-d, %Y" }}</p>
  </header>

  <div class="post-content">
    {{ content }}
  </div>
</article>"#;

        fs::write(dest_layouts_dir.join("post.html"), post_layout)
            .map_err(|e| format!("Failed to write post layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/post.html".to_string(),
            change_type: ChangeType::Created,
            description: "Post layout created".to_string(),
        });
        
        // Create a page layout
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

        fs::write(dest_layouts_dir.join("page.html"), page_layout)
            .map_err(|e| format!("Failed to write page layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/page.html".to_string(),
            change_type: ChangeType::Created,
            description: "Page layout created".to_string(),
        });
        
        // Create a home layout
        let home_layout = r#"---
layout: default
---
<div class="home">
  <h1 class="page-heading">{{ page.title }}</h1>
  
  {{ content }}
  
  <h2 class="post-list-heading">Posts</h2>
  <ul class="post-list">
    {% for post in site.posts %}
    <li>
      <span class="post-meta">{{ post.date | date: "%b %-d, %Y" }}</span>
      <h3>
        <a class="post-link" href="{{ post.url | relative_url }}">{{ post.title }}</a>
      </h3>
    </li>
    {% endfor %}
  </ul>
</div>"#;

        fs::write(dest_layouts_dir.join("home.html"), home_layout)
            .map_err(|e| format!("Failed to write home layout: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_layouts/home.html".to_string(),
            change_type: ChangeType::Created,
            description: "Home layout created".to_string(),
        });
        
        Ok(())
    }
}

// Helper function to convert EJS layout to Liquid
fn convert_ejs_layout_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert yield tag
    result = result.replace("<%- yield %>", "{{ content }}");
    
    // Convert other EJS syntax
    result = super::content::convert_ejs_to_liquid(&result);
    
    result
}

// Helper function to convert Jade layout to Liquid
fn convert_jade_layout_to_liquid(content: &str) -> String {
    let mut result = super::content::convert_jade_to_liquid(content);
    
    // In Jade layouts, the "block content" is where page content goes
    result = result.replace("block content", "{{ content }}");
    
    result
}

// Helper function to convert HTML layout to Liquid
fn convert_html_layout_to_liquid(content: &str) -> String {
    let mut result = content.to_string();
    
    // Convert yield tag (common in Harp layouts)
    result = result.replace("{{ yield }}", "{{ content }}");
    
    result
} 