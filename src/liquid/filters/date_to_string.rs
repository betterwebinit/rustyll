use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};
use chrono::{TimeZone, Utc, NaiveDateTime};

/// DateToString filter implementation
#[derive(Debug, Clone)]
pub struct DateToStringFilter;

impl fmt::Display for DateToStringFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "date_to_string")
    }
}

impl liquid_core::Filter for DateToStringFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let date_str = input.to_kstr().to_string();
        
        // Try to parse the date in various formats
        let date = if date_str.len() >= 10 {
            // Try ISO 8601 format (YYYY-MM-DD)
            let iso_date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d");
            if let Ok(date) = iso_date {
                Some(date)
            } else {
                // Try Jekyll date format (YYYY-MM-DD HH:MM:SS)
                let jekyll_date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S");
                jekyll_date.ok()
            }
        } else {
            None
        };
        
        if let Some(dt) = date {
            let formatted = Utc.from_utc_datetime(&dt).format("%d %b %Y").to_string();
            Ok(Value::scalar(formatted))
        } else {
            // If can't parse, return the original string
            Ok(Value::scalar(date_str))
        }
    }
}

/// Parse filter factory for date_to_string
#[derive(Debug, Clone)]
pub struct DateToStringFilterParser;

impl FilterReflection for DateToStringFilterParser {
    fn name(&self) -> &str {
        "date_to_string"
    }
    
    fn description(&self) -> &str {
        "Formats a date according to Jekyll's date_to_string format (%d %b %Y)"
    }
    
    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
    
    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for DateToStringFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(DateToStringFilter))
    }
    
    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
} 