use std::path::Path;
use std::fs;
use chrono::Local;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

impl super::EleventyMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create the main README.md for the migrated site
        let readme_path = dest_dir.join("README.md");
        let date = Local::now().format("%Y-%m-%d").to_string();
        
        let readme_content = format!(r#"# Migrated Eleventy Site

This site was migrated from Eleventy to Rustyll on {}.

## Directory Structure

- `_config.yml` - Site configuration
- `_posts/` - Blog posts 
- `_includes/` - Template includes/partials
- `_data/` - Data files
- `assets/` - Static assets (CSS, JavaScript, images, etc.)

## Migration Notes

Eleventy and Rustyll have some key differences:

1. **Template Language**: Eleventy supports multiple template languages (Nunjucks, Liquid, Handlebars, etc.), while Rustyll primarily uses Liquid templates.

2. **JavaScript Configuration**: Eleventy uses JavaScript for configuration, while Rustyll uses YAML. The original Eleventy config files are preserved for reference.

3. **Data Files**: Eleventy allows JavaScript data files that export objects or functions. These need to be converted to static JSON/YAML data for Rustyll.

4. **Collections**: Eleventy has a flexible collections system, while Rustyll uses a more Jekyll-like collection approach.

5. **Filters/Shortcodes**: Custom Eleventy filters and shortcodes need to be reimplemented using Rustyll plugins.

## Automatic Migration Limitations

The automatic migration tool has copied files and made basic conversions, but you may need to:

1. Update template syntax to be compatible with Liquid
2. Convert JavaScript data files to JSON/YAML
3. Update asset references to match the new directory structure
4. Review and update collection definitions and front matter

For more information on Rustyll, visit the documentation at: https://rustyll.org
"#, date);
        
        fs::write(&readme_path, readme_content)
            .map_err(|e| format!("Failed to write README.md: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "README.md".to_string(),
            change_type: ChangeType::Created,
            description: "Created main README with migration information".to_string(),
        });
        
        Ok(())
    }
} 