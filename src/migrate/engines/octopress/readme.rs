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
    let readme_content = r#"# Migrated Octopress Site

This Jekyll site was automatically migrated from an Octopress site using Rustyll.

## Structure

The site has been updated to use modern Jekyll structure:

- `_posts/` - Contains blog posts migrated from the Octopress site
- `_pages/` - Contains static pages migrated from the Octopress site
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

The following Octopress components were migrated:

- Blog posts and pages
- Layouts and templates (converted from Octopress/Jekyll hybrid format to standard Jekyll)
- Styles (Sass/CSS)
- Configuration settings
- Plugins where possible (some may require modern Jekyll alternatives)
- Assets and includes

Some manual adjustments may be necessary for:

- Custom Octopress plugins or Ruby helpers
- Third-party integrations
- Complex Liquid templates specific to Octopress

## About Octopress

Octopress was a popular blogging framework built on top of Jekyll that added various features and conventions. This site has been updated to a modern Jekyll site while preserving the content and design of the original Octopress site.

## Credits

Originally generated from an Octopress site using Rustyll, a migration tool for static site generators.
"#;
    
    // Write the readme file
    fs::write(&readme_path, readme_content)
        .map_err(|e| format!("Failed to write README.md: {}", e))?;
    
    // Add to changes
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "README.md".into(),
        description: "Created README.md file with migration information".into(),
    });
    
    Ok(())
} 