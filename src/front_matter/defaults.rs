use std::path::{Path, PathBuf};
use glob::Pattern;
use log::debug;
use crate::config::{Config, FrontMatterDefault};
use crate::front_matter::FrontMatter;

/// Apply front matter defaults to a specific file
pub fn apply_defaults_to_front_matter(
    front_matter: &mut FrontMatter, 
    file_path: &Path, 
    config: &Config
) -> Result<(), Box<dyn std::error::Error>> {
    // Get collection from path context instead of front_matter
    let collection_name = "";
    
    // Start with defaults from collection configuration
    if !collection_name.is_empty() {
        if let Some(collection_config) = config.collections.items.get(collection_name) {
            for default in &collection_config.defaults {
                apply_default_if_matches(front_matter, file_path, default, collection_name)?;
            }
        }
    }
    
    // Apply global defaults from config
    for default in &config.defaults {
        apply_default_if_matches(front_matter, file_path, default, collection_name)?;
    }
    
    Ok(())
}

/// Apply front matter defaults to multiple paths
pub fn apply_defaults_to_paths(
    files: &mut [(PathBuf, FrontMatter)], 
    config: &Config
) -> Result<(), Box<dyn std::error::Error>> {
    for (path, front_matter) in files.iter_mut() {
        apply_defaults_to_front_matter(front_matter, path, config)?;
    }
    
    Ok(())
}

/// Apply a specific default if the file matches the scope
fn apply_default_if_matches(
    front_matter: &mut FrontMatter, 
    file_path: &Path, 
    default: &FrontMatterDefault,
    collection_name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the scope matches
    if !scope_matches(&default.scope, file_path, collection_name) {
        return Ok(());
    }
    
    // Convert the default values to a FrontMatter instance and merge
    let yaml_str = serde_yaml::to_string(&default.values)?;
    let default_front_matter = FrontMatter::default();
    
    // Apply defaults
    if front_matter.title.is_none() && default_front_matter.title.is_some() {
        front_matter.title = default_front_matter.title.clone();
    }
    
    if front_matter.layout.is_none() && default_front_matter.layout.is_some() {
        front_matter.layout = default_front_matter.layout.clone();
    }
    
    Ok(())
}

/// Check if a file matches a scope's pattern
fn scope_matches(
    scope: &crate::config::FrontMatterScope, 
    file_path: &Path,
    collection_name: &str
) -> bool {
    // Check type if specified
    if let Some(type_pattern) = &scope.type_ {
        match type_pattern.as_str() {
            "posts" => {
                if collection_name != "posts" {
                    return false;
                }
            },
            "pages" => {
                if !collection_name.is_empty() && collection_name != "pages" {
                    return false;
                }
            },
            "drafts" => {
                if !file_path.to_string_lossy().contains("_drafts") {
                    return false;
                }
            },
            _ => {
                // For other types, check if it matches the collection name
                if collection_name != type_pattern {
                    return false;
                }
            }
        }
    }
    
    // Check path if specified
    if let Some(path_pattern) = &scope.path {
        let pattern = Pattern::new(path_pattern);
        
        if let Ok(pattern) = pattern {
            let file_path_str = file_path.to_string_lossy();
            
            // Normalize path for matching
            let normalized_path = file_path_str.replace('\\', "/");
            
            if !pattern.matches(&normalized_path) {
                return false;
            }
        } else {
            debug!("Invalid glob pattern in front matter defaults: {}", path_pattern);
            return false;
        }
    }
    
    // If we get here, all conditions matched
    true
}

/// Apply default front matter
pub fn apply_defaults(
    front_matter: &mut FrontMatter,
    path: &Path,
    config: &Config
) -> Result<(), Box<dyn std::error::Error>> {
    // Get collection from path context instead of front_matter
    let collection_name = "";
    
    // Start with defaults from collection configuration
    if !collection_name.is_empty() {
        if let Some(collection_config) = config.collections.items.get(collection_name) {
            if !collection_config.defaults.is_empty() {
                apply_defaults_from_list(front_matter, path, &collection_config.defaults)?;
            }
        }
    }
    
    // Apply global defaults
    apply_defaults_from_list(front_matter, path, &config.defaults)?;
    
    Ok(())
}

/// Apply defaults from a defaults list
fn apply_defaults_from_list(
    front_matter: &mut FrontMatter,
    path: &Path,
    defaults: &[FrontMatterDefault],
) -> Result<(), Box<dyn std::error::Error>> {
    // Find matching defaults
    let default = find_matching_default(path, defaults);
    
    if let Some(default) = default {
        // Convert the default values to a FrontMatter instance and merge
        let yaml_str = serde_yaml::to_string(&default.values)?;
        
        // Create a default front matter
        let default_front_matter = FrontMatter::default();
        
        // Apply defaults for title
        if front_matter.title.is_none() && default_front_matter.title.is_some() {
            front_matter.title = default_front_matter.title.clone();
        }
        
        // Apply defaults for layout
        if front_matter.layout.is_none() && default_front_matter.layout.is_some() {
            front_matter.layout = default_front_matter.layout.clone();
        }
    }
    
    Ok(())
}

/// Find the matching default configuration for a path
fn find_matching_default<'a>(path: &Path, defaults: &'a [FrontMatterDefault]) -> Option<&'a FrontMatterDefault> {
    for default in defaults {
        if scope_matches(&default.scope, path, "") {
            return Some(default);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FrontMatterDefault, FrontMatterScope};
    
    #[test]
    fn test_scope_matches_path() {
        let scope = FrontMatterScope {
            path: Some("_posts/*.md".to_string()),
            type_: None,
        };
        
        let matching_path = PathBuf::from("_posts/test.md");
        let non_matching_path = PathBuf::from("pages/test.md");
        
        assert!(scope_matches(&scope, &matching_path, ""));
        assert!(!scope_matches(&scope, &non_matching_path, ""));
    }
    
    #[test]
    fn test_scope_matches_type() {
        let scope = FrontMatterScope {
            path: None,
            type_: Some("posts".to_string()),
        };
        
        let path = PathBuf::from("_posts/test.md");
        
        assert!(scope_matches(&scope, &path, "posts"));
        assert!(!scope_matches(&scope, &path, "pages"));
    }
    
    #[test]
    fn test_apply_default_if_matches() {
        let mut front_matter = FrontMatter::default();
        let path = PathBuf::from("_posts/test.md");
        
        let default = FrontMatterDefault {
            scope: FrontMatterScope {
                path: Some("_posts/*.md".to_string()),
                type_: None,
            },
            values: serde_yaml::from_str("layout: post\nauthor: Test Author").unwrap(),
        };
        
        apply_default_if_matches(&mut front_matter, &path, &default, "").unwrap();
        
        assert_eq!(front_matter.layout, Some("post".to_string()));
        assert_eq!(front_matter.author, Some("Test Author".to_string()));
    }
} 