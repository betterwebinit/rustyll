use std::path::Path;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, write_readme};

impl super::HugoMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Main README
        let main_readme = r#"# Migrated Hugo Site

This site has been migrated from Hugo to Rustyll.

## Directory Structure

- `_posts/`: Blog posts
- `_layouts/`: Template layouts
- `_includes/`: Reusable template fragments (converted from Hugo partials)
- `_data/`: Data files (YAML, JSON, CSV)
- `assets/`: Asset files
- Various static files in root directory

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

## Important Differences from Hugo

1. Template syntax: Hugo uses Go templates, Rustyll uses Liquid templates
2. Front matter: Variable names and structure may differ
3. Content organization: Different conventions for content structure
4. Asset handling: Different asset pipeline approach
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