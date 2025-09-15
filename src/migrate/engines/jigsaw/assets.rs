use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Jigsaw assets...");
    }
    
    // In Jigsaw, assets are typically in source/assets
    let assets_dir = source_dir.join("source").join("assets");
    if !assets_dir.exists() {
        // Check other common locations
        let alternatives = [
            source_dir.join("source").join("_assets"),
            source_dir.join("assets"),
        ];
        
        let mut found = false;
        for alt_dir in alternatives.iter() {
            if alt_dir.exists() {
                found = true;
                migrate_asset_dir(alt_dir, dest_dir, verbose, result)?;
            }
        }
        
        if !found {
            result.warnings.push("Could not find Jigsaw assets directory".into());
        }
        
        return Ok(());
    }
    
    migrate_asset_dir(&assets_dir, dest_dir, verbose, result)?;
    
    // Look for webpack.mix.js or package.json to detect build system
    if source_dir.join("webpack.mix.js").exists() {
        migrate_laravel_mix(source_dir, dest_dir, result)?;
    } else if source_dir.join("package.json").exists() {
        result.warnings.push("Found package.json. You may need to manually adapt the build process.".into());
    }
    
    Ok(())
}

fn migrate_asset_dir(
    assets_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create destination assets directory
    let dest_assets_dir = dest_dir.join("assets");
    create_dir_if_not_exists(&dest_assets_dir)?;
    
    // Process all asset files and subdirectories
    let asset_files = WalkDir::new(assets_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file());
    
    for entry in asset_files {
        let relative_path = entry.path().strip_prefix(assets_dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        let dest_file = dest_assets_dir.join(relative_path);
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file.parent() {
            create_dir_if_not_exists(parent)?;
        }
        
        // Copy the file
        fs::copy(entry.path(), &dest_file)
            .map_err(|e| format!("Failed to copy asset file {}: {}", entry.path().display(), e))?;
        
        // Add to migration results
        result.changes.push(MigrationChange {
            file_path: format!("assets/{}", relative_path.display()),
            change_type: ChangeType::Copied,
            description: "Copied asset file".into(),
        });
    }
    
    Ok(())
}

fn migrate_laravel_mix(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a Jekyll-friendly way to handle assets
    let config_path = dest_dir.join("_config.yml");
    let mut config_content = if config_path.exists() {
        fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read _config.yml: {}", e))?
    } else {
        "# Site settings\ntitle: Migrated Site\n".to_string()
    };
    
    // Add sass configuration if it doesn't exist
    if !config_content.contains("sass:") {
        config_content.push_str("\n# Sass settings\nsass:\n  style: compressed\n");
        
        fs::write(&config_path, &config_content)
            .map_err(|e| format!("Failed to update _config.yml: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".into(),
            change_type: ChangeType::Modified,
            description: "Added Sass configuration".into(),
        });
    }
    
    // Create a README file to explain asset handling
    let assets_readme = dest_dir.join("assets").join("README.md");
    let readme_content = r#"# Migrated Assets

These assets have been migrated from a Jigsaw site that used Laravel Mix.

## Changes Required

1. Jekyll handles Sass processing differently than Laravel Mix. If you have Sass files, they will need to be placed in the `_sass` directory.
2. For JavaScript files, you may need to set up a custom build process using webpack or another tool.
3. Image and other static assets can remain in this directory.

## Original Configuration

The original site used Laravel Mix (webpack.mix.js). You may need to manually adapt your build process.

"#;
    
    fs::write(&assets_readme, readme_content)
        .map_err(|e| format!("Failed to write assets README: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "assets/README.md".into(),
        change_type: ChangeType::Created,
        description: "Created README for asset migration guidance".into(),
    });
    
    // If SCSS files exist, create a proper structure for Jekyll
    if source_dir.join("source").join("assets").join("sass").exists() || 
       source_dir.join("source").join("assets").join("scss").exists() {
        let sass_dir = dest_dir.join("_sass");
        create_dir_if_not_exists(&sass_dir)?;
        
        // Add note about Sass files
        result.warnings.push("Sass/SCSS files found. You may need to reorganize them to work with Jekyll's Sass processing.".into());
    }
    
    Ok(())
} 