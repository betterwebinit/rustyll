use std::error::Error;

/// Common boxed result type for the builder module
pub type BoxResult<T> = Result<T, Box<dyn Error>>; 