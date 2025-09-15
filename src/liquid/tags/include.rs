use liquid_core::{Runtime, ValueView, model::{Value, Object, ScalarCow}, Error, ParseTag, Renderable, TagReflection, TagTokenIter};
use crate::config::Config;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use log::{info, error};
use regex;
use super::utils::create_default_include_globals;
use html_escape;
use crate::liquid::filters;

/// Jekyll-compatible include tag
#[derive(Debug, Clone)]
pub struct IncludeTag {
    config: Config,
}

impl IncludeTag {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    fn read_include_file(&self, name: &str) -> Result<String, Error> {
        let includes_dir = self.config.source.join(&self.config.includes_dir);
        
        // First try with the exact name
        let mut file_path = includes_dir.join(name);
        
        // If it doesn't exist, try with .html extension
        if !file_path.exists() && !name.contains('.') {
            file_path = includes_dir.join(format!("{}.html", name));
        }
        
        info!("Looking for include file at: {}", file_path.display());
        
        // Check if this is a binary file that we should skip
        if crate::builder::processor::is_binary_file(&file_path) {
            log::info!("Skipping binary file in include: {}", file_path.display());
            return Ok(format!("<!-- Binary file skipped: {} -->", name));
        }
        
        // Try to read the file
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                info!("Successfully read include file: {}", file_path.display());
                Ok(content)
            },
            Err(e) => {
                error!("Failed to read include file '{}': {}", file_path.display(), e);
                
                // Check if there's a similar file with .html extension as fallback
                if !name.ends_with(".html") {
                    let html_path = includes_dir.join(format!("{}.html", name));
                    if html_path.exists() {
                        info!("Found alternative include file with .html extension: {}", html_path.display());
                        match fs::read_to_string(&html_path) {
                            Ok(content) => {
                                info!("Successfully read alternative include file: {}", html_path.display());
                                return Ok(content);
                            },
                            Err(e2) => {
                                error!("Failed to read alternative include file '{}': {}", html_path.display(), e2);
                            }
                        }
                    }
                }
                
                // Generate a useful error message with debugging info
                let error_msg = format!(
                    "Could not read include file '{}': {}. Includes dir: {}, File path: {}", 
                    name, e, includes_dir.display(), file_path.display()
                );
                
                // List available include files for debugging
                if let Ok(entries) = fs::read_dir(&includes_dir) {
                    let mut available_files = Vec::new();
                    for entry in entries {
                        if let Ok(entry) = entry {
                            available_files.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                    error!("Available include files: {:?}", available_files);
                }
                
                Err(Error::with_msg(error_msg))
            }
        }
    }
    
    // Add helper method to parse parameters
    fn parse_parameters(&self, params_str: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        // Parse parameters using regex - this regex handles key=value pairs where value can be quoted or unquoted
        let re = regex::Regex::new(r#"([^=\s]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|(\S+))"#).unwrap();
        for cap in re.captures_iter(params_str) {
            let key = cap[1].trim().to_string();
            
            // The value could be in capture group 2 (double quotes), 3 (single quotes), or 4 (unquoted)
            let value = if let Some(v) = cap.get(2) {
                // Double quoted value
                v.as_str().to_string()
            } else if let Some(v) = cap.get(3) {
                // Single quoted value
                v.as_str().to_string()
            } else if let Some(v) = cap.get(4) {
                // Unquoted value
                v.as_str().to_string()
            } else {
                // Default to empty if no value is captured
                String::new()
            };
            
            info!("Include param: {}='{}'", key, value);
            params.insert(key, value);
        }
        
        params
    }
}

struct IncludeTagReflection;

impl TagReflection for IncludeTagReflection {
    fn tag(&self) -> &str {
        "include"
    }

    fn description(&self) -> &str {
        "Include content from another file"
    }
}

impl ParseTag for IncludeTag {
    fn reflection(&self) -> &dyn TagReflection {
        &IncludeTagReflection
    }
    
