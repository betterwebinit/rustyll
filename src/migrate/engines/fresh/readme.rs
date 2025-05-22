use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

pub(super) fn generate_readme(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Main README
    let main_readme = r#"# Migrated Fresh Site

This site has been migrated from Fresh (Deno-based framework) to Rustyll.

## Directory Structure

- `_layouts/`: Template layouts (converted from Fresh components)
- `_includes/`: Reusable template fragments (converted from Fresh islands)
- `_data/`: Data files from your Fresh application
- `assets/`: CSS, JavaScript, images, and other static files
- Various content pages converted from Fresh routes

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

## Changes from Fresh

The Rustyll system has several differences from Fresh:

1. Fresh islands have been converted to static components
2. Fresh routes have been converted to static pages
3. Dynamic functionality needs to be re-implemented with static site patterns
4. TypeScript JSX components have been converted to Liquid templates
"#;
    
    write_readme(dest_dir, main_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Created main README file with migration guidance".to_string(),
    });
    
    Ok(())
} 