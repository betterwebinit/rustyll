pub mod document;
pub mod data;
pub mod utils;
pub mod types;

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use log::{info, debug};
use liquid::model::Value;
use liquid::Object;

use crate::config::Config;

pub use document::{Collection, Document, load_collections, document_to_liquid};
pub use data::load_data_files;
pub use utils::collections_to_liquid;
pub use types::BoxResult; 