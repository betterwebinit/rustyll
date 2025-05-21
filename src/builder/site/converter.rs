use std::collections::HashMap;
use liquid::model::Value;
use liquid::Object;

use crate::builder::page::Page;

/// Convert a page to a Liquid value for template rendering
pub fn page_to_liquid(page: &Page) -> liquid::model::Value {
    let mut obj = liquid::Object::new();
    
    // Basic properties
    obj.insert("path".into(), Value::scalar(page.path.to_string_lossy().to_string()));
    obj.insert("content".into(), Value::scalar(page.content.clone()));
    
    if let Some(output_path) = &page.output_path {
        obj.insert("output_path".into(), Value::scalar(output_path.to_string_lossy().to_string()));
    }
    
    if let Some(url) = &page.url {
        obj.insert("url".into(), Value::scalar(url.clone()));
    }
    
    if let Some(date) = &page.date {
        // Convert DateTime to string in RFC3339 format
        obj.insert("date".into(), Value::scalar(date.to_rfc3339()));
    }
    
    // Front matter properties
    if let Some(title) = &page.front_matter.title {
        obj.insert("title".into(), Value::scalar(title.clone()));
    }
    
    if let Some(description) = &page.front_matter.description {
        obj.insert("description".into(), Value::scalar(description.clone()));
    }
    
    if let Some(permalink) = &page.front_matter.permalink {
        obj.insert("permalink".into(), Value::scalar(permalink.clone()));
    }
    
    if let Some(layout) = &page.front_matter.layout {
        obj.insert("layout".into(), Value::scalar(layout.clone()));
    }
    
    if let Some(published) = page.front_matter.published {
        obj.insert("published".into(), Value::scalar(published));
    }
    
    // Categories and tags
    if let Some(categories) = &page.front_matter.categories {
        let categories_array = categories.iter()
            .map(|c| Value::scalar(c.clone()))
            .collect::<Vec<Value>>();
        obj.insert("categories".into(), Value::Array(categories_array));
    }
    
    if let Some(tags) = &page.front_matter.tags {
        let tags_array = tags.iter()
            .map(|t| Value::scalar(t.clone()))
            .collect::<Vec<Value>>();
        obj.insert("tags".into(), Value::Array(tags_array));
    }
    
    // Custom front matter fields
    for (key, value) in &page.front_matter.custom {
        if !obj.contains_key::<str>(key) {
            obj.insert(key.clone().into(), crate::builder::processor::yaml_to_liquid(value.clone()));
        }
    }
    
    Value::Object(obj)
}

/// Convert a data object to a Liquid object
pub fn data_to_liquid(data: &HashMap<String, liquid::model::Value>) -> liquid::Object {
    let mut obj = liquid::Object::new();
    
    for (key, value) in data {
        obj.insert(key.clone().into(), value.clone());
    }
    
    obj
} 