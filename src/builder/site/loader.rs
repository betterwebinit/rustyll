use std::collections::HashMap;
use std::fs;
use log::debug;

use crate::directory::DirectoryStructure;
use crate::builder::types::BoxResult;
use crate::front_matter::FrontMatter;

/// Layout information including content and metadata
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub content: String,
    pub front_matter: FrontMatter,
}

/// Load layouts from the layouts directory
pub fn load_layouts(dirs: &DirectoryStructure) -> BoxResult<HashMap<String, LayoutInfo>> {
    let mut layouts = HashMap::new();
    
    // First check the site layouts
    if dirs.layouts_dir.exists() {
        for entry in fs::read_dir(&dirs.layouts_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let layout_name = path.file_stem().unwrap().to_string_lossy().to_string();
                
                // Read the layout file
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        // Extract front matter from layout if it exists
                        let (front_matter, processed_content) = if content.starts_with("---") {
                            // Parse front matter and extract both front matter and content
                            match crate::front_matter::utils::extract_front_matter(&content) {
                                Ok((front_matter, body)) => (front_matter, body),
                                Err(_) => (FrontMatter::default(), content), // If parsing fails, use original content
                            }
                        } else {
                            (FrontMatter::default(), content)
                        };

                        let layout_info = LayoutInfo {
                            content: processed_content,
                            front_matter,
                        };
                        layouts.insert(layout_name, layout_info);
                    },
                    Err(e) => {
                        debug!("Error reading layout file {}: {}", file_name, e);
                    }
                }
            }
        }
    }
    
    // Check theme layouts if they exist
    if let Some(theme_layouts_dir) = &dirs.theme_layouts_dir {
        if theme_layouts_dir.exists() {
            for entry in fs::read_dir(theme_layouts_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let layout_name = path.file_stem().unwrap().to_string_lossy().to_string();
                    
                    // Only add if not already defined in site layouts
                    if !layouts.contains_key(&layout_name) {
                        // Read the layout file
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                // Extract front matter from layout if it exists
                                let (front_matter, processed_content) = if content.starts_with("---") {
                                    // Parse front matter and extract both front matter and content
                                    match crate::front_matter::utils::extract_front_matter(&content) {
                                        Ok((front_matter, body)) => (front_matter, body),
                                        Err(_) => (FrontMatter::default(), content), // If parsing fails, use original content
                                    }
                                } else {
                                    (FrontMatter::default(), content)
                                };

                                let layout_info = LayoutInfo {
                                    content: processed_content,
                                    front_matter,
                                };
                                layouts.insert(layout_name, layout_info);
                            },
                            Err(e) => {
                                debug!("Error reading theme layout file {}: {}", file_name, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(layouts)
}

/// Load includes from the _includes directory
pub fn load_includes(dirs: &DirectoryStructure) -> BoxResult<HashMap<String, String>> {
    let mut includes = HashMap::new();
    
    // First check site includes
    if dirs.includes_dir.exists() {
        load_includes_from_dir(&dirs.includes_dir, &mut includes, "")?;
    }
    
    // Then check theme includes if they exist
    if let Some(theme_includes_dir) = &dirs.theme_includes_dir {
        if theme_includes_dir.exists() {
            // Only add if not already defined in site includes
            load_includes_from_dir(theme_includes_dir, &mut includes, "")?;
        }
    }
    
    Ok(includes)
}

/// Helper to load includes from a directory, with subdirectory support
fn load_includes_from_dir(dir: &std::path::Path, includes: &mut HashMap<String, String>, prefix: &str) -> BoxResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let include_name = if prefix.is_empty() {
                file_name.to_string()
            } else {
                format!("{}/{}", prefix, file_name)
            };
            
            // Only add if not already defined
            if !includes.contains_key(&include_name) {
                // Read the include file
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        includes.insert(include_name, content);
                    },
                    Err(e) => {
                        debug!("Error reading include file {}: {}", file_name, e);
                    }
                }
            }
        } else if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            let new_prefix = if prefix.is_empty() {
                dir_name.to_string()
            } else {
                format!("{}/{}", prefix, dir_name)
            };
            
            load_includes_from_dir(&path, includes, &new_prefix)?;
        }
    }
    
    Ok(())
} 