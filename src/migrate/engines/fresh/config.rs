use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Fresh configuration...");
    }

    // Check for Fresh config files
    let import_map_path = source_dir.join("import_map.json");
    let dev_path = source_dir.join("dev.ts");
    let deno_config_path = source_dir.join("deno.json");
    
    // Set up default values
    let mut site_title = "Fresh Site".to_string();
    let mut site_description = "A site migrated from Fresh".to_string();

    // Extract information from deno.json if available
    if deno_config_path.exists() {
        if verbose {
            log::info!("Found deno.json configuration");
        }
        
        // Read and parse the JSON file
        let content = fs::read_to_string(&deno_config_path)
            .map_err(|e| format!("Failed to read deno.json: {}", e))?;
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                site_title = name.to_string();
            }
            
            if let Some(description) = json.get("description").and_then(|v| v.as_str()) {
                site_description = description.to_string();
            }
        }
    } else if verbose {
        log::info!("No deno.json found. Using default values.");
    }
    
    // Create Jekyll _config.yml
    let jekyll_config = format!(r#"# Jekyll configuration converted from Fresh
title: "{}"
description: "{}"
email: ""
baseurl: "" # the subpath of your site, e.g. /blog
url: "" # the base hostname & protocol for your site, e.g. http://example.com
twitter_username: ""
github_username: ""
date: {}

# Build settings
markdown: kramdown
theme: minima
plugins:
  - jekyll-feed
  - jekyll-sitemap

# Exclude from processing
exclude:
  - .sass-cache/
  - .jekyll-cache/
  - gemfiles/
  - Gemfile
  - Gemfile.lock
  - node_modules/
  - vendor/
  - fresh/
  - deno.json
  - import_map.json
  - dev.ts
  - fresh.gen.ts

# Include from processing
include:
  - _pages
  - _assets
"#, site_title, site_description, chrono::Local::now().format("%Y-%m-%d"));

    // Write Jekyll config file
    let jekyll_config_path = dest_dir.join("_config.yml");
    fs::write(&jekyll_config_path, jekyll_config)
        .map_err(|e| format!("Failed to write Jekyll config file: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_config.yml".into(),
        description: "Created Jekyll configuration from Fresh project".into(),
    });

    // Create Gemfile for Jekyll
    let gemfile_content = r#"source "https://rubygems.org"

# Jekyll and plugins
gem "jekyll", "~> 4.2.0"
gem "minima", "~> 2.5"
gem "jekyll-feed", "~> 0.12"
gem "jekyll-sitemap"

# Windows and JRuby does not include zoneinfo files, so bundle the tzinfo-data gem
# and associated library.
platforms :mingw, :x64_mingw, :mswin, :jruby do
  gem "tzinfo", "~> 1.2"
  gem "tzinfo-data"
end

# Performance-booster for watching directories on Windows
gem "wdm", "~> 0.1.1", :platforms => [:mingw, :x64_mingw, :mswin]
"#;

    // Write Gemfile
    let gemfile_path = dest_dir.join("Gemfile");
    fs::write(&gemfile_path, gemfile_content)
        .map_err(|e| format!("Failed to write Gemfile: {}", e))?;

    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "Gemfile".into(),
        description: "Created Gemfile for Jekyll project".into(),
    });

    Ok(())
} 