use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

impl super::NanocMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Write main README.md
        let main_readme = r#"# Migrated Nanoc Site

This site was migrated from Nanoc to Rustyll.

## Directory Structure

- `_config.yml` - Site configuration
- `_data/` - Data files
- `_layouts/` - Layout templates
- `_includes/` - Reusable template fragments
- `_posts/` - Blog posts
- `_pages/` - Site pages
- `assets/` - Static files (CSS, JS, images, etc.)

## Migration Notes

1. **Templates**: Nanoc ERB/HAML templates have been converted to Liquid templates
   - `<%= yield %>` → `{{ content }}`
   - `<% ... %>` → `{% ... %}`
   - `<%= ... %>` → `{{ ... }}`

2. **Content**: Nanoc content files have been migrated to Jekyll-style pages/posts
   - Blog posts are in the `_posts/` directory with date prefixes
   - Regular pages are in the `_pages/` collection

3. **Configuration**: Nanoc config has been migrated to `_config.yml`
   - Some configuration may require manual adjustment

4. **Assets**: Static files have been moved to the `assets/` directory

## Development

To build the site:

```bash
# Install dependencies
bundle install

# Build the site
bundle exec rustyll build

# Serve the site locally
bundle exec rustyll serve
```

## Manual Adjustments

Some aspects of the site may require manual adjustment:

1. Complex Ruby helpers need to be reimplemented in Liquid
2. Nanoc Rules (routing) have no direct equivalent in Rustyll
3. Complex data sources may need manual conversion
"#;

        let main_readme_path = dest_dir.join("README.md");
        fs::write(&main_readme_path, main_readme)
            .map_err(|e| format!("Failed to write main README.md: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Migration documentation created".to_string(),
        });
        
        // Write README for includes directory
        let includes_dir = dest_dir.join("_includes");
        if !includes_dir.exists() {
            create_dir_if_not_exists(&includes_dir)?;
            
            // Create a basic header and footer
            let header_html = r#"<header class="site-header">
  <div class="container">
    <a class="site-title" href="{{ '/' | relative_url }}">{{ site.title | escape }}</a>
    <nav class="site-nav">
      <ul>
        {% for item in site.data.navigation.main_nav %}
          <li><a href="{{ item.url | relative_url }}">{{ item.title }}</a></li>
        {% endfor %}
      </ul>
    </nav>
  </div>
</header>"#;

            let footer_html = r#"<footer class="site-footer">
  <div class="container">
    <p>&copy; {{ site.time | date: '%Y' }} {{ site.title | escape }}</p>
  </div>
</footer>"#;

            fs::write(includes_dir.join("header.html"), header_html)
                .map_err(|e| format!("Failed to write header include: {}", e))?;
                
            fs::write(includes_dir.join("footer.html"), footer_html)
                .map_err(|e| format!("Failed to write footer include: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: "_includes/header.html".to_string(),
                change_type: ChangeType::Created,
                description: "Basic header include created".to_string(),
            });
            
            result.changes.push(MigrationChange {
                file_path: "_includes/footer.html".to_string(),
                change_type: ChangeType::Created,
                description: "Basic footer include created".to_string(),
            });
        }
        
        let includes_readme = r#"# Includes Directory

This directory contains reusable template fragments that can be included in layouts and pages.

## Usage

To include a template fragment in another template:

```liquid
{% include header.html %}
```

To pass variables to an include:

```liquid
{% include product.html name="Product Name" price="$9.99" %}
```

## Converted Includes

Some includes may have been converted from Nanoc partials:
- Partials from layouts/ directory
- ERB/HAML partials have been converted to Liquid syntax

## Basic Includes

- `header.html` - Site header with navigation
- `footer.html` - Site footer
"#;

        let includes_readme_path = includes_dir.join("README.md");
        fs::write(&includes_readme_path, includes_readme)
            .map_err(|e| format!("Failed to write includes README.md: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "_includes/README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Includes documentation created".to_string(),
        });
        
        // Create a basic CSS file if it doesn't exist
        let css_dir = dest_dir.join("assets").join("css");
        create_dir_if_not_exists(&css_dir)?;
        
        let css_path = css_dir.join("main.css");
        if !css_path.exists() {
            let basic_css = r#"/* Basic styles for migrated Nanoc site */
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  line-height: 1.6;
  color: #333;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 1rem;
}

.container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 0 1rem;
}

/* Header styles */
.site-header {
  border-bottom: 1px solid #e8e8e8;
  padding: 1rem 0;
  margin-bottom: 2rem;
}

.site-title {
  font-size: 1.5rem;
  font-weight: 700;
  text-decoration: none;
  color: #333;
}

.site-nav ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
}

.site-nav li {
  margin-right: 1rem;
}

.site-nav a {
  text-decoration: none;
  color: #666;
}

.site-nav a:hover {
  color: #0366d6;
}

/* Content styles */
.post-title, .page-title {
  margin-top: 0;
}

.post-meta {
  color: #666;
  font-size: 0.9rem;
  margin-bottom: 1rem;
}

/* Footer styles */
.site-footer {
  border-top: 1px solid #e8e8e8;
  padding: 1rem 0;
  margin-top: 2rem;
  color: #666;
}
"#;
            
            fs::write(&css_path, basic_css)
                .map_err(|e| format!("Failed to write basic CSS: {}", e))?;
                
            result.changes.push(MigrationChange {
                file_path: "assets/css/main.css".to_string(),
                change_type: ChangeType::Created,
                description: "Basic CSS stylesheet created".to_string(),
            });
        }
        
        // Create a Gemfile for the Rustyll site
        let gemfile_path = dest_dir.join("Gemfile");
        let gemfile_content = r#"source "https://rubygems.org"

gem "rustyll", "~> 0.1.0"  # Replace with actual version when Rustyll is released

# Plugins can be added here
# gem "rustyll-feed", "~> 0.1.0"
# gem "rustyll-seo-tag", "~> 0.1.0"
"#;

        fs::write(&gemfile_path, gemfile_content)
            .map_err(|e| format!("Failed to write Gemfile: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "Gemfile".to_string(),
            change_type: ChangeType::Created,
            description: "Gemfile created for Rustyll dependencies".to_string(),
        });
        
        Ok(())
    }
} 