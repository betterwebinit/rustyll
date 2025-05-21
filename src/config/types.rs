use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::collections::hash_map;
use liquid::Object;
use liquid::model::Value;

use crate::config::defaults;
use crate::builder::processor::yaml_to_liquid;

/// Site data configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiteData {
    /// Site title
    #[serde(default)]
    pub title: Option<String>,
    
    /// Site description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Site author
    #[serde(default)]
    pub author: Option<String>,
    
    /// Custom site data
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

/// Site configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Source directory for Jekyll site
    #[serde(default = "defaults::default_source")]
    pub source: PathBuf,
    
    /// Destination directory for generated site
    #[serde(default = "defaults::default_destination")]
    pub destination: PathBuf,
    
    /// Layouts directory
    #[serde(default = "defaults::default_layouts_dir")]
    pub layouts_dir: PathBuf,
    
    /// Data directory
    #[serde(default = "defaults::default_data_dir")]
    pub data_dir: PathBuf,
    
    /// Includes directory
    #[serde(default = "defaults::default_includes_dir")]
    pub includes_dir: PathBuf,
    
    /// Base URL for the site
    #[serde(default = "defaults::default_base_url")]
    pub base_url: String,
    
    /// Site title
    #[serde(default = "defaults::default_site_title")]
    pub title: String,
    
    /// Site description
    #[serde(default = "defaults::default_site_description")]
    pub description: String,
    
    /// Repository URL (e.g., GitHub repository)
    #[serde(default)]
    pub repository: Option<String>,
    
    /// Safe mode (disables plugins and some features)
    #[serde(default = "defaults::default_safe_mode")]
    pub safe_mode: bool,
    
    /// Site-wide front matter defaults
    #[serde(default)]
    pub defaults: Vec<FrontMatterDefault>,
    
    /// Exclude patterns (glob patterns of files to exclude)
    #[serde(default = "defaults::default_exclude")]
    pub exclude: Vec<String>,
    
    /// Include patterns (glob patterns of files that would otherwise be excluded)
    #[serde(default)]
    pub include: Vec<String>,
    
    /// Collections configuration
    #[serde(default)]
    pub collections: Collections,
    
    /// Plugin settings
    #[serde(default)]
    pub plugins: Vec<String>,
    
    /// Markdown extensions
    #[serde(default = "default_markdown_extensions")]
    pub markdown_ext: Vec<String>,
    
    /// Keep files (files to not delete during clean operation)
    #[serde(default)]
    pub keep_files: Vec<String>,
    
    /// Posts directory (default "_posts")
    #[serde(default = "default_posts_dir")]
    pub posts_dir: String,
    
    /// Drafts directory (default "_drafts")
    #[serde(default = "default_drafts_dir")]
    pub drafts_dir: String,
    
    /// URL for site
    #[serde(default)]
    pub url: Option<String>,
    
    /// Highlighter setting
    #[serde(default = "default_highlighter")]
    pub highlighter: String,
    
    /// Permalink format
    #[serde(default = "default_permalink")]
    pub permalink: String,
    
    /// Site data
    #[serde(default)]
    pub site_data: SiteData,
}

/// Default front matter for specific paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatterDefault {
    /// Scope for the default (path, type)
    pub scope: FrontMatterScope,
    
    /// Values to apply
    pub values: serde_yaml::Value,
}

/// Scope for a front matter default
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatterScope {
    /// Path pattern to match
    pub path: Option<String>,
    
    /// Type of document to match (post, page, draft, etc.)
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

/// Collections configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collections {
    /// Individual collections
    #[serde(flatten)]
    pub items: HashMap<String, CollectionConfig>,
}

// Add IntoIterator implementation for &Collections to iterate over items
impl<'a> IntoIterator for &'a Collections {
    type Item = (&'a String, &'a CollectionConfig);
    type IntoIter = hash_map::Iter<'a, String, CollectionConfig>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Configuration for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Whether to output the collection files as individual pages
    #[serde(default)]
    pub output: bool,
    
