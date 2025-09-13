pub mod types;
pub mod utils;
pub mod parser;
pub mod defaults;

// Re-export the most common items for convenience
pub use types::front_matter::FrontMatter;
pub use parser::parse_front_matter as parse;
pub use utils::extract_front_matter;
