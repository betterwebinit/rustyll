use std::path::Path;
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
        log::info!("Migrating Metalsmith configuration...");
    }

    // Handle metalsmith.json
    if let Ok(config_content) = fs::read_to_string(source_dir.join("metalsmith.json")) {
        let rustyll_config = convert_json_to_yaml(&config_content)?;
        fs::write(
            dest_dir.join("_config.yml"),
            rustyll_config,
        ).map_err(|e| format!("Failed to write _config.yml: {}", e))?;

        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "_config.yml".into(),
            description: "Converted metalsmith.json to Rustyll _config.yml".into(),
        });
    }

    // Handle metalsmith.js if json not found
    if !source_dir.join("metalsmith.json").exists() {
        if let Ok(js_content) = fs::read_to_string(source_dir.join("metalsmith.js")) {
            let rustyll_config = convert_js_to_yaml(&js_content)?;
            fs::write(
                dest_dir.join("_config.yml"),
                rustyll_config,
            ).map_err(|e| format!("Failed to write _config.yml: {}", e))?;

            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: "_config.yml".into(),
                description: "Converted metalsmith.js to Rustyll _config.yml".into(),
            });
        }
    }

    // Handle package.json for dependencies
    if let Ok(package_content) = fs::read_to_string(source_dir.join("package.json")) {
        let dependencies = extract_dependencies(&package_content)?;
        fs::write(
            dest_dir.join("Gemfile"),
            dependencies,
        ).map_err(|e| format!("Failed to write Gemfile: {}", e))?;

        result.changes.push(MigrationChange {
            change_type: ChangeType::Created,
            file_path: "Gemfile".into(),
            description: "Created Gemfile from package.json dependencies".into(),
        });
    }

    Ok(())
}

fn convert_json_to_yaml(json_content: &str) -> Result<String, String> {
    let json_value: Value = serde_json::from_str(json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut yaml_lines = Vec::new();
    yaml_lines.push("# Converted from metalsmith.json".to_string());
    yaml_lines.push("title: Rustyll Site".to_string());
    yaml_lines.push("baseurl: \"\"".to_string());

    // Convert Metalsmith-specific configurations
    if let Some(metadata) = json_value.get("metadata") {
        if let Some(obj) = metadata.as_object() {
            for (key, value) in obj {
                yaml_lines.push(format!("{}: {}", key, value));
            }
        }
    }

    // Map Metalsmith plugins to Rustyll/Jekyll equivalents
    if let Some(plugins) = json_value.get("plugins") {
        yaml_lines.push("plugins:".to_string());
        if let Some(obj) = plugins.as_object() {
            for (plugin, _) in obj {
                match plugin.as_str() {
                    "metalsmith-markdown" => {
                        yaml_lines.push("  - jekyll-kramdown".to_string());
                    },
                    "metalsmith-layouts" => {
                        yaml_lines.push("  - jekyll-layouts".to_string());
                    },
                    "metalsmith-permalinks" => {
                        yaml_lines.push("  - jekyll-permalinks".to_string());
                    },
                    _ => {
                        // Add mappings for other common plugins
                    }
                }
            }
        }
    }

    Ok(yaml_lines.join("\n"))
}

fn convert_js_to_yaml(js_content: &str) -> Result<String, String> {
    let mut yaml_lines = Vec::new();
    yaml_lines.push("# Converted from metalsmith.js".to_string());
    yaml_lines.push("title: Rustyll Site".to_string());
    yaml_lines.push("baseurl: \"\"".to_string());

    // Basic parsing of metalsmith.js configuration
    for line in js_content.lines() {
        if line.contains(".metadata(") {
            yaml_lines.push("# Site metadata".to_string());
        } else if line.contains(".use(") {
            if let Some(plugin) = extract_plugin_name(line) {
                match plugin {
                    "markdown" => yaml_lines.push("markdown: kramdown".to_string()),
                    "layouts" => yaml_lines.push("layouts: true".to_string()),
                    "permalinks" => yaml_lines.push("permalink: pretty".to_string()),
                    _ => {}
                }
            }
        }
    }

    Ok(yaml_lines.join("\n"))
}

fn extract_plugin_name(line: &str) -> Option<&str> {
    if let Some(start) = line.find(".use(") {
        if let Some(end) = line[start..].find(")") {
            let plugin = line[start + 5..start + end].trim();
            return Some(plugin.trim_matches('"').trim_matches('\''));
        }
    }
    None
}

fn extract_dependencies(package_content: &str) -> Result<String, String> {
    let package_json: Value = serde_json::from_str(package_content)
        .map_err(|e| format!("Failed to parse package.json: {}", e))?;

    let mut gemfile_lines = Vec::new();
    gemfile_lines.push("source \"https://rubygems.org\"".to_string());
    gemfile_lines.push("".to_string());
    gemfile_lines.push("gem \"rustyll\"".to_string());

    // Map Node.js/Metalsmith dependencies to Ruby/Jekyll equivalents
    if let Some(deps) = package_json.get("dependencies") {
        if let Some(obj) = deps.as_object() {
            for (dep, _) in obj {
                match dep.as_str() {
                    "metalsmith" => {
                        gemfile_lines.push("gem \"jekyll\"".to_string());
                    },
                    "metalsmith-markdown" => {
                        gemfile_lines.push("gem \"kramdown\"".to_string());
                    },
                    "metalsmith-layouts" => {
                        gemfile_lines.push("gem \"jekyll-layouts\"".to_string());
                    },
                    "metalsmith-sass" => {
                        gemfile_lines.push("gem \"jekyll-sass-converter\"".to_string());
                    },
                    _ => {
                        // Add mappings for other common dependencies
                    }
                }
            }
        }
    }

    Ok(gemfile_lines.join("\n"))
} 