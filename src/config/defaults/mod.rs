pub mod values;
pub mod front_matter;

// Export the default values
pub use values::*;

/// Default excludes - returns Option<Vec<String>> to match the expected type
pub fn default_exclude() -> Option<Vec<String>> {
    Some(vec![
        ".git".to_string(),
        ".svn".to_string(),
        ".hg".to_string(),
        "_site".to_string(),
        "vendor".to_string(),
        "node_modules".to_string(),
        ".jekyll-cache".to_string(),
        ".jekyll-metadata".to_string(),
        ".sass-cache".to_string(),
        ".rustyll-cache".to_string(),
        "Gemfile".to_string(),
        "Gemfile.lock".to_string(),
    ])
} 