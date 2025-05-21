pub mod types;
pub mod structure;
pub mod utils;

// Re-export common types and functions
pub use types::DirectoryType;
pub use structure::DirectoryStructure;
pub use utils::{
    clean_destination, 
    find_files, 
    is_convertible_file
}; 