use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_yaml;
use chrono::{DateTime, Utc, TimeZone};
use liquid::Object;
use liquid::model::Value;
use super::deserializers::deserialize_string_or_seq;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Pagination configuration in front matter
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Pagination {
    /// Enable pagination
    #[serde(default)]
    pub enabled: bool,
    
    /// Number of items per page
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    
    /// Current page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: usize,
    
    /// Total number of pages
    #[serde(skip_deserializing)]
    pub total_pages: usize,
    
    /// Total number of items
    #[serde(skip_deserializing)]
    pub total_items: usize,
    
    /// Path format for pagination pages
    pub path: Option<String>,
    
    /// Sort field for paginated items
    #[serde(default = "default_sort")]
    pub sort: String,
    
    /// Sort direction (asc or desc)
    #[serde(default = "default_sort_direction")]
    pub direction: String,
    
    /// Collection to paginate (default: posts)
    #[serde(default = "default_collection")]
    pub collection: String,
    
    /// Category filter
    pub category: Option<String>,
    
    /// Tag filter
    pub tag: Option<String>,
    
    /// Post IDs (filled in by pagination processor)
    #[serde(skip_deserializing)]
    pub posts: Vec<String>,
}

/// SEO-related metadata
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SeoMetadata {
    /// SEO title
    pub title: Option<String>,
    
    /// SEO description
    pub description: Option<String>,
    
    /// Open Graph image
    pub image: Option<String>,
    
    /// Twitter card type
    pub twitter_card: Option<String>,
}

