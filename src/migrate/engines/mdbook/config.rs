use std::path::{Path, PathBuf};
use std::fs;
use toml;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MDBook configuration...");
    }

    // Handle book.toml
    if let Ok(config_content) = fs::read_to_string(source_dir.join("book.toml")) {
        let rustyll_config = convert_toml_to_yaml(&config_content)?;
        fs::write(
            dest_dir.join("_config.yml"),
            rustyll_config,
        ).map_err(|e| format!("Failed to write _config.yml: {}", e))?;

        result.changes.push(MigrationChange {
            change_type: ChangeType::Converted,
            file_path: "_config.yml".into(),
            description: "Converted book.toml to Rustyll _config.yml".into(),
        });
    }

    // Handle Cargo.toml for dependencies
    if let Ok(cargo_content) = fs::read_to_string(source_dir.join("Cargo.toml")) {
        let dependencies = extract_dependencies(&cargo_content)?;
        fs::write(
            dest_dir.join("Gemfile"),
            dependencies,
        ).map_err(|e| format!("Failed to write Gemfile: {}", e))?;

        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "Gemfile".into(),
            description: "Created Gemfile from Cargo.toml dependencies".into(),
        });
    }

    // Handle theme configuration
    migrate_theme_config(source_dir, dest_dir, verbose, result)?;

    Ok(())
}

fn convert_toml_to_yaml(toml_content: &str) -> Result<String, String> {
    // Parse TOML content
    let toml_value: toml::Value = toml::from_str(toml_content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    // Convert to YAML-compatible structure
    let mut yaml_config = serde_yaml::Mapping::new();

    if let Some(book) = toml_value.get("book") {
        if let Some(title) = book.get("title") {
            yaml_config.insert(
                serde_yaml::Value::String("title".into()),
                serde_yaml::Value::String(title.as_str().unwrap_or("").into()),
            );
        }
        if let Some(authors) = book.get("authors") {
            yaml_config.insert(
                serde_yaml::Value::String("authors".into()),
                serde_yaml::to_value(authors).unwrap_or_default(),
            );
        }
        if let Some(description) = book.get("description") {
            yaml_config.insert(
                serde_yaml::Value::String("description".into()),
                serde_yaml::Value::String(description.as_str().unwrap_or("").into()),
            );
        }
    }

    // Add Rustyll-specific configurations
    yaml_config.insert(
        serde_yaml::Value::String("markdown".into()),
        serde_yaml::Value::String("kramdown".into()),
    );
    yaml_config.insert(
        serde_yaml::Value::String("highlighter".into()),
        serde_yaml::Value::String("rouge".into()),
    );

    // Convert to YAML string
    serde_yaml::to_string(&yaml_config)
        .map_err(|e| format!("Failed to generate YAML: {}", e))
}

fn extract_dependencies(cargo_content: &str) -> Result<String, String> {
    let mut gemfile_lines = Vec::new();
    gemfile_lines.push("source \"https://rubygems.org\"".to_string());
    gemfile_lines.push("".to_string());
    gemfile_lines.push("gem \"rustyll\"".to_string());
    gemfile_lines.push("gem \"kramdown\"".to_string());
    gemfile_lines.push("gem \"rouge\"".to_string());

    // Parse Cargo.toml and map relevant dependencies
    if let Ok(cargo_toml) = toml::from_str::<toml::Value>(cargo_content) {
        if let Some(deps) = cargo_toml.get("dependencies") {
            if deps.get("mdbook").is_some() {
                gemfile_lines.push("gem \"jekyll-toc\"".to_string());
            }
            if deps.get("mdbook-mermaid").is_some() {
                gemfile_lines.push("gem \"jekyll-mermaid\"".to_string());
            }
            if deps.get("mdbook-katex").is_some() {
                gemfile_lines.push("gem \"jekyll-katex\"".to_string());
            }
        }
    }

    Ok(gemfile_lines.join("\n"))
}

fn migrate_theme_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let theme_dir = source_dir.join("theme");
    if theme_dir.exists() {
        // Handle custom theme configuration
        if let Ok(content) = fs::read_to_string(theme_dir.join("index.hbs")) {
            let theme_config = extract_theme_config(&content)?;
            let dest_path = dest_dir.join("_config.theme.yml");
            
            fs::write(&dest_path, theme_config)
                .map_err(|e| format!("Failed to write theme config: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: "_config.theme.yml".into(),
                description: "Created theme configuration from MDBook theme".into(),
            });
        }
    }

    Ok(())
}

fn extract_theme_config(theme_content: &str) -> Result<String, String> {
    let mut yaml_lines = Vec::new();
    yaml_lines.push("# Theme configuration converted from MDBook".to_string());
    yaml_lines.push("theme:".to_string());
    yaml_lines.push("  name: rustyll-book".to_string());
    yaml_lines.push("  syntax_highlighting: true".to_string());
    yaml_lines.push("  search: true".to_string());

    Ok(yaml_lines.join("\n"))
} 