    fn parse(&self, mut arguments: TagTokenIter, _options: &liquid_core::parser::Language) -> Result<Box<dyn Renderable>, Error> {
        // Collect all arguments as raw tokens first to handle paths with slashes correctly
        let mut all_tokens = Vec::new();
        while let Some(token) = arguments.next() {
            all_tokens.push(token.as_str().to_string());
        }

        if all_tokens.is_empty() {
            return Err(Error::with_msg("Include tag requires a filename argument"));
        }

        // Log all incoming tokens for debugging
        info!("Include tag all tokens: {:?}", all_tokens);

        // Handle the filename - it might be spread across multiple tokens if it contains slashes
        let mut filename_parts = Vec::new();
        let mut token_index = 0;

        // Reconstruct the filename by looking for patterns like: "components", "/", "blog", "/", "hero.html"
        while token_index < all_tokens.len() {
            let token = &all_tokens[token_index];

            // Stop if we hit a keyword like "with"
            if token == "with" {
                break;
            }

            // Add this token to the filename
            filename_parts.push(token.clone());
            token_index += 1;

            // If the next token is "/" and we're not at the end, continue building the path
            if token_index < all_tokens.len() && all_tokens[token_index] == "/" {
                filename_parts.push("/".to_string());
                token_index += 1;
            }
        }

        // Join the filename parts
        let mut first_arg_str = filename_parts.join("");
        info!("Include tag reconstructed filename: '{}'", first_arg_str);

        // Get remaining tokens after the filename
        let all_remaining_tokens: Vec<String> = all_tokens[token_index..].to_vec();
        info!("Include tag remaining tokens: {:?}", all_remaining_tokens);
        
        // Reset arguments iterator
        let mut arguments = all_remaining_tokens.iter();
        
        // Handle "-" trim directive at the end of the filename
        if first_arg_str.ends_with('-') {
            first_arg_str = first_arg_str[..first_arg_str.len()-1].to_string();
            info!("Removed trailing dash (trim directive) from filename: '{}'", first_arg_str);
        }
        
        // Check if this is a Liquid variable (starts with {{ and ends with }})
        let is_variable = first_arg_str.trim().starts_with("{{") && first_arg_str.trim().ends_with("}}");
        
        // Clean up the filename - remove quotes if present
        let filename = if !is_variable {
            first_arg_str.trim_matches('"').trim_matches('\'').to_string()
        } else {
            first_arg_str
        };
        
        // Parse parameters
        let mut params = HashMap::new();
        let mut current_key = String::new();
        
        // Check for the "with" keyword to process parameters
        if let Some(next_token) = arguments.next() {
            if next_token == "with" {
                // Process key=value pairs after "with"
                while let Some(param) = arguments.next() {
                    let param_str = param.to_string();
                    
                    if param_str.contains('=') {
                        // This is a key=value pair
                        let parts: Vec<&str> = param_str.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim().to_string();
                            let value = parts[1].trim().to_string();
                            params.insert(key, value);
                        }
                    } else if current_key.is_empty() {
                        // This is a key
                        current_key = param_str;
                    } else {
                        // This is a value for the previous key
                        params.insert(current_key, param_str);
                        current_key = String::new();
                    }
                }
            }
        }
        
        // Debug log the parsed values
        info!("Include tag parsed: filename='{}', is_variable={}, params={:?}", 
              filename, is_variable, params);
        
        Ok(Box::new(IncludeTagRenderer {
            config: self.config.clone(),
            filename,
            is_variable,
            params,
        }))
    }
}

/// Renderer for the include tag
#[derive(Debug)]
struct IncludeTagRenderer {
    config: Config,
    filename: String,
    is_variable: bool,
    params: HashMap<String, String>,
}

