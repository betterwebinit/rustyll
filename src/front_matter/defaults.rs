use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use crate::config::Config;
use crate::front_matter::FrontMatter;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Apply default front matter values to files based on path scopes
pub fn apply_defaults_to_paths<'a>(config: &Config, paths: &'a [&'a Path]) -> HashMap<&'a Path, FrontMatter> {
    let mut result = HashMap::new();
    
    // Initialize with empty front matter for all paths
    for path in paths {
        result.insert(*path, FrontMatter::new());
    }
    
    // Apply defaults in order
    for default in &config.defaults {
        // Check paths against scope
        for path in paths {
            let front_matter = result.get_mut(path).unwrap();
            
            // Check if this path matches the scope
            if path_matches_scope(path, &default.scope, config) {
                // Apply default values
                apply_defaults_to_front_matter(front_matter, &default.values);
            }
        }
    }
    
    result
}

/// Apply defaults to a single front matter
pub fn apply_defaults_to_front_matter(front_matter: &mut FrontMatter, values: &serde_yaml::Value) {
    if let serde_yaml::Value::Mapping(map) = values {
        for (key, value) in map {
            if let Some(key_str) = key.as_str() {
                match key_str {
                    "title" => {
                        if front_matter.title.is_none() {
                            if let Some(title) = value.as_str() {
                                front_matter.title = Some(title.to_string());
                            }
                        }
                    },
                    "layout" => {
                        if front_matter.layout.is_none() {
                            if let Some(layout) = value.as_str() {
                                front_matter.layout = Some(layout.to_string());
                            }
                        }
                    },
                    "permalink" => {
                        if front_matter.permalink.is_none() {
                            if let Some(permalink) = value.as_str() {
                                front_matter.permalink = Some(permalink.to_string());
                            }
                        }
                    },
                    "published" => {
                        if front_matter.published.is_none() {
                            if let Some(published) = value.as_bool() {
                                front_matter.published = Some(published);
                            }
                        }
                    },
                    _ => {
                        // Add to custom if not already present
                        let key_string = key_str.to_string();
                        if !front_matter.custom.contains_key(&key_string) {
                            front_matter.custom.insert(key_string, value.clone());
                        }
                    }
                }
            }
        }
    }
}

/// Check if a path matches a scope
fn path_matches_scope(path: &Path, scope: &crate::config::FrontMatterScope, config: &Config) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check path pattern if it exists
    if let Some(pattern) = &scope.path {
        let scope_path = pattern.trim_start_matches('/');
        if !path_str.contains(scope_path) {
            return false;
        }
    }
    
    // Check type pattern if it exists
    if let Some(type_pattern) = &scope.type_ {
        let file_type = match type_pattern.as_str() {
            "posts" => {
                let posts_dir = config.source.join(&config.posts_dir);
                let posts_path = posts_dir.to_string_lossy();
                path_str.starts_with(&*posts_path)
            },
            "drafts" => {
                let drafts_dir = config.source.join(&config.drafts_dir);
                let drafts_path = drafts_dir.to_string_lossy();
                path_str.starts_with(&*drafts_path)
            },
            "pages" => {
                // Pages are anything that's not a post, draft, or collection
                let posts_dir = config.source.join(&config.posts_dir);
                let posts_path = posts_dir.to_string_lossy();
                let drafts_dir = config.source.join(&config.drafts_dir);
                let drafts_path = drafts_dir.to_string_lossy();
                
                // Check if this is a page (not post or draft)
                !path_str.starts_with(&*posts_path) && 
                !path_str.starts_with(&*drafts_path) &&
                !is_collection_path(path, config)
            },
            // Check if it's a collection type
            collection_name => {
                // Check if this path is in the collection
                let collection_dir = if let Some(coll) = config.collections.items.get(collection_name) {
                    // Default collection directory pattern (_name)
                    config.source.join(format!("_{}", collection_name))
                } else {
                    config.source.join(format!("_{}", collection_name))
                };
                let collection_path = collection_dir.to_string_lossy();
                path_str.starts_with(&*collection_path)
            }
        };
        
        if !file_type {
            return false;
        }
    }
    
    // If we got here, all scope conditions matched
    true
}

/// Check if a path is in a collection
fn is_collection_path(path: &Path, config: &Config) -> bool {
    let path_str = path.to_string_lossy();
    
    for (name, _) in &config.collections.items {
        // Default collection directory pattern (_name)
        let collection_dir = config.source.join(format!("_{}", name));
        
        let collection_path = collection_dir.to_string_lossy();
        
        if path_str.starts_with(&*collection_path) {
            return true;
        }
    }
    
    false
} 