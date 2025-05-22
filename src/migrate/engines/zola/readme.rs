use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

// Public function to generate readme files
pub fn generate_readme(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Main README
    let main_readme = r#"# Migrated Zola Site

This site has been migrated from Zola to Rustyll.

## Directory Structure

- `_posts/`: Blog posts (converted from Zola sections)
- `_layouts/`: Template layouts (converted from Zola templates)
- `_includes/`: Reusable template fragments (converted from Zola partials/macros)
- `_sass/`: Sass files for styling (if present)
- `assets/`: CSS, JavaScript, images, and other static files (from Zola's static directory)
- Various content pages in respective directories

## Building the Site

To build the site with Rustyll:

```
rustyll build
```

## Serving the Site Locally

To serve the site locally:

```
rustyll serve
```

## Migration Notes

Check the MIGRATION.md file for a detailed report of the migration process,
including any warnings or issues that need to be addressed.

Each directory also contains a README.md file with specific information about
the migrated content in that directory.

## Important Differences from Zola

1. Template syntax: Zola uses Tera templates, Rustyll uses Liquid templates
2. Front matter: Zola uses TOML, Rustyll uses YAML
3. Content organization: Different conventions for content structure
4. URL handling: Different path generation rules
"#;
    
    write_readme(dest_dir, main_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Created main README file with migration guidance".to_string(),
    });
    
    Ok(())
}

impl super::ZolaMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Delegate to the module function
        generate_readme(dest_dir, result)
    }
} 