use std::error::Error;
use std::fmt;
use std::io;

/// Common result type for Rustyll operations
pub type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Error types for Rustyll operations
#[derive(Debug)]
pub enum RustyllError {
    /// IO error wrapper
    Io(io::Error),
    /// Configuration error
    Config(String),
    /// Template processing error
    Template(String),
    /// Front matter parsing error
    FrontMatter(String),
    /// Markdown processing error
    Markdown(String),
    /// File handling error
    File(String),
    /// Server error
    Server(String),
    /// Generic error message
    Generic(String),
}

impl fmt::Display for RustyllError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RustyllError::Io(err) => write!(f, "IO error: {}", err),
            RustyllError::Config(msg) => write!(f, "Configuration error: {}", msg),
            RustyllError::Template(msg) => write!(f, "Template error: {}", msg),
            RustyllError::FrontMatter(msg) => write!(f, "Front matter error: {}", msg),
            RustyllError::Markdown(msg) => write!(f, "Markdown error: {}", msg),
            RustyllError::File(msg) => write!(f, "File error: {}", msg),
            RustyllError::Server(msg) => write!(f, "Server error: {}", msg),
            RustyllError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for RustyllError {}

impl From<io::Error> for RustyllError {
    fn from(err: io::Error) -> Self {
        RustyllError::Io(err)
    }
}

impl From<String> for RustyllError {
    fn from(msg: String) -> Self {
        RustyllError::Generic(msg)
    }
}

impl From<&str> for RustyllError {
    fn from(msg: &str) -> Self {
        RustyllError::Generic(msg.to_string())
    }
} 