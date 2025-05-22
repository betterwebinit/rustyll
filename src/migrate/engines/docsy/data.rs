use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};
use yaml_rust::{YamlLoader, YamlEmitter};
use toml;

pub(super) fn migrate_data(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy data files...");
    }

    // Create data directory
    let data_dir = dest_dir.join("_data");
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    // Look for data files in various locations
    let data_sources = [
        source_dir.join("data"),
        source_dir.join("themes/docsy/data"),
        source_dir.join("config"),
        source_dir.join("config/_default"),
    ];

    let mut found_data = false;

    for source in data_sources.iter() {
        if source.exists() {
            found_data = true;
            migrate_data_files(source, &data_dir, verbose, result)?;
        }
    }

    // Migrate menus from config files to _data/menu.yml
    migrate_menus(source_dir, &data_dir, verbose, result)?;

    // Create default data files if none found
    if !found_data {
        create_default_data_files(&data_dir, result)?;
    }

    Ok(())
}

fn migrate_data_files(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir).min_depth(1) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            
            // Process data files based on extension
            match extension {
                "toml" => migrate_toml_data(path, dest_dir, verbose, result)?,
                "yaml" | "yml" => migrate_yaml_data(path, dest_dir, verbose, result)?,
                "json" => migrate_json_data(path, dest_dir, verbose, result)?,
                _ => continue, // Skip unknown file types
            }
        }
    }

    Ok(())
}

fn migrate_toml_data(
    source_path: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read TOML data file: {}", e))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    // Convert to YAML
    let yaml_content = convert_toml_to_yaml_string(&toml_value)?;

    // Get filename without extension
    let file_stem = source_path.file_stem().unwrap().to_string_lossy();
    let dest_path = dest_dir.join(format!("{}.yml", file_stem));

    fs::write(&dest_path, yaml_content)
        .map_err(|e| format!("Failed to write YAML data file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("_data/{}.yml", file_stem),
        description: format!("Converted TOML data from {}", source_path.display()),
    });

    Ok(())
}

fn migrate_yaml_data(
    source_path: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read YAML data file: {}", e))?;

    // Just copy YAML files directly
    let file_stem = source_path.file_stem().unwrap().to_string_lossy();
    let dest_path = dest_dir.join(format!("{}.yml", file_stem));

    fs::write(&dest_path, content)
        .map_err(|e| format!("Failed to write YAML data file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("_data/{}.yml", file_stem),
        description: format!("Copied YAML data from {}", source_path.display()),
    });

    Ok(())
}

fn migrate_json_data(
    source_path: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read JSON data file: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Convert JSON to YAML
    let yaml_content = serde_yaml::to_string(&json)
        .map_err(|e| format!("Failed to convert JSON to YAML: {}", e))?;

    let file_stem = source_path.file_stem().unwrap().to_string_lossy();
    let dest_path = dest_dir.join(format!("{}.yml", file_stem));

    fs::write(&dest_path, yaml_content)
        .map_err(|e| format!("Failed to write YAML data file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: format!("_data/{}.yml", file_stem),
        description: format!("Converted JSON data from {}", source_path.display()),
    });

    Ok(())
}

fn migrate_menus(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Check for menus in config files
    let config_files = [
        source_dir.join("config.toml"),
        source_dir.join("config.yaml"),
        source_dir.join("config.yml"),
        source_dir.join("config/_default/config.toml"),
        source_dir.join("config/_default/config.yaml"),
        source_dir.join("config/_default/config.yml"),
        source_dir.join("config/_default/menus.toml"),
        source_dir.join("config/_default/menus.yaml"),
        source_dir.join("config/_default/menus.yml"),
    ];

    let mut menu_data = "# Default menu structure\nmain:\n  - name: Documentation\n    url: /docs/\n    weight: 10\n  - name: Blog\n    url: /blog/\n    weight: 20\n".to_string();
    let mut found_menu = false;

    for config_file in config_files.iter() {
        if config_file.exists() {
            let extension = config_file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            
            let extracted_menu = match extension {
                "toml" => extract_menu_from_toml(config_file)?,
                "yaml" | "yml" => extract_menu_from_yaml(config_file)?,
                _ => continue,
            };
            
            if !extracted_menu.is_empty() {
                menu_data = extracted_menu;
                found_menu = true;
                break;
            }
        }
    }

    // Write the menu data
    let dest_path = dest_dir.join("menu.yml");
    fs::write(&dest_path, menu_data)
        .map_err(|e| format!("Failed to write menu data file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/menu.yml".into(),
        description: if found_menu {
            "Migrated menu structure from config".into()
        } else {
            "Created default menu structure".into()
        },
    });

    Ok(())
}

