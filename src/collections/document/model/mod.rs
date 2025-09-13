use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate};
use serde::{Serialize, Deserialize};

use crate::front_matter::FrontMatter;
use crate::config::{Config, CollectionConfig};

/// A collection of documents
#[derive(Debug, Clone)]
pub struct Collection {
    /// The name of the collection
    pub label: String,
    
    /// Whether to output the collection as individual pages
    pub output: bool,
    
    /// The permalink pattern for this collection
    pub permalink: Option<String>,
    
    /// Documents in the collection
    pub documents: Vec<Document>,
    
    /// The relative directory path for this collection
    pub relative_directory: PathBuf,
    
    /// The absolute directory path for this collection
    pub directory: PathBuf,
    
    /// Sort order for documents in this collection
    pub sort_by: String,
    
    /// Metadata for this collection
    pub metadata: CollectionMetadata,
}

/// Metadata for a collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionMetadata {
    /// Custom data for this collection
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

/// Document state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// Published document
    Published,
    
    /// Draft document
    Draft,
    
    /// Unpublished document (contains published: false in front matter)
    Unpublished,
}

/// A document within a collection
#[derive(Debug, Clone)]
pub struct Document {
    /// Unique identifier for the document
    pub id: String,
    
    /// Absolute path to the document file
    pub path: PathBuf,
    
    /// Path relative to the source directory
    pub relative_path: PathBuf,
    
    /// Output path where the document will be written
    pub output_path: Option<PathBuf>,
    
    /// URL for the document
    pub url: Option<String>,
    
    /// Collection this document belongs to
    pub collection: String,
    
    /// Date from filename or front matter
    pub date: Option<DateTime<Utc>>,
    
    /// Raw content of the document
    pub content: String,
    
    /// Rendered content after processing
    pub rendered_content: Option<String>,
    
    /// Front matter data
    pub front_matter: FrontMatter,
    
    /// Excerpt from the document
    pub excerpt: Option<String>,
    
    /// Document state (published, draft, unpublished)
    pub state: DocumentState,
    
    /// Categories for this document
    pub categories: Vec<String>,
    
    /// Tags for this document
    pub tags: Vec<String>,
    
    /// Next document in the collection (for navigation)
    pub next: Option<Box<Document>>,
    
    /// Previous document in the collection (for navigation)
    pub previous: Option<Box<Document>>,
    
    /// Related documents
    pub related: Vec<String>,
    
    /// Modification time of the file
    pub mtime: Option<DateTime<Utc>>,
    
    /// Creation time of the file
    pub ctime: Option<DateTime<Utc>>,
}

impl Document {
    /// Create a new document
    pub fn new(
        id: String,
        path: PathBuf,
        relative_path: PathBuf,
        collection: String,
        content: String,
        front_matter: FrontMatter,
    ) -> Self {
        // Extract date from front matter or use current time
        let date = front_matter.date.clone();
        
        // Extract categories and tags
        let categories = front_matter.categories.clone().unwrap_or_default();
        let tags = front_matter.tags.clone().unwrap_or_default();
        
        // Determine document state
        let state = if front_matter.published.unwrap_or(true) == false {
            DocumentState::Unpublished
        } else {
            DocumentState::Published
        };
        
        // Try to get file modification and creation times
        let (mtime, ctime) = if let Ok(metadata) = std::fs::metadata(&path) {
            let mtime = metadata.modified().ok().and_then(|t| {
                t.elapsed().ok().map(|e| {
                    let now = Utc::now();
                    now - chrono::Duration::from_std(e).unwrap_or_default()
                })
            });
            
            let ctime = metadata.created().ok().and_then(|t| {
                t.elapsed().ok().map(|e| {
                    let now = Utc::now();
                    now - chrono::Duration::from_std(e).unwrap_or_default()
                })
            });
            
            (mtime, ctime)
        } else {
            (None, None)
        };
        
        Document {
            id,
            path,
            relative_path,
            output_path: None,
            url: None,
            collection,
            date: parse_date_string(date),
            content,
            rendered_content: None,
            front_matter,
            excerpt: None,
            state,
            categories,
            tags,
            next: None,
            previous: None,
            related: Vec::new(),
            mtime,
            ctime,
        }
    }
    
    /// Check if this document should be written to the output
    pub fn should_write(&self, config: &Config) -> bool {
        match self.state {
            DocumentState::Published => true,
            DocumentState::Draft => config.show_drafts.unwrap_or(false),
            DocumentState::Unpublished => config.unpublished.unwrap_or(false),
        }
    }
    
