use std::path::{Path, PathBuf};
use std::fs;
use log::debug;
use std::collections::HashMap;

use crate::config::types::Config;
use crate::config::validation;
use crate::utils::error::{BoxResult, RustyllError};

/// Configuration file names to look for
const CONFIG_FILES: [&str; 3] = ["_config.yml", "_config.yaml", "_config.toml"];

/// Load site configuration from config files
pub fn load_config<P: AsRef<Path>>(
    source_dir: P, 
    config_files: Option<Vec<PathBuf>>
) -> BoxResult<Config> {
    // Start with default configuration
    let mut config = Config::default();
    
    // Load configuration from specified files or defaults
    let config_paths = match config_files {
        Some(paths) => paths,
        None => find_default_config_files(&source_dir)?,
    };
    
    if config_paths.is_empty() {
        debug!("No configuration files found, using defaults");
    } else {
        for path in config_paths {
            debug!("Loading configuration from {}", path.display());
            merge_config_file(&mut config, &path)?;
        }
    }
    
    // Set source directory if not already set
    if config.source == PathBuf::from(".") {
        config.source = source_dir.as_ref().to_path_buf();
    }
    
    // Validate the config
    validation::validate_config(&config)?;
    
    debug!("Configuration loaded: {:?}", config);
    Ok(config)
}

/// Find default configuration files
fn find_default_config_files<P: AsRef<Path>>(source_dir: P) -> BoxResult<Vec<PathBuf>> {
    let mut config_paths = Vec::new();
    for &config_file in &CONFIG_FILES {
        let config_path = source_dir.as_ref().join(config_file);
        if config_path.exists() {
            config_paths.push(config_path);
        }
    }
    
    Ok(config_paths)
}

/// Merge a configuration file into the current configuration
fn merge_config_file(config: &mut Config, config_path: &Path) -> BoxResult<()> {
    if !config_path.exists() {
        return Err(RustyllError::Config(format!(
            "Configuration file not found: {}", config_path.display()
        )).into());
    }
    
    let content = fs::read_to_string(config_path)
        .map_err(|e| RustyllError::Config(format!(
            "Failed to read configuration file {}: {}", config_path.display(), e
        )))?;
    
    // Parse based on file extension
    let file_config: Config = if let Some(ext) = config_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "yml" | "yaml" => parse_yaml_config(&content, config_path)?,
            "toml" => parse_toml_config(&content, config_path)?,
            "json" => parse_json_config(&content, config_path)?,
            _ => {
                return Err(RustyllError::Config(format!(
                    "Unsupported configuration file format: {}", ext.to_string_lossy()
                )).into());
            }
        }
    } else {
        // Assume YAML if no extension
        parse_yaml_config(&content, config_path)?
    };
    
    // Merge the configurations
    merge_configs(config, &file_config);
    
    Ok(())
}

/// Parse a YAML configuration file
fn parse_yaml_config(content: &str, path: &Path) -> BoxResult<Config> {
    // First parse the content as a generic YAML Value to get all top-level keys
    let yaml_value = serde_yaml::from_str::<serde_yaml::Value>(content)
        .map_err(|e| RustyllError::Config(format!(
            "Failed to parse YAML configuration ({}): {}", path.display(), e
        )))?;
    
    // Extract all top-level keys and values
    let mut top_level_keys = HashMap::new();
    if let serde_yaml::Value::Mapping(map) = &yaml_value {
        // Collect keys into a Vec so we can print them with debug!
        let key_list: Vec<_> = map.keys().collect();
        debug!("Top-level keys found in config: {:?}", key_list);
        
        for (key, value) in map {
            if let serde_yaml::Value::String(key_str) = key {
                // Store all top-level keys
                debug!("Processing top-level config key: {} = {:?}", key_str, value);
                
                // Skip collection-specific keys that will be handled by the normal deserialization
                // Also skip Config struct fields that are handled by serde
                if !["defaults", "exclude", "include", "plugins", "collections",
                     "source", "destination", "layouts_dir", "data_dir", "includes_dir",
                     "collections_dir", "plugins_dir", "cache_dir", "base_url", "baseurl",
                     "title", "description", "repository", "safe_mode", "markdown_ext",
                     "markdown_extensions", "markdown_config", "keep_files", "posts_dir",
                     "drafts_dir", "url", "highlighter", "permalink", "site_data",
                     "incremental", "build_report", "show_drafts", "future", "unpublished",
                     "limit_posts", "lsi", "encoding", "timezone", "excerpt_separator",
                     "paginate", "paginate_path", "kramdown", "liquid", "jekyll", "server",
                     "strict_front_matter", "category_dir", "tag_dir", "liquid_config",
                     "sass", "webrick", "quiet", "verbose", "trace", "strict_variables"].contains(&key_str.as_str()) {
                    debug!("Adding key '{}' to top_level_keys", key_str);
                    top_level_keys.insert(key_str.clone(), value.clone());
                } else {
                    debug!("Skipping special key: {}", key_str);
                }
            }
        }
    }
    
    // Then parse into the Config struct
    let mut config: Config = serde_yaml::from_value(yaml_value.clone())
        .map_err(|e| RustyllError::Config(format!(
            "Failed to parse YAML configuration ({}): {}", path.display(), e
        )))?;

    // Special handling for author field - extract it directly from YAML
    if let serde_yaml::Value::Mapping(map) = &yaml_value {
        if let Some(author_value) = map.get(&serde_yaml::Value::String("author".to_string())) {
            config.site_data.author = Some(author_value.clone());
        }
    }
    
    // Add all top-level keys to site_data.custom to make them available in templates
    debug!("Adding top-level keys to site_data.custom:");
    for (key, value) in top_level_keys {
        debug!("  - {}: {:?}", key, value);
        if !config.site_data.custom.contains_key(&key) {
            config.site_data.custom.insert(key.clone(), value.clone());
            // Special handling for author field - also add to site_data.author
            if key == "author" {
                config.site_data.author = Some(value);
            }
        } else {
            debug!("    (already exists in site_data.custom)");
        }
    }
    
    // Print all site_data.custom keys for debugging
    debug!("Final site_data.custom keys: {:?}", config.site_data.custom.keys().collect::<Vec<_>>());
    
    Ok(config)
}

