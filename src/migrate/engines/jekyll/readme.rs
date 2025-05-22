use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

impl super::JekyllMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Main README
        let main_readme = r#"# Migrated Jekyll Site

This site has been migrated from Jekyll to Rustyll.

## Directory Structure

- `_posts/`: Blog posts
- `_layouts/`: Template layouts
- `_includes/`: Reusable template fragments
- `_data/`: Data files (YAML, JSON, CSV)
- `_sass/`: Sass files for styling
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

## Changes from Jekyll

The Rustyll system is designed to be compatible with Jekyll sites, but there
may be some differences in advanced features:

1. Plugin functionality may need to be reimplemented
2. Some Liquid filters or tags might have different behavior
3. Build process and configuration options might differ
"#;
        
        write_readme(dest_dir, main_readme)?;
        
        result.changes.push(MigrationChange {
            file_path: "README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Created main README file with migration guidance".to_string(),
        });
        
        Ok(())
    }
} 