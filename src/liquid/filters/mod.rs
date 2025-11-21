mod markdownify;
mod relative_url;
mod absolute_url;
mod date_to_string;
mod date_to_xmlschema;
mod date;
mod number_with_delimiter;

use liquid::ParserBuilder;
use crate::config::Config;

/// Register custom filters for use in Liquid templates
pub fn register_filters(parser_builder: ParserBuilder, config: &Config) -> ParserBuilder {
    // Add markdownify filter
    let parser_builder = parser_builder
        .filter(markdownify::MarkdownifyFilterParser);
    
    // Add relative_url filter
    let parser_builder = parser_builder
        .filter(relative_url::RelativeUrlFilterParser { 
            base_url: config.base_url.clone() 
        });
    
    // Add absolute_url filter
    let parser_builder = parser_builder
        .filter(absolute_url::AbsoluteUrlFilterParser { 
            base_url: config.base_url.clone(),
            site_url: config.url.clone()
        });
    
    // Add date_to_string filter
    let parser_builder = parser_builder
        .filter(date_to_string::DateToStringFilterParser);

    // Add date_to_xmlschema filter
    let parser_builder = parser_builder
        .filter(date_to_xmlschema::DateToXmlSchemaFilterParser);

    // Add generic date filter (Jekyll-compatible)
    let parser_builder = parser_builder
        .filter(date::DateFilterParser);

    // Add number_with_delimiter filter
    let parser_builder = parser_builder
        .filter(number_with_delimiter::NumberWithDelimiterFilterParser);

    parser_builder
}

 