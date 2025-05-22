use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn generate_readme(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"# Migrated MDBook Site

This site was migrated from MDBook to Rustyll format. Below is information about the migrated site structure and how to work with it.

## Directory Structure

```
.
├── _config.yml          # Main configuration file
├── _data/              # Data files including table of contents
├── _includes/          # Reusable components and theme files
├── _layouts/           # Page layouts
├── _plugins/           # Site plugins (converted from MDBook preprocessors)
├── assets/             # Static files, images, CSS, and JavaScript
│   ├── css/           # Stylesheets
│   ├── js/            # JavaScript files
│   ├── images/        # Images
│   └── theme/         # Theme-specific assets
├── content/           # Main content directory (converted from src/)
└── Gemfile           # Ruby dependencies
```

## Features

The following MDBook features have been migrated:

1. Content
   - All markdown files from the `src` directory
   - Table of contents from `SUMMARY.md`
   - Front matter and metadata

2. Theme
   - Custom theme files (if present)
   - Default theme styles
   - Responsive layout

3. Preprocessors
   - Links processing
   - Index generation
   - Mermaid diagrams
   - KaTeX math rendering

4. Search
   - Full-text search functionality
   - Search index generation
   - Search results page

5. Static Files
   - Images and other assets
   - Theme-specific files
   - Additional configured static files

## Usage

1. Install dependencies:
   ```bash
   bundle install
   ```

2. Run the site locally:
   ```bash
   bundle exec rustyll serve
   ```

3. Build the site:
   ```bash
   bundle exec rustyll build
   ```

## Configuration

The site configuration is in `_config.yml`. This includes:
- Site metadata
- Theme settings
- Build options
- Plugin configurations

## Theme Customization

The theme files are located in:
- `_layouts/` for page layouts
- `_includes/theme/` for theme components
- `assets/theme/` for theme assets

## Plugins

Converted MDBook preprocessors are available as Jekyll plugins in the `_plugins` directory:
- Search functionality
- Preprocessors (links, index, etc.)
- Renderers
- Custom extensions

## Notes

- The site uses Liquid templates instead of Handlebars
- Math equations use KaTeX (converted from MDBook's math support)
- Mermaid diagrams are supported through a Jekyll plugin
- Search is implemented using a Jekyll plugin and JavaScript

## Migration Details

This site was automatically migrated using the Rustyll migration tool. The following changes were made:
- Converted MDBook's TOML configuration to YAML
- Transformed Handlebars templates to Liquid syntax
- Migrated preprocessors to Jekyll plugins
- Preserved all content and assets
- Set up equivalent functionality for MDBook features

## Support

For issues or questions about the migrated site, please refer to the Rustyll documentation or open an issue in the repository.
"#;

    fs::write(dest_dir.join("README.md"), content)
        .map_err(|e| format!("Failed to write README.md: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "README.md".into(),
        description: "Created README with migration documentation".into(),
    });

    // Create a .gitignore file
    let gitignore_content = r#"_site/
.sass-cache/
.jekyll-cache/
.jekyll-metadata
.bundle/
vendor/
Gemfile.lock
"#;

    fs::write(dest_dir.join(".gitignore"), gitignore_content)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: ".gitignore".into(),
        description: "Created .gitignore file".into(),
    });

    Ok(())
} 