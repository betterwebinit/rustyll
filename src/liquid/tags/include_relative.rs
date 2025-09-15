use liquid_core::{Runtime, ValueView, model::{Value, Object, ScalarCow}, Error, ParseTag, Renderable, TagReflection, TagTokenIter};
use crate::config::Config;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use log::{info, error};
use regex;
use super::utils::create_default_include_globals;
use html_escape;
use crate::liquid::filters;

/// Jekyll-compatible include_relative tag
#[derive(Debug, Clone)]
pub struct IncludeRelativeTag {
    config: Config,
}

impl IncludeRelativeTag {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    fn read_include_file(&self, name: &str, current_path: &Path) -> Result<String, Error> {
        // For include_relative, we use the current file's directory as the base
        let base_dir = current_path.parent()
            .ok_or_else(|| Error::with_msg("Cannot determine parent directory for current template"))?;
        
        let file_path = base_dir.join(name);
        
        info!("Looking for include_relative file at: {}", file_path.display());
        
        // Check if this is a binary file that we should skip
        if crate::builder::processor::is_binary_file(&file_path) {
            log::info!("Skipping binary file in include_relative: {}", file_path.display());
            return Ok(format!("<!-- Binary file skipped: {} -->", name));
        }
        
        // Try to read the file
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                info!("Successfully read include_relative file: {}", file_path.display());
                Ok(content)
            },
            Err(e) => {
                error!("Failed to read include_relative file: {} - Error: {}", file_path.display(), e);
                
                // Also check for alternative path formulations (replace slashes with underscores, etc.)
                // This is for compatibility with some Jekyll themes
                let alt_name = name.replace('/', "_");
                if alt_name != name {
                    let alt_path = base_dir.join(alt_name);
                    info!("Trying alternative include_relative path: {}", alt_path.display());
                    
                    if let Ok(content) = fs::read_to_string(&alt_path) {
                        info!("Successfully read include_relative file from alternative path: {}", alt_path.display());
                        return Ok(content);
                    }
                }
                
                // Also check in the _includes directory as a fallback
                let includes_dir = self.config.source.join(&self.config.includes_dir);
                let includes_path = includes_dir.join(name);
                info!("Trying fallback in _includes directory: {}", includes_path.display());
                
                if let Ok(content) = fs::read_to_string(&includes_path) {
                    info!("Successfully read include_relative file from _includes directory: {}", includes_path.display());
                    return Ok(content);
                }
                
                // List files in the base directory for debugging
                if let Ok(entries) = fs::read_dir(base_dir) {
                    info!("Available files in relative directory ({}):", base_dir.display());
                    for entry in entries {
                        if let Ok(entry) = entry {
                            info!("  {}", entry.path().display());
                        }
                    }
                }
                
                Err(Error::with_msg(format!("Could not read include_relative file: {}", file_path.display())))
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
            
            info!("Include_relative param: {}='{}'", key, value);
            params.insert(key, value);
        }
        
        params
    }
}

struct IncludeRelativeTagReflection;

impl TagReflection for IncludeRelativeTagReflection {
    fn tag(&self) -> &str {
        "include_relative"
    }

    fn description(&self) -> &str {
        "Include content from another file relative to the current file"
    }
}

impl ParseTag for IncludeRelativeTag {
    fn reflection(&self) -> &dyn TagReflection {
        &IncludeRelativeTagReflection
    }
    
    fn parse(&self, mut arguments: TagTokenIter, _options: &liquid_core::parser::Language) -> Result<Box<dyn Renderable>, Error> {
        // First get the argument which might be a filename or a path with parameters
        let first_arg = arguments.next().ok_or_else(|| Error::with_msg("Include_relative tag requires a filename argument"))?;
        let mut first_arg_str = first_arg.as_str().to_string();
        
        // Log all incoming tokens for debugging
        info!("Include_relative tag first argument: '{}'", first_arg_str);
        let mut all_remaining_tokens = Vec::new();
        while let Some(token) = arguments.next() {
            all_remaining_tokens.push(token.as_str().to_string());
        }
        info!("Include_relative tag remaining tokens: {:?}", all_remaining_tokens);
        
        // Reset arguments iterator
        let arguments = all_remaining_tokens.iter();
        
        // Handle "-" trim directive at the end of the filename
        if first_arg_str.ends_with('-') {
            first_arg_str = first_arg_str[..first_arg_str.len()-1].to_string();
            info!("Removed trailing dash (trim directive) from filename: '{}'", first_arg_str);
        }
        
        // Check if this is a Liquid variable (starts with {{ and ends with }})
        let is_variable = first_arg_str.trim().starts_with("{{") && first_arg_str.trim().ends_with("}}");
        
        // If it's a variable, keep it as is
        if is_variable {
            let filename = first_arg_str;
            info!("Include_relative filename (variable): '{}'", filename);
            
            // Collect all the parameter arguments
            let mut params = HashMap::new();
            // Parse parameters if any are left
            if !all_remaining_tokens.is_empty() {
                let params_str = all_remaining_tokens.join(" ");
                params = self.parse_parameters(&params_str);
            }
            
            return Ok(Box::new(IncludeRelativeTagRenderer {
                config: self.config.clone(),
                filename,
                is_variable,
                params,
            }));
        }
        
        // For regular paths, handle slashes directly in the filename
        // Remove quotes if present
        let filename = first_arg_str.trim_matches('"').trim_matches('\'').to_string();
        let mut tokens_for_params = Vec::new();
        
        // Collect parameter arguments
        for token_str in arguments {
            // If it contains an equals sign, it's a parameter
            if token_str.contains('=') {
                tokens_for_params.push(token_str.clone());
            } else {
                // Otherwise it's likely part of a parameter value
                tokens_for_params.push(token_str.clone());
            }
        }
        
        // Join all parameter tokens
        let params_str = tokens_for_params.join(" ");
        
        info!("Include_relative final filename: '{}'", filename);
        if !params_str.is_empty() {
            info!("Include_relative parameters: '{}'", params_str);
        }
        
        // Parse parameters
        let params = self.parse_parameters(&params_str);
        
        Ok(Box::new(IncludeRelativeTagRenderer {
            config: self.config.clone(),
            filename,
            is_variable: false,
            params,
        }))
    }
}

