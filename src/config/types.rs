use std::path::PathBuf;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::collections::hash_map;
use liquid::Object;
use liquid::model::Value;
use glob_match::glob_match;

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

/// Liquid template engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidConfig {
    /// Error mode (lax, warn, strict)
    #[serde(default = "default_liquid_error_mode")]
    pub error_mode: String,
    
    /// Whether to use strict filters
    #[serde(default = "default_false")]
    pub strict_filters: bool,
    
    /// Whether to use strict variables
    #[serde(default = "default_false")]
    pub strict_variables: bool,
}

impl Default for LiquidConfig {
    fn default() -> Self {
        LiquidConfig {
            error_mode: default_liquid_error_mode(),
            strict_filters: false,
            strict_variables: false,
        }
    }
}

/// Kramdown markdown engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KramdownConfig {
    /// Whether to auto-generate IDs for headings
    #[serde(default = "default_true")]
    pub auto_ids: bool,
    
    /// TOC levels to include
    #[serde(default = "default_toc_levels")]
    pub toc_levels: Vec<u8>,
    
    /// Entity output format
    #[serde(default = "default_entity_output")]
    pub entity_output: String,
    
    /// Smart quotes configuration
    #[serde(default = "default_smart_quotes")]
    pub smart_quotes: String,
    
    /// Input format
    #[serde(default = "default_kramdown_input")]
    pub input: String,
    
    /// Whether to use hard wraps
    #[serde(default = "default_false")]
    pub hard_wrap: bool,
    
    /// Whether to guess language for code blocks
    #[serde(default = "default_true")]
    pub guess_lang: bool,
    
    /// Footnote number start
    #[serde(default = "default_footnote_nr")]
    pub footnote_nr: u8,
    
    /// Whether to show warnings
    #[serde(default = "default_false")]
    pub show_warnings: bool,
}

impl Default for KramdownConfig {
    fn default() -> Self {
        KramdownConfig {
            auto_ids: default_true(),
            toc_levels: default_toc_levels(),
            entity_output: default_entity_output(),
            smart_quotes: default_smart_quotes(),
            input: default_kramdown_input(),
            hard_wrap: false,
            guess_lang: true,
            footnote_nr: default_footnote_nr(),
            show_warnings: false,
        }
    }
}

/// Markdown extensions configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarkdownExtensions {
    /// Whether to enable math support
    #[serde(default)]
    pub math: bool,
    
    /// Whether to enable diagram support
    #[serde(default)]
    pub diagrams: bool,
    
    /// Whether to enable typographic enhancements
    #[serde(default = "default_true")]
    pub typographic: bool,
    
    /// Whether to enable table of contents generation
    #[serde(default)]
    pub toc: bool,
    
    /// Whether to enable footnotes
    #[serde(default)]
    pub footnotes: bool,
    
    /// Whether to enable task lists
    #[serde(default = "default_true")]
    pub task_lists: bool,
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
    
    /// Collections directory (where collections are stored)
    #[serde(default = "default_collections_dir")]
    pub collections_dir: String,
    
    /// Plugins directory
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,
    
    /// Cache directory
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
    
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
    pub exclude: Option<Vec<String>>,
    
    /// Include patterns (glob patterns of files that would otherwise be excluded)
    #[serde(default = "default_include")]
    pub include: Option<Vec<String>>,
    
    /// Collections configuration
    #[serde(default)]
    pub collections: Collections,
    
    /// Plugin settings
    #[serde(default)]
    pub plugins: Vec<String>,
    
    /// Markdown extensions
    #[serde(default = "default_markdown_extensions")]
    pub markdown_ext: Vec<String>,
    
    /// Additional markdown extensions (features)
    #[serde(default)]
    pub markdown_extensions: Option<Vec<String>>,
    
    /// Advanced markdown configuration
    #[serde(default)]
    pub markdown_config: MarkdownExtensions,
    
    /// Keep files (files to not delete during clean operation)
    #[serde(default = "default_keep_files")]
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
    
    /// Whether to use incremental rebuilds
    #[serde(default)]
    pub incremental: Option<bool>,
    
    /// Whether to generate a build report
    #[serde(default)]
    pub build_report: Option<bool>,
    
    /// Whether to show drafts
    #[serde(default)]
    pub show_drafts: Option<bool>,
    
    /// Whether to show future posts
    #[serde(default)]
    pub future: Option<bool>,
    
    /// Whether to show unpublished posts
    #[serde(default)]
    pub unpublished: Option<bool>,
    
    /// Limit on number of posts to build
    #[serde(default)]
    pub limit_posts: Option<usize>,
    
    /// Enable LSI (Latent Semantic Indexing) for better related posts
    #[serde(default)]
    pub lsi: Option<bool>,
    
    /// Encoding for reading files
    #[serde(default = "default_encoding")]
    pub encoding: String,
    
    /// Time zone
    #[serde(default)]
    pub timezone: Option<String>,
    
    /// Excerpt separator
    #[serde(default = "default_excerpt_separator")]
    pub excerpt_separator: String,
    
    /// Paginate settings
    #[serde(default)]
    pub paginate: Option<usize>,
    
    /// Paginate path
    #[serde(default = "default_paginate_path")]
    pub paginate_path: String,
    
    /// Whether to be quiet in output
    #[serde(default)]
    pub quiet: Option<bool>,
    
    /// Whether to be verbose in output
    #[serde(default)]
    pub verbose: Option<bool>,
    
    /// Whether to do strict front matter parsing
    #[serde(default)]
    pub strict_front_matter: Option<bool>,
    
    /// Kramdown markdown options
    #[serde(default)]
    pub kramdown: Option<KramdownConfig>,
    
    /// Liquid template engine options
    #[serde(default)]
    pub liquid: Option<LiquidConfig>,
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
    vec!["md".to_string(), "markdown".to_string(), "mkd".to_string(), "mkdn".to_string()]
}