/// Parse a TOML configuration file
fn parse_toml_config(content: &str, path: &Path) -> BoxResult<Config> {
    toml::from_str(content)
        .map_err(|e| RustyllError::Config(format!(
            "Failed to parse TOML configuration ({}): {}", path.display(), e
        )).into())
}

/// Parse a JSON configuration file
fn parse_json_config(content: &str, path: &Path) -> BoxResult<Config> {
    serde_json::from_str(content)
        .map_err(|e| RustyllError::Config(format!(
            "Failed to parse JSON configuration ({}): {}", path.display(), e
        )).into())
}

/// Merge two configurations
fn merge_configs(target: &mut Config, source: &Config) {
    // Only override non-default values from source config
    if source.source != crate::config::defaults::default_source() {
        target.source = source.source.clone();
    }
    
    if source.destination != crate::config::defaults::default_destination() {
        target.destination = source.destination.clone();
    }
    
    if source.layouts_dir != crate::config::defaults::default_layouts_dir() {
        target.layouts_dir = source.layouts_dir.clone();
    }
    
    if source.includes_dir != crate::config::defaults::default_includes_dir() {
        target.includes_dir = source.includes_dir.clone();
    }
    
    if source.data_dir != crate::config::defaults::default_data_dir() {
        target.data_dir = source.data_dir.clone();
    }
    
    if source.base_url != crate::config::defaults::default_base_url() {
        target.base_url = source.base_url.clone();
    }
    
    if source.title != crate::config::defaults::default_site_title() {
        target.title = source.title.clone();
    }
    
    if source.description != crate::config::defaults::default_site_description() {
        target.description = source.description.clone();
    }
    
    // Merge repository field if set
    if source.repository.is_some() {
        debug!("Merging repository: {:?}", source.repository);
        target.repository = source.repository.clone();
    }
    
    // Boolean flags are simply set if they're true in the source
    if source.safe_mode {
        target.safe_mode = true;
    }
    
    // Merge collections
    if !source.collections.items.is_empty() {
        for (key, value) in &source.collections.items {
            target.collections.items.insert(key.clone(), value.clone());
        }
    }
    
    // Merge arrays if they're not empty in the source
    if !source.defaults.is_empty() {
        target.defaults = source.defaults.clone();
    }
    
    if let Some(exclude) = &source.exclude {
        if !exclude.is_empty() {
            target.exclude = source.exclude.clone();
        }
    }
    
    if let Some(include) = &source.include {
        if !include.is_empty() {
            target.include = source.include.clone();
        }
    }
    
    if !source.plugins.is_empty() {
        target.plugins = source.plugins.clone();
    }
    
    // Merge site_data fields
    if let Some(title) = &source.site_data.title {
        target.site_data.title = Some(title.clone());
    }
    
    if let Some(description) = &source.site_data.description {
        target.site_data.description = Some(description.clone());
    }
    
    if let Some(author) = &source.site_data.author {
        target.site_data.author = Some(author.clone());
    }
    
    // Merge custom site_data fields (like repository, etc.)
    for (key, value) in &source.site_data.custom {
        debug!("Merging custom config value: {} = {:?}", key, value);
        target.site_data.custom.insert(key.clone(), value.clone());
    }
    
    // Merge URL if set
    if source.url.is_some() {
        target.url = source.url.clone();
    }
    
    // Merge other config fields if they're set
    if !source.permalink.is_empty() && source.permalink != "date" {
        target.permalink = source.permalink.clone();
    }
    
    if !source.highlighter.is_empty() && source.highlighter != "rouge" {
        target.highlighter = source.highlighter.clone();
    }
} 