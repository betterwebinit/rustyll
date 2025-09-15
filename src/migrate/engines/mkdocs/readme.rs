use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn generate_readme(
    dest_dir: &Path,
    result: &mut MigrationResult
) -> Result<(), String> {
    // Create README.md in the root directory
    let content = r#"# Migrated MkDocs Site

This site was migrated from MkDocs to Rustyll format. Below is information about the migrated site structure and how to work with it.

## Directory Structure

```
.
├── _config.yml          # Main configuration file
├── _data/               # Data files including navigation
├── _docs/               # Documentation pages migrated from MkDocs
├── assets/              # Static files, images, CSS, and JavaScript
│   ├── css/             # Stylesheets
│   ├── js/              # JavaScript files
│   └── images/          # Images and other media
└── index.md             # Home page
```

## Features

The following MkDocs features have been migrated:

1. Content
   - All markdown files from the `docs` directory
   - Front matter and metadata
   - Admonitions (converted to blockquotes)

2. Navigation
   - Navigation structure preserved in `_data/navigation.yml`
   - Section-based organization

3. Styling
   - CSS files preserved
   - Basic responsive design

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
- Navigation settings 
- Build options
- Plugin configurations

## Theme Customization

You can customize the site appearance by modifying:
- `assets/css/main.css` for overall styles
- `_layouts/` for page layouts
- `_includes/` for components

## Notes

- MkDocs admonitions (`!!! note`) have been converted to blockquotes
- Some advanced MkDocs features might require further customization
- The folder structure follows Jekyll conventions rather than MkDocs

## Migration Details

This site was automatically migrated using the Rustyll migration tool. The following changes were made:
- Converted MkDocs YAML configuration to Jekyll YAML
- Transformed MkDocs-specific syntax to standard Markdown/Liquid
- Preserved all content and assets
- Set up equivalent navigation structure

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