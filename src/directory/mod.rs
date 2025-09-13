pub mod types;
pub mod structure;
pub mod utils;

// Re-export common types and functions
pub use structure::DirectoryStructure;
pub use utils::clean_destination; 