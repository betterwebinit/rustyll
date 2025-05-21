use crate::config::Config;
use crate::directory::{DirectoryStructure, clean_destination};
use crate::collections::{load_collections, load_data_files, collections_to_liquid};
use crate::liquid::{create_jekyll_parser, create_site_object};
use crate::markdown::MarkdownRenderer;
use crate::builder::page::{Page, collect_pages};
use crate::builder::types::BoxResult;
use crate::builder::site::{
    load_layouts, 
    load_includes, 
    process_collections, 
    process_pages,
    data_to_liquid
};

use std::collections::HashMap;
use std::fs;
use log::{info, debug};
use liquid::Object;
use liquid::model::Value;

/// Build a Jekyll-compatible static site
pub fn build_site(config: &Config, include_drafts: bool, include_unpublished: bool) -> BoxResult<()> {
    let start_time = std::time::Instant::now();

    // Create directory structure
    let mut dirs = DirectoryStructure::from_config(config);
    info!("Using source directory: {}", dirs.source.display());
    info!("Output will be generated in: {}", dirs.destination.display());

    // Clean the destination directory
    clean_destination(config)?;

    // Create destination directory and other required directories
    dirs.create_site_directories()?;

    // Load collections (includes posts)
    info!("Loading collections...");
    let mut collections = load_collections(config)?;
    
    // Get posts collection for convenience
    let posts = collections.get("posts")
        .ok_or("Posts collection not found")?
        .clone();

    // Load pages
    info!("Loading pages...");
    let pages = collect_pages(&dirs)?;

    // Load layouts
    info!("Loading layouts...");
    let layouts = load_layouts(&dirs)?;

    // Load includes
    info!("Loading includes...");
    let includes = load_includes(&dirs)?;

    // Load data files
    info!("Loading data files...");
    let data = load_data_files(config)?;

    // Copy static files
    info!("Copying static files...");
    let copied_count = dirs.copy_static_files()?;
    info!("Copied {} static files", copied_count);

    // Create the Liquid parser with custom tags and filters
    info!("Setting up template engine...");
    let parser = create_jekyll_parser(config, includes)?;

    // Create the Markdown renderer
    let markdown_renderer = MarkdownRenderer::new(config);

    // Create the site object with all collections and data
    let mut site_data = create_site_object(config);
    
    // Add collections to site object
    let collections_data = collections_to_liquid(&collections, config);
    site_data.insert("collections".into(), Value::Object(collections_data));
    
    // Add pages to site object
    let pages_array = pages.iter()
        .map(|page| crate::builder::site::page_to_liquid(page))
        .collect::<Vec<Value>>();
    site_data.insert("pages".into(), Value::Array(pages_array));
    
    // Add posts directly to site object
    if let Some(posts_collection) = collections.get("posts") {
        let posts_array = posts_collection.documents.iter()
            .map(|doc| crate::collections::document_to_liquid(doc))
            .collect::<Vec<Value>>();
        site_data.insert("posts".into(), Value::Array(posts_array.clone()));
        
        // Add HTML pages subset
        let html_pages = pages.iter()
            .filter(|page| {
                if let Some(ext) = page.path.extension() {
                    return ext == "html";
                }
                false
            })
            .map(|page| crate::builder::site::page_to_liquid(page))
            .collect::<Vec<Value>>();
        site_data.insert("html_pages".into(), Value::Array(html_pages));
        
        // Collect categories and tags from posts
        let mut categories = std::collections::HashMap::new();
        let mut tags = std::collections::HashMap::new();
        
        for doc in &posts_collection.documents {
            // Process categories
            if let Some(doc_categories) = &doc.front_matter.categories {
                for category in doc_categories {
                    let category_posts = categories.entry(category.clone())
                        .or_insert_with(Vec::new);
                    category_posts.push(crate::collections::document_to_liquid(doc));
                }
            }
            
            // Process tags
            if let Some(doc_tags) = &doc.front_matter.tags {
                for tag in doc_tags {
                    let tag_posts = tags.entry(tag.clone())
                        .or_insert_with(Vec::new);
                    tag_posts.push(crate::collections::document_to_liquid(doc));
                }
            }
        }
        
        // Convert categories and tags to Liquid objects
        let mut categories_obj = liquid::Object::new();
        for (category, posts) in categories {
            categories_obj.insert(category.into(), Value::Array(posts));
        }
        site_data.insert("categories".into(), Value::Object(categories_obj));
        
        let mut tags_obj = liquid::Object::new();
        for (tag, posts) in tags {
            tags_obj.insert(tag.into(), Value::Array(posts));
        }
        site_data.insert("tags".into(), Value::Object(tags_obj));
    }
    
    // Add data files to site object
    let data_object = data_to_liquid(&data);
    site_data.insert("data".into(), Value::Object(data_object));

    // Process collections
    for (name, collection) in &mut collections {
        // First collect all documents that need URLs
        let docs_count = collection.documents.len();
        let mut urls = Vec::with_capacity(docs_count);
        
        // Generate URLs without mutating documents
        for doc in &collection.documents {
            let url = collection.generate_url(doc, config);
            urls.push(url);
        }
        
        // Now assign the URLs to documents
        for (i, doc) in collection.documents.iter_mut().enumerate() {
            doc.url = urls.get(i).cloned().flatten();
        }
    }
    
    // Process and render collections (including posts)
    process_collections(&mut collections, &layouts, &parser, &site_data, &markdown_renderer, &dirs, config)?;
    
    // Process and render pages
    process_pages(pages, &layouts, &parser, &site_data, &markdown_renderer, config)?;

    // Debug output all available data
    log::debug!("Data objects:");
    for (key, value) in &data {
        log::debug!("- data[{}]: {:?}", key, value);
    }

    // Debug output the complete site object
    log::debug!("Complete site object:");
    for (key, value) in &site_data {
        log::debug!("- site.{}: {:?}", key, value);
    }

    let elapsed = start_time.elapsed();
    info!("Site built in {:.2?}", elapsed);

    Ok(())
} 