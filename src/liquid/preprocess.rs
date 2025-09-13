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
    
    // Apply each preprocessor function in sequence
    let content = normalize_hyphenated_variables(&content);
    let content = convert_include_equals_to_colons(&content);
    
    // Return the preprocessed content
    debug!("Preprocessed liquid content");
    content
}

/// Convert equals signs to colons in include tag parameters
/// For example: {% include file.html param=value %} -> {% include file.html param: value %}
fn convert_include_equals_to_colons(content: &str) -> String {
    lazy_static::lazy_static! {
        // This regex finds include tags with parameters using equals signs
        static ref INCLUDE_PARAMS_RE: Regex = Regex::new(
            r#"(\{%\s*include\s+(?:"[^"]+"|'[^']+'|[^\s"']+)(?:\s+[a-zA-Z0-9_-]+))=([^%}]+%\})"#
        ).unwrap();
    }
    
    let result = INCLUDE_PARAMS_RE.replace_all(content, |caps: &regex::Captures| {
        let before_equals = &caps[1]; // {% include file.html param
        let after_equals = &caps[2];  // value %}
        
        let result = format!("{}: {}", before_equals, after_equals);
        info!("Converting include parameter: '{}={}' -> '{}'", 
              before_equals, after_equals, result);
        
        result
    }).to_string();
    
    result
}

/// Normalize hyphenated variable names to use bracket notation
/// For example: {{site.data.primary-nav-items}} -> {{site.data["primary-nav-items"]}}
fn normalize_hyphenated_variables(content: &str) -> String {
    lazy_static::lazy_static! {
        // This regex looks for variable references with hyphens that aren't already using bracket notation
        static ref HYPHEN_VAR_RE: Regex = Regex::new(
            r#"(\{\{\s*|\{\{-\s*|\|\s*)([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)\.([a-zA-Z0-9_]+-[a-zA-Z0-9_-]+)([^}|.]*)(\s*\}\}|\s*-\}\}|\s*\|)"#
        ).unwrap();
    }
    
    let result = HYPHEN_VAR_RE.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps[1];    // {{ or {{- or | 
        let base = &caps[2];      // site.data
        let hyphen_part = &caps[3]; // primary-nav-items (matched group with hyphen)
        let suffix_part = &caps[4];  // any trailing part after the hyphenated name
        let end = &caps[5];       // }} or -}} or |
        
        let bracket_format = format!("{}{}[\"{}\"]{}{}",
            prefix, base, hyphen_part, suffix_part, end);
        
        info!("Normalized hyphenated variable: {} -> {}", 
              format!("{}{}.{}{}{}", prefix, base, hyphen_part, suffix_part, end),
              bracket_format);
              
        bracket_format
    }).to_string();
    
    result
} 