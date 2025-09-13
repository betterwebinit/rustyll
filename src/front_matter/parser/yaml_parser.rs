use std::error::Error;
use log::{warn};
use crate::front_matter::types::FrontMatter;
use serde_yaml;

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

/// Parse YAML front matter from text
pub fn parse_yaml(content: &str) -> Result<(serde_yaml::Value, String), Box<dyn Error>> {
    // Check if the content starts with front matter delimiter
    if !content.starts_with("---") {
        return Ok((serde_yaml::Value::Null, content.to_string()));
    }
    
    // Find the end of the front matter
    if let Some(end_index) = content[3..].find("---") {
        let yaml_content = &content[3..(end_index + 3)];
        let remaining_content = &content[(end_index + 6)..];
        
        // Parse the YAML content
        match serde_yaml::from_str(yaml_content) {
            Ok(yaml) => Ok((yaml, remaining_content.to_string())),
            Err(e) => Err(Box::new(e)),
        }
    } else {
        // No end delimiter found
        Ok((serde_yaml::Value::Null, content.to_string()))
    }
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
        let mut fm1 = FrontMatter::default();
        fm1.title = Some("Original Title".to_string());
        fm1.custom.insert("original_field".to_string(), serde_yaml::Value::String("value".to_string()));
        
        let mut fm2 = FrontMatter::default();
        fm2.title = Some("New Title".to_string());
        fm2.layout = Some("default".to_string());
        fm2.permalink = Some("/custom/url/".to_string());
        fm2.custom.insert("new_field".to_string(), serde_yaml::Value::String("new_value".to_string()));
        
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