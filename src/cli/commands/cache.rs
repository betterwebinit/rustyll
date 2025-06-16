use crate::cli::types::{Commands, CacheAction};
use crate::directory::utils::clean_directory;
use std::path::PathBuf;

pub async fn handle_cache_command(command: &Commands) {
    if let Commands::Cache { action } = command {
        match action {
            CacheAction::Clear { kind } => {
                clear_caches(kind.as_deref());
            }
            CacheAction::Status {} => {
                show_cache_status();
            }
        }
    }
}

const CACHE_DIRS: &[&str] = &[".jekyll-cache", ".sass-cache"];
const CACHE_FILES: &[&str] = &[".jekyll-metadata"];

fn clear_caches(kind: Option<&str>) {
    match kind {
        Some("sass") => remove_path(".sass-cache"),
        Some("jekyll") => remove_path(".jekyll-cache"),
        Some("metadata") => remove_path(".jekyll-metadata"),
        Some(_) => println!("Unknown cache type"),
        None => {
            for d in CACHE_DIRS.iter().chain(CACHE_FILES.iter()) {
                remove_path(d);
            }
        }
    }
}

fn show_cache_status() {
    for d in CACHE_DIRS.iter().chain(CACHE_FILES.iter()) {
        let path = PathBuf::from(d);
        if path.exists() {
            println!("{}: present", d);
        } else {
            println!("{}: not found", d);
        }
    }
}

fn remove_path(path: &str) {
    let p = PathBuf::from(path);
    if p.is_dir() {
        if let Err(e) = clean_directory(&p) {
            println!("Failed to clean {}: {}", path, e);
        }
    } else if p.exists() {
        if let Err(e) = std::fs::remove_file(&p) {
            println!("Failed to remove {}: {}", path, e);
        }
    }
}

