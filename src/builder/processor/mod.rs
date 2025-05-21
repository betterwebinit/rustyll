mod yaml;

pub use yaml::{yaml_to_liquid, json_to_liquid};

use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};

/// Check if a file is binary (non-text)
pub fn is_binary_file(path: &Path) -> bool {
    // Open the file
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 512]; // Read first 512 bytes
    
    // Read a chunk from the file
    let bytes_read = match reader.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };
    
    if bytes_read == 0 {
        return false; // Empty file is not binary
    }
    
    // Check for null bytes or control characters that aren't whitespace
    for byte in &buffer[..bytes_read] {
        if *byte == 0 || (*byte < 32 && *byte != 9 && *byte != 10 && *byte != 13) {
            return true;
        }
    }
    
    false
} 