/// Default posts directory
fn default_posts_dir() -> String {
    "_posts".to_string()
}

/// Default drafts directory
fn default_drafts_dir() -> String {
    "_drafts".to_string()
}

/// Default collections directory
fn default_collections_dir() -> String {
    "".to_string()  // Empty string means collections are at root level
}

/// Default plugins directory
fn default_plugins_dir() -> String {
    "_plugins".to_string()
}

/// Default cache directory
fn default_cache_dir() -> String {
    ".rustyll-cache".to_string()
}

/// Default highlighter
fn default_highlighter() -> String {
    "rouge".to_string()
}

/// Default permalink format
fn default_permalink() -> String {
    "date".to_string()
}

/// Default encoding
fn default_encoding() -> String {
    "utf-8".to_string()
}

/// Default excerpt separator
fn default_excerpt_separator() -> String {
    "\n\n".to_string()
}

/// Default paginate path
fn default_paginate_path() -> String {
    "/page:num".to_string()
}

/// Default keep files
fn default_keep_files() -> Vec<String> {
    vec![".git".to_string(), ".svn".to_string()]
}

/// Default include patterns
fn default_include() -> Option<Vec<String>> {
    Some(vec![".htaccess".to_string()])
}

/// Default true boolean
fn default_true() -> bool {
    true
}

/// Default false boolean
fn default_false() -> bool {
    false
}

/// Default toc levels
fn default_toc_levels() -> Vec<u8> {
    (1..=6).collect()
}

/// Default entity output
fn default_entity_output() -> String {
    "as_char".to_string()
}

/// Default smart quotes
fn default_smart_quotes() -> String {
    "lsquo,rsquo,ldquo,rdquo".to_string()
}

/// Default kramdown input format
fn default_kramdown_input() -> String {
    "GFM".to_string()
}

/// Default footnote number
fn default_footnote_nr() -> u8 {
    1
}

