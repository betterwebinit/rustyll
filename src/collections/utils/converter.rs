use std::collections::HashMap;
use liquid::model::Value;
use liquid::Object;

use crate::config::Config;
use crate::collections::document::model::Collection;
use crate::collections::document::converter::document_to_liquid;

/// Convert collections to a Liquid object for template rendering
pub fn collections_to_liquid(
    collections: &HashMap<String, Collection>,
    _config: &Config
) -> liquid::Object {
    let mut liquid_collections = liquid::Object::new();
    
    // Add each collection
    for (label, collection) in collections {
        let mut liquid_collection = liquid::Object::new();
        
        // Collection metadata
        liquid_collection.insert("label".into(), Value::scalar(collection.label.clone()));
        liquid_collection.insert("output".into(), Value::scalar(collection.output));
        liquid_collection.insert("relative_directory".into(), 
            Value::scalar(collection.relative_directory.to_string_lossy().to_string()));
        liquid_collection.insert("directory".into(),
            Value::scalar(collection.directory.to_string_lossy().to_string()));
        
        // Collection documents
        let docs: Vec<Value> = collection.documents.iter()
            .map(|doc| document_to_liquid(doc))
            .collect();
        
        liquid_collection.insert("docs".into(), Value::Array(docs.clone()));
        
        // Add collection to collections map
        liquid_collections.insert(label.clone().into(), Value::Object(liquid_collection));
        
        // For each collection, also add its documents directly to site.<collection_name>
        // This is how Jekyll makes site.posts, site.projects, etc. available
        if collection.output {
            liquid_collections.insert(collection.label.clone().into(), Value::Array(docs));
        }
    }
    
    liquid_collections
} 