use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use log::{info, debug, error, warn};
use liquid::Object;
use rayon::prelude::*;

use crate::config::Config;
use crate::directory::DirectoryStructure;
use crate::markdown::MarkdownRenderer;
use crate::collections::Collection;
use crate::builder::page::Page;
use crate::builder::types::BoxResult;
use crate::liquid::create_globals;
use html_escape;

/// Process and render collections
pub fn process_collections(
    collections: &mut HashMap<String, Collection>,
    layouts: &HashMap<String, String>,
    parser: &liquid::Parser,
    site_data: &Object,
    markdown_renderer: &MarkdownRenderer,
    dirs: &DirectoryStructure,
    config: &Config
) -> BoxResult<()> {
    info!("Processing collections...");
    
    // Use a thread-safe counter for statistics
    let processed_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));
    
    // Get CPU count to optimize parallelism
    let cpu_count = num_cpus::get();
    info!("Using {} CPU cores for parallel processing", cpu_count);
    
    // Process each collection
    for (label, collection) in collections.iter_mut() {
        // Skip collections that don't output
        if !collection.output {
            debug!("Skipping collection '{}' (output: false)", label);
            continue;
        }
        
        info!("Processing collection '{}'", label);
        let collection_docs_count = collection.documents.len();
        
        // Create parent directories for output files first (this avoids race conditions)
        let output_paths: Vec<_> = collection.documents.iter()
            .filter_map(|doc| {
                // Determine output path
                if let Some(url) = &doc.url {
                    // Remove leading slash from URL
                    let path_str = if url.starts_with('/') {
                        &url[1..]
                    } else {
                        url
                    };
                    let relative_path = Path::new(path_str).to_path_buf();
                    let output_path = dirs.destination.join(&relative_path);
                    Some(output_path)
                } else {
                    // If no URL, use the relative path with .html extension
                    let mut output_path = doc.relative_path.clone();
                    output_path.set_extension("html");
                    let output_path = dirs.destination.join(&output_path);
                    Some(output_path)
                }
            })
            .collect();
            
        // Create all output directories in parallel
        output_paths.par_iter().for_each(|path| {
            if let Some(parent) = path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("Failed to create directory {}: {}", parent.display(), e);
                }
            }
        });
        
        // Thread-safe wrappers for resources
        let layouts = Arc::new(layouts.clone());
        let parser = Arc::new(parser.clone());
        let site_data = Arc::new(site_data.clone());
        let markdown_renderer = Arc::new(markdown_renderer.clone());
        let config = Arc::new(config.clone());
        let dirs = Arc::new(dirs.clone());
        
        // Process documents in parallel
        collection.documents.par_iter_mut().for_each(|doc| {
            // Create the output path for the document
            let relative_path = if let Some(url) = &doc.url {
                // Remove leading slash from URL
                let path_str = if url.starts_with('/') {
                    &url[1..]
                } else {
                    url
                };
                Path::new(path_str).to_path_buf()
            } else {
                // If no URL, use the relative path with .html extension
                let mut output_path = doc.relative_path.clone();
                output_path.set_extension("html");
                output_path
            };
            
            // Set the absolute output path
            let output_path = dirs.destination.join(&relative_path);
            doc.output_path = Some(output_path.clone());
            
            // Create globals for this document
            let mut globals = create_globals(&config, Some(&site_data));
            globals.insert("page".into(), crate::collections::document_to_liquid(doc));
            globals.insert("content".into(), liquid::model::Value::scalar(doc.content.clone()));
            
            // Render content (markdown or liquid)
            let rendered_content = match if is_markdown_file(&doc.path, &config) {
                // Render markdown to HTML
                let html_content = markdown_renderer.render(&doc.content);
                
                // Then process with Liquid - decode HTML entities before parsing
                let decoded_content = html_escape::decode_html_entities(&html_content).to_string();
                match parser.parse(&decoded_content) {
                    Ok(template) => match template.render(&globals) {
                        Ok(content) => Ok(content),
                        Err(e) => Err(format!("Error rendering liquid in markdown: {}", e))
                    },
                    Err(e) => Err(format!("Error parsing liquid in markdown: {}", e))
                }
            } else {
                // Just process with Liquid - decode HTML entities before parsing
                let decoded_content = html_escape::decode_html_entities(&doc.content).to_string();
                match parser.parse(&decoded_content) {
                    Ok(template) => match template.render(&globals) {
                        Ok(content) => Ok(content),
                        Err(e) => Err(format!("Error rendering liquid: {}", e))
                    },
                    Err(e) => Err(format!("Error parsing liquid: {}", e))
                }
            } {
                Ok(content) => content,
                Err(e) => {
                    error!("Error processing document {}: {}", doc.path.display(), e);
                    let mut error_count = error_count.lock().unwrap();
                    *error_count += 1;
                    return;
                }
            };
            
            // Store the rendered content
            doc.rendered_content = Some(rendered_content.clone());
            
            // Update the globals with the rendered content
            globals.insert("content".into(), liquid::model::Value::scalar(rendered_content.clone()));
            
            // Apply layout if specified
            let final_content = if let Some(layout) = &doc.front_matter.layout {
                match apply_layout(&rendered_content, layout, &layouts, &parser, &globals, &config) {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Error applying layout to {}: {}", doc.path.display(), e);
                        let mut error_count = error_count.lock().unwrap();
                        *error_count += 1;
                        return;
                    }
                }
            } else {
                rendered_content
            };
            
            // Write the final content to the output file
            match File::create(&output_path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(final_content.as_bytes()) {
                        error!("Error writing to {}: {}", output_path.display(), e);
                        let mut error_count = error_count.lock().unwrap();
                        *error_count += 1;
                    } else {
                        debug!("Generated {}", output_path.display());
                        let mut processed_count = processed_count.lock().unwrap();
                        *processed_count += 1;
                    }
                },
                Err(e) => {
                    error!("Error creating file {}: {}", output_path.display(), e);
                    let mut error_count = error_count.lock().unwrap();
                    *error_count += 1;
                }
            }
        });
        
        info!("Processed {}/{} documents in collection '{}'", 
              collection_docs_count - *error_count.lock().unwrap(), 
              collection_docs_count,
              label);
    }
    
    let total_processed = *processed_count.lock().unwrap();
    let total_errors = *error_count.lock().unwrap();
    if total_errors > 0 {
        warn!("Completed with {} documents processed and {} errors", total_processed, total_errors);
    } else {
        info!("Successfully processed {} documents", total_processed);
    }
    
    Ok(())
}