/// Default liquid error mode
fn default_liquid_error_mode() -> String {
    "warn".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            source: defaults::default_source(),
            destination: defaults::default_destination(),
            layouts_dir: defaults::default_layouts_dir(),
            includes_dir: defaults::default_includes_dir(),
            data_dir: defaults::default_data_dir(),
            collections_dir: default_collections_dir(),
            plugins_dir: default_plugins_dir(),
            cache_dir: default_cache_dir(),
            base_url: defaults::default_base_url(),
            title: defaults::default_site_title(),
            description: defaults::default_site_description(),
            repository: None,
            safe_mode: defaults::default_safe_mode(),
            defaults: Vec::new(),
            exclude: defaults::default_exclude(),
            include: default_include(),
            collections: Collections {
                items: defaults::default_collections(),
            },
            plugins: Vec::new(),
            markdown_ext: default_markdown_extensions(),
            markdown_extensions: None,
            markdown_config: MarkdownExtensions::default(),
            keep_files: default_keep_files(),
            posts_dir: default_posts_dir(),
            drafts_dir: default_drafts_dir(),
            url: None,
            highlighter: default_highlighter(),
            permalink: default_permalink(),
            site_data: SiteData::default(),
            incremental: None,
            build_report: None,
            show_drafts: None,
            future: None,
            unpublished: None,
            limit_posts: None,
            lsi: None,
            encoding: default_encoding(),
            timezone: None,
            excerpt_separator: default_excerpt_separator(),
            paginate: None,
            paginate_path: default_paginate_path(),
            quiet: None,
            verbose: None,
            strict_front_matter: None,
            kramdown: None,
            liquid: None,
        }
    }
}

/// Available permalink styles for easier config
pub enum PermalinkStyle {
    Date,
    Pretty,
    Ordinal,
    None,
    Custom(String),
}

impl From<&str> for PermalinkStyle {
    fn from(s: &str) -> Self {
        match s {
            "date" => PermalinkStyle::Date,
            "pretty" => PermalinkStyle::Pretty,
            "ordinal" => PermalinkStyle::Ordinal,
            "none" => PermalinkStyle::None,
            custom => PermalinkStyle::Custom(custom.to_string()),
        }
    }
}

impl PermalinkStyle {
    pub fn to_pattern(&self) -> String {
        match self {
            PermalinkStyle::Date => "/:categories/:year/:month/:day/:title.html".to_string(),
            PermalinkStyle::Pretty => "/:categories/:year/:month/:day/:title/".to_string(),
            PermalinkStyle::Ordinal => "/:categories/:year/:y_day/:title.html".to_string(),
            PermalinkStyle::None => "/:categories/:title.html".to_string(),
            PermalinkStyle::Custom(pattern) => pattern.clone(),
        }
    }
}

impl Config {
    /// Convert config to a Liquid Object for use in templates
    pub fn to_liquid(&self) -> Object {
        let mut obj = Object::new();
        
        // Add basic properties
        obj.insert("source".into(), Value::scalar(self.source.to_string_lossy().to_string()));
        obj.insert("destination".into(), Value::scalar(self.destination.to_string_lossy().to_string()));
        obj.insert("title".into(), Value::scalar(self.title.clone()));
        obj.insert("description".into(), Value::scalar(self.description.clone()));
        
        if let Some(url) = &self.url {
            obj.insert("url".into(), Value::scalar(url.clone()));
        }
        
        obj.insert("baseurl".into(), Value::scalar(self.base_url.clone()));
        
        // Add version and environment info
        obj.insert("version".into(), Value::scalar(env!("CARGO_PKG_VERSION")));
        
        let env = std::env::var("RUSTYLL_ENV").unwrap_or_else(|_| "development".to_string());
        obj.insert("environment".into(), Value::scalar(env));
        
        // Convert site data
        if let Some(title) = &self.site_data.title {
            obj.insert("title".into(), Value::scalar(title.clone()));
        }
        
        if let Some(description) = &self.site_data.description {
            obj.insert("description".into(), Value::scalar(description.clone()));
        }
        
        if let Some(author) = &self.site_data.author {
            obj.insert("author".into(), Value::scalar(author.clone()));
        }
        
        // Add all custom site data
        for (key, value) in &self.site_data.custom {
            let liquid_value = yaml_to_liquid(value.clone());
            obj.insert(key.clone().into(), liquid_value);
        }
        
        obj
    }
    
    /// Check if a file should be excluded based on exclude/include patterns
    pub fn is_excluded(&self, path: &Path) -> bool {
        let rel_path_str = path.to_string_lossy();
        
        // Check include patterns first (these override excludes)
        if let Some(include_patterns) = &self.include {
            for pattern in include_patterns {
                if glob_match(pattern, &rel_path_str) {
                    return false;
                }
            }
        }
        
        // Then check exclude patterns
        if let Some(exclude_patterns) = &self.exclude {
            for pattern in exclude_patterns {
                if glob_match(pattern, &rel_path_str) {
                    return true;
                }
            }
        }
        
        false
    }
} 