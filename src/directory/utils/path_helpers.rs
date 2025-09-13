use std::path::{Path, PathBuf};
use log::warn;
use glob::glob;
use glob_match::glob_match;
use crate::config::Config;
use crate::directory::types::BoxResult;

/// Check if a file is a convertible file (e.g., Markdown, HTML)
pub fn is_convertible_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return ext_str == "md" || ext_str == "markdown" || ext_str == "html" || ext_str == "htm";
    }
    false
}

/// Check if a path matches a pattern
pub fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
        return glob_pattern.matches(path);
    }
    
    // Simple substring match as fallback
    path.contains(pattern)
}

/// Convert a list of patterns to a regex
pub fn patterns_to_regex(patterns: &[String]) -> String {
    let mut regex_parts = Vec::new();
    
    for pattern in patterns {
        // Escape special regex characters but handle glob wildcards
        let escaped = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        
        regex_parts.push(format!("({})", escaped));
    }
    
    regex_parts.join("|")
}

/// Convert an Option<Vec<String>> list of patterns to a regex
pub fn option_patterns_to_regex(patterns: &Option<Vec<String>>) -> String {
    if let Some(patterns) = patterns {
        patterns_to_regex(patterns)
    } else {
        String::new()
    }
}

/// Find all files matching patterns in the source directory
pub fn find_files(config: &Config, patterns: &[&str]) -> BoxResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    let source = &config.source;
    
    for pattern in patterns {
        let full_pattern = source.join(pattern).to_string_lossy().to_string();
        for entry in glob(&full_pattern)? {
            match entry {
                Ok(path) => {
                    // Skip directories and excluded files
                    if path.is_file() && !path_matches_pattern(
                        &path.to_string_lossy().to_string(), 
                        &option_patterns_to_regex(&config.exclude)
                    ) {
                        files.push(path);
                    }
                },
                Err(e) => {
                    warn!("Error matching pattern {}: {}", pattern, e);
                }
            }
        }
    }
    
    Ok(files)
}

/// Check if a path is safe to delete in safe mode
pub fn is_safe_delete_path(path: &Path, config: &Config) -> bool {
    if !config.safe_mode {
        return true; // Safe mode not enabled, any path is safe
    }
    
    // In safe mode, only allow deleting within the source or an explicit destination
    path.starts_with(&config.source) || path.starts_with(&config.destination)
}

pub fn is_excluded(path: &Path, config: &Config) -> bool {
    // First check includes (they override excludes)
    if let Some(include_patterns) = &config.include {
        for pattern in include_patterns {
            if glob_match(pattern, path.to_str().unwrap_or("")) {
                return false;
            }
        }
    }
    
    // Then check excludes
    if let Some(exclude_patterns) = &config.exclude {
        for pattern in exclude_patterns {
            if glob_match(pattern, path.to_str().unwrap_or("")) {
                return true;
            }
        }
    }
    
    false
} 