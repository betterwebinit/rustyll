use std::error::Error;

/// Box type for directory operations that can fail
pub type BoxResult<T> = Result<T, Box<dyn Error>>; 