use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

pub(super) fn write_readme_files(dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
    // Main README
    let main_readme = r#"# Migrated Cobalt Site

This site has been migrated from Cobalt to Rustyll.

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

## Changes from Cobalt

The Rustyll system is designed to be compatible with Cobalt sites, but there
may be some differences:

1. Liquid template usage is similar but may have different filters/tags
2. Front matter handling may differ slightly
3. Some advanced features may need to be reimplemented
"#;
    
    write_readme(dest_dir, main_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "README.md".to_string(),
        change_type: ChangeType::Created,
        description: "Created main README file with migration guidance".to_string(),
    });
    
    Ok(())
} 