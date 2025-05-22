use std::path::{Path, PathBuf};
use std::fs;
use serde_yaml;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating MkDocs configuration...");
    }

    // Look for mkdocs.yml or mkdocs.yaml in the source directory
    let config_path = if source_dir.join("mkdocs.yml").exists() {
        source_dir.join("mkdocs.yml")
    } else if source_dir.join("mkdocs.yaml").exists() {
        source_dir.join("mkdocs.yaml")
    } else {
        result.warnings.push("No MkDocs configuration file found.".into());
        return Ok(());
    };

    // Read the MkDocs config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read MkDocs config file: {}", e))?;

    // Parse the YAML
    let yaml_docs = serde_yaml::from_str::<serde_yaml::Value>(&config_content)
        .map_err(|e| format!("Failed to parse MkDocs config file: {}", e))?;

    // Extract site information
    let site_name = yaml_docs
        .get("site_name")
        .and_then(|v| v.as_str())
        .unwrap_or("MkDocs Site");

    let site_description = yaml_docs
        .get("site_description")
        .and_then(|v| v.as_str())
        .unwrap_or("A MkDocs site migrated to Jekyll");

    let site_author = yaml_docs
        .get("site_author")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Create Jekyll _config.yml
    let jekyll_config = format!(
        r#"# Jekyll configuration converted from MkDocs
title: "{}"
description: "{}"
author: "{}"
url: "" # the base hostname & protocol for your site, e.g. http://example.com
baseurl: "" # the subpath of your site, e.g. /blog

# Build settings
markdown: kramdown
theme: minima
plugins:
  - jekyll-feed
  - jekyll-sitemap

# Collection settings
collections:
  docs:
    output: true
    permalink: /:collection/:path/

defaults:
  - scope:
      path: "_docs"
      type: "docs"
    values:
      layout: "page"
  - scope:
      path: ""
      type: "posts"
    values:
      layout: "post"
  - scope:
      path: ""
    values:
      layout: "default"

# Exclude from processing
exclude:
  - .sass-cache/
  - .jekyll-cache/
  - gemfiles/
  - Gemfile
  - Gemfile.lock
  - node_modules/
  - vendor/
"#,
        site_name, site_description, site_author
    );

    // Write Jekyll config file
    let jekyll_config_path = dest_dir.join("_config.yml");
    fs::write(&jekyll_config_path, jekyll_config)
        .map_err(|e| format!("Failed to write Jekyll config file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_config.yml".into(),
        description: "Created Jekyll configuration file from MkDocs config".into(),
    });

    Ok(())
} 