    /// Permalink pattern for the collection
    pub permalink: Option<String>,
    
    /// Sort order for the collection
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    
    /// Defaults for this collection
    #[serde(default)]
    pub defaults: Vec<FrontMatterDefault>,
}

/// Default sort by field for collections
fn default_sort_by() -> String {
    "date".to_string()
}

/// Default markdown extensions
fn default_markdown_extensions() -> Vec<String> {
    vec!["md".to_string(), "markdown".to_string()]
}

/// Default posts directory
fn default_posts_dir() -> String {
    "_posts".to_string()
}

/// Default drafts directory
fn default_drafts_dir() -> String {
    "_drafts".to_string()
}

/// Default highlighter
fn default_highlighter() -> String {
    "rouge".to_string()
}

/// Default permalink format
fn default_permalink() -> String {
    "date".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            source: defaults::default_source(),
            destination: defaults::default_destination(),
            layouts_dir: defaults::default_layouts_dir(),
            includes_dir: defaults::default_includes_dir(),
            data_dir: defaults::default_data_dir(),
            base_url: defaults::default_base_url(),
            title: defaults::default_site_title(),
            description: defaults::default_site_description(),
            repository: None,
            safe_mode: defaults::default_safe_mode(),
            defaults: Vec::new(),
            exclude: defaults::default_exclude(),
            include: Vec::new(),
            collections: Collections {
                items: defaults::default_collections(),
            },
            plugins: Vec::new(),
            markdown_ext: default_markdown_extensions(),
            keep_files: Vec::new(),
            posts_dir: default_posts_dir(),
            drafts_dir: default_drafts_dir(),
            url: None,
            highlighter: default_highlighter(),
            permalink: default_permalink(),
            site_data: SiteData::default(),
        }
    }
}

impl Config {
    /// Convert config to a Liquid Object for use in templates
    pub fn to_liquid(&self) -> Object {
        let mut obj = Object::new();
        
        // Log site_data.custom keys
        log::debug!("site_data.custom keys: {:?}", self.site_data.custom.keys().collect::<Vec<_>>());
        
        // Add all config values to the object
        
        // Basic settings
        if let Some(title) = &self.site_data.title {
            obj.insert("title".into(), Value::scalar(title.clone()));
        } else {
            obj.insert("title".into(), Value::scalar(self.title.clone()));
        }
        
        if let Some(description) = &self.site_data.description {
            obj.insert("description".into(), Value::scalar(description.clone()));
        } else if !self.description.is_empty() {
            obj.insert("description".into(), Value::scalar(self.description.clone()));
        }
        
        if let Some(author) = &self.site_data.author {
            obj.insert("author".into(), Value::scalar(author.clone()));
        }
        
        // Repository URL - explicit field
        if let Some(repo) = &self.repository {
            log::debug!("Adding repository: {}", repo);
            obj.insert("repository".into(), Value::scalar(repo.clone()));
        }
        
        // URL settings
        obj.insert("baseurl".into(), Value::scalar(self.base_url.clone()));
        
        if let Some(url) = &self.url {
            obj.insert("url".into(), Value::scalar(url.clone()));
        }
        
        // Custom variables from site_data.custom - the most important part!
        log::debug!("Processing custom variables from site_data.custom:");
        for (key, value) in &self.site_data.custom {
            log::debug!("  - Adding to site object: '{}' = {:?}", key, value);
            obj.insert(key.clone().into(), yaml_to_liquid(value.clone()));
        }
        
        // Other Jekyll settings
        obj.insert("markdown".into(), Value::scalar(self.markdown_ext.join(", ")));
        obj.insert("highlighter".into(), Value::scalar(self.highlighter.clone()));
        obj.insert("permalink".into(), Value::scalar(self.permalink.clone()));
        
        // Debug which keys are available
        log::debug!("Final site object keys: {:?}", obj.keys().collect::<Vec<_>>());
        
        obj
    }
} 