use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_yaml;
use chrono::{DateTime, Utc, NaiveDate, TimeZone};
use liquid::Object;
use liquid::model::Value;
use super::deserializers::deserialize_string_or_seq;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Front matter for a document or layout
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrontMatter {
    /// Document title
    pub title: Option<String>,
    
    /// Layout to use
    pub layout: Option<String>,
    
    /// Custom permalink
    pub permalink: Option<String>,
    
    /// Page description
    pub description: Option<String>,
    
    /// Date as a string (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
    pub date: Option<String>,
    
    /// Categories
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_seq")]
    pub categories: Option<Vec<String>>,
    
    /// Tags
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_seq")]
    pub tags: Option<Vec<String>>,
    
    /// Author name
    pub author: Option<String>,
    
    /// Whether the content is published
    pub published: Option<bool>,
    
    /// Page-specific excerpt separator
    pub excerpt_separator: Option<String>,
    
    /// Custom front matter fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter {
            title: None,
            layout: None,
            permalink: None,
            description: None,
            date: None,
            categories: None,
            tags: None,
            author: None,
            published: None,
            excerpt_separator: None,
            custom: HashMap::new(),
        }
    }
}

impl FrontMatter {
    /// Create a new empty front matter
    pub fn new() -> Self {
        FrontMatter::default()
    }
    
    /// Parse front matter from content
    pub fn parse(content: &str) -> BoxResult<Self> {
        crate::front_matter::parser::parse(content)
    }
    
    /// Extract the content without front matter - static method for backward compatibility
    pub fn extract_content(content: &str) -> String {
        crate::front_matter::extract_content(content)
    }
    
    /// Convert front matter to YAML string
    pub fn to_yaml(&self) -> BoxResult<String> {
        let yaml = serde_yaml::to_string(self)?;
        Ok(yaml)
    }
    
    /// Add front matter to content
    pub fn add_to_content(&self, content: &str) -> BoxResult<String> {
        let yaml = self.to_yaml()?;
        Ok(format!("---\n{}---\n\n{}", yaml, content))
    }
    
    /// Convert front matter to a Liquid object for templates
    pub fn to_liquid_object(&self) -> Object {
        let mut obj = Object::new();
        
        // Add basic fields
        if let Some(title) = &self.title {
            obj.insert("title".into(), Value::scalar(title.clone()));
        }
        
        if let Some(layout) = &self.layout {
            obj.insert("layout".into(), Value::scalar(layout.clone()));
        }
        
        if let Some(permalink) = &self.permalink {
            obj.insert("permalink".into(), Value::scalar(permalink.clone()));
        }
        
        if let Some(description) = &self.description {
            obj.insert("description".into(), Value::scalar(description.clone()));
        }
        
        if let Some(date_str) = &self.date {
            obj.insert("date".into(), Value::scalar(date_str.clone()));
        }
        
        if let Some(categories) = &self.categories {
            let cat_values: Vec<Value> = categories.iter()
                .map(|c| Value::scalar(c.clone()))
                .collect();
            obj.insert("categories".into(), Value::Array(cat_values));
        }
        
        if let Some(tags) = &self.tags {
            let tag_values: Vec<Value> = tags.iter()
                .map(|t| Value::scalar(t.clone()))
                .collect();
            obj.insert("tags".into(), Value::Array(tag_values));
        }
        
        if let Some(author) = &self.author {
            obj.insert("author".into(), Value::scalar(author.clone()));
        }
        
        if let Some(published) = &self.published {
            obj.insert("published".into(), Value::scalar(*published));
        }
        
        if let Some(excerpt_separator) = &self.excerpt_separator {
            obj.insert("excerpt_separator".into(), Value::scalar(excerpt_separator.clone()));
        }
        
        // Add custom fields
        for (key, value) in &self.custom {
            // Skip keys we've already added
            if !obj.contains_key::<str>(key) {
                obj.insert(key.clone().into(), crate::builder::processor::yaml_to_liquid(value.clone()));
            }
        }
        
        obj
    }
    
    /// Merge with another front matter, keeping existing values if present
    pub fn merge(&mut self, other: &FrontMatter) {
        if self.title.is_none() && other.title.is_some() {
            self.title = other.title.clone();
        }
        
        if self.layout.is_none() && other.layout.is_some() {
            self.layout = other.layout.clone();
        }
        
        if self.permalink.is_none() && other.permalink.is_some() {
            self.permalink = other.permalink.clone();
        }
        
        if self.date.is_none() && other.date.is_some() {
            self.date = other.date.clone();
        }
        
        if self.categories.is_none() && other.categories.is_some() {
            self.categories = other.categories.clone();
        }
        
        if self.tags.is_none() && other.tags.is_some() {
            self.tags = other.tags.clone();
        }
        
        if self.published.is_none() && other.published.is_some() {
            self.published = other.published;
        }
        
        if self.excerpt_separator.is_none() && other.excerpt_separator.is_some() {
            self.excerpt_separator = other.excerpt_separator.clone();
        }
        
        // Merge custom fields, keeping existing values
        for (key, value) in &other.custom {
            if !self.custom.contains_key(key) {
                self.custom.insert(key.clone(), value.clone());
            }
        }
    }
    
    /// Get parsed date if available
    pub fn get_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        if let Some(date_str) = &self.date {
            // Try ISO 8601 format (YYYY-MM-DD)
            if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let naive_dt = dt.and_hms_opt(0, 0, 0)?;
                return Some(Utc.from_utc_datetime(&naive_dt));
            }
            
            // Try Jekyll date format (YYYY-MM-DD HH:MM:SS)
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
                return Some(Utc.from_utc_datetime(&dt));
            }
            
            // Try RFC3339 format
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
                return Some(dt.with_timezone(&Utc));
            }
        }
        None
    }
    
    /// Get categories as a single string
    pub fn get_category_string(&self) -> Option<String> {
        self.categories.as_ref().map(|cats| cats.join(", "))
    }
    
    /// Get tags as a single string
    pub fn get_tag_string(&self) -> Option<String> {
        self.tags.as_ref().map(|tags| tags.join(", "))
    }
    
    /// Check if a document should be published
    pub fn is_published(&self) -> bool {
        self.published.unwrap_or(true)
    }
} 