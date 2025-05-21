use std::error::Error;
use crate::front_matter::types::FrontMatter;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Extract content without front matter
pub fn extract_content(content: &str) -> String {
    // Check if content starts with ---
    if !has_front_matter(content) {
        return content.to_string();
    }
    
    // Find the second --- marker
    if let Some(end_pos) = content[3..].find("---") {
        // Skip past the second --- and any following newlines
        let start_pos = 3 + end_pos + 3;
        if start_pos < content.len() {
            return content[start_pos..].trim_start().to_string();
        }
    }
    
    // If we can't find the second marker, just return the original content
    content.to_string()
}

/// Check if content has front matter
pub fn has_front_matter(content: &str) -> bool {
    content.trim_start().starts_with("---")
}

/// Extract front matter and content
pub fn extract_front_matter(content: &str) -> BoxResult<(FrontMatter, String)> {
    if !has_front_matter(content) {
        return Ok((FrontMatter::default(), content.to_string()));
    }
    
    // Find the second --- marker
    if let Some(end_pos) = content[3..].find("---") {
        let yaml_content = &content[3..3 + end_pos].trim();
        
        // Parse the YAML content
        match serde_yaml::from_str::<FrontMatter>(yaml_content) {
            Ok(front_matter) => {
                // Extract the content (skipping past the second ---)
                let content_start = 3 + end_pos + 3;
                let content = if content_start < content.len() {
                    content[content_start..].trim_start().to_string()
                } else {
                    String::new()
                };
                
                Ok((front_matter, content))
            },
            Err(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Error parsing front matter: {}", e)
            ))),
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Malformed front matter: missing closing delimiter"
        )))
    }
}

/// Extract only the front matter without the content
pub fn extract_front_matter_only(content: &str) -> BoxResult<FrontMatter> {
    let (front_matter, _) = extract_front_matter(content)?;
    Ok(front_matter)
} 