impl Renderable for IncludeTagRenderer {
    fn render(&self, runtime: &dyn Runtime) -> Result<String, Error> {
        info!("Rendering include tag with filename: {}", self.filename);
        info!("Parameters: {:?}", self.params);
        
        // Evaluate the filename
        let filename = if self.is_variable {
            let var_name = self.filename.trim_start_matches("{{").trim_end_matches("}}").trim();
            // For variable resolution, we need to parse it as a path
            let path: Vec<_> = var_name.split('.').map(ScalarCow::from).collect();
            let value_cow = runtime.get(&path)?;
            let value_str = value_cow.to_kstr().to_string();
            value_str
        } else {
            self.filename.clone()
        };
        
        // Read the include file
        let include_tag = IncludeTag::new(self.config.clone());
        let content = match include_tag.read_include_file(&filename) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read include file '{}': {}", filename, e);
                return Err(e);
            }
        };
        
        // Create a new scope for the include with parameters
        let mut include_scope = create_default_include_globals();
        
        // Add the filename to the include scope
        include_scope.insert("path".into(), Value::scalar(filename.clone()));
        
        // Extract just the name without path or extension
        let name = PathBuf::from(&filename)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| filename.clone());
            
        include_scope.insert("name".into(), Value::scalar(name));
        
        // Create a new context with the include scope
        let mut globals = Object::new();
        
        // Add site-wide variables from parent context
        if let Ok(site_value) = runtime.get(&[ScalarCow::from("site")]) {
            if let Some(site_obj) = site_value.as_object() {
                // We need to convert &dyn ObjectView to Object
                let mut new_site_obj = Object::new();
                
                // Copy all values from site_obj to the new object
                for (key, value) in site_obj.iter() {
                    let v = value.to_value();
                    new_site_obj.insert(key.clone().into(), v);
                }
                
                globals.insert("site".into(), Value::Object(new_site_obj));
            }
        }
        
        // Add page-wide variables from parent context
        if let Ok(page_value) = runtime.get(&[ScalarCow::from("page")]) {
            if let Some(page_obj) = page_value.as_object() {
                // We need to convert &dyn ObjectView to Object
                let mut new_page_obj = Object::new();
                
                // Copy all values from page_obj to the new object
                for (key, value) in page_obj.iter() {
                    let v = value.to_value();
                    new_page_obj.insert(key.clone().into(), v);
                }
                
                globals.insert("page".into(), Value::Object(new_page_obj));
            }
        }
        
        // Add the parameters directly to the global scope
        for (key, value_str) in &self.params {
            // Try to evaluate the value
            if value_str == "true" {
                // Boolean true
                info!("Adding boolean param: {}=true", key);
                globals.insert(key.clone().into(), Value::scalar(true));
            } else if value_str == "false" {
                // Boolean false
                info!("Adding boolean param: {}=false", key);
                globals.insert(key.clone().into(), Value::scalar(false));
            } else if let Ok(num) = value_str.parse::<f64>() {
                // Number
                info!("Adding numeric param: {}={}", key, num);
                globals.insert(key.clone().into(), Value::scalar(num));
            } else {
                // Try to parse as a variable reference
                let path: Vec<_> = value_str.split('.').map(ScalarCow::from).collect();
                match runtime.get(&path) {
                    Ok(val) => {
                        info!("Adding variable param: {}={:?}", key, val.to_value());
                        globals.insert(key.clone().into(), val.to_value());
                    },
                    Err(_) => {
                        // Fall back to treating it as a string - create an owned string
                        info!("Adding string param: {}='{}'", key, value_str);
                        globals.insert(key.clone().into(), Value::scalar(value_str.clone()));
                    }
                }
            }
        }
        
        // Add the include scope to globals
        globals.insert("include".into(), Value::Object(include_scope));
        
        info!("Full globals context: {:?}", globals);
        
        // Set up a new liquid parser - with custom filters
        let mut parser_builder = liquid::ParserBuilder::with_stdlib();
        
        // Register custom filters - specifically relative_url
        parser_builder = filters::register_filters(parser_builder, &self.config);
        
        // Build the parser
        let options = parser_builder.build()?;
        
        // Preprocess the content to fix any include tags inside it
        let preprocessed_content = crate::liquid::preprocess::preprocess_liquid(&content);
        
        // Parse and render the include content - decode HTML entities first
        let decoded_content = html_escape::decode_html_entities(&preprocessed_content).to_string();
        
        // Parse the decoded content
        let template = match options.parse(&decoded_content) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to parse include file '{}': {}", filename, e);
                return Err(Error::with_msg(format!("Error parsing include file '{}': {}", filename, e)));
            }
        };
        
        // Render with the globals context
        let rendered = match template.render(&globals) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to render include file '{}': {}", filename, e);
                return Err(Error::with_msg(format!("Error rendering include file '{}': {}", filename, e)));
            }
        };
        
        info!("Successfully rendered include file: {}", filename);
        
        Ok(rendered)
    }

    fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn Runtime) -> Result<(), Error> {
        let s = self.render(runtime)?;
        writer.write_all(s.as_bytes()).map_err(|e| Error::with_msg(format!("Failed to write to output: {}", e)))?;
        Ok(())
    }
} 