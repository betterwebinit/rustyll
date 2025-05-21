use std::path::PathBuf;
use chrono::{DateTime, Utc};

use crate::front_matter::FrontMatter;

/// A page in the site
#[derive(Debug, Clone)]
pub struct Page {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub url: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub content: String,
    pub front_matter: FrontMatter,
    pub process: bool,
} 