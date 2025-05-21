use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};

use crate::utils::error::BoxResult;

/// Check if a path exists and is a directory
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Check if a path exists and is a file
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Create a directory and any parent directories if they don't exist
pub fn create_directory<P: AsRef<Path>>(path: P) -> BoxResult<()> {
    fs::create_dir_all(path.as_ref())?;
    Ok(())
}

/// Remove a directory and all its contents
pub fn remove_directory<P: AsRef<Path>>(path: P) -> BoxResult<()> {
    if path.as_ref().exists() && path.as_ref().is_dir() {
        fs::remove_dir_all(path.as_ref())?;
    }
    Ok(())
}

/// Read a file to string
pub fn read_file<P: AsRef<Path>>(path: P) -> BoxResult<String> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Write a string to a file, creating the file if it doesn't exist
pub fn write_file<P: AsRef<Path>>(path: P, contents: &str) -> BoxResult<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        create_directory(parent)?;
    }
    
    let mut file = fs::File::create(path.as_ref())?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

/// Copy a file from source to destination
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> BoxResult<u64> {
    // Create parent directories if they don't exist
    if let Some(parent) = to.as_ref().parent() {
        create_directory(parent)?;
    }
    
    let bytes_copied = fs::copy(from, to)?;
    Ok(bytes_copied)
}

/// List all files in a directory recursively
pub fn list_files<P: AsRef<Path>>(dir: P) -> BoxResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !dir.as_ref().is_dir() {
        return Ok(files);
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively list files in subdirectories
            let mut subdir_files = list_files(&path)?;
            files.append(&mut subdir_files);
        } else {
            files.push(path);
        }
    }
    
    Ok(files)
} 