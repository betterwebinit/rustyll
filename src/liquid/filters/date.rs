use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};
use chrono::{NaiveDateTime, Utc, TimeZone, DateTime};

/// Date filter implementation
#[derive(Debug, Clone)]
pub struct DateFilter {
    format: String,
}

impl fmt::Display for DateFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "date")
    }
}

impl liquid_core::Filter for DateFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let date_str = input.to_kstr().to_string();

        // Try to parse the date in various formats
        let date = if date_str == "now" || date_str == "today" {
            // Special case for "now" and "today"
            Some(Utc::now())
        } else if let Ok(timestamp) = date_str.parse::<i64>() {
            // Unix timestamp
            Some(Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(Utc::now))
        } else {
            // Try various date string formats
            parse_date_string(&date_str)
        };

        if let Some(dt) = date {
            // Format the date according to the provided format string
            let formatted = format_date(&dt, &self.format);
            Ok(Value::scalar(formatted))
        } else {
            // If can't parse, return the original string
            Ok(Value::scalar(date_str))
        }
    }
}

fn parse_date_string(date_str: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339 format first (used by Jekyll/Liquid)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try ISO 8601 format (YYYY-MM-DD HH:MM:SS)
    if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }

    // Try date only (YYYY-MM-DD)
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }

    None
}

fn format_date(dt: &DateTime<Utc>, format: &str) -> String {
    // Convert strftime-like format to chrono format
    // This handles Jekyll/Liquid date formatting strings
    let chrono_format = convert_jekyll_to_chrono_format(format);
    dt.format(&chrono_format).to_string()
}

fn convert_jekyll_to_chrono_format(format: &str) -> String {
    // Jekyll uses strftime-style formatting
    // This is already compatible with chrono's format strings
    // But we need to handle a few special cases
    format
        .replace("%s", "%s") // Unix timestamp (already compatible)
        .replace("%Q", "%3f") // Milliseconds
        .to_string()
}

/// Parse filter factory for date
#[derive(Debug, Clone)]
pub struct DateFilterParser;

impl FilterReflection for DateFilterParser {
    fn name(&self) -> &str {
        "date"
    }

    fn description(&self) -> &str {
        "Formats a date according to the specified format string"
    }

    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }

    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for DateFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        // For now, we'll use a default format
        // TODO: Parse the format string from arguments when the API is clearer
        Ok(Box::new(DateFilter {
            format: "%B %d, %Y".to_string()  // Default Jekyll format
        }))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
}