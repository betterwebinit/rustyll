use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

impl super::AssembleMigrator {
    pub(super) fn generate_readme(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Generate main README for the migrated site
        let readme_content = format!(r#"# Rustyll Site (Migrated from Assemble)

This site was automatically migrated from an [Assemble](https://assemble.io/) site to [Rustyll](https://rustyll.org) format.

## Site Structure

- `_config.yml` - Main site configuration
- `_layouts/` - Layout templates for pages and posts
- `_includes/` - Reusable template fragments (partials)
- `_pages/` - Static site pages
- `_posts/` - Blog posts
- `_data/` - Data files (YAML/JSON)
- `assets/` - Static assets (CSS, JavaScript, images, etc.)

## Migration Details

This site was migrated on {} using the Assemble to Rustyll migrator.

### Templating Changes

The following transformations were applied during migration:

- Handlebars templates (`{{variable}}`) were converted to Liquid syntax (`{{ variable }}`)
- Handlebars partials (`{{> partial}}`) were converted to Liquid includes (`{{{{ include "partial.html" }}}}`)
- Handlebars conditionals (`{{#if condition}}`) were converted to Liquid conditionals (`{{{{ if condition }}}}`)
- Handlebars loops (`{{#each items}}`) were converted to Liquid loops (`{{{{ for item in items }}}}`)

### Content Migration

- Content files from `pages/` or similar directories were moved to `_pages/`
- Blog posts were moved to `_posts/` with Jekyll-style date prefixes
- Front matter (YAML metadata) was preserved and extended

### Data Migration

- JSON data files were converted to YAML format
- JavaScript data files may require manual conversion

## Next Steps

1. Run your site with `rustyll serve` to preview it
2. Check for any migration warnings in the migration report
3. Review templates for any complex Handlebars helpers that need manual conversion
4. Customize your site configuration in `_config.yml`

## Rustyll Documentation

For more information about Rustyll, visit [https://rustyll.org](https://rustyll.org).
"#, chrono::Local::now().format("%Y-%m-%d"));

        // Write the main README
        fs::write(dest_dir.join("README.md"), readme_content)
            .map_err(|e| format!("Failed to write main README file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Main README file created".to_string(),
        });
        
        // Generate README for the root directory explaining how to use Rustyll
        let rustyll_readme = r#"# Using Your Migrated Rustyll Site

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
- `_data/` - Data files (YAML/JSON)
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

        // Create a USAGE.md file
        fs::write(dest_dir.join("USAGE.md"), rustyll_readme)
            .map_err(|e| format!("Failed to write USAGE.md file: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "USAGE.md".to_string(),
            change_type: ChangeType::Created,
            description: "Rustyll usage guide created".to_string(),
        });
        
        Ok(())
    }
} 