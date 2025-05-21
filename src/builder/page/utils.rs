use std::path::{Path, PathBuf};

use crate::directory::DirectoryStructure;
use crate::front_matter::FrontMatter;

/// Determine the output path for a page
pub fn determine_output_path(
    input_path: &Path, 
    relative_path: &Path, 
    front_matter: &FrontMatter,
    dirs: &DirectoryStructure
) -> Option<PathBuf> {
    // If front matter has a permalink, use that
    if let Some(permalink) = &front_matter.permalink {
        let path_str = if permalink.starts_with('/') {
            permalink[1..].to_string()
        } else {
            permalink.clone()
        };
        
        // Handle index.html appending
        let output_path = if path_str.ends_with('/') {
            format!("{}index.html", path_str)
        } else {
            path_str
        };
        
        return Some(dirs.destination.join(output_path));
    }
    
    // Default behavior: preserve directory structure
    let mut output_path = dirs.destination.join(relative_path);
    
    // Special handling for markdown files - convert to HTML
    if is_markdown_file(input_path) {
        // Change extension to .html
        output_path.set_extension("html");
    }
    
    Some(output_path)
}

/// Check if a file is a markdown file based on extension
fn is_markdown_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return ext_str == "md" || ext_str == "markdown";
    }
    false
} 