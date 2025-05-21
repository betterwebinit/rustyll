mod filters;
mod tags;

use std::collections::HashMap;
use std::error::Error;
use liquid::{Parser, ParserBuilder, Object, ValueView};
use liquid::model::Value;
use liquid::partials::{InMemorySource, EagerCompiler};
use crate::config::Config;
use log;
use html_escape;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Parse content with Liquid
pub fn parse_liquid(content: &str, parser: &liquid::Parser, globals: &Object) -> BoxResult<String> {
    // Decode HTML entities before parsing
    let decoded_content = html_escape::decode_html_entities(content).to_string();
    
    match parser.parse(&decoded_content) {
        Ok(template) => match template.render(globals) {
            Ok(result) => Ok(result),
            Err(e) => {
                // Log available globals for debugging
                log::debug!("Error rendering Liquid template: {}", e);
                log::debug!("Available globals:");
                for (key, value) in globals {
                    log::debug!("- {}: {:?}", key, value);
                    
                    // If this is an object, just log that it's an object without trying to iterate
                    if value.is_object() {
                        log::debug!("  - {}.* is an Object", key);
                    }
                }
                
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error rendering Liquid template: {}", e)
                )))
            },
        },
        Err(e) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error parsing Liquid template: {}", e)
        ))),
    }
}

/// Create a Jekyll-compatible Liquid parser with custom filters/tags
pub fn create_jekyll_parser(
    config: &Config, 
    includes: HashMap<String, String>
) -> BoxResult<Parser> {
    // Note: We don't use the includes map directly anymore since we're using custom tags
    // for handling includes in a Jekyll-compatible way
    
    // Create the parser builder and register custom extensions
    let mut parser_builder = ParserBuilder::with_stdlib();
    
    // Register custom filters
    parser_builder = filters::register_filters(parser_builder, config);
    
    // Register custom tags
    parser_builder = tags::register_tags(parser_builder, config);
    
    // Build the parser
    let parser = parser_builder.build()?;
    
    Ok(parser)
}

/// Create the site object with metadata for templates
pub fn create_site_object(config: &Config) -> Object {
    // Start with all config values as a base
    let mut site = config.to_liquid();
    
    // Add dynamic values that aren't in the config
    
    // Add time
    let now = chrono::Utc::now();
    site.insert("time".into(), Value::scalar(now.to_rfc3339()));
    
    // Add Jekyll version variable
    let jekyll_env = std::env::var("JEKYLL_ENV").unwrap_or_else(|_| "development".to_string());
    let mut jekyll = Object::new();
    jekyll.insert("environment".into(), Value::scalar(jekyll_env));
    jekyll.insert("version".into(), Value::scalar("0.1.0".to_string())); // Rustyll version
    site.insert("jekyll".into(), Value::Object(jekyll));
    
    // Debug output all available site variables
    log::debug!("Site variables available in templates:");
    for (key, value) in &site {
        log::debug!("- site.{}: {:?}", key, value);
    }
    
    site
}

// Add a separate function to create global variables for templates
pub fn create_globals(config: &Config, site_data: Option<&Object>) -> Object {
    let mut globals = Object::new();
    
    // Add site object - either use provided site_data or create a new one
    let site = if let Some(site) = site_data {
        site.clone()
    } else {
        create_site_object(config)
    };
    
    globals.insert("site".into(), Value::Object(site));
    
    // Add default include object with pre-defined variables for Jekyll compatibility
    globals.insert("include".into(), Value::Object(tags::utils::create_default_include_globals()));
    
    log::debug!("Global variables:");
    for (key, value) in &globals {
        log::debug!("- {}: {:?}", key, value);
    }
    
    globals
} 