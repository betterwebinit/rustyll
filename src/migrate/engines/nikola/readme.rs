use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn generate_readme(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Path for the readme file
    let readme_path = dest_dir.join("README.md");
    
    // Content for the readme
    let readme_content = r#"# Migrated Nikola Site

This Jekyll site was automatically migrated from a Nikola site using Rustyll.

## Structure

- `_posts/` - Contains blog posts migrated from the Nikola site
- `_pages/` - Contains static pages migrated from the Nikola site
- `_layouts/` - Contains the layout templates for the site
- `_includes/` - Contains reusable parts of the site
- `assets/` - Contains styles, scripts, and images
- `_data/` - Contains data files used by the site

## Usage

This site uses Jekyll, a static site generator. To run the site locally:

1. Install Ruby and Bundler
2. Run `bundle install` to install dependencies
3. Run `bundle exec jekyll serve` to start the development server
4. Visit `http://localhost:4000` in your browser

## Migration Notes

The following Nikola components were migrated:

- Content files (markdown, reStructuredText, HTML)
- Templates and themes
- Static assets and assets
- Configuration settings (where possible)
- Taxonomies (categories, tags)
- Multilingual content (if present)

Some manual adjustments may be necessary for:

- Custom Nikola plugins
- Python-specific functionality
- Complex Mako/Jinja2 templates

## Credits

Originally generated from a Nikola site using Rustyll, a migration tool for static site generators.
"#;
    
    // Write the readme file
    fs::write(&readme_path, readme_content)
        .map_err(|e| format!("Failed to write README.md: {}", e))?;
    
    // Add to changes
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "README.md".into(),
        description: "Created README.md file".into(),
    });
    
    Ok(())
} 