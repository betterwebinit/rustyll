use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};

/// AbsoluteUrl filter implementation
#[derive(Debug, Clone)]
pub struct AbsoluteUrlFilter {
    base_url: String,
    site_url: Option<String>,
}

impl liquid_core::Filter for AbsoluteUrlFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let path = input.to_kstr().to_string();
        
        // Get site URL, defaulting to empty string if not provided
        let site_url = match &self.site_url {
            Some(url) => {
                if url.ends_with('/') {
                    url.clone()
                } else {
                    format!("{}/", url)
                }
            },
            None => String::new(),
        };
        
        // First create the relative URL
        let mut rel_url = if self.base_url.is_empty() {
            path
        } else {
            let base = if self.base_url.ends_with('/') {
                self.base_url.clone()
            } else {
                format!("{}/", self.base_url)
            };
            
            let path_without_leading_slash = path.trim_start_matches('/');
            format!("{}{}", base, path_without_leading_slash)
        };
        
        // Ensure URL starts with /
        if !rel_url.starts_with('/') {
            rel_url = format!("/{}", rel_url);
        }
        
        // Now prepend site URL
        let mut absolute_url = format!("{}{}", site_url, rel_url);
        
        // Clean up any double slashes (except at the start of protocol)
        let protocol_pos = absolute_url.find("://");
        if let Some(pos) = protocol_pos {
            let (protocol, path) = absolute_url.split_at(pos + 3);
            let clean_path = path.replace("//", "/");
            absolute_url = format!("{}{}", protocol, clean_path);
        } else {
            // No protocol found, just clean up all double slashes
            while absolute_url.contains("//") {
                absolute_url = absolute_url.replace("//", "/");
            }
        }
        
        Ok(Value::scalar(absolute_url))
    }
}

impl fmt::Display for AbsoluteUrlFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "absolute_url")
    }
}

/// Parse filter factory for absolute_url
#[derive(Debug, Clone)]
pub struct AbsoluteUrlFilterParser {
    pub base_url: String,
    pub site_url: Option<String>,
}

impl FilterReflection for AbsoluteUrlFilterParser {
    fn name(&self) -> &str {
        "absolute_url"
    }
    
    fn description(&self) -> &str {
        "Creates an absolute URL by prepending the site url and baseurl to the input"
    }
    
    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
    
    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for AbsoluteUrlFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(AbsoluteUrlFilter {
            base_url: self.base_url.clone(),
            site_url: self.site_url.clone(),
        }))
    }
    
    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
} 