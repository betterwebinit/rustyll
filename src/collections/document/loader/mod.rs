mod parsers;

use std::collections::HashMap;
use std::path::Path;
use log::{info, debug};
use walkdir::WalkDir;

use crate::config::{Config, CollectionConfig};
use crate::collections::document::model::{Collection, Document};
use crate::collections::types::BoxResult;

pub use parsers::{parse_document, parse_post, parse_draft};

/// Load collections from the filesystem based on configuration
pub fn load_collections(config: &Config) -> BoxResult<HashMap<String, Collection>> {
    info!("Loading collections...");
    let mut collections = HashMap::new();
    
    // Get posts collection config or create default
    let posts_config = match config.collections.items.get("posts") {
        Some(config) => config,
        None => {
            // Default posts config
            &CollectionConfig {
                output: true,
                permalink: None,
                sort_by: "date".to_string(),
                defaults: Vec::new(),
            }
        }
    };
    
    let mut posts_collection = Collection::new(
        "posts",
        posts_config,
        &config.source,
        None // No collections_dir
    );
    
    // Load user-defined collections
    for (label, collection_config) in &config.collections {
        if label == "posts" {
            // We already handled posts separately
            continue;
        }
        
        let collection = Collection::new(
            label,
            collection_config,
            &config.source,
            None // No collections_dir
        );
        
        collections.insert(label.clone(), collection);
    }
    
    // Load documents for each collection
    for (label, collection) in &mut collections {
        load_collection_documents(collection, config)?;
    }
    
    // Load posts separately since they have special handling
    load_posts(&mut posts_collection, config, false, false)?;
    collections.insert("posts".to_string(), posts_collection);
    
    Ok(collections)
}

/// Load documents for a regular collection
fn load_collection_documents(collection: &mut Collection, config: &Config) -> BoxResult<()> {
    debug!("Loading documents for collection: {}", collection.label);
    
    if !collection.directory.exists() {
        debug!("Collection directory does not exist: {}", collection.directory.display());
        return Ok(());
    }
    
    // Walk the directory and find all markdown files
    for entry in WalkDir::new(&collection.directory).follow_links(true) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension().unwrap_or_default().to_string_lossy();
            if extension == "md" || extension == "markdown" {
                if let Some(doc) = parse_document(path, &collection.directory, &collection.label)? {
                    // Generate URL for the document
                    let url = collection.generate_url(&doc, config);
                    
                    // Add document to collection with URL
                    let mut doc = doc;
                    doc.url = url;
                    collection.documents.push(doc);
                }
            }
        }
    }
    
    debug!("Loaded {} documents for collection: {}", collection.documents.len(), collection.label);
    
    Ok(())
}

/// Load posts including optional drafts
fn load_posts(
    collection: &mut Collection,
    config: &Config,
    include_drafts: bool,
    include_unpublished: bool
) -> BoxResult<()> {
    debug!("Loading posts...");
    
    // Load regular posts
    let posts_dir = collection.directory.clone();
    if posts_dir.exists() {
        // Walk the directory and find all markdown files
        for entry in WalkDir::new(&posts_dir).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let extension = path.extension().unwrap_or_default().to_string_lossy();
                if extension == "md" || extension == "markdown" {
                    if let Some(doc) = parse_post(path, &posts_dir, include_unpublished)? {
                        // Generate URL for the document
                        let url = collection.generate_url(&doc, config);
                        
                        // Add document to collection with URL
                        let mut doc = doc;
                        doc.url = url;
                        collection.documents.push(doc);
                    }
                }
            }
        }
    }
    
    // Load drafts if requested
    if include_drafts {
        let drafts_dir = config.source.join("_drafts");
        if drafts_dir.exists() {
            // Walk the directory and find all markdown files
            for entry in WalkDir::new(&drafts_dir).follow_links(true) {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let extension = path.extension().unwrap_or_default().to_string_lossy();
                    if extension == "md" || extension == "markdown" {
                        if let Some(doc) = parse_draft(path, &drafts_dir, include_unpublished)? {
                            // Generate URL for the document
                            let url = collection.generate_url(&doc, config);
                            
                            // Add document to collection with URL
                            let mut doc = doc;
                            doc.url = url;
                            collection.documents.push(doc);
                        }
                    }
                }
            }
        }
    }
    
    debug!("Loaded {} posts", collection.documents.len());
    
    Ok(())
}

// Helper function to load documents from a directory
fn load_documents(dir: &Path, collection: &str, _config: &Config) -> BoxResult<Vec<Document>> {
    // ... existing implementation ...
    Ok(Vec::new()) // Replace with actual implementation
} 