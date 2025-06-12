use std::error::Error;
use log::{warn};
use crate::front_matter::types::FrontMatter;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Parse front matter from content
pub fn parse(content: &str) -> BoxResult<FrontMatter> {
    // Check if content has front matter (starts with ---)
    if content.starts_with("---\n") || content.starts_with("---\r\n") {
        // Find the closing delimiter
        if let Some(end_pos) = content[3..].find("\n---") {
            let front_matter_str = &content[3..end_pos+3];
            
            // Parse YAML front matter
            match serde_yaml::from_str::<FrontMatter>(front_matter_str) {
                Ok(front_matter) => {
                    return Ok(front_matter);
                },
                Err(e) => {
                    warn!("Error parsing front matter: {}", e);
                    // Return default if parsing fails
                    return Ok(FrontMatter::default());
                }
            }
        }
    }
    
    // No front matter found
    Ok(FrontMatter::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_parse_front_matter() {
        let content = "---\ntitle: Test Page\nlayout: default\n---\n\nPage content here";
        let front_matter = parse(content).unwrap();
        
        assert_eq!(front_matter.title, Some("Test Page".to_string()));
        assert_eq!(front_matter.layout, Some("default".to_string()));
        assert_eq!(front_matter.permalink, None);
    }
    
    #[test]
    fn test_merge_front_matter() {
        let mut fm1 = FrontMatter {
            title: Some("Original Title".to_string()),
            layout: None,
            permalink: None,
            description: None,
            date: None,
            categories: None,
            tags: None,
            author: None,
            published: None,
            excerpt_separator: None,
            custom: {
                let mut map = HashMap::new();
                map.insert("original_field".to_string(), serde_yaml::Value::String("value".to_string()));
                map
            },
        };
        
        let fm2 = FrontMatter {
            title: Some("New Title".to_string()),
            layout: Some("default".to_string()),
            permalink: Some("/custom/url/".to_string()),
            description: None,
            date: None,
            categories: None,
            tags: None,
            author: None,
            published: None,
            excerpt_separator: None,
            custom: {
                let mut map = HashMap::new();
                map.insert("new_field".to_string(), serde_yaml::Value::String("new_value".to_string()));
                map
            },
        };
        
        fm1.merge(&fm2);
        
        // Original fields should be preserved
        assert_eq!(fm1.title, Some("Original Title".to_string()));
        assert_eq!(fm1.custom.get("original_field").unwrap().as_str().unwrap(), "value");
        
        // Missing fields should be filled
        assert_eq!(fm1.layout, Some("default".to_string()));
        assert_eq!(fm1.permalink, Some("/custom/url/".to_string()));
        assert_eq!(fm1.custom.get("new_field").unwrap().as_str().unwrap(), "new_value");
    }
} 