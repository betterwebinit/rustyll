use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter};
use liquid_core::FilterReflection;
use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate};

/// DateToXmlSchema filter implementation
#[derive(Debug, Clone)]
pub struct DateToXmlSchemaFilter;

impl fmt::Display for DateToXmlSchemaFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "date_to_xmlschema")
    }
}

impl liquid_core::Filter for DateToXmlSchemaFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let date_str = input.to_kstr().to_string();

        // Try various date formats
        let date = if let Ok(dt) = DateTime::parse_from_rfc3339(&date_str) {
            dt.with_timezone(&Utc)
        } else if let Ok(dt) = DateTime::parse_from_rfc2822(&date_str) {
            dt.with_timezone(&Utc)
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S") {
            dt.and_utc()
        } else if let Ok(dt) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            // For date only, create a datetime at midnight
            dt.and_hms_opt(0, 0, 0).unwrap().and_utc()
        } else {
            // Default to current time if we can't parse
            Utc::now()
        };

        // Format as XML Schema (ISO 8601)
        Ok(Value::scalar(date.to_rfc3339()))
    }
}

/// Parse filter factory for date_to_xmlschema
#[derive(Debug, Clone)]
pub struct DateToXmlSchemaFilterParser;

impl FilterReflection for DateToXmlSchemaFilterParser {
    fn name(&self) -> &str {
        "date_to_xmlschema"
    }

    fn description(&self) -> &str {
        "Converts a date to XML Schema (ISO 8601) format"
    }

    fn positional_parameters(&self) -> &'static [liquid_core::parser::ParameterReflection] {
        &[]
    }

    fn keyword_parameters(&self) -> &'static [liquid_core::parser::ParameterReflection] {
        &[]
    }
}

impl ParseFilter for DateToXmlSchemaFilterParser {
    fn parse(&self, _arguments: FilterArguments) -> liquid_core::Result<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(DateToXmlSchemaFilter))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
}

impl fmt::Display for DateToXmlSchemaFilterParser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "date_to_xmlschema")
    }
}