use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

pub(super) fn generate_readme(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Main README
    let main_readme = r#"# Migrated Jigsaw Site

This site has been migrated from Jigsaw (PHP) to Rustyll.

## Directory Structure

- `_layouts/`: Template layouts migrated from Blade templates
- `_includes/`: Reusable template fragments
- `_posts/`: Blog posts and other collection items
- `_data/`: Data files converted from PHP arrays
- `assets/`: CSS, JavaScript, images, and other static files
- Various pages and content files in the root directory

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

## Changes from Jigsaw

The Rustyll system has several differences from Jigsaw:

1. Blade templates have been converted to Liquid templates
2. PHP functions and helpers have been converted to Liquid filters/tags
3. Webpack/Mix config has been replaced with Rustyll's asset pipeline
4. Collections handling is different and follows Jekyll-style conventions
"#;
    
    write_readme(dest_dir, main_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Created main README file with migration guidance".to_string(),
    });
    
    Ok(())
} 