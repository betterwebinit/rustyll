use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use log::{info, debug, error};
use liquid::Object;

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
    
    // Process each collection
    for (label, collection) in collections.iter_mut() {
        // Skip collections that don't output
        if !collection.output {
            debug!("Skipping collection '{}' (output: false)", label);
            continue;
        }
        
        info!("Processing collection '{}'", label);
        
        // Process each document in the collection
        for doc in &mut collection.documents {
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
            
            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Create globals for this document
            let mut globals = create_globals(config, Some(&site_data));
            globals.insert("page".into(), crate::collections::document_to_liquid(doc));
            globals.insert("content".into(), liquid::model::Value::scalar(doc.content.clone()));
            
            // Render content (markdown or liquid)
            let rendered_content = if is_markdown_file(&doc.path, config) {
                // Render markdown to HTML
                let html_content = markdown_renderer.render(&doc.content);
                
                // Then process with Liquid - decode HTML entities before parsing
                let decoded_content = html_escape::decode_html_entities(&html_content).to_string();
                let template = parser.parse(&decoded_content)?;
                template.render(&globals)?
            } else {
                // Just process with Liquid - decode HTML entities before parsing
                let decoded_content = html_escape::decode_html_entities(&doc.content).to_string();
                let template = parser.parse(&decoded_content)?;
                template.render(&globals)?
            };
            
            // Store the rendered content
            doc.rendered_content = Some(rendered_content.clone());
            
            // Update the globals with the rendered content
            globals.insert("content".into(), liquid::model::Value::scalar(rendered_content.clone()));
            
            // Apply layout if specified
            let final_content = if let Some(layout) = &doc.front_matter.layout {
                apply_layout(&rendered_content, layout, layouts, parser, &globals, config)?
            } else {
                rendered_content
            };
            
            // Write the final content to the output file
            let mut file = File::create(&output_path)?;
            file.write_all(final_content.as_bytes())?;
            
            debug!("Generated {}", output_path.display());
        }
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
    
    for page in pages {
        // Skip pages that don't have an output path
        if page.output_path.is_none() {
            debug!("Skipping page {} (no output path)", page.path.display());
            continue;
        }
        
        let output_path = page.output_path.as_ref().unwrap();
        
        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Handle special static files
        if !page.process {
            // Just copy the file
            if let Err(e) = fs::copy(&page.path, output_path) {
                error!("Error copying static file {}: {}", page.path.display(), e);
            }
            continue;
        }
        
        // Create globals for this page
        let mut globals = create_globals(config, Some(&site_data));
        globals.insert("page".into(), crate::builder::site::page_to_liquid(&page));
        globals.insert("content".into(), liquid::model::Value::scalar(page.content.clone()));
        
        // Render content (markdown or liquid)
        let rendered_content = if is_markdown_file(&page.path, config) {
            // Render markdown to HTML
            let html_content = markdown_renderer.render(&page.content);
            
            // Then process with Liquid - decode HTML entities before parsing
            let decoded_content = html_escape::decode_html_entities(&html_content).to_string();
            let template = parser.parse(&decoded_content)?;
            template.render(&globals)?
        } else {
            // Just process with Liquid - decode HTML entities before parsing
            let decoded_content = html_escape::decode_html_entities(&page.content).to_string();
            let template = parser.parse(&decoded_content)?;
            template.render(&globals)?
        };
        
        // Update the globals with the rendered content
        globals.insert("content".into(), liquid::model::Value::scalar(rendered_content.clone()));
        
        // Apply layout if specified
        let final_content = if let Some(layout) = &page.front_matter.layout {
            apply_layout(&rendered_content, layout, layouts, parser, &globals, config)?
        } else {
            rendered_content
        };
        
        // Write the final content to the output file
        let mut file = File::create(output_path)?;
        file.write_all(final_content.as_bytes())?;
        
        debug!("Generated {}", output_path.display());
    }
    
    Ok(())
}

/// Apply a layout to content
pub fn apply_layout(
    content: &str,
    layout_name: &str,
    layouts: &HashMap<String, String>,
    parser: &liquid::Parser,
    globals: &Object,
    config: &Config
) -> BoxResult<String> {
    // Get the layout content
    let layout_content = layouts.get(layout_name)
        .ok_or_else(|| format!("Layout '{}' not found", layout_name))?;
    
    // Create a copy of globals with content included
    let mut layout_globals = globals.clone();
    layout_globals.insert("content".into(), liquid::model::Value::scalar(content.to_string()));
    
    // Parse and render the layout - decode HTML entities before parsing
    let decoded_content = html_escape::decode_html_entities(layout_content).to_string();
    let template = parser.parse(&decoded_content)?;
    let rendered = template.render(&layout_globals)?;
    
    // Check if the layout itself has a parent layout
    let layout_front_matter = crate::front_matter::extract_front_matter_only(layout_content)?;
    
    if let Some(parent_layout) = &layout_front_matter.layout {
        // Apply the parent layout
        apply_layout(&rendered, parent_layout, layouts, parser, globals, config)
    } else {
        // No parent layout, just return the rendered content
        Ok(rendered)
    }
}

/// Check if a file is a markdown file based on extension
pub fn is_markdown_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return config.markdown_ext.contains(&ext_str);
    }
    false
} 