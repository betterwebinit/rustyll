use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc, Timelike, Datelike};

use crate::front_matter::types::FrontMatter;
use crate::utils::path;

/// Permalink template patterns
pub enum PermalinkStyle {
    /// Date style: /:categories/:year/:month/:day/:title.html
    Date,
    /// Pretty style: /:categories/:year/:month/:day/:title/
    Pretty,
    /// Ordinal style: /:categories/:year/:y_day/:title.html
    Ordinal,
    /// None style: /:categories/:title.html
    None,
    /// Custom pattern
    Custom(String),
}

impl From<&str> for PermalinkStyle {
    fn from(s: &str) -> Self {
        match s {
            "date" => PermalinkStyle::Date,
            "pretty" => PermalinkStyle::Pretty,
            "ordinal" => PermalinkStyle::Ordinal,
            "none" => PermalinkStyle::None,
            _ => PermalinkStyle::Custom(s.to_string()),
        }
    }
}

impl PermalinkStyle {
    /// Get the pattern string for this permalink style
    pub fn pattern(&self) -> String {
        match self {
            PermalinkStyle::Date => "/:categories/:year/:month/:day/:title.html".to_string(),
            PermalinkStyle::Pretty => "/:categories/:year/:month/:day/:title/".to_string(),
            PermalinkStyle::Ordinal => "/:categories/:year/:y_day/:title.html".to_string(),
            PermalinkStyle::None => "/:categories/:title.html".to_string(),
            PermalinkStyle::Custom(pattern) => pattern.clone(),
        }
    }
}

/// Process a permalink pattern with front matter data
pub fn process_permalink(
    pattern: &str,
    front_matter: &FrontMatter,
    collection: Option<&str>,
    source_path: &Path,
) -> String {
    let mut result = pattern.to_string();
    let mut placeholders = HashMap::<String, String>::new();
    
    // Add basic placeholders
    if let Some(title) = &front_matter.title {
        placeholders.insert("title".to_string(), slugify(title));
    } else if let Some(stem) = path::get_stem(source_path) {
        placeholders.insert("title".to_string(), slugify(&stem));
    } else {
        placeholders.insert("title".to_string(), "untitled".to_string());
    }
    
    // Add date-based placeholders if available
    if let Some(date) = front_matter.get_date() {
        add_date_placeholders(&mut placeholders, date);
    }
    
    // Add collection name if available
    if let Some(coll) = collection {
        placeholders.insert("collection".to_string(), coll.to_string());
    }
    
    // Add categories if available
    if let Some(categories) = &front_matter.categories {
        let categories_path = categories.join("/");
        placeholders.insert("categories".to_string(), categories_path);
    } else {
        // Replace with empty if no categories
        result = result.replace("/:categories", "");
        result = result.replace(":categories/", "");
        result = result.replace(":categories", "");
    }
    
    // Process output extension
    let output_ext = if result.ends_with('/') {
        "".to_string()
    } else if !result.contains(".") {
        ".html".to_string()
    } else {
        "".to_string()
    };
    placeholders.insert("output_ext".to_string(), output_ext);
    
    // Replace placeholders
    for (key, value) in placeholders {
        result = result.replace(&format!(":{}", key), &value);
    }
    
    // Clean up any double slashes
    while result.contains("//") {
        result = result.replace("//", "/");
    }
    
    result
}

/// Add date-based placeholders to the map
fn add_date_placeholders(placeholders: &mut HashMap<String, String>, date: DateTime<Utc>) {
    placeholders.insert("year".to_string(), date.year().to_string());
    placeholders.insert("month".to_string(), format!("{:02}", date.month()));
    placeholders.insert("day".to_string(), format!("{:02}", date.day()));
    placeholders.insert("hour".to_string(), format!("{:02}", date.hour()));
    placeholders.insert("minute".to_string(), format!("{:02}", date.minute()));
    placeholders.insert("second".to_string(), format!("{:02}", date.second()));
    
    // Ordinal day of year
    let ordinal_day = date.ordinal();
    placeholders.insert("y_day".to_string(), format!("{:03}", ordinal_day));
    
    // Short versions
    placeholders.insert("i_month".to_string(), date.month().to_string());
    placeholders.insert("i_day".to_string(), date.day().to_string());
}

/// Convert a string to a URL-friendly slug
fn slugify(input: &str) -> String {
    let mut slug = input
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-");
    
    // Replace multiple consecutive dashes with a single dash
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    
    // Remove leading and trailing dashes
    slug = slug.trim_start_matches('-').trim_end_matches('-').to_string();
    
    slug
} 