mod generator;
mod parser;

pub use generator::generate_toc;
pub use parser::extract_headings;

/// Options for table of contents generation
#[derive(Debug, Clone)]
pub struct TocOptions {
    /// Minimum level to include (h1 = 1, h2 = 2, etc.)
    pub min_level: usize,
    /// Maximum level to include
    pub max_level: usize,
    /// Whether to include the page title (h1)
    pub include_title: bool,
    /// CSS class for the TOC list
    pub list_class: String,
} 