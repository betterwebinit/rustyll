mod include;
mod include_relative;
mod link;
mod raw;
mod highlight;
pub mod utils;

use crate::config::Config;
use liquid::ParserBuilder;

/// Register custom tags for use in Liquid templates
pub fn register_tags(parser_builder: ParserBuilder, config: &Config) -> ParserBuilder {
    // Register the include tag
    let parser_builder = parser_builder.tag(include::IncludeTag::new(config.clone()));
    
    // Register the include_relative tag
    let parser_builder = parser_builder.tag(include_relative::IncludeRelativeTag::new(config.clone()));
    
    // Register the link tag
    let parser_builder = parser_builder.tag(link::LinkTag::new(config.clone()));
    
    // Register the raw block tag
    let parser_builder = parser_builder.block(raw::RawBlock::new());
    
    // Register the highlight block tag
    let parser_builder = parser_builder.block(highlight::HighlightBlock::new(config.clone()));
    
    // If highlighting is enabled
    if config.highlighter == "rouge" || config.highlighter == "pygments" {
        // Just return the parser as is - we'll rely on the built-in highlight support from liquid
        parser_builder
    } else {
        parser_builder
    }
}

 