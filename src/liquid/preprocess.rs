use regex::Regex;
use log::{info, debug, error};

/// Preprocesses liquid templates to fix issues with include tags containing slashes
pub fn preprocess_liquid(content: &str) -> String {
    // Create regex to find include tags with paths containing slashes
    let include_regex = Regex::new(r#"(\{%\s*include\s+)([^\s"']+/[^\s"']+)(\s+.*?%\}|\s*%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile include regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });
    
    // Create regex to find include_relative tags with paths containing slashes
    let include_relative_regex = Regex::new(r#"(\{%\s*include_relative\s+)([^\s"']+/[^\s"']+)(\s+.*?%\}|\s*%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile include_relative regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });
    
    // Find and quote paths with slashes in include tags
    let content = include_regex.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps[1];   // The part before the path ({% include )
        let path = &caps[2];     // The path with slashes
        let suffix = &caps[3];   // The part after the path (parameters and closing %})
        
        info!("Preprocessing include tag: path '{}' found with slashes", path);
        let quoted_path = format!("\"{}\"", path);
        
        // Return the fixed include tag with quoted path
        format!("{}{}{}", prefix, quoted_path, suffix)
    }).to_string();
    
    // Find and quote paths with slashes in include_relative tags
    let content = include_relative_regex.replace_all(&content, |caps: &regex::Captures| {
        let prefix = &caps[1];   // The part before the path ({% include_relative )
        let path = &caps[2];     // The path with slashes
        let suffix = &caps[3];   // The part after the path (parameters and closing %})
        
        info!("Preprocessing include_relative tag: path '{}' found with slashes", path);
        let quoted_path = format!("\"{}\"", path);
        
        // Return the fixed include tag with quoted path
        format!("{}{}{}", prefix, quoted_path, suffix)
    }).to_string();
    
    // Also handle trim modifiers at the end of include paths
    let trim_regex = Regex::new(r#"(\{%\s*include\s+(?:"[^"]+"|'[^']+'|[^\s"']+))(-\s*%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile trim regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });
    
    let content = trim_regex.replace_all(&content, |caps: &regex::Captures| {
        let prefix = &caps[1];   // The part before the trim marker
        let suffix = &caps[2];   // The trim marker and closing %}
        
        info!("Preprocessing include tag with trim marker");
        
        // Move the trim marker to before the closing tag
        format!("{} -%}}", prefix)
    }).to_string();
    
    // Return the preprocessed content
    debug!("Preprocessed liquid content");
    content
} 