    /// Generate excerpt from content
    pub fn generate_excerpt(&mut self, separator: &str) {
        if self.excerpt.is_some() {
            return;
        }
        
        if let Some(idx) = self.content.find(separator) {
            self.excerpt = Some(self.content[..idx].trim().to_string());
        } else {
            // No explicit separator, use the first paragraph
            let first_para_end = self.content.find("\n\n").unwrap_or_else(|| self.content.len());
            let first_para = self.content[..first_para_end].trim();
            
            if !first_para.is_empty() {
                self.excerpt = Some(first_para.to_string());
            }
        }
    }
    
    /// Get the title of the document
    pub fn title(&self) -> Option<String> {
        self.front_matter.title.clone()
    }
    
    /// Get the slug for the document
    pub fn slug(&self) -> String {
        // Try to get slug from front matter
        if let Some(slug) = &self.front_matter.slug {
            return slug.clone();
        }
        
        // Try to get from title
        if let Some(title) = &self.front_matter.title {
            return slug::slugify(title);
        }
        
        // Fall back to file stem
        if let Some(stem) = self.path.file_stem() {
            return stem.to_string_lossy().to_string();
        }
        
        // Last resort
        "unnamed".to_string()
    }
}

impl Collection {
    /// Create a new collection
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
            sort_by: config.sort_by.clone(),
            metadata: CollectionMetadata::default(),
        }
    }
    
    /// Sort the documents in this collection
    pub fn sort_documents(&mut self) {
        match self.sort_by.as_str() {
            "date" => {
                self.documents.sort_by(|a, b| {
                    let a_date = a.date.unwrap_or_else(|| Utc::now());
                    let b_date = b.date.unwrap_or_else(|| Utc::now());
                    b_date.cmp(&a_date) // Reverse chronological order
                });
            },
            "title" => {
                self.documents.sort_by(|a, b| {
                    let a_title = a.front_matter.title.clone().unwrap_or_default();
                    let b_title = b.front_matter.title.clone().unwrap_or_default();
                    a_title.cmp(&b_title)
                });
            },
            _ => {
                // Custom sort field
                self.documents.sort_by(|a, b| {
                    let a_val = a.front_matter.custom.get(&self.sort_by);
                    let b_val = b.front_matter.custom.get(&self.sort_by);
                    
                    match (a_val, b_val) {
                        (Some(a_val), Some(b_val)) => {
                            // Try to compare as strings
                            let a_str = a_val.as_str().unwrap_or("");
                            let b_str = b_val.as_str().unwrap_or("");
                            a_str.cmp(b_str)
                        },
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
        }
    }
    
    /// Set up next/previous links between documents
    pub fn set_next_prev_links(&mut self) {
        if self.documents.is_empty() {
            return;
        }
        
        // We need to clone the documents to avoid borrow checker issues
        let docs = self.documents.clone();
        
        for i in 0..self.documents.len() {
            // Set next document
            if i < self.documents.len() - 1 {
                self.documents[i].next = Some(Box::new(docs[i + 1].clone()));
            }
            
            // Set previous document
            if i > 0 {
                self.documents[i].previous = Some(Box::new(docs[i - 1].clone()));
            }
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
            
            // Also add ordinal day
            replacements.insert(":y_day".to_string(), date.format("%j").to_string());
        }
        
        // Title replacement
        replacements.insert(":title".to_string(), doc.slug());
        
        // Slug replacement
        replacements.insert(":slug".to_string(), doc.slug());
        
        // Categories
        if !doc.categories.is_empty() {
            let categories_path = doc.categories.join("/");
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
        
        // Add baseurl if configured
        let mut final_url = url;
        if !site_config.base_url.is_empty() && !site_config.base_url.starts_with("http") {
            let base = if site_config.base_url.ends_with('/') {
                site_config.base_url.clone()
            } else {
                format!("{}/", site_config.base_url)
            };
            
            let url_without_leading_slash = if final_url.starts_with('/') {
                final_url[1..].to_string()
            } else {
                final_url
            };
            
            final_url = format!("/{}{}", base, url_without_leading_slash);
        } else if !final_url.starts_with('/') {
            // Ensure URL starts with /
            final_url = format!("/{}", final_url);
        }
        
        Some(final_url)
    }
}

/// Parse a date string into a DateTime<Utc>
fn parse_date_string(date: Option<String>) -> Option<DateTime<Utc>> {
    if let Some(date_str) = date {
        // Try to parse the date using various formats
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&date_str) {
            return Some(parsed.with_timezone(&Utc));
        }
        
        // Try other date formats
        if let Ok(parsed) = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S") {
            return Some(parsed.and_utc());
        }

        if let Ok(parsed) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            return Some(parsed.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
    }
    
    None
} 