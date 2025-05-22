use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

impl super::HarpMigrator {
    pub(super) fn generate_readme(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Generate main README.md for the migrated site
        let readme_content = format!(r#"# Rustyll Site (Migrated from Harp)

This site was automatically migrated from a [Harp](https://harpjs.com/) site to [Rustyll](https://rustyll.org) format.

## Site Structure

- `_config.yml` - Main site configuration
- `_layouts/` - Layout templates for pages and posts
- `_includes/` - Reusable template fragments (partials)
- `_pages/` - Static site pages
- `_posts/` - Blog posts
- `_data/` - Data files (YAML)
- `assets/` - Static assets (CSS, JavaScript, images, etc.)

## Migration Details

This site was migrated on {} using the Harp to Rustyll migrator.

### Templating Changes

The following transformations were applied during migration:

- Jade templates were converted to HTML/Liquid syntax
- EJS templates (`<%= variable %>`) were converted to Liquid syntax (`{{ variable }}`)
- Partials (files starting with `_`) were moved to the `_includes` directory
- Layout files (`_layout.*`) were converted to Jekyll-compatible layouts

### Content Migration

- Content files were moved to `_pages/` and `_posts/` directories
- Front matter (YAML metadata) was added to all content files
- JSON data from `_data.json` files was extracted and converted to YAML

## Next Steps

1. Run your site with `rustyll serve` to preview it
2. Check for any migration warnings in the migration report
3. Review templates for any complex Jade/EJS syntax that needs manual conversion
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
        
        // Create USAGE.md with instructions for using Rustyll
        let usage_content = r#"# Using Your Migrated Rustyll Site

## Prerequisites

1. Install Rust and Cargo if you don't have them already:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Rustyll:
   ```
   cargo install rustyll
   ```

## Development Workflow

1. **Preview your site**:
   ```
   rustyll serve
   ```

2. **Build your site**:
   ```
   rustyll build
   ```

3. **Create a new post**:
   ```
   rustyll new post "My New Post Title"
   ```

4. **Create a new page**:
   ```
   rustyll new page "About Us"
   ```

## Directory Structure

- `_config.yml` - Site configuration
- `_layouts/` - Layout templates
- `_includes/` - Reusable snippets
- `_pages/` - Static pages
- `_posts/` - Blog posts
- `_data/` - Data files (YAML)
- `assets/` - Static assets (CSS, JS, images)

## Liquid Templating

Rustyll uses Liquid templating. Here are some common patterns:

- **Variables**: `{{ page.title }}`
- **Loops**: `{% for post in site.posts %} ... {% endfor %}`
- **Conditionals**: `{% if page.featured %} ... {% endif %}`
- **Includes**: `{% include header.html %}`
- **Links**: `{{ "/about/" | relative_url }}` or `{{ post.url | absolute_url }}`

## Front Matter

Each content file starts with YAML front matter between triple dashes:

```yaml
---
layout: post
title: "My Post Title"
date: 2023-01-01
categories: [news, updates]
---
```

## Deployment

After building your site with `rustyll build`, deploy the contents of the `_site` directory to your web host.
"#;

        // Write USAGE.md
        fs::write(dest_dir.join("USAGE.md"), usage_content)
            .map_err(|e| format!("Failed to write USAGE.md file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "USAGE.md".to_string(),
            change_type: ChangeType::Created,
            description: "Rustyll usage guide created".to_string(),
        });
        
        // Create a MIGRATION.md file with key differences between Harp and Rustyll
        let migration_notes = r#"# Migration Notes: Harp to Rustyll

This document highlights key differences between Harp and Rustyll to help you adapt to the new system.

## Configuration

- **Harp**: Uses `harp.json` or `_harp.json` with a `globals` object for site configuration
- **Rustyll**: Uses `_config.yml` in YAML format, with global variables at the root level

## Content

- **Harp**: Uses any file in the root with EJS or Jade preprocessing
- **Rustyll**: Uses directories like `_posts/` and `_pages/` with front matter for metadata

## Templates

- **Harp**: Uses EJS, Jade, or plain HTML with `{{ yield }}` to include content
- **Rustyll**: Uses Liquid templates with `{{ content }}` to include content

## Partials

- **Harp**: Uses files starting with `_` (e.g., `_header.jade`)
- **Rustyll**: Uses files in the `_includes/` directory

## Data

- **Harp**: Uses `_data.json` files in the same directory as content
- **Rustyll**: Uses YAML files in the `_data/` directory

## Compilation

- **Harp**: Uses `harp compile` to build the site
- **Rustyll**: Uses `rustyll build` to build the site

## Development Server

- **Harp**: Uses `harp server` to run a development server
- **Rustyll**: Uses `rustyll serve` to run a development server

## Liquid vs. EJS/Jade Syntax

| Harp (EJS) | Rustyll (Liquid) |
|------------|------------------|
| `<%= title %>` | `{{ page.title }}` |
| `<%- partial("header") %>` | `{% include header.html %}` |
| `<% if (condition) { %>` | `{% if condition %}` |
| `<% } %>` | `{% endif %}` |
| `<% for (var i in items) { %>` | `{% for item in items %}` |
| `<% } %>` | `{% endfor %}` |

## Front Matter

Harp used `_data.json` to provide metadata for files, whereas Rustyll uses YAML front matter directly in the content files. For example:

```yaml
---
title: "My Page Title"
layout: page
date: 2023-01-01
---

Page content goes here...
```
"#;

        // Write MIGRATION.md
        fs::write(dest_dir.join("MIGRATION_NOTES.md"), migration_notes)
            .map_err(|e| format!("Failed to write MIGRATION_NOTES.md file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "MIGRATION_NOTES.md".to_string(),
            change_type: ChangeType::Created,
            description: "Migration notes created".to_string(),
        });
        
        Ok(())
    }
} 