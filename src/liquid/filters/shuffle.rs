use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;
use liquid_core::{Runtime, ValueView, Value, Result as LiquidResult, Error as LiquidError};
use liquid_core::parser::{FilterArguments, ParseFilter, ParameterReflection};
use liquid_core::{FilterReflection};

/// Shuffle filter implementation that randomizes arrays or strings
#[derive(Debug, Clone)]
pub struct ShuffleFilter;

impl liquid_core::Filter for ShuffleFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> LiquidResult<Value> {
        // If input is an array
        if let Some(array) = input.as_array() {
            let mut vec: Vec<Value> = array.values().map(|v| v.to_value()).collect();
            let mut rng = thread_rng();
            vec.shuffle(&mut rng);
            Ok(Value::Array(vec))
        } else if input.is_scalar() {
            // Otherwise, treat it as string and shuffle characters
            let mut chars: Vec<char> = input.to_kstr().chars().collect();
            let mut rng = thread_rng();
            chars.shuffle(&mut rng);
            Ok(Value::scalar(chars.into_iter().collect::<String>()))
        } else {
            Err(LiquidError::with_msg("shuffle filter expects array or string input").into())
        }
    }
}

impl fmt::Display for ShuffleFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shuffle")
    }
}

/// Parse filter factory for shuffle
#[derive(Debug, Clone)]
pub struct ShuffleFilterParser;

impl FilterReflection for ShuffleFilterParser {
    fn name(&self) -> &str { "shuffle" }
    fn description(&self) -> &str { "Randomly shuffles an array or a string" }
    fn positional_parameters(&self) -> &'static [ParameterReflection] { &[] }
    fn keyword_parameters(&self) -> &'static [ParameterReflection] { &[] }
}

impl ParseFilter for ShuffleFilterParser {
    fn parse(&self, _args: FilterArguments) -> LiquidResult<Box<dyn liquid_core::Filter>> {
        Ok(Box::new(ShuffleFilter))
    }

    fn reflection(&self) -> &dyn FilterReflection { self }
}


#[cfg(test)]
mod tests {
    use super::*;
    use liquid_core::model::Value;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::Filter;

    #[test]
    fn shuffle_array_retains_elements() {
        let filter = ShuffleFilter;
        let input = Value::array(vec![Value::scalar(1i64), Value::scalar(2i64), Value::scalar(3i64)]);
        let runtime = RuntimeBuilder::new().build();
        let result = filter.evaluate(input.as_view(), &runtime).unwrap();
        let arr = result.into_array().unwrap();
        assert_eq!(arr.len(), 3);
        let mut numbers: Vec<i64> = arr.iter()
            .map(|v| v.as_scalar().and_then(|s| s.to_integer()).unwrap())
            .collect();
        numbers.sort();
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn shuffle_string_retains_chars() {
        let filter = ShuffleFilter;
        let input = Value::scalar("abc");
        let runtime = RuntimeBuilder::new().build();
        let result = filter.evaluate(input.as_view(), &runtime).unwrap();
        let out = result.into_scalar().unwrap().to_kstr().into_owned();
        let mut chars: Vec<char> = out.chars().collect();
        chars.sort();
        assert_eq!(chars, vec!['a', 'b', 'c']);
    }
}
