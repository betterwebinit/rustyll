mod builder;
mod loader;
mod processor;
mod converter;

pub use builder::build_site;
pub use loader::{load_layouts, load_includes};
pub use processor::{process_collections, process_pages};
pub use converter::{page_to_liquid, data_to_liquid};

use std::path::Path;
use crate::config::Config;

/// Check if a file is a markdown file
pub fn is_markdown_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        
        // Check against markdown extensions list from config
        for md_ext in &config.markdown_ext {
            if ext_str == md_ext.to_lowercase() {
                return true;
            }
        }
    }
    
    false
} 