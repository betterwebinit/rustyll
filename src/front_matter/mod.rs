pub mod types;
pub mod utils;
pub mod parser;
pub mod extractor;
pub mod defaults;

// Re-export the most common items for convenience
pub use types::FrontMatter;
pub use parser::parse;
pub use utils::{extract_content, has_front_matter, extract_front_matter, extract_front_matter_only};
pub use defaults::{apply_defaults_to_paths, apply_defaults_to_front_matter};
pub use extractor::{extract_title_from_content, extract_excerpt};
