use std::path::Path;
use std::fs;
use chrono::Local;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

impl super::MiddlemanMigrator {
    pub(super) fn write_readme_files(&self, dest_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create the main README.md for the migrated site
        let readme_path = dest_dir.join("README.md");
        let date = Local::now().format("%Y-%m-%d").to_string();
        
        let readme_content = format!(r#"# Migrated Middleman Site

This site was migrated from Middleman to Rustyll on {}.

## Directory Structure

- `_config.yml` - Site configuration
- `_posts/` - Blog posts 
- `_layouts/` - Layout templates
- `_includes/` - Partial templates
- `_data/` - Data files
- `assets/` - Static assets (CSS, JavaScript, images, fonts)

## Migration Notes

Middleman and Rustyll have some key differences:

1. **Template Language**: Middleman uses ERB/HAML/Slim, while Rustyll uses Liquid templates. Template files have been converted where possible, but may need manual adjustment.

2. **Ruby Configuration**: Middleman uses Ruby for configuration (config.rb), while Rustyll uses YAML (_config.yml). The original Middleman config is preserved for reference.

3. **Asset Pipeline**: Middleman has a sophisticated asset pipeline, while Rustyll uses a simpler approach. Asset paths and helpers need to be updated.

4. **Helpers & Extensions**: Middleman Ruby helpers need to be replaced with Liquid filters or Rustyll plugins.

5. **Layout System**: Middleman's `yield` mechanic is replaced with Rustyll's `{{ content }}` variable.

## Automatic Migration Limitations

The automatic migration tool has copied files and made basic conversions, but you may need to:

1. Update ERB/HAML/Slim template syntax to be compatible with Liquid
2. Convert Ruby code in templates and data files to YAML/Liquid equivalents
3. Update asset references to match the new directory structure
4. Implement any custom helpers from Middleman as Rustyll plugins

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