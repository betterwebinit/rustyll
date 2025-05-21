pub mod site;
pub mod page;
pub mod processor;
pub mod watcher;
pub mod types;

use crate::config::Config;
use std::error::Error;

pub use site::build_site;
pub use watcher::watch_site;
// pub use types::BoxResult; 