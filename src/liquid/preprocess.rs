use regex::Regex;
use log::{info, debug, error};

/// Preprocesses liquid templates to fix issues with include tags containing slashes
pub fn preprocess_liquid(content: &str) -> String {
    // Create regex to find include tags with paths containing slashes
    // This regex handles paths like "components/rankings/criteria-methodology.html" and "content/devops/introducao.md"
    let include_regex = Regex::new(r#"(\{%\s*-?\s*include\s+)([a-zA-Z0-9_./-]+(?:/[a-zA-Z0-9_./-]+)*)(\s+.*?-?\s*%\}|\s*-?\s*%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile include regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });
    
    // Create regex to find include_relative tags with paths containing slashes
    let include_relative_regex = Regex::new(r#"(\{%\s*-?\s*include_relative\s+)([a-zA-Z0-9_./-]+(?:/[a-zA-Z0-9_./-]+)*)(\s+.*?-?\s*%\}|\s*-?\s*%\})"#)
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

    // Handle the specific pattern: {% capture var %}{% include path/with/slash.ext %}{% endcapture %}
    let capture_inline_include_regex = Regex::new(r#"(\{%\s*capture\s+[^%]+%\}\s*\{%\s*include\s+)([a-zA-Z0-9_./-]+/[a-zA-Z0-9_./-]+)(\s*%\}\s*\{%\s*endcapture\s*%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile capture inline include regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });

    let content = capture_inline_include_regex.replace_all(&content, |caps: &regex::Captures| {
        let prefix = &caps[1];   // {% capture var %}{% include
        let path = &caps[2];     // The path with slashes
        let suffix = &caps[3];   // %}{% endcapture %}

        // Check if the path is already quoted
        if path.starts_with('"') && path.ends_with('"') {
            // Already quoted, return as is
            format!("{}{}{}", prefix, path, suffix)
        } else {
            info!("Preprocessing capture inline include: path '{}' found with slashes", path);
            let quoted_path = format!("\"{}\"", path);
            // Return the fixed include tag with quoted path
            format!("{}{}{}", prefix, quoted_path, suffix)
        }
    }).to_string();

    // Handle any remaining include statements that weren't caught by the above patterns
    // This is a more general pattern that catches include statements with slashes anywhere
    let general_include_regex = Regex::new(r#"(\{%\s*-?\s*include\s+)([a-zA-Z0-9_./-]+/[a-zA-Z0-9_./-]+)(\s+.*?%\}|%\})"#)
        .unwrap_or_else(|e| {
            error!("Failed to compile general include regex: {}", e);
            Regex::new(r"a^").unwrap() // This will never match anything
        });

    let content = general_include_regex.replace_all(&content, |caps: &regex::Captures| {
        let prefix = &caps[1];   // The part before the path ({% include )
        let path = &caps[2];     // The path with slashes
        let suffix = &caps[3];   // The part after the path (parameters and closing %})

        // Check if the path is already quoted
        if path.starts_with('"') && path.ends_with('"') {
            // Already quoted, return as is
            format!("{}{}{}", prefix, path, suffix)
        } else {
            info!("Preprocessing general include tag: path '{}' found with slashes", path);
            let quoted_path = format!("\"{}\"", path);
            // Return the fixed include tag with quoted path
            format!("{}{}{}", prefix, quoted_path, suffix)
        }
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
    let content = protect_date_filter_formats(&content);
    let content = protect_markdown_date_formats(&content);

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

/// Protect date filter format strings from being misinterpreted
/// Wraps date filter format strings in quotes if they aren't already
fn protect_date_filter_formats(content: &str) -> String {
    lazy_static::lazy_static! {
        // Match date filter with unquoted format string
        // This regex matches: | date: format_string where format_string may contain special chars
        static ref DATE_FILTER_RE: Regex = Regex::new(
            r#"\|\s*date:\s*([^"'\s][^}|]*?)(?:\s*(?:\||}}|-}}))"#
        ).unwrap();
    }

    let result = DATE_FILTER_RE.replace_all(content, |caps: &regex::Captures| {
        let full_match = &caps[0];
        let format = &caps[1].trim();

        // If the format already starts with a quote, leave it as is
        if format.starts_with('"') || format.starts_with('\'') {
            full_match.to_string()
        } else {
            // Find the ending delimiter (|, }}, or -}})
            let end_delim = if full_match.ends_with("}}") {
                "}}"
            } else if full_match.ends_with("-}}") {
                "-}}"
            } else {
                "|"
            };

            // Wrap the format in quotes
            let quoted = format!("| date: \"{}\" {}", format, end_delim);
            debug!("Protected date format: '{}' -> '{}'", full_match, quoted);
            quoted
        }
    }).to_string();

    result
}

/// Special handling for date formats in markdown context
/// This handles the case where date formats in markdown links confuse the parser
fn protect_markdown_date_formats(content: &str) -> String {
    lazy_static::lazy_static! {
        // Match date filter in markdown context (inside [])
        static ref MD_DATE_FILTER_RE: Regex = Regex::new(
            r#"\[([^\]]*\{\{[^\}]*\|\s*date:\s*)("[^"]+"|'[^']+'|[^}]+)(\s*\}\}[^\]]*)\]"#
        ).unwrap();
    }

    MD_DATE_FILTER_RE.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let format = &caps[2];
        let suffix = &caps[3];

        // If format is not quoted, quote it
        let quoted_format = if format.starts_with('"') || format.starts_with('\'') {
            format.to_string()
        } else {
            format!("\"{}\"", format)
        };

        format!("[{}{}{}]", prefix, quoted_format, suffix)
    }).to_string()
} 