mod filters;
mod tags;
mod preprocess;

use std::collections::HashMap;
use std::error::Error;
use liquid::{Parser, ParserBuilder, Object, ValueView};
use liquid::model::Value;
use crate::config::Config;
use log;
use html_escape;
use std::path::Path;
use std::fs;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Detect GitHub repository information from various sources
fn detect_github_repository(config: &Config) -> (String, String, String) {
    // Default empty values
    let mut repo_name = String::new();
    let mut owner_name = String::new();
    let mut project_name = String::new();
    
    // 1. Try to get from config.yml if user has specified it
    if let Some(repository) = config.repository.as_ref() {
        repo_name = repository.clone();
        if let Some(idx) = repo_name.find('/') {
            owner_name = repo_name[..idx].to_string();
            project_name = repo_name[idx+1..].to_string();
        }
        return (repo_name, owner_name, project_name);
    }
    
    // 2. Try to get from environment variables (GitHub Actions, etc)
    if let Ok(gh_repo) = std::env::var("GITHUB_REPOSITORY") {
        repo_name = gh_repo;
        if let Some(idx) = repo_name.find('/') {
            owner_name = repo_name[..idx].to_string();
            project_name = repo_name[idx+1..].to_string();
        }
        return (repo_name, owner_name, project_name);
    }
    
    // 3. Try to get from .git/config if it exists
    let git_config_path = Path::new(&config.source).join(".git/config");
    if git_config_path.exists() {
        if let Ok(content) = fs::read_to_string(git_config_path) {
            // Look for github.com URLs in the git config
            for line in content.lines() {
                if line.contains("github.com") && line.contains("url = ") {
                    // Extract the repository from URLs like:
                    // url = https://github.com/username/repo.git
                    // or git@github.com:username/repo.git
                    let parts: Vec<&str> = line.split(&['/', ':', '.'][..]).collect();
                    for (i, part) in parts.iter().enumerate() {
                        if *part == "github.com" || *part == "github" {
                            if i + 2 < parts.len() {
                                owner_name = parts[i+1].trim().to_string();
                                // Remove .git suffix if present
                                project_name = parts[i+2].trim().to_string();
                                if project_name.ends_with(".git") {
                                    project_name = project_name[..project_name.len()-4].to_string();
                                }
                                repo_name = format!("{}/{}", owner_name, project_name);
                                return (repo_name, owner_name, project_name);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Return defaults if we couldn't detect
    (repo_name, owner_name, project_name)
}

/// Parse content with Liquid
pub fn parse_liquid(content: &str, parser: &liquid::Parser, globals: &Object) -> BoxResult<String> {
    // First preprocess to fix include tags with slashes
    let preprocessed_content = preprocess::preprocess_liquid(content);
    
    // Decode HTML entities before parsing
    let decoded_content = html_escape::decode_html_entities(&preprocessed_content).to_string();
    
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
    
    // Add GitHub repository information for GitHub Pages compatibility
    let (repo_name, owner_name, project_name) = detect_github_repository(config);
    site.insert("repository".into(), Value::scalar(repo_name.clone()));
    
    // Add github object (mimicking jekyll-github-metadata plugin)
    let mut github = Object::new();
    github.insert("repository_name".into(), Value::scalar(repo_name.clone()));
    github.insert("owner_name".into(), Value::scalar(owner_name.clone()));
    github.insert("project_title".into(), Value::scalar(project_name));
    
    // Add repository URLs - this matches what jekyll-github-metadata would provide
    github.insert("repository_url".into(), Value::scalar(format!("https://github.com/{}", repo_name)));
    github.insert("owner_url".into(), Value::scalar(format!("https://github.com/{}", owner_name)));
    github.insert("api_url".into(), Value::scalar(format!("https://api.github.com/repos/{}", repo_name)));
    
    // Add pages variables - some themes expect these
    let mut pages = Object::new();
    pages.insert("build_revision".into(), Value::scalar("rustyll-static"));
    github.insert("pages".into(), Value::Object(pages));
    
    site.insert("github".into(), Value::Object(github));

    // Add plugins array (for Jekyll compatibility)
    let plugins = if !config.plugins.is_empty() {
        config.plugins.iter()
            .map(|p| Value::scalar(p.clone()))
            .collect()
    } else {
        // Default Jekyll plugins that themes might expect
        vec![
            Value::scalar("jekyll-feed"),
            Value::scalar("jekyll-seo-tag"),
            Value::scalar("jekyll-sitemap"),
        ]
    };
    site.insert("plugins".into(), Value::Array(plugins));

    // Debug output all available site variables
    log::debug!("Site variables available in templates:");
    for (key, _value) in &site {
        log::debug!("- site.{}", key);
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