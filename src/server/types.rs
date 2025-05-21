use std::error::Error;

/// Common result type for server operations
pub type BoxResult<T> = Result<T, Box<dyn Error>>; 