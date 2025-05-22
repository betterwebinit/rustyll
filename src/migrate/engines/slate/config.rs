use std::path::{Path, PathBuf};
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Slate configuration...");
    }
    
    // Look for config.rb in source directory
    let config_path = source_dir.join("config.rb");
    
    // Default values
    let mut site_title = "Slate API Documentation".to_string();
    let mut site_description = "API documentation converted from Slate".to_string();
    let mut site_url = "".to_string();
    
    // Extract values from config.rb if it exists
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            // Try to extract site title
            if let Some(title) = extract_ruby_setting(&content, "set :title") {
                site_title = title.to_string(); // Clone the value
            }
            
            // Try to extract site description 
            if let Some(description) = extract_ruby_setting(&content, "set :description") {
                site_description = description.to_string(); // Clone the value
            }
            
            // Try to extract site URL
            if let Some(url) = extract_ruby_setting(&content, "set :url") {
                site_url = url.to_string(); // Clone the value
            }
        }
    }
    
    // Create Jekyll _config.yml
    let config_content = format!(r#"# Jekyll configuration converted from Slate

# Site settings
title: "{}"
description: "{}"
url: "{}"
baseurl: ""

# Build settings
markdown: kramdown
highlighter: rouge
permalink: pretty

# Slate-specific settings
collections:
  documentation:
    output: true
    permalink: /:collection/:path/

defaults:
  - scope:
      path: ""
      type: documentation
    values:
      layout: documentation
      
sass:
  style: compressed

plugins:
  - jekyll-seo-tag
  - jekyll-sitemap
  
exclude:
  - Gemfile
  - Gemfile.lock
  - README.md
  - CNAME
"#, 
        site_title, site_description, site_url
    );
    
    // Write the config file
    let config_file = dest_dir.join("_config.yml");
    fs::write(&config_file, config_content)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_config.yml".to_string(),
        change_type: ChangeType::Created,
        description: "Created Jekyll configuration from Slate config.rb".to_string(),
    });
    
    // Create Gemfile
    let gemfile_content = r#"source "https://rubygems.org"

# Jekyll and plugins
gem "jekyll", "~> 4.2.0"
gem "jekyll-feed", "~> 0.15.1"
gem "jekyll-seo-tag", "~> 2.7.1"

# Theme
gem "minima", "~> 2.5.1"

# Markdown processor
gem "kramdown", "~> 2.3.1"
gem "kramdown-parser-gfm", "~> 1.1.0"

# Syntax highlighting
gem "rouge", "~> 3.26.0"

# Windows and JRuby does not include zoneinfo files, so bundle the tzinfo-data gem
platforms :mingw, :x64_mingw, :mswin, :jruby do
  gem "tzinfo", "~> 1.2"
  gem "tzinfo-data"
end

# Performance-booster for watching directories on Windows
gem "wdm", "~> 0.1.1", :platforms => [:mingw, :x64_mingw, :mswin]
"#;

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

fn generate_jekyll_config(source_dir: &Path) -> Result<String, String> {
    // Try to extract info from Slate config
    let mut site_title = "Slate Documentation".to_string();
    let mut site_description = "API Documentation".to_string();
    let mut site_url = "".to_string();
    
    // Look for config.rb or similar in the source directory
    let config_path = source_dir.join("config.rb");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            // Try to extract site title
            if let Some(title) = extract_ruby_setting(&content, "set :title") {
                site_title = title.to_string();
            }
            
            // Try to extract other settings
            if let Some(desc) = extract_ruby_setting(&content, "set :description") {
                site_description = desc.to_string();
            }
            
            if let Some(url) = extract_ruby_setting(&content, "set :url") {
                site_url = url.to_string();
            }
        }
    }
    
    // Create Jekyll config content
    let config = format!(
        r#"# Jekyll configuration converted from Slate
title: "{}"
description: "{}"
url: "{}" # the base hostname & protocol for your site
baseurl: "" # the subpath of your site, e.g. /blog

# Build settings
markdown: kramdown
highlighter: rouge
theme: minima
plugins:
  - jekyll-feed
  - jekyll-seo-tag

# Collection settings
collections:
  pages:
    output: true
    permalink: /:path/

# Default front matter
defaults:
  - scope:
      path: ""
      type: "pages"
    values:
      layout: "page"
  - scope:
      path: ""
    values:
      layout: "default"

# Exclude from processing
exclude:
  - .sass-cache/
  - .jekyll-cache/
  - Gemfile
  - Gemfile.lock
  - node_modules/
  - vendor/
"#,
        site_title, site_description, site_url
    );
    
    Ok(config)
}

fn extract_ruby_setting<'a>(content: &'a str, prefix: &str) -> Option<&'a str> {
    let lines: Vec<&str> = content.lines().collect();
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with(prefix) {
            let parts: Vec<&str> = trimmed.splitn(2, ',').collect();
            if parts.len() > 1 {
                return Some(parts[1].trim().trim_matches(|c| c == '\'' || c == '"'));
            }
        }
    }
    
    None
} 