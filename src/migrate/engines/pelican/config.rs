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
        log::info!("Migrating Pelican configuration...");
    }
    
    // Extract settings from Pelican config files
    let (site_title, site_description, site_author, site_url) = extract_settings(source_dir);
    
    // Create Jekyll _config.yml content
    let mut config_content = String::from("# Jekyll configuration converted from Pelican\n\n");
    
    // Site settings
    config_content.push_str(&format!("title: \"{}\"\n", site_title));
    config_content.push_str(&format!("description: \"{}\"\n", site_description));
    
    if !site_author.is_empty() {
        config_content.push_str(&format!("author: \"{}\"\n", site_author));
    }
    
    if !site_url.is_empty() {
        config_content.push_str(&format!("url: \"{}\"\n", site_url));
    }
    
    config_content.push_str("baseurl: \"\"\n\n");
    
    // Build settings
    config_content.push_str("# Build settings\n");
    config_content.push_str("markdown: kramdown\n");
    config_content.push_str("permalink: /:categories/:year/:month/:day/:title/\n\n");
    
    // Theme settings
    if has_custom_theme(source_dir) {
        config_content.push_str("# Theme settings\n");
        config_content.push_str("theme: pelican-theme\n\n");
    }
    
    // Pagination settings 
    config_content.push_str("# Pagination settings\n");
    config_content.push_str("paginate: 10\n");
    config_content.push_str("paginate_path: \"/page:num/\"\n\n");
    
    // Extract plugin configurations and add them
    let plugins = extract_plugins(source_dir);
    if !plugins.is_empty() {
        config_content.push_str("# Plugins\n");
        config_content.push_str("plugins:\n");
        for plugin in plugins {
            config_content.push_str(&format!("  - {}\n", plugin));
        }
        config_content.push_str("\n");
    }
    
    // Add some default excludes
    config_content.push_str("# Excludes\n");
    config_content.push_str("exclude:\n");
    config_content.push_str("  - Gemfile\n");
    config_content.push_str("  - Gemfile.lock\n");
    config_content.push_str("  - node_modules\n");
    config_content.push_str("  - vendor\n");
    
    // Write the config file
    let config_path = dest_dir.join("_config.yml");
    fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_config.yml".to_string(),
        change_type: ChangeType::Created,
        description: "Created Jekyll configuration from Pelican settings".to_string(),
    });
    
    Ok(())
}

// Helper function to extract Python string variables
fn extract_python_string<'a>(content: &'a str, var_name: &str) -> Option<&'a str> {
    let regex = regex::Regex::new(&format!(r#"(?m)^{}\s*=\s*["'](.*?)["']"#, var_name)).unwrap();
    regex.captures(content).and_then(|caps| caps.get(1)).map(|m| m.as_str())
}

// Fix the extract_settings function to clone the values
fn extract_settings(source_dir: &Path) -> (String, String, String, String) {
    // Default values
    let mut site_title = "Pelican Site".to_string();
    let mut site_description = "A site migrated from Pelican to Jekyll".to_string();
    let mut site_author = "".to_string();
    let mut site_url = "".to_string();
    
    // Extract from pelicanconf.py
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            if let Some(title) = extract_python_string(&content, "SITENAME") {
                site_title = title.to_string();
            }
            
            if let Some(desc) = extract_python_string(&content, "SITESUBTITLE") {
                site_description = desc.to_string();
            } else if let Some(desc) = extract_python_string(&content, "DESCRIPTION") {
                site_description = desc.to_string();
            }
            
            if let Some(author) = extract_python_string(&content, "AUTHOR") {
                site_author = author.to_string();
            }
        }
    }
    
    // Extract from publishconf.py
    let publishconf_path = source_dir.join("publishconf.py");
    if publishconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&publishconf_path) {
            if let Some(url) = extract_python_string(&content, "SITEURL") {
                site_url = url.to_string();
            }
        }
    }
    
    (site_title, site_description, site_author, site_url)
}

// Check if the Pelican site has a custom theme
fn has_custom_theme(source_dir: &Path) -> bool {
    // Look for a theme directory in the source
    let theme_dir = source_dir.join("theme");
    if theme_dir.exists() && theme_dir.is_dir() {
        return true;
    }
    
    // Check if theme is specified in pelicanconf.py
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            if let Some(theme) = extract_python_string(&content, "THEME") {
                return theme != "simple" && theme != "notmyidea";  // These are default themes
            }
        }
    }
    
    false
}

// Extract plugin list from Pelican configuration
fn extract_plugins(source_dir: &Path) -> Vec<String> {
    let mut plugins = Vec::new();
    
    // Check pelicanconf.py for plugin settings
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            // Look for PLUGINS list
            let plugins_regex = regex::Regex::new(r"PLUGINS\s*=\s*\[(.*?)\]").unwrap();
            if let Some(captures) = plugins_regex.captures(&content) {
                let plugins_str = &captures[1];
                
                // Extract individual plugin names
                let plugin_regex = regex::Regex::new(r#"["']([^"']+)["']"#).unwrap();
                for plugin_match in plugin_regex.captures_iter(plugins_str) {
                    plugins.push(plugin_match[1].to_string());
                }
            }
        }
    }
    
    // Map Pelican plugins to Jekyll equivalents
    let mut jekyll_plugins = Vec::new();
    for plugin in plugins {
        match plugin.as_str() {
            "sitemap" => jekyll_plugins.push("jekyll-sitemap".to_string()),
            "feed" | "atom" => jekyll_plugins.push("jekyll-feed".to_string()),
            "seo" | "meta_tags" => jekyll_plugins.push("jekyll-seo-tag".to_string()),
            "pagination" => jekyll_plugins.push("jekyll-paginate".to_string()),
            "related_posts" => jekyll_plugins.push("jekyll-related-posts".to_string()),
            "toc" | "table_of_contents" => jekyll_plugins.push("jekyll-toc".to_string()),
            "archives" => jekyll_plugins.push("jekyll-archives".to_string()),
            "redirect" => jekyll_plugins.push("jekyll-redirect-from".to_string()),
            "i18n" | "i18n_subsites" => jekyll_plugins.push("jekyll-polyglot".to_string()),
            _ => {}, // No direct mapping for other plugins
        }
    }
    
    jekyll_plugins
} 