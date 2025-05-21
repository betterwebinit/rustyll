use std::path::Path;
use serde_yaml::Value;
use log::debug;

use crate::config::Config;
use crate::front_matter::types::FrontMatter;

/// Apply front matter defaults to a document
pub fn apply_defaults(
    front_matter: &mut FrontMatter,
    path: &Path,
    doc_type: &str,
    config: &Config
) {
    // If there are no defaults, we can return early
    if config.defaults.is_empty() {
        return;
    }
    
    debug!("Applying front matter defaults to {}", path.display());
    
    // Convert path to a string for matching
    let path_str = path.to_string_lossy().to_string();
    
    // Find matching defaults based on path and type
    for default in &config.defaults {
        let mut matches = true;
        
        // Check if the path matches
        if let Some(scope_path) = &default.scope.path {
            if !path_str.contains(scope_path) {
                matches = false;
            }
        }
        
        // Check if the type matches
        if let Some(scope_type) = &default.scope.type_ {
            if scope_type != doc_type {
                matches = false;
            }
        }
        
        // If it matches, apply the default values
        if matches {
            debug!("Found matching default for {}", path.display());
            apply_default_values(front_matter, &default.values);
        }
    }
}

/// Apply default values to front matter
fn apply_default_values(front_matter: &mut FrontMatter, values: &Value) {
    if let Value::Mapping(map) = values {
        for (key, value) in map {
            if let Value::String(key_str) = key {
                match key_str.as_str() {
                    "title" => {
                        if front_matter.title.is_none() {
                            if let Value::String(title) = value {
                                front_matter.title = Some(title.clone());
                            }
                        }
                    },
                    "layout" => {
                        if front_matter.layout.is_none() {
                            if let Value::String(layout) = value {
                                front_matter.layout = Some(layout.clone());
                            }
                        }
                    },
                    "permalink" => {
                        if front_matter.permalink.is_none() {
                            if let Value::String(permalink) = value {
                                front_matter.permalink = Some(permalink.clone());
                            }
                        }
                    },
                    "published" => {
                        if front_matter.published.is_none() {
                            if let Value::Bool(published) = value {
                                front_matter.published = Some(*published);
                            }
                        }
                    },
                    // Other fields can be added here as needed
                    _ => {
                        // For any other field, add to the custom data if not already present
                        if !front_matter.custom.contains_key(key_str) {
                            front_matter.custom.insert(key_str.clone(), value.clone());
                        }
                    }
                }
            }
        }
    }
} 