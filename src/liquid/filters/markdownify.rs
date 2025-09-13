use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};

/// Markdownify filter implementation that converts markdown to HTML
#[derive(Debug, Clone)]
pub struct MarkdownifyFilter;

impl liquid_core::Filter for MarkdownifyFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let markdown = input.to_kstr().to_string();
        let html = comrak::markdown_to_html(&markdown, &comrak::ComrakOptions::default());
        Ok(Value::scalar(html))
    }
}

impl fmt::Display for MarkdownifyFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "markdownify")
    }
}

/// Parse filter factory for markdownify
#[derive(Debug, Clone)]
pub struct MarkdownifyFilterParser;

impl FilterReflection for MarkdownifyFilterParser {
    fn name(&self) -> &str {
        "markdownify"
    }
    
    fn description(&self) -> &str {
        "Converts markdown text to HTML"
    }
    
    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
    
    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for MarkdownifyFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(MarkdownifyFilter))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
} 