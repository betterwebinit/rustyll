use std::path::{Path, PathBuf};
use std::fs;
use serde_json::Value;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Jigsaw configuration...");
    }

    // Handle config.php
    if let Ok(config_content) = fs::read_to_string(source_dir.join("config.php")) {
        let rustyll_config = convert_php_config_to_yaml(&config_content)?;
        fs::write(
            dest_dir.join("_config.yml"),
            rustyll_config,
        ).map_err(|e| format!("Failed to write _config.yml: {}", e))?;

        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "_config.yml".into(),
            description: "Converted Jigsaw config.php to Rustyll _config.yml".into(),
        });
    }

    // Handle composer.json for dependencies
    if let Ok(composer_content) = fs::read_to_string(source_dir.join("composer.json")) {
        if let Ok(composer_json) = serde_json::from_str::<Value>(&composer_content) {
            let dependencies = extract_dependencies(&composer_json)?;
            fs::write(
                dest_dir.join("Gemfile"),
                dependencies,
            ).map_err(|e| format!("Failed to write Gemfile: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: "Gemfile".into(),
                description: "Created Gemfile from composer.json dependencies".into(),
            });
        }
    }

    // Handle environment-specific configs
    migrate_env_configs(source_dir, dest_dir, verbose, result)?;

    Ok(())
}

fn convert_php_config_to_yaml(php_content: &str) -> Result<String, String> {
    // Extract configuration values from PHP array syntax
    let mut yaml_lines = Vec::new();
    yaml_lines.push("# Converted from Jigsaw config.php".to_string());
    yaml_lines.push("title: Rustyll Site".to_string());
    yaml_lines.push("baseurl: \"\"".to_string());

    // Parse PHP array syntax and convert to YAML
    for line in php_content.lines() {
        if line.contains("=>") {
            let parts: Vec<&str> = line.split("=>").collect();
            if parts.len() == 2 {
                let key = parts[0].trim().trim_matches('\'').trim_matches('"');
                let value = parts[1].trim().trim_matches(';').trim_matches('\'').trim_matches('"');
                yaml_lines.push(format!("{}: {}", key, value));
            }
        }
    }

    Ok(yaml_lines.join("\n"))
}

fn extract_dependencies(composer_json: &Value) -> Result<String, String> {
    let mut gemfile_lines = Vec::new();
    gemfile_lines.push("source \"https://rubygems.org\"".to_string());
    gemfile_lines.push("".to_string());
    gemfile_lines.push("gem \"rustyll\"".to_string());

    // Map PHP/Composer dependencies to Ruby/Jekyll equivalents
    if let Some(deps) = composer_json.get("require") {
        if let Some(obj) = deps.as_object() {
            for (dep, _) in obj {
                match dep.as_str() {
                    "tightenco/jigsaw" => {
                        gemfile_lines.push("gem \"jekyll\"".to_string());
                    },
                    "symfony/yaml" => {
                        gemfile_lines.push("gem \"yaml\"".to_string());
                    },
                    _ => {
                        // Add other relevant dependency mappings
                    }
                }
            }
        }
    }

    Ok(gemfile_lines.join("\n"))
}

fn migrate_env_configs(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Handle environment-specific config files
    let env_configs = ["config.production.php", "config.local.php"];
    
    for config_file in env_configs.iter() {
        if let Ok(content) = fs::read_to_string(source_dir.join(config_file)) {
            let env_name = config_file
                .strip_prefix("config.")
                .and_then(|s| s.strip_suffix(".php"))
                .unwrap_or("production");
            
            let yaml_content = convert_php_config_to_yaml(&content)?;
            let dest_path = dest_dir.join(format!("_config.{}.yml", env_name));
            
            fs::write(&dest_path, yaml_content)
                .map_err(|e| format!("Failed to write {}: {}", dest_path.display(), e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_config.{}.yml", env_name),
                description: format!("Converted {} to Rustyll environment config", config_file),
            });
        }
    }

    Ok(())
} 