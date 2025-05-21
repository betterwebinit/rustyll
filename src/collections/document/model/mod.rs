use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use liquid::model::Value;

use crate::front_matter::FrontMatter;
use crate::config::{Config, CollectionConfig};

/// A collection of documents
#[derive(Debug, Clone)]
pub struct Collection {
    pub label: String,
    pub output: bool,
    pub permalink: Option<String>,
    pub documents: Vec<Document>,
    pub relative_directory: PathBuf,
    pub directory: PathBuf,
}

/// A document within a collection
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub url: Option<String>,
    pub collection: String,
    pub date: Option<DateTime<Utc>>,
    pub content: String,
    pub rendered_content: Option<String>,
    pub front_matter: FrontMatter,
    pub excerpt: Option<String>,
}

impl Collection {
    pub fn new(
        label: &str,
        config: &CollectionConfig,
        site_source: &Path,
        collections_dir: Option<&Path>
    ) -> Self {
        let relative_directory = PathBuf::from(format!("_{}", label));
        let directory = if let Some(coll_dir) = collections_dir {
            coll_dir.join(format!("_{}", label))
        } else {
            site_source.join(format!("_{}", label))
        };
        
        Collection {
            label: label.to_string(),
            output: config.output,
            permalink: config.permalink.clone(),
            documents: Vec::new(),
            relative_directory,
            directory,
        }
    }
    
    /// Generate the URL for a document in this collection based on permalink pattern
    pub fn generate_url(&self, doc: &Document, site_config: &Config) -> Option<String> {
        if !self.output {
            return None;
        }
        
        // Get permalink template, either from collection config or default
        let permalink_template = if let Some(permalink) = &self.permalink {
            permalink.clone()
        } else if self.label == "posts" {
            // Default permalink for posts
            String::from("/:categories/:year/:month/:day/:title:output_ext")
        } else {
            // Default permalink for other collections
            String::from("/:collection/:path:output_ext")
        };
        
        // Prepare replacements based on document properties
        let mut replacements = HashMap::new();
        
        // Collection name
        replacements.insert(":collection".to_string(), self.label.clone());
        
        // Path and name (without extension)
        if let Some(stem) = doc.path.file_stem() {
            replacements.insert(":name".to_string(), stem.to_string_lossy().to_string());
        }
        
        if let Some(rel_path) = doc.path.strip_prefix(&self.directory).ok() {
            let path_str = rel_path.to_string_lossy().to_string();
            replacements.insert(":path".to_string(), path_str);
        }
        
        // Output extension
        replacements.insert(":output_ext".to_string(), String::from(".html"));
        
        // Date-based replacements
        if let Some(date) = &doc.date {
            replacements.insert(":year".to_string(), date.format("%Y").to_string());
            replacements.insert(":month".to_string(), date.format("%m").to_string());
            replacements.insert(":day".to_string(), date.format("%d").to_string());
            replacements.insert(":hour".to_string(), date.format("%H").to_string());
            replacements.insert(":minute".to_string(), date.format("%M").to_string());
            replacements.insert(":second".to_string(), date.format("%S").to_string());
            replacements.insert(":i_day".to_string(), date.format("%-d").to_string());
            replacements.insert(":i_month".to_string(), date.format("%-m").to_string());
        }
        
        // Title replacement
        if let Some(title) = &doc.front_matter.title {
            replacements.insert(":title".to_string(), slug::slugify(title));
        }
        
        // Categories
        if let Some(categories) = &doc.front_matter.categories {
            let categories_path = categories.join("/");
            replacements.insert(":categories".to_string(), categories_path);
        } else {
            replacements.insert(":categories".to_string(), String::new());
        }
        
        // Now apply the replacements to the pattern
        let mut url = permalink_template.clone();
        
        for (placeholder, value) in &replacements {
            url = url.replace(placeholder, value);
        }
        
        // Clean up any empty path segments
        while url.contains("//") {
            url = url.replace("//", "/");
        }
        
        // Ensure URL starts with /
        if !url.starts_with('/') {
            url = format!("/{}", url);
        }
        
        Some(url)
    }
} 