use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};
use yaml_rust::{YamlLoader, YamlEmitter};
use toml;

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Docsy configuration...");
    }

    // Check for Hugo config files in order of preference
    let config_files = [
        "config.toml",
        "config.yaml",
        "config.yml",
        "config.json",
        "config/_default/config.toml",
        "config/_default/config.yaml",
        "config/_default/config.yml",
    ];

    let mut found_config = false;

    for config_file in config_files.iter() {
        let source_config = source_dir.join(config_file);
        if source_config.exists() {
            found_config = true;
            migrate_specific_config(&source_config, dest_dir, verbose, result)?;
            break;
        }
    }

    if !found_config {
        create_default_config(dest_dir, verbose, result)?;
    }

    // Create Gemfile 
    create_gemfile(dest_dir, result)?;

    Ok(())
}

fn migrate_specific_config(
    source_config: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let extension = source_config.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    
    let config_content = match extension {
        "toml" => convert_toml_to_yaml(source_config)?,
        "yaml" | "yml" => convert_yaml_to_yaml(source_config)?,
        "json" => convert_json_to_yaml(source_config)?,
        _ => create_default_config_content(),
    };

    fs::write(dest_dir.join("_config.yml"), config_content)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_config.yml".into(),
        description: format!("Converted configuration from {}", source_config.display()),
    });

    Ok(())
}

fn convert_toml_to_yaml(source_config: &Path) -> Result<String, String> {
    let content = fs::read_to_string(source_config)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    let yaml_content = format!(
        "# Converted from Docsy TOML configuration
# Original file: {}
        
title: {}
description: {}
baseurl: {}
theme: docsy-jekyll
markdown: kramdown
        
# Additional settings (migrated from Docsy)
plugins:
  - jekyll-feed
  - jekyll-seo-tag
  - jekyll-sitemap
        
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor
        
# Former Docsy settings
docsy:
  version: '{}'
",
        source_config.display(),
        toml_value.get("title").and_then(|v| v.as_str()).unwrap_or("Migrated Docsy Site"),
        toml_value.get("params").and_then(|p| p.get("description")).and_then(|v| v.as_str()).unwrap_or("A site migrated from Docsy"),
        toml_value.get("baseURL").and_then(|v| v.as_str()).unwrap_or("/"),
        toml_value.get("module").and_then(|m| m.get("hugoVersion")).and_then(|v| v.as_table()).and_then(|t| t.get("min")).and_then(|v| v.as_str()).unwrap_or("0.0.1")
    );

    Ok(yaml_content)
}

fn convert_yaml_to_yaml(source_config: &Path) -> Result<String, String> {
    let content = fs::read_to_string(source_config)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let docs = YamlLoader::load_from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    if docs.is_empty() {
        return Ok(create_default_config_content());
    }

    let yaml_content = format!(
        "# Converted from Docsy YAML configuration
# Original file: {}
        
title: {}
description: {}
baseurl: {}
theme: docsy-jekyll
markdown: kramdown
        
# Additional settings (migrated from Docsy)
plugins:
  - jekyll-feed
  - jekyll-seo-tag
  - jekyll-sitemap
        
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor
",
        source_config.display(),
        docs[0]["title"].as_str().unwrap_or("Migrated Docsy Site"),
        docs[0]["params"]["description"].as_str().unwrap_or("A site migrated from Docsy"),
        docs[0]["baseURL"].as_str().unwrap_or("/")
    );

    Ok(yaml_content)
}

fn convert_json_to_yaml(source_config: &Path) -> Result<String, String> {
    let content = fs::read_to_string(source_config)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let yaml_content = format!(
        "# Converted from Docsy JSON configuration
# Original file: {}
        
title: {}
description: {}
baseurl: {}
theme: docsy-jekyll
markdown: kramdown
        
# Additional settings (migrated from Docsy)
plugins:
  - jekyll-feed
  - jekyll-seo-tag
  - jekyll-sitemap
        
exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor
",
        source_config.display(),
        json["title"].as_str().unwrap_or("Migrated Docsy Site"),
        json["params"]["description"].as_str().unwrap_or("A site migrated from Docsy"),
        json["baseURL"].as_str().unwrap_or("/")
    );

    Ok(yaml_content)
}

fn create_default_config(
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = create_default_config_content();

    fs::write(dest_dir.join("_config.yml"), content)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_config.yml".into(),
        description: "Created default configuration".into(),
    });

    Ok(())
}

fn create_default_config_content() -> String {
    String::from(
        "# Default configuration for migrated Docsy site
title: Migrated Docsy Site
description: A site migrated from Docsy to Rustyll
baseurl: /
theme: docsy-jekyll
markdown: kramdown

# Additional settings (migrated from Docsy)
plugins:
  - jekyll-feed
  - jekyll-seo-tag
  - jekyll-sitemap

exclude:
  - Gemfile
  - Gemfile.lock
  - node_modules
  - vendor
"
    )
}

fn create_gemfile(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let content = r#"source 'https://rubygems.org'

gem 'jekyll', '~> 4.2'
gem 'webrick', '~> 1.7'

# Migrated from Docsy theme
gem 'docsy-jekyll', '~> 1.0'

group :jekyll_plugins do
  gem 'jekyll-feed', '~> 0.16'
  gem 'jekyll-seo-tag', '~> 2.7'
  gem 'jekyll-sitemap', '~> 1.4'
end
"#;

    fs::write(dest_dir.join("Gemfile"), content)
        .map_err(|e| format!("Failed to write Gemfile: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "Gemfile".into(),
        description: "Created Gemfile with Jekyll dependencies".into(),
    });

    Ok(())
} 