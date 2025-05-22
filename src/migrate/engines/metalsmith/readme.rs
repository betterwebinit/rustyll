use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

pub(super) fn generate_readme(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Main README
    let main_readme = r#"# Migrated Metalsmith Site

This site has been migrated from Metalsmith to Rustyll.

## Directory Structure

- `_layouts/`: Template layouts
- `_includes/`: Reusable template fragments
- `_data/`: Data files (YAML, JSON, CSV)
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

## Changes from Metalsmith

The Rustyll system is designed to be compatible with Metalsmith sites, but there
may be some differences:

1. Metalsmith plugins have been converted to Rustyll/Jekyll equivalents
2. Template handling might differ depending on the original template engine
3. JavaScript-based processing has been replaced with Jekyll-style processing
"#;
    
    write_readme(dest_dir, main_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Created main README file with migration guidance".to_string(),
    });
    
    Ok(())
} 