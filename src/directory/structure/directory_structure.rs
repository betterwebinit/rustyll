use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::directory::types::{DirectoryType, BoxResult};

/// Manages the directory structure for a Jekyll site
#[derive(Debug, Clone)]
pub struct DirectoryStructure {
    // Core directories
    pub source: PathBuf,
    pub destination: PathBuf,
    
    // Special directories
    pub layouts_dir: PathBuf,
    pub includes_dir: PathBuf,
    pub posts_dir: PathBuf,
    pub drafts_dir: PathBuf,
    pub data_dir: PathBuf,
    pub sass_dir: PathBuf,
    pub plugins_dir: PathBuf,
    
    // Theme directories (if using gem-based theme)
    pub theme_layouts_dir: Option<PathBuf>,
    pub theme_includes_dir: Option<PathBuf>,
    pub theme_sass_dir: Option<PathBuf>,
    pub theme_assets_dir: Option<PathBuf>,
    
    // Exclude/include patterns
    pub exclude_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
}

impl DirectoryStructure {
    /// Create a new directory structure from config
    pub fn from_config(config: &Config) -> Self {
        // Create source-relative path builders for common directories
        let source = &config.source;
        let destination = &config.destination;
        
        // Create structure with paths relative to source
        let layouts_dir = source.join(&config.layouts_dir);
        let includes_dir = source.join(&config.includes_dir);
        let posts_dir = source.join(&config.posts_dir);
        let drafts_dir = source.join(&config.drafts_dir);
        let data_dir = source.join(&config.data_dir);
        
        // Default directory names for sass and plugins
        let sass_dir = source.join("_sass");
        let plugins_dir = source.join("_plugins");
        
        // Theme directories (not implemented yet, would come from gem-based themes)
        let theme_layouts_dir = None;
        let theme_includes_dir = None;
        let theme_sass_dir = None;
        let theme_assets_dir = None;
        
        DirectoryStructure {
            source: config.source.clone(),
            destination: config.destination.clone(),
            layouts_dir: config.layouts_dir.clone(),
            includes_dir: config.includes_dir.clone(),
            data_dir: config.data_dir.clone(),
            posts_dir: posts_dir,
            drafts_dir: drafts_dir,
            sass_dir,
            plugins_dir,
            theme_layouts_dir,
            theme_includes_dir,
            theme_sass_dir,
            theme_assets_dir,
            exclude_patterns: config.exclude.clone().unwrap_or_default(),
            include_patterns: config.include.clone().unwrap_or_default(),
        }
    }
    
    /// Get a specific directory based on type
    pub fn get_directory(&self, dir_type: DirectoryType) -> PathBuf {
        match dir_type {
            DirectoryType::Layouts => self.layouts_dir.clone(),
            DirectoryType::Includes => self.includes_dir.clone(),
            DirectoryType::Posts => self.posts_dir.clone(),
            DirectoryType::Drafts => self.drafts_dir.clone(),
            DirectoryType::Data => self.data_dir.clone(),
            DirectoryType::Collections => self.source.clone(), // Collections are at root
            DirectoryType::Sass => self.sass_dir.clone(),
            DirectoryType::Plugins => self.plugins_dir.clone(),
            DirectoryType::Site => self.destination.clone(),
            DirectoryType::Static => self.source.clone(),
        }
    }
    
    /// Check if a path should be excluded
    pub fn is_excluded(&self, path: &Path) -> bool {
        // Skip _site directory
        if path.starts_with(&self.destination) {
            return true;
        }
        
        // Skip target directory - binary build files
        if path.starts_with(&self.source.join("target")) {
            return true;
        }
        
        // Skip excluded paths
        for exclude in &self.exclude_patterns {
            let exclude_path = self.source.join(exclude);
            if path.starts_with(&exclude_path) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if path is in a special directory like _layouts, _includes, etc.
    pub fn is_special_directory(&self, path: &Path) -> bool {
        path.starts_with(&self.layouts_dir) ||
        path.starts_with(&self.includes_dir) ||
        path.starts_with(&self.data_dir) ||
        path.starts_with(&self.plugins_dir)
    }
    
    /// Get static files in the source directory
    pub fn get_static_files(&self) -> BoxResult<Vec<(PathBuf, PathBuf)>> {
        let mut static_files = Vec::new();
        let source_path = &self.source;
        let dest_path = &self.destination;
        
        let walker = walkdir::WalkDir::new(source_path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                let path = e.path();
                !self.is_excluded(path) && !self.is_special_directory(path)
            });
            
        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // Skip directories
            if path.is_dir() {
                continue;
            }
            
            // Skip special directories
            if self.is_special_directory(path) {
                continue;
            }
            
            // Compute destination path
            let rel_path = path.strip_prefix(source_path)?;
            let dest_file = dest_path.join(rel_path);
            
            static_files.push((path.to_path_buf(), dest_file));
        }
        
        Ok(static_files)
    }
} 