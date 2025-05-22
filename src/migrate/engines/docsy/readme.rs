use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn generate_readme(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"# Migrated Docsy Site

This site was migrated from Docsy (Hugo) to a Jekyll-based site using Rustyll. Below is information about the migrated site structure and how to work with it.

## Directory Structure

```
.
├── _config.yml          # Main configuration file
├── _data/               # Data files including menu and sidebar structure
├── _includes/           # Reusable components (converted from partials)
├── _layouts/            # Page layouts
├── assets/              # Static files, images, CSS, and JavaScript
│   ├── css/             # Stylesheets
│   ├── js/              # JavaScript files
│   ├── images/          # Images and other media
│   └── fonts/           # Web fonts
├── content/             # Main content directory (converted from Hugo content)
└── Gemfile             # Ruby dependencies
```

## Features

The following Docsy features have been migrated:

1. Content
   - All markdown files from the `content` directory
   - Front matter and metadata
   - Shortcodes converted to Liquid includes

2. Templates
   - Layouts converted from Go templates to Liquid
   - Partials converted to includes

3. Navigation
   - Menu structure preserved
   - Sidebar navigation

4. Styling
   - CSS/SCSS files preserved
   - Responsive design

## Usage

1. Install dependencies:
   ```bash
   bundle install
   ```

2. Run the site locally:
   ```bash
   bundle exec jekyll serve
   ```

3. Build the site:
   ```bash
   bundle exec jekyll build
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
- `_includes/` for components
- `assets/css/` for stylesheets

## Notes

- The site uses Liquid templates instead of Hugo's Go templates
- Shortcodes are implemented as includes
- The folder structure has been adapted to Jekyll conventions
- Some advanced Hugo features might need manual adjustment

## Migration Details

This site was automatically migrated using the Rustyll migration tool. The following changes were made:
- Converted Hugo's TOML/YAML configuration to Jekyll YAML
- Transformed Go templates to Liquid syntax
- Preserved all content and assets
- Set up equivalent functionality for Docsy features

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