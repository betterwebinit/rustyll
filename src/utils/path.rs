use std::path::{Path, PathBuf};

/// Normalize a path, resolving ".." and "." components
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut result = PathBuf::new();
    
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                // Go up one level unless we're at the root
                if !result.as_os_str().is_empty() {
                    result.pop();
                }
            },
            std::path::Component::CurDir => {
                // Skip "." components
            },
            _ => {
                // Add other components
                result.push(component);
            }
        }
    }
    
    result
}

/// Get file extension as a string
pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string())
}

/// Get file name without extension
pub fn get_stem<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
}

/// Check if a path has a specific extension
pub fn has_extension<P: AsRef<Path>>(path: P, ext: &str) -> bool {
    get_extension(path)
        .map_or(false, |e| e == ext)
}

/// Make a path relative to another path
pub fn make_relative<P: AsRef<Path>, B: AsRef<Path>>(path: P, base: B) -> Option<PathBuf> {
    let path = path.as_ref();
    let base = base.as_ref();
    
    let path = normalize_path(path);
    let base = normalize_path(base);
    
    // Simple implementation of path difference
    if let (Some(path_str), Some(base_str)) = (path.to_str(), base.to_str()) {
        if path_str.starts_with(base_str) {
            let relative = path_str
                .trim_start_matches(base_str)
                .trim_start_matches(['/', '\\']);
            return Some(PathBuf::from(relative));
        }
    }
    
    None
}

/// Convert a URI path to a filesystem path
pub fn uri_to_path<P: AsRef<Path>>(root: P, uri: &str) -> PathBuf {
    let mut path = PathBuf::from(root.as_ref());
    
    // Remove leading slash if present
    let uri = uri.trim_start_matches('/');
    
    path.push(uri);
    path
}

/// Join multiple path components
pub fn join<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
    let mut result = PathBuf::from(base.as_ref());
    result.push(path);
    result
}