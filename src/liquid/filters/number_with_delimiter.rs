use std::fmt;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};

/// NumberWithDelimiter filter implementation
/// Formats numbers with thousands separators (commas)
#[derive(Debug, Clone)]
pub struct NumberWithDelimiterFilter {
    delimiter: String,
}

impl fmt::Display for NumberWithDelimiterFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "number_with_delimiter")
    }
}

impl liquid_core::Filter for NumberWithDelimiterFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        let input_str = input.to_kstr().to_string();

        // Try to parse as a number
        if let Ok(num) = input_str.parse::<i64>() {
            // Format the number with delimiters
            let formatted = format_number_with_delimiter(num, &self.delimiter);
            Ok(Value::scalar(formatted))
        } else if let Ok(num) = input_str.parse::<f64>() {
            // Handle floating point numbers
            let formatted = if num.fract() == 0.0 {
                // No decimal part, format as integer
                format_number_with_delimiter(num as i64, &self.delimiter)
            } else {
                // Format with decimal part
                let int_part = num.trunc() as i64;
                let formatted_int = format_number_with_delimiter(int_part, &self.delimiter);
                let decimal_formatted = format!("{:.2}", num.fract());
                let decimal_part = decimal_formatted.trim_start_matches("0.");
                format!("{}.{}", formatted_int, decimal_part)
            };
            Ok(Value::scalar(formatted))
        } else {
            // If can't parse as number, return the original string
            Ok(Value::scalar(input_str))
        }
    }
}

fn format_number_with_delimiter(num: i64, delimiter: &str) -> String {
    let num_str = num.to_string();
    let mut result = String::new();

    for (i, ch) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push_str(delimiter);
        }
        result.push(ch);
    }

    result.chars().rev().collect()
}

/// Parse filter factory for number_with_delimiter
#[derive(Debug, Clone)]
pub struct NumberWithDelimiterFilterParser;

impl FilterReflection for NumberWithDelimiterFilterParser {
    fn name(&self) -> &str {
        "number_with_delimiter"
    }

    fn description(&self) -> &str {
        "Formats a number with thousands separators (commas by default)"
    }

    fn positional_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }

    fn keyword_parameters(&self) -> &'static [ParameterReflection] {
        &[]
    }
}

impl ParseFilter for NumberWithDelimiterFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        // Default delimiter is comma
        Ok(Box::new(NumberWithDelimiterFilter {
            delimiter: ",".to_string(),
        }))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
}