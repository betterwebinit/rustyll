use liquid::model::Value;
use liquid::Object;

use crate::collections::document::model::Document;

/// Convert a document to a Liquid value for template rendering
pub fn document_to_liquid(doc: &Document) -> liquid::model::Value {
    let mut obj = liquid::Object::new();
    
    // Basic properties
    obj.insert("id".into(), Value::scalar(doc.id.clone()));
    obj.insert("collection".into(), Value::scalar(doc.collection.clone()));
    obj.insert("content".into(), Value::scalar(doc.content.clone()));
    
    if let Some(rendered) = &doc.rendered_content {
        obj.insert("output".into(), Value::scalar(rendered.clone()));
    }
    
    if let Some(excerpt) = &doc.excerpt {
        obj.insert("excerpt".into(), Value::scalar(excerpt.clone()));
    }
    
    if let Some(url) = &doc.url {
        obj.insert("url".into(), Value::scalar(url.clone()));
    }
    
    if let Some(date) = &doc.date {
        // Convert DateTime to string in RFC3339 format
        obj.insert("date".into(), Value::scalar(date.to_rfc3339()));
    }
    
    // Path properties
    obj.insert("path".into(), Value::scalar(doc.path.to_string_lossy().to_string()));
    obj.insert("relative_path".into(), Value::scalar(doc.relative_path.to_string_lossy().to_string()));
    
    if let Some(output_path) = &doc.output_path {
        obj.insert("output_path".into(), Value::scalar(output_path.to_string_lossy().to_string()));
    }
    
    // Front matter
    let front_matter_obj = doc.front_matter.to_liquid_object();
    for (key, value) in front_matter_obj {
        obj.insert(key, value);
    }
    
    Value::Object(obj)
} 