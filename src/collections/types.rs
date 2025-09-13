use std::error::Error;
use std::collections::HashMap;
use liquid::model::Value;

/// Common boxed result type for collections
pub type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Type alias for data collections
pub type DataCollection = HashMap<String, Value>; 