/// Process and render pages
pub fn process_pages(
    pages: Vec<Page>,
    layouts: &HashMap<String, String>,
    parser: &liquid::Parser,
    site_data: &Object,
    markdown_renderer: &MarkdownRenderer,
    config: &Config
) -> BoxResult<()> {
    info!("Processing pages...");
    
    // Use a thread-safe counter for statistics
    let processed_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));
    let _total_pages = pages.len();
    
    // Create output directories first to avoid race conditions
    let output_paths: Vec<_> = pages.iter()
        .filter_map(|page| page.output_path.as_ref().cloned())
        .collect();
        
    output_paths.par_iter().for_each(|path| {
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create directory {}: {}", parent.display(), e);
            }
        }
    });
    
    // Thread-safe wrappers for resources
    let layouts = Arc::new(layouts.clone());
    let parser = Arc::new(parser.clone());
    let site_data = Arc::new(site_data.clone());
    let markdown_renderer = Arc::new(markdown_renderer.clone());
    let config = Arc::new(config.clone());
    
    // Process pages in parallel
    pages.into_par_iter().for_each(|page| {
        // Skip pages that don't have an output path
        if page.output_path.is_none() {
            debug!("Skipping page {} (no output path)", page.path.display());
            return;
        }
        
        let output_path = page.output_path.as_ref().unwrap();
        
        // Handle special static files
        if !page.process {
            // Just copy the file
            if let Err(e) = fs::copy(&page.path, output_path) {
                error!("Error copying static file {}: {}", page.path.display(), e);
                let mut error_count = error_count.lock().unwrap();
                *error_count += 1;
            } else {
                let mut processed_count = processed_count.lock().unwrap();
                *processed_count += 1;
            }
            return;
        }
        
        // Create globals for this page
        let mut globals = create_globals(&config, Some(&site_data));
        globals.insert("page".into(), crate::builder::site::page_to_liquid(&page));
        globals.insert("content".into(), liquid::model::Value::scalar(page.content.clone()));
        
        // Render content (markdown or liquid)
        let rendered_content = match if is_markdown_file(&page.path, &config) {
            // Render markdown to HTML
            let html_content = markdown_renderer.render(&page.content);
            
            // Then process with Liquid - decode HTML entities before parsing
            let decoded_content = html_escape::decode_html_entities(&html_content).to_string();
            match parser.parse(&decoded_content) {
                Ok(template) => match template.render(&globals) {
                    Ok(content) => Ok(content),
                    Err(e) => Err(format!("Error rendering liquid in markdown: {}", e))
                },
                Err(e) => Err(format!("Error parsing liquid in markdown: {}", e))
            }
        } else {
            // Just process with Liquid - decode HTML entities before parsing
            let decoded_content = html_escape::decode_html_entities(&page.content).to_string();
            match parser.parse(&decoded_content) {
                Ok(template) => match template.render(&globals) {
                    Ok(content) => Ok(content),
                    Err(e) => Err(format!("Error rendering liquid: {}", e))
                },
                Err(e) => Err(format!("Error parsing liquid: {}", e))
            }
        } {
            Ok(content) => content,
            Err(e) => {
                error!("Error processing page {}: {}", page.path.display(), e);
                let mut error_count = error_count.lock().unwrap();
                *error_count += 1;
                return;
            }
        };
        
        // Update the globals with the rendered content
        globals.insert("content".into(), liquid::model::Value::scalar(rendered_content.clone()));
        
        // Apply layout if specified
        let final_content = if let Some(layout) = &page.front_matter.layout {
            match apply_layout(&rendered_content, layout, &layouts, &parser, &globals, &config) {
                Ok(content) => content,
                Err(e) => {
                    error!("Error applying layout to {}: {}", page.path.display(), e);
                    let mut error_count = error_count.lock().unwrap();
                    *error_count += 1;
                    return;
                }
            }
        } else {
            rendered_content
        };
        
        // Write the final content to the output file
        match File::create(output_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(final_content.as_bytes()) {
                    error!("Error writing to {}: {}", output_path.display(), e);
                    let mut error_count = error_count.lock().unwrap();
                    *error_count += 1;
                } else {
                    debug!("Generated {}", output_path.display());
                    let mut processed_count = processed_count.lock().unwrap();
                    *processed_count += 1;
                }
            },
            Err(e) => {
                error!("Error creating file {}: {}", output_path.display(), e);
                let mut error_count = error_count.lock().unwrap();
                *error_count += 1;
            }
        }
    });
    
    let total_processed = *processed_count.lock().unwrap();
    let total_errors = *error_count.lock().unwrap();
    if total_errors > 0 {
        warn!("Completed with {} pages processed and {} errors", total_processed, total_errors);
    } else {
        info!("Successfully processed {} pages", total_processed);
    }
    
    Ok(())
}

