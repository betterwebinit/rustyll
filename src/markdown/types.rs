use std::error::Error;

/// Common result type for markdown operations
pub type BoxResult<T> = Result<T, Box<dyn Error>>; 