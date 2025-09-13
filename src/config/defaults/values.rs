use std::path::PathBuf;
use std::collections::HashMap;


/// Get the default permalink pattern based on collection name
pub fn get_permalink_pattern(collection_name: &str) -> String {
    match collection_name {
        "posts" => "/:categories/:year/:month/:day/:title".to_string(),
        _ => format!("/{}/{{}}:output_ext", collection_name),
    }
}

/// Default source directory
pub fn default_source() -> PathBuf {
    PathBuf::from(".")
}

/// Default destination directory
pub fn default_destination() -> PathBuf {
    PathBuf::from("_site")
}

/// Default layouts directory
pub fn default_layouts_dir() -> PathBuf {
    PathBuf::from("_layouts")
}

/// Default includes directory
pub fn default_includes_dir() -> PathBuf {
    PathBuf::from("_includes")
}

/// Default data directory
pub fn default_data_dir() -> PathBuf {
    PathBuf::from("_data")
}

/// Default site title
pub fn default_site_title() -> String {
    "Your awesome site".to_string()
}

/// Default site description
pub fn default_site_description() -> String {
    "".to_string()
}

/// Default base URL
pub fn default_base_url() -> String {
    "".to_string()
}

/// Default excluded files
pub fn default_exclude() -> Vec<String> {
    vec![
        "_config.yml".to_string(),
        "Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        "target".to_string(),
        "vendor/bundle/".to_string(),
    ]
}

/// Default safe mode
pub fn default_safe_mode() -> bool {
    false
}

/// Default collections configuration
pub fn default_collections() -> HashMap<String, crate::config::CollectionConfig> {
    let mut collections = HashMap::new();
    
    // Add posts collection by default
    let posts_config = crate::config::CollectionConfig {
        output: true,
        permalink: Some(get_permalink_pattern("posts")),
        sort_by: "date".to_string(),
        defaults: Vec::new(),
    };
    
    collections.insert("posts".to_string(), posts_config);
    
    collections
} 