use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use chrono::{DateTime, Utc, NaiveDate, TimeZone};
use regex::Regex;
use log::{debug, warn, error};

use crate::front_matter::FrontMatter;
use crate::collections::document::model::Document;
use crate::collections::types::BoxResult;

/// Parse a document file for a regular collection
pub fn parse_document(
    path: &Path,
    collection_dir: &Path,
    collection_label: &str
) -> BoxResult<Option<Document>> {
    debug!("Parsing document: {}", path.display());
    
    // Read the file content
    let content = fs::read_to_string(path)?;
    
    // Extract front matter and content
    let (front_matter, content) = crate::front_matter::extract_front_matter(&content)?;
    
    // Skip if draft and published is false
    if let Some(false) = front_matter.published {
        debug!("Skipping unpublished document: {}", path.display());
        return Ok(None);
    }
    
    // Create document ID
    let id = if let Some(rel_path) = path.strip_prefix(collection_dir).ok() {
        rel_path.to_string_lossy().to_string()
    } else {
        path.file_name().unwrap().to_string_lossy().to_string()
    };
    
    // Create document with extracted data
    let document = Document {
        id,
        path: path.to_path_buf(),
        relative_path: path.strip_prefix(collection_dir)?.to_path_buf(),
        output_path: None,
        url: None,
        collection: collection_label.to_string(),
        date: front_matter.get_date(),
        content,
        rendered_content: None,
        front_matter,
        excerpt: None,
    };
    
    Ok(Some(document))
}

/// Parse a post file (includes special date handling)
pub fn parse_post(
    path: &Path,
    posts_dir: &Path,
    include_unpublished: bool
) -> BoxResult<Option<Document>> {
    debug!("Parsing post: {}", path.display());
    
    // Read the file content
    let content = fs::read_to_string(path)?;
    
    // Extract front matter and content
    let (mut front_matter, content) = crate::front_matter::extract_front_matter(&content)?;
    
    // Skip if published is false and we're not including unpublished
    if let Some(false) = front_matter.published {
        if !include_unpublished {
            debug!("Skipping unpublished post: {}", path.display());
            return Ok(None);
        }
    }
    
    // Try to extract date from filename if not in front matter
    if front_matter.date.is_none() {
        if let Some(dt) = extract_date_from_filename(path) {
            front_matter.date = Some(dt.to_string());
        }
    }
    
    // Create document ID
    let id = if let Some(rel_path) = path.strip_prefix(posts_dir).ok() {
        rel_path.to_string_lossy().to_string()
    } else {
        path.file_name().unwrap().to_string_lossy().to_string()
    };
    
    // Create document with extracted data
    let document = Document {
        id,
        path: path.to_path_buf(),
        relative_path: path.strip_prefix(posts_dir)?.to_path_buf(),
        output_path: None,
        url: None,
        collection: "posts".to_string(),
        date: front_matter.get_date(),
        content,
        rendered_content: None,
        front_matter,
        excerpt: None,
    };
    
    Ok(Some(document))
}

/// Parse a draft post
pub fn parse_draft(
    path: &Path,
    drafts_dir: &Path,
    include_unpublished: bool
) -> BoxResult<Option<Document>> {
    debug!("Parsing draft: {}", path.display());
    
    // Read the file content
    let content = fs::read_to_string(path)?;
    
    // Extract front matter and content
    let (mut front_matter, content) = crate::front_matter::extract_front_matter(&content)?;
    
    // Skip if published is false and we're not including unpublished
    if let Some(false) = front_matter.published {
        if !include_unpublished {
            debug!("Skipping unpublished draft: {}", path.display());
            return Ok(None);
        }
    }
    
    // Drafts always get the current date
    let now = Utc::now();
    front_matter.date = Some(now.to_string());
    
    // Create document ID
    let id = if let Some(rel_path) = path.strip_prefix(drafts_dir).ok() {
        rel_path.to_string_lossy().to_string()
    } else {
        path.file_name().unwrap().to_string_lossy().to_string()
    };
    
    // Create document with extracted data
    let document = Document {
        id,
        path: path.to_path_buf(),
        relative_path: path.strip_prefix(drafts_dir)?.to_path_buf(),
        output_path: None,
        url: None,
        collection: "posts".to_string(),
        date: front_matter.get_date(),
        content,
        rendered_content: None,
        front_matter,
        excerpt: None,
    };
    
    Ok(Some(document))
}

/// Extract date from a filename in the format YYYY-MM-DD-title.md
pub fn extract_date_from_filename(path: &Path) -> Option<DateTime<Utc>> {
    let filename = path.file_name()?.to_string_lossy();
    let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})-").ok()?;
    
    if let Some(caps) = re.captures(&filename) {
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        let day: u32 = caps.get(3)?.as_str().parse().ok()?;
        
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)?;
        let datetime = naive_date.and_hms_opt(0, 0, 0)?;
        
        return Some(Utc.from_utc_datetime(&datetime));
    }
    
    None
} 