/// Apply a layout to content
pub fn apply_layout(
    _content: &str,
    layout_name: &str,
    layouts: &HashMap<String, String>,
    parser: &liquid::Parser,
    globals: &Object,
    config: &Config
) -> BoxResult<String> {
    // Get the layout content
    let layout_content = layouts.get(layout_name)
        .ok_or_else(|| format!("Layout '{}' not found", layout_name))?;
    
    // Create new template with the layout content
    let template = parser.parse(layout_content)?;
    
    // Render the layout with the content
    let rendered = template.render(globals)?;
    
    // Check if the layout has a parent layout
    if let Some(parent_layout) = get_parent_layout(layout_name, layouts) {
        // Recursively apply parent layout
        let mut new_globals = globals.clone();
        new_globals.insert("content".into(), liquid::model::Value::scalar(rendered.clone()));
        apply_layout(&rendered, &parent_layout, layouts, parser, &new_globals, config)
    } else {
        // No parent layout, return the rendered content
        Ok(rendered)
    }
}

/// Extract the parent layout name from a layout
fn get_parent_layout(layout_name: &str, layouts: &HashMap<String, String>) -> Option<String> {
    if let Some(layout_content) = layouts.get(layout_name) {
        // Look for front matter in the layout
        if layout_content.starts_with("---") {
            if let Some(end_index) = layout_content.find("---\n") {
                let front_matter = &layout_content[3..end_index];
                
                // Look for layout: line
                for line in front_matter.lines() {
                    if line.trim().starts_with("layout:") {
                        let parent = line.trim()
                            .strip_prefix("layout:")
                            .map(|s| s.trim().to_string());
                        
                        if let Some(parent) = parent {
                            // Don't return empty layouts
                            if !parent.is_empty() {
                                return Some(parent);
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Check if a file is a markdown file
pub fn is_markdown_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        
        // Check against markdown extensions list from config
        for md_ext in &config.markdown_ext {
            if ext_str == md_ext.to_lowercase() {
                return true;
            }
        }
    }
    
    false
} 