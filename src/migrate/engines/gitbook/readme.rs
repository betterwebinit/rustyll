use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

impl super::GitbookMigrator {
    pub(super) fn generate_readme(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Generate main README.md for the migrated site
        let readme_content = format!(r#"# Rustyll Site (Migrated from GitBook)

This site was automatically migrated from a [GitBook](https://www.gitbook.com/) site to [Rustyll](https://rustyll.org) format.

## Site Structure

- `_config.yml` - Main site configuration
- `_layouts/` - Layout templates for pages
- `_includes/` - Reusable template fragments
- `_pages/` - Content pages
- `_data/` - Data files (e.g., navigation structure)
- `assets/` - Static assets (CSS, JavaScript, images, etc.)

## Migration Details

This site was migrated on {} using the GitBook to Rustyll migrator.

### Structure Changes

The following changes were made during migration:

- GitBook's `SUMMARY.md` structure was converted to Jekyll/Rustyll's page system
- Navigation was extracted to `_data/navigation.yml`
- Content pages were placed in `_pages/` directory
- GitBook README.md was converted to the site's homepage

### Asset Migration

- CSS styles were migrated to `assets/css/`
- JavaScript was migrated to `assets/js/`
- Images and other media were migrated to `assets/images/` and other appropriate directories
- GitBook plugins were preserved in `assets/plugins/`

## Next Steps

1. Run your site with `rustyll serve` to preview it
2. Check for any migration warnings in the migration report
3. Review imported content and templates
4. Customize your site configuration in `_config.yml`

## Rustyll Documentation

For more information about Rustyll, visit [https://rustyll.org](https://rustyll.org).
"#, chrono::Local::now().format("%Y-%m-%d"));

        // Write main README.md
        fs::write(dest_dir.join("README.md"), readme_content)
            .map_err(|e| format!("Failed to write main README file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Main README file created".to_string(),
        });
        
        // Create MIGRATION_GUIDE.md with detailed instructions
        let migration_guide = r#"# GitBook to Rustyll Migration Guide

This document provides guidance on how to work with your site after migrating from GitBook to Rustyll.

## Understanding the Structure

### GitBook Structure
GitBook typically organizes content using:
- `README.md` - Main landing page
- `SUMMARY.md` - Navigation structure
- Markdown files throughout the repository
- `book.json` for configuration

### Rustyll Structure
Rustyll/Jekyll organizes content using:
- `_config.yml` - Site configuration
- `_layouts/` - HTML templates
- `_includes/` - Reusable HTML fragments
- `_pages/` - Content pages
- `_data/` - Data files in YAML format
- `assets/` - Static files (CSS, JS, images)

## Working with Content

### Pages

All GitBook pages have been migrated to the `_pages/` directory with appropriate front matter:

```yaml
---
title: "Page Title"
layout: page
permalink: /path/to/page/
weight: 1  # Corresponds to the order in SUMMARY.md
---
```

### Navigation

The navigation structure from SUMMARY.md has been extracted to `_data/navigation.yml`:

```yaml
main:
  - title: "Introduction"
    url: /
  - title: "Getting Started"
    url: /getting-started/
```

To reference navigation in templates:

```liquid
{% for item in site.data.navigation.main %}
  <a href="{{ item.url | relative_url }}">{{ item.title }}</a>
{% endfor %}
```

### Links

GitBook relative links have been preserved, but you may need to update some links:

- GitBook: `[Link text](page.md)`
- Rustyll: `[Link text]({% link _pages/page.md %})` or `[Link text]({{ "/page/" | relative_url }})`

## Templates

The GitBook template has been converted to Liquid templates in `_layouts/` and `_includes/`.

Key template files:
- `_layouts/default.html` - Main layout with GitBook-like structure
- `_layouts/page.html` - Page template
- `_includes/navigation.html` - Navigation menu

## Assets

Assets have been organized under the `assets/` directory:
- `assets/css/` - Stylesheets 
- `assets/js/` - JavaScript files
- `assets/images/` - Images
- `assets/plugins/` - GitBook plugins (client-side assets only)

## Configuration

The `_config.yml` file contains all site configuration. Notable settings:

```yaml
title: "Your Site Title"
description: "Site description"
baseurl: ""  # Set this for subdirectory sites
markdown: kramdown  # Markdown processor

# GitBook-specific settings preserved from book.json
gitbook:
  plugins: [...]
  structure: {...}
```

## Common Tasks

### Adding a new page

1. Create a new markdown file in `_pages/`
2. Add front matter:
   ```yaml
   ---
   title: "New Page Title"
   layout: page
   permalink: /new-page/
   ---
   ```
3. Add the page to navigation in `_data/navigation.yml`

### Customizing the appearance

1. Edit CSS files in `assets/css/`
2. Modify layout templates in `_layouts/`
3. Update includes in `_includes/`

## Further Help

For more information about Rustyll, visit [https://rustyll.org](https://rustyll.org).
"#;

        // Write MIGRATION_GUIDE.md
        fs::write(dest_dir.join("MIGRATION_GUIDE.md"), migration_guide)
            .map_err(|e| format!("Failed to write migration guide: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "MIGRATION_GUIDE.md".to_string(),
            change_type: ChangeType::Created,
            description: "Migration guide created".to_string(),
        });
        
        // Create _layouts/README.md
        let layouts_dir = dest_dir.join("_layouts");
        if layouts_dir.exists() {
            let layouts_readme = r#"# Layouts Directory

This directory contains the HTML templates that define the structure of your site.

## Layout Files

- `default.html` - The main layout with the GitBook-like structure (sidebar, content area)
- `page.html` - Template for regular content pages
- `home.html` - Template for the home page

## Using Layouts

Layouts are specified in the front matter of content files:

```yaml
---
layout: page
---
```

## Customizing Layouts

Feel free to modify these templates to customize the appearance of your site.
Each layout file is a Liquid template with HTML structure.

Key elements in the default layout:
- Navigation sidebar
- Content area with `{{ content }}` placeholder
- Header and footer areas
"#;

            write_readme(&layouts_dir, layouts_readme)?;
            
            result.changes.push(MigrationChange {
                file_path: "_layouts/README.md".to_string(),
                change_type: ChangeType::Created,
                description: "Layouts directory README created".to_string(),
            });
        }
        
        // Create _includes/README.md
        let includes_dir = dest_dir.join("_includes");
        if includes_dir.exists() {
            let includes_readme = r#"# Includes Directory

This directory contains reusable HTML fragments that can be included in layouts and pages.

## Include Files

- `head.html` - HTML head content (metadata, CSS links)
- `header.html` - Site header
- `footer.html` - Site footer
- `navigation.html` - Navigation sidebar

## Using Includes

To use an include in a layout or page:

```liquid
{% include header.html %}
```

You can also pass variables to includes:

```liquid
{% include navigation.html active_page="home" %}
```

## Customizing Includes

These includes were generated based on the GitBook theme structure.
Feel free to modify them to match your desired design.
"#;

            write_readme(&includes_dir, includes_readme)?;
            
            result.changes.push(MigrationChange {
                file_path: "_includes/README.md".to_string(),
                change_type: ChangeType::Created,
                description: "Includes directory README created".to_string(),
            });
        }
        
        // Create _data/README.md
        let data_dir = dest_dir.join("_data");
        if data_dir.exists() {
            let data_readme = r#"# Data Directory

This directory contains data files used by your site templates.

## Data Files

- `navigation.yml` - Site navigation structure extracted from SUMMARY.md

## Using Data Files

Data from these files can be accessed in templates using the `site.data` object:

```liquid
{% for item in site.data.navigation.main %}
  <a href="{{ item.url | relative_url }}">{{ item.title }}</a>
{% endfor %}
```

## Data Structure

### navigation.yml

This file contains the navigation structure for your site:

```yaml
# Section with grouped items
section_name:
  - title: "Page Title"
    url: /path/to/page/
  - title: "Another Page"
    url: /another-page/

# Top-level items
- title: "Standalone Page"
  url: /standalone/
```

You can modify this file to customize your site navigation.
"#;

            write_readme(&data_dir, data_readme)?;
            
            result.changes.push(MigrationChange {
                file_path: "_data/README.md".to_string(),
                change_type: ChangeType::Created,
                description: "Data directory README created".to_string(),
            });
        }
        
        // Create assets/README.md
        let assets_dir = dest_dir.join("assets");
        if assets_dir.exists() {
            let assets_readme = r#"# Assets Directory

This directory contains static assets for your migrated site.

## Subdirectories

- `css/` - Stylesheets
- `js/` - JavaScript files
- `images/` - Images
- `plugins/` - GitBook plugins (client-side assets only)

## GitBook Style

The CSS has been adapted from GitBook's style with these key components:
- `style.css` - Main styles for the GitBook-like layout
- `website.css` - Website-specific styles

## JavaScript

- `main.js` - Main JavaScript with GitBook-like functionality
- `plugins.js` - Loader for GitBook plugins

## Using Assets

To reference assets in templates:

```liquid
<link rel="stylesheet" href="{{ "/assets/css/style.css" | relative_url }}">
<script src="{{ "/assets/js/main.js" | relative_url }}"></script>
<img src="{{ "/assets/images/logo.png" | relative_url }}" alt="Logo">
```
"#;

            write_readme(&assets_dir, assets_readme)?;
            
            result.changes.push(MigrationChange {
                file_path: "assets/README.md".to_string(),
                change_type: ChangeType::Created,
                description: "Assets directory README created".to_string(),
            });
        }
        
        Ok(())
    }
} 