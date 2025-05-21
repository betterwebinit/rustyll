use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult, Error as LiquidError};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};

/// RelativeUrl filter implementation
#[derive(Debug, Clone)]
pub struct RelativeUrlFilter {
    base_url: String,
}

impl liquid_core::Filter for RelativeUrlFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let path = input.to_kstr().to_string();
        
        // Combine baseurl with input path, ensuring there's just one slash between them
        let mut url = if self.base_url.is_empty() {
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
        if !url.starts_with('/') {
            url = format!("/{}", url);
        }
        
        // Clean up any double slashes (except at the start of protocol)
        while url.contains("//") {
            url = url.replace("//", "/");
        }
        
        Ok(Value::scalar(url))
    }
}

impl fmt::Display for RelativeUrlFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "relative_url")
    }
}

/// Parse filter factory for relative_url
#[derive(Debug, Clone)]
pub struct RelativeUrlFilterParser {
    pub base_url: String,
}

impl FilterReflection for RelativeUrlFilterParser {
    fn name(&self) -> &str {
        "relative_url"
    }
    
    fn description(&self) -> &str {
        "Prepends the site's baseurl to the input"
    }
    
    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
    
    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for RelativeUrlFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(RelativeUrlFilter {
            base_url: self.base_url.clone(),
        }))
    }
    
    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
} 