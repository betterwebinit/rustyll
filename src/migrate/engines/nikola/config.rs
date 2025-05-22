use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Nikola configuration...");
    }

    // Look for conf.py in the source directory
    let conf_path = source_dir.join("conf.py");
    if !conf_path.exists() {
        result.warnings.push("No conf.py found in Nikola source.".into());
        return Ok(());
    }
    
    // Read the configuration file
    let config_content = fs::read_to_string(&conf_path)
        .map_err(|e| format!("Failed to read conf.py: {}", e))?;
    
    // Extract configuration values
    let config_values = extract_nikola_config(&config_content);
    
    // Create Jekyll _config.yml
    generate_jekyll_config(&config_values, dest_dir, result)?;
    
    // Create Gemfile
    generate_gemfile(dest_dir, result)?;
    
    Ok(())
}

fn extract_nikola_config(content: &str) -> HashMap<String, String> {
    let mut config = HashMap::new();
    
    // Extract basic site information
    extract_string_value(content, "BLOG_TITLE", &mut config);
    extract_string_value(content, "BLOG_DESCRIPTION", &mut config);
    extract_string_value(content, "BLOG_AUTHOR", &mut config);
    extract_string_value(content, "SITE_URL", &mut config);
    extract_string_value(content, "BLOG_EMAIL", &mut config);
    extract_string_value(content, "DEFAULT_LANG", &mut config);
    extract_string_value(content, "TIMEZONE", &mut config);
    
    // Extract theme information
    extract_string_value(content, "THEME", &mut config);
    
    // Extract permalink style
    extract_string_value(content, "PERMALINK_STRUCTURE", &mut config);
    
    // Extract translation information
    if let Some(langs) = extract_list_value(content, "TRANSLATIONS") {
        config.insert("translations".to_string(), langs.join(","));
    }
    
    // Extract RSS feed configuration
    if let Some(value) = extract_boolean_value(content, "GENERATE_RSS") {
        config.insert("generate_rss".to_string(), value.to_string());
    }
    
    // Extract other useful configuration
    if let Some(value) = extract_boolean_value(content, "SHOW_SOURCELINK") {
        config.insert("show_source".to_string(), value.to_string());
    }
    
    if let Some(value) = extract_boolean_value(content, "COPY_SOURCES") {
        config.insert("copy_sources".to_string(), value.to_string());
    }
    
    config
}

fn extract_string_value(content: &str, key: &str, config: &mut HashMap<String, String>) {
    let pattern = format!(r#"(?m)^{}\s*=\s*["']([^"']+)["']"#, key);
    let regex = regex::Regex::new(&pattern).unwrap();
    
    if let Some(captures) = regex.captures(content) {
        let value = captures[1].to_string();
        config.insert(key.to_lowercase(), value);
    }
}

fn extract_boolean_value(content: &str, key: &str) -> Option<bool> {
    let pattern = format!(r"(?m)^{}\s*=\s*(True|False)", key);
    let regex = regex::Regex::new(&pattern).unwrap();
    
    if let Some(captures) = regex.captures(content) {
        return Some(captures[1] == *"True");
    }
    
    None
}

fn extract_list_value(content: &str, key: &str) -> Option<Vec<String>> {
    let pattern = format!(r"(?m)^{}\s*=\s*\{{([^}}]+)}}", key);
    let regex = regex::Regex::new(&pattern).unwrap();
    
    if let Some(captures) = regex.captures(content) {
        let list_str = captures[1].to_string();
        
        // Extract individual values - this is a simplification
        let value_regex = regex::Regex::new(r#"["']([^"']+)["']"#).unwrap();
        let values: Vec<String> = value_regex.captures_iter(&list_str)
            .map(|cap| cap[1].to_string())
            .collect();
        
        return Some(values);
    }
    
    None
}

fn generate_jekyll_config(
    config_values: &HashMap<String, String>,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Set default values
    let title = config_values.get("blog_title").cloned().unwrap_or_else(|| "Migrated Nikola Site".to_string());
    let description = config_values.get("blog_description").cloned().unwrap_or_else(|| "A site migrated from Nikola".to_string());
    let author = config_values.get("blog_author").cloned().unwrap_or_default();
    let url = config_values.get("site_url").cloned().unwrap_or_else(|| "http://example.com".to_string());
    let timezone = config_values.get("timezone").cloned().unwrap_or_else(|| "UTC".to_string());
    
    // Handle multilingual settings
    let has_translations = config_values.contains_key("translations");
    let default_lang = config_values.get("default_lang").cloned().unwrap_or_else(|| "en".to_string());
    
    // Create Jekyll config
    let mut jekyll_config = format!(
        r#"# Jekyll configuration migrated from Nikola
title: "{}"
description: "{}"
author: "{}"
url: "{}" # the base hostname & protocol for your site
baseurl: "" # the subpath of your site, e.g. /blog
timezone: {}

# Build settings
markdown: kramdown
theme: minima
plugins:
  - jekyll-feed
  - jekyll-sitemap
  - jekyll-paginate
"#,
        title, description, author, url, timezone
    );
    
    // Add multilingual settings if needed
    if has_translations {
        let translations = config_values.get("translations").unwrap();
        let langs: Vec<&str> = translations.split(',').collect();
        
        jekyll_config.push_str("\n# Multilingual settings\n");
        jekyll_config.push_str(&format!("languages: [\"{}\"", default_lang));
        
        for lang in &langs {
            if *lang != default_lang {
                jekyll_config.push_str(&format!(", \"{}\"", lang));
            }
        }
        
        jekyll_config.push_str("]\n");
        jekyll_config.push_str(&format!("default_lang: \"{}\"\n", default_lang));
        jekyll_config.push_str("exclude_from_localization: [\"assets\", \"images\"]\n");
        jekyll_config.push_str("parallel_localization: true\n");
        
        // Add the polyglot plugin
        jekyll_config.push_str("  - jekyll-polyglot\n");
    }
    
    // Collection settings
    jekyll_config.push_str(r#"
# Collection settings
collections:
  pages:
    output: true
    permalink: /:path/

# Default front matter
defaults:
  - scope:
      path: ""
      type: "posts"
    values:
      layout: "post"
  - scope:
      path: "_pages"
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
  - gemfiles/
  - Gemfile
  - Gemfile.lock
  - node_modules/
  - vendor/
"#);
    
    // Write the config file
    let config_path = dest_dir.join("_config.yml");
    fs::write(&config_path, jekyll_config)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "_config.yml".into(),
        description: "Created Jekyll configuration from Nikola config".into(),
    });
    
    Ok(())
}

fn generate_gemfile(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let gemfile_content = r#"source "https://rubygems.org"

# Jekyll and plugins
gem "jekyll", "~> 4.2.0"
gem "minima", "~> 2.5"
gem "jekyll-feed", "~> 0.12"
gem "jekyll-sitemap"
gem "jekyll-paginate"

# Windows and JRuby does not include zoneinfo files, so bundle the tzinfo-data gem
# and associated library.
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
        description: "Created Gemfile for Jekyll site".into(),
    });
    
    Ok(())
} 