fn extract_menu_from_toml(config_file: &Path) -> Result<String, String> {
    let content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read TOML file: {}", e))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    let mut menu_yaml = String::new();

    // Look for menus in different locations
    if let Some(menu) = toml_value.get("menu") {
        menu_yaml = convert_toml_to_yaml_string(menu)?;
    } else if let Some(menus) = toml_value.get("menus") {
        menu_yaml = convert_toml_to_yaml_string(menus)?;
    } else if let Some(params) = toml_value.get("params") {
        if let Some(ui) = params.get("ui") {
            if let Some(sidebar_menu) = ui.get("sidebar_menu_foldable") {
                if sidebar_menu.as_bool().unwrap_or(false) {
                    menu_yaml = "# Extracted sidebar menu structure\nsidebar:\n  - title: Documentation\n    links:\n      - title: Getting Started\n        url: /docs/getting-started/\n      - title: Examples\n        url: /docs/examples/\n".to_string();
                }
            }
        }
    }

    Ok(menu_yaml)
}

fn extract_menu_from_yaml(config_file: &Path) -> Result<String, String> {
    let content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read YAML file: {}", e))?;

    let docs = YamlLoader::load_from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    if docs.is_empty() {
        return Ok(String::new());
    }

    let doc = &docs[0];
    
    // Try to find menus in various locations
    if doc["menu"].is_badvalue() && doc["menus"].is_badvalue() {
        return Ok(String::new());
    }

    let menu_section = if !doc["menu"].is_badvalue() {
        &doc["menu"]
    } else {
        &doc["menus"]
    };

    // Convert the YAML to string
    let mut yaml_string = String::new();
    let mut emitter = YamlEmitter::new(&mut yaml_string);
    emitter.dump(menu_section).map_err(|e| format!("Failed to emit YAML: {}", e))?;

    // Remove the initial "---" that YamlEmitter adds
    if yaml_string.starts_with("---") {
        yaml_string = yaml_string.trim_start_matches("---").trim().to_string();
    }

    Ok(yaml_string)
}

fn convert_toml_to_yaml_string(toml_value: &toml::Value) -> Result<String, String> {
    // Convert TOML to a JSON value first
    let json_value = serde_json::to_value(toml_value)
        .map_err(|e| format!("Failed to convert TOML to JSON: {}", e))?;

    // Then convert JSON to YAML
    let yaml_content = serde_yaml::to_string(&json_value)
        .map_err(|e| format!("Failed to convert JSON to YAML: {}", e))?;

    Ok(yaml_content)
}

fn create_default_data_files(
    data_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create default sidebar data
    let sidebar_content = r#"# Default sidebar navigation
- title: Documentation
  url: /docs/
  links:
    - title: Getting Started
      url: /docs/getting-started/
    - title: Core Concepts
      url: /docs/concepts/
    - title: Examples
      url: /docs/examples/

- title: Reference
  url: /reference/
  links:
    - title: API
      url: /reference/api/
    - title: CLI
      url: /reference/cli/
"#;

    fs::write(data_dir.join("sidebar.yml"), sidebar_content)
        .map_err(|e| format!("Failed to write sidebar data: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/sidebar.yml".into(),
        description: "Created default sidebar navigation data".into(),
    });

    // Create default links data
    let links_content = r#"# Community and project links
user:
  - name: User Guide
    url: /docs/getting-started/
  - name: Examples
    url: /docs/examples/
  - name: Tutorials
    url: /docs/tutorials/

developer:
  - name: GitHub
    url: https://github.com/example/project
  - name: Documentation
    url: /docs/
"#;

    fs::write(data_dir.join("links.yml"), links_content)
        .map_err(|e| format!("Failed to write links data: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_data/links.yml".into(),
        description: "Created default links data".into(),
    });

    Ok(())
} 