/// Renderer for the include_relative tag
#[derive(Debug)]
struct IncludeRelativeTagRenderer {
    config: Config,
    filename: String,
    is_variable: bool,
    params: HashMap<String, String>,
}

impl Renderable for IncludeRelativeTagRenderer {
    fn render(&self, runtime: &dyn Runtime) -> Result<String, Error> {
        info!("Rendering include_relative tag with filename: {}", self.filename);
        info!("Parameters: {:?}", self.params);
        
        // Get the current template path from the runtime
        let current_path = match runtime.get(&[ScalarCow::from("page"), ScalarCow::from("path")]) {
            Ok(path_value) => {
                let path_str = path_value.to_kstr().to_string();
                PathBuf::from(&path_str)
            },
            Err(_) => {
                return Err(Error::with_msg("Cannot determine current file path for include_relative"));
            }
        };
        
        // Evaluate the filename
        let filename = if self.is_variable {
            let var_name = self.filename.trim_start_matches("{{").trim_end_matches("}}").trim();
            // For variable resolution, we need to parse it as a path
            let path: Vec<_> = var_name.split('.').map(ScalarCow::from).collect();
            let value_cow = runtime.get(&path)?;
            let value_str = value_cow.to_kstr().to_string();
            value_str
        } else {
            self.filename.trim_matches('"').trim_matches('\'').to_string()
        };
        
        // Read the include file relative to the current file's path
        let include_tag = IncludeRelativeTag::new(self.config.clone());
        let content = match include_tag.read_include_file(&filename, &current_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read include_relative file '{}': {}", filename, e);
                return Err(e);
            }
        };
        
        // Create a new scope for the include with parameters
        let mut include_scope = create_default_include_globals();
        
        info!("Initial include_relative scope with defaults: {:?}", include_scope);
        
        // Add the parameters to the include scope
        for (key, value_str) in &self.params {
            info!("Processing param: {}='{}'", key, value_str);
            
            // Parse and evaluate the value
            if value_str.starts_with('"') && value_str.ends_with('"') || 
                value_str.starts_with('\'') && value_str.ends_with('\'') {
                // String literal - make sure to create an owned string
                let clean_value = value_str.trim_matches('"').trim_matches('\'').to_string();
                info!("Adding literal param: {}='{}'", key, clean_value);
                include_scope.insert(key.clone().into(), Value::scalar(clean_value));
            } else if value_str == "true" {
                // Boolean true
                info!("Adding boolean param: {}=true", key);
                include_scope.insert(key.clone().into(), Value::scalar(true));
            } else if value_str == "false" {
                // Boolean false
                info!("Adding boolean param: {}=false", key);
                include_scope.insert(key.clone().into(), Value::scalar(false));
            } else if let Ok(num) = value_str.parse::<f64>() {
                // Number
                info!("Adding numeric param: {}={}", key, num);
                include_scope.insert(key.clone().into(), Value::scalar(num));
            } else {
                // Try to parse as a variable reference
                let path: Vec<_> = value_str.split('.').map(ScalarCow::from).collect();
                match runtime.get(&path) {
                    Ok(val) => {
                        info!("Adding variable param: {}={:?}", key, val.to_value());
                        include_scope.insert(key.clone().into(), val.to_value());
                    },
                    Err(_) => {
                        // Fall back to treating it as a string - create an owned string
                        info!("Adding string param: {}='{}'", key, value_str);
                        include_scope.insert(key.clone().into(), Value::scalar(value_str.clone()));
                    }
                }
            }
        }
        
        info!("Final include_relative scope after parameters: {:?}", include_scope);
        
        // Set up a new liquid parser with custom filters
        let mut parser_builder = liquid::ParserBuilder::with_stdlib();
        
        // Register custom filters - specifically relative_url
        parser_builder = filters::register_filters(parser_builder, &self.config);
        
        // Build the parser
        let options = parser_builder.build()?;
        
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
        
        // Add the include scope to globals
        globals.insert("include".into(), Value::Object(include_scope));
        
        info!("Full globals context: {:?}", globals);
        
        // Preprocess the content to fix any include tags inside it
        let preprocessed_content = crate::liquid::preprocess::preprocess_liquid(&content);
        
        // Parse and render the include content - decode HTML entities first
        let decoded_content = html_escape::decode_html_entities(&preprocessed_content).to_string();
        let template = match options.parse(&decoded_content) {
            Ok(t) => t,
            Err(e) => {
                error!("Error parsing include_relative template '{}': {}", filename, e);
                return Err(Error::with_msg(format!("Error parsing include_relative {}: {}", filename, e)));
            }
        };
        
        // Render the template with the globals
        match template.render(&globals) {
            Ok(result) => {
                Ok(result)
            },
            Err(e) => {
                error!("Error rendering include_relative file '{}': {}", filename, e);
                Err(Error::with_msg(format!("Error rendering include_relative {}: {}", filename, e)))
            }
        }
    }

    fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn Runtime) -> Result<(), Error> {
        let s = self.render(runtime)?;
        writer.write_all(s.as_bytes()).map_err(|e| Error::with_msg(format!("Failed to write to output: {}", e)))?;
        Ok(())
    }
} 