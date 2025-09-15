pub mod yaml_parser;


use crate::front_matter::types::front_matter::FrontMatter;
use std::error::Error;

/// Parse front matter from text
pub fn parse_front_matter(content: &str) -> Result<FrontMatter, Box<dyn Error>> {
    // First parse YAML
    let (yaml, _) = yaml_parser::parse_yaml(content)?;
    
    // Then convert to FrontMatter
    let front_matter: FrontMatter = match serde_yaml::from_value(yaml) {
        Ok(fm) => fm,
        Err(e) => return Err(Box::new(e)),
    };
    
    Ok(front_matter)
} 