/// Front matter for a document or layout
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrontMatter {
    /// Document title
    pub title: Option<String>,
    
    /// Custom slug for URL generation
    pub slug: Option<String>,
    
    /// Layout to use
    pub layout: Option<String>,
    
    /// Custom permalink
    pub permalink: Option<String>,
    
    /// Page description
    pub description: Option<String>,
    
    /// Date as a string (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
    pub date: Option<String>,
    
    /// Modified date as a string
    pub modified_date: Option<String>,
    
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
    
    /// Author information
    pub authors: Option<Vec<serde_yaml::Value>>,
    
    /// Whether the content is published
    pub published: Option<bool>,
    
    /// Whether to show in RSS feed
    #[serde(default = "default_true")]
    pub rss: bool,
    
    /// Whether content is a draft
    #[serde(default)]
    pub draft: Option<bool>,
    
    /// Whether to skip processing this content
    #[serde(default)]
    pub sitemap: Option<bool>,
    
    /// Order value for sorting
    pub order: Option<i32>,
    
    /// Weight value for sorting
    pub weight: Option<i32>,
    
    /// Page-specific excerpt separator
    pub excerpt_separator: Option<String>,
    
    /// Custom excerpt
    pub excerpt: Option<String>,
    
    /// Image for the content
    pub image: Option<String>,
    
    /// Thumbnail image
    pub thumbnail: Option<String>,
    
    /// Pagination configuration
    pub pagination: Option<Pagination>,
    
    /// SEO metadata
    pub seo: Option<SeoMetadata>,
    
    /// Redirect from URL
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_or_seq")]
    pub redirect_from: Option<Vec<String>>,
    
    /// Redirect to URL
    pub redirect_to: Option<String>,
    
    /// Table of contents settings
    pub toc: Option<bool>,
    
    /// Custom css classes
    pub css_class: Option<String>,
    
    /// Custom front matter fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

fn default_true() -> bool {
    true
}

fn default_per_page() -> usize {
    10
}

fn default_page() -> usize {
    1
}

fn default_sort() -> String {
    "date".to_string()
}

fn default_sort_direction() -> String {
    "desc".to_string()
}

fn default_collection() -> String {
    "posts".to_string()
}

impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter {
            title: None,
            slug: None,
            layout: None,
            permalink: None,
            description: None,
            date: None,
            modified_date: None,
            categories: None,
            tags: None,
            author: None,
            authors: None,
            published: None,
            rss: true,
            draft: None,
            sitemap: None,
            order: None,
            weight: None,
            excerpt_separator: None,
            excerpt: None,
            image: None,
            thumbnail: None,
            pagination: None,
            seo: None,
            redirect_from: None,
            redirect_to: None,
            toc: None,
            css_class: None,
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
        crate::front_matter::parse(content)
    }
    
    /// Extract the content without front matter - static method for backward compatibility
    pub fn extract_content(content: &str) -> String {
        crate::front_matter::utils::extract_content(content)
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
        
        if let Some(slug) = &self.slug {
            obj.insert("slug".into(), Value::scalar(slug.clone()));
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
        
        // Handle dates
        if let Some(date_str) = &self.date {
            obj.insert("date".into(), Value::scalar(date_str.clone()));
            
            // Also add parsed date if possible
            if let Some(date) = self.get_date() {
                let datetime_obj = self.datetime_to_liquid_object(date);
                obj.insert("date_obj".into(), Value::Object(datetime_obj));
            }
        }
        
        if let Some(modified_date) = &self.modified_date {
            obj.insert("modified_date".into(), Value::scalar(modified_date.clone()));
            
            // Also add parsed date if possible
            if let Ok(date) = DateTime::parse_from_rfc3339(modified_date) {
                let datetime_obj = self.datetime_to_liquid_object(date.with_timezone(&Utc));
                obj.insert("modified_date_obj".into(), Value::Object(datetime_obj));
            }
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
        
        if let Some(authors) = &self.authors {
            let authors_values: Vec<Value> = authors.iter()
                .map(|a| crate::builder::processor::yaml_to_liquid(a.clone()))
                .collect();
            obj.insert("authors".into(), Value::Array(authors_values));
        }
        
        if let Some(published) = &self.published {
            obj.insert("published".into(), Value::scalar(*published));
        }
        
        obj.insert("rss".into(), Value::scalar(self.rss));
        
        if let Some(draft) = &self.draft {
            obj.insert("draft".into(), Value::scalar(*draft));
        }
        
        if let Some(sitemap) = &self.sitemap {
            obj.insert("sitemap".into(), Value::scalar(*sitemap));
        }
        
        if let Some(order) = &self.order {
            obj.insert("order".into(), Value::scalar(*order));
        }
        
        if let Some(weight) = &self.weight {
            obj.insert("weight".into(), Value::scalar(*weight));
        }
        
        if let Some(excerpt_separator) = &self.excerpt_separator {
            obj.insert("excerpt_separator".into(), Value::scalar(excerpt_separator.clone()));
        }
        
        if let Some(excerpt) = &self.excerpt {
            obj.insert("excerpt".into(), Value::scalar(excerpt.clone()));
        }
        
        if let Some(image) = &self.image {
            obj.insert("image".into(), Value::scalar(image.clone()));
        }
        
        if let Some(thumbnail) = &self.thumbnail {
            obj.insert("thumbnail".into(), Value::scalar(thumbnail.clone()));
        }
        
        if let Some(pagination) = &self.pagination {
            let mut pagination_obj = Object::new();
            
            pagination_obj.insert("enabled".into(), Value::scalar(pagination.enabled));
            pagination_obj.insert("per_page".into(), Value::scalar(pagination.per_page.to_string()));
            pagination_obj.insert("page".into(), Value::scalar(pagination.page.to_string()));
            pagination_obj.insert("total_pages".into(), Value::scalar(pagination.total_pages.to_string()));
            pagination_obj.insert("total_items".into(), Value::scalar(pagination.total_items.to_string()));
            
            if let Some(path) = &pagination.path {
                pagination_obj.insert("path".into(), Value::scalar(path.clone()));
            }
            
            pagination_obj.insert("sort".into(), Value::scalar(pagination.sort.clone()));
            pagination_obj.insert("direction".into(), Value::scalar(pagination.direction.clone()));
            pagination_obj.insert("collection".into(), Value::scalar(pagination.collection.clone()));
            
            if let Some(category) = &pagination.category {
                pagination_obj.insert("category".into(), Value::scalar(category.clone()));
            }
            
            if let Some(tag) = &pagination.tag {
                pagination_obj.insert("tag".into(), Value::scalar(tag.clone()));
            }
            
            obj.insert("pagination".into(), Value::Object(pagination_obj));
        }
        
        if let Some(seo) = &self.seo {
            let mut seo_obj = Object::new();
            
            if let Some(title) = &seo.title {
                seo_obj.insert("title".into(), Value::scalar(title.clone()));
            }
            
            if let Some(description) = &seo.description {
                seo_obj.insert("description".into(), Value::scalar(description.clone()));
            }
            
            if let Some(image) = &seo.image {
                seo_obj.insert("image".into(), Value::scalar(image.clone()));
            }
            
            if let Some(twitter_card) = &seo.twitter_card {
                seo_obj.insert("twitter_card".into(), Value::scalar(twitter_card.clone()));
            }
            
            obj.insert("seo".into(), Value::Object(seo_obj));
        }
        
        if let Some(redirect_from) = &self.redirect_from {
            let redirect_values: Vec<Value> = redirect_from.iter()
                .map(|r| Value::scalar(r.clone()))
                .collect();
            obj.insert("redirect_from".into(), Value::Array(redirect_values));
        }
        
        if let Some(redirect_to) = &self.redirect_to {
            obj.insert("redirect_to".into(), Value::scalar(redirect_to.clone()));
        }
        
        if let Some(toc) = &self.toc {
            obj.insert("toc".into(), Value::scalar(*toc));
        }
        
        if let Some(css_class) = &self.css_class {
            obj.insert("css_class".into(), Value::scalar(css_class.clone()));
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
    
    /// Convert a datetime to a liquid object with Jekyll-compatible date parts
    fn datetime_to_liquid_object(&self, dt: DateTime<Utc>) -> Object {
        let mut obj = Object::new();
        
        // Add date components
        obj.insert("year".into(), Value::scalar(dt.format("%Y").to_string()));
        obj.insert("month".into(), Value::scalar(dt.format("%m").to_string()));
        obj.insert("day".into(), Value::scalar(dt.format("%d").to_string()));
        obj.insert("hour".into(), Value::scalar(dt.format("%H").to_string()));
        obj.insert("minute".into(), Value::scalar(dt.format("%M").to_string()));
        obj.insert("second".into(), Value::scalar(dt.format("%S").to_string()));
        
        // Add formatted versions
        obj.insert("iso8601".into(), Value::scalar(dt.to_rfc3339()));
        obj.insert("rfc822".into(), Value::scalar(dt.format("%a, %d %b %Y %H:%M:%S %z").to_string()));
        obj.insert("string".into(), Value::scalar(dt.format("%Y-%m-%d %H:%M:%S %z").to_string()));
        obj.insert("unix".into(), Value::scalar(dt.timestamp()));
        
        // Add additional Jekyll-compatible components
        obj.insert("y_day".into(), Value::scalar(dt.format("%j").to_string())); // Day of year
        obj.insert("w_day".into(), Value::scalar(dt.format("%w").to_string())); // Day of week (0-6)
        
        obj
    }
    
    /// Merge with another front matter, keeping existing values if present
    pub fn merge(&mut self, other: &FrontMatter) {
        if self.title.is_none() && other.title.is_some() {
            self.title = other.title.clone();
        }
        
        if self.slug.is_none() && other.slug.is_some() {
            self.slug = other.slug.clone();
        }
        
        if self.layout.is_none() && other.layout.is_some() {
            self.layout = other.layout.clone();
        }
        
        if self.permalink.is_none() && other.permalink.is_some() {
            self.permalink = other.permalink.clone();
        }
        
        if self.description.is_none() && other.description.is_some() {
            self.description = other.description.clone();
        }
        
        if self.date.is_none() && other.date.is_some() {
            self.date = other.date.clone();
        }
        
        if self.modified_date.is_none() && other.modified_date.is_some() {
            self.modified_date = other.modified_date.clone();
        }
        
        if self.categories.is_none() && other.categories.is_some() {
            self.categories = other.categories.clone();
        }
        
        if self.tags.is_none() && other.tags.is_some() {
            self.tags = other.tags.clone();
        }
        
        if self.author.is_none() && other.author.is_some() {
            self.author = other.author.clone();
        }
        
        if self.authors.is_none() && other.authors.is_some() {
            self.authors = other.authors.clone();
        }
        
        if self.published.is_none() && other.published.is_some() {
            self.published = other.published;
        }
        
        if self.draft.is_none() && other.draft.is_some() {
            self.draft = other.draft;
        }
        
        if self.sitemap.is_none() && other.sitemap.is_some() {
            self.sitemap = other.sitemap;
        }
        
        if self.order.is_none() && other.order.is_some() {
            self.order = other.order;
        }
        
        if self.weight.is_none() && other.weight.is_some() {
            self.weight = other.weight;
        }
        
        if self.excerpt_separator.is_none() && other.excerpt_separator.is_some() {
            self.excerpt_separator = other.excerpt_separator.clone();
        }
        
        if self.excerpt.is_none() && other.excerpt.is_some() {
            self.excerpt = other.excerpt.clone();
        }
        
        if self.image.is_none() && other.image.is_some() {
            self.image = other.image.clone();
        }
        
        if self.thumbnail.is_none() && other.thumbnail.is_some() {
            self.thumbnail = other.thumbnail.clone();
        }
        
        if self.pagination.is_none() && other.pagination.is_some() {
            self.pagination = other.pagination.clone();
        }
        
        if self.seo.is_none() && other.seo.is_some() {
            self.seo = other.seo.clone();
        }
        
        if self.redirect_from.is_none() && other.redirect_from.is_some() {
            self.redirect_from = other.redirect_from.clone();
        }
        
        if self.redirect_to.is_none() && other.redirect_to.is_some() {
            self.redirect_to = other.redirect_to.clone();
        }
        
        if self.toc.is_none() && other.toc.is_some() {
            self.toc = other.toc;
        }
        
        if self.css_class.is_none() && other.css_class.is_some() {
            self.css_class = other.css_class.clone();
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
    
    /// Get modified date if available
    pub fn get_modified_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        if let Some(date_str) = &self.modified_date {
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
    
    /// Check if a document is a draft
    pub fn is_draft(&self) -> bool {
        self.draft.unwrap_or(false)
    }
    
    /// Get the document state
    pub fn get_state(&self) -> crate::collections::document::model::DocumentState {
        use crate::collections::document::model::DocumentState;
        
        if self.is_draft() {
            DocumentState::Draft
        } else if !self.is_published() {
            DocumentState::Unpublished
        } else {
            DocumentState::Published
        }
    }
} 