//! Hook system for plugins

use std::collections::HashMap;
use serde_yaml::Value;

/// Available hooks in the build process
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Hook {
    /// Before site initialization
    PreInit,
    /// After site initialization
    PostInit,
    /// Before reading source files
    PreRead,
    /// After reading source files
    PostRead,
    /// Before generating pages
    PreGenerate,
    /// After generating pages
    PostGenerate,
    /// Before rendering content
    PreRender,
    /// After rendering content
    PostRender,
    /// Before writing files
    PreWrite,
    /// After writing files
    PostWrite,
    /// Before cleaning site
    PreClean,
    /// After cleaning site
    PostClean,
    /// Custom hook
    Custom(String),
}

impl Hook {
    /// Get the name of the hook
    pub fn name(&self) -> &str {
        match self {
            Hook::PreInit => "pre_init",
            Hook::PostInit => "post_init",
            Hook::PreRead => "pre_read",
            Hook::PostRead => "post_read",
            Hook::PreGenerate => "pre_generate",
            Hook::PostGenerate => "post_generate",
            Hook::PreRender => "pre_render",
            Hook::PostRender => "post_render",
            Hook::PreWrite => "pre_write",
            Hook::PostWrite => "post_write",
            Hook::PreClean => "pre_clean",
            Hook::PostClean => "post_clean",
            Hook::Custom(name) => name,
        }
    }

    /// Create a hook from its name
    pub fn from_name(name: &str) -> Self {
        match name {
            "pre_init" => Hook::PreInit,
            "post_init" => Hook::PostInit,
            "pre_read" => Hook::PreRead,
            "post_read" => Hook::PostRead,
            "pre_generate" => Hook::PreGenerate,
            "post_generate" => Hook::PostGenerate,
            "pre_render" => Hook::PreRender,
            "post_render" => Hook::PostRender,
            "pre_write" => Hook::PreWrite,
            "post_write" => Hook::PostWrite,
            "pre_clean" => Hook::PreClean,
            "post_clean" => Hook::PostClean,
            _ => Hook::Custom(name.to_string()),
        }
    }
}

/// Context passed to hook handlers
#[derive(Debug)]
pub struct HookContext {
    /// Data available to the hook
    pub data: HashMap<String, Value>,
    /// Site configuration
    pub site_config: HashMap<String, Value>,
    /// Current page being processed (if applicable)
    pub current_page: Option<String>,
    /// Output directory
    pub output_dir: String,
    /// Source directory
    pub source_dir: String,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(source_dir: String, output_dir: String) -> Self {
        Self {
            data: HashMap::new(),
            site_config: HashMap::new(),
            current_page: None,
            output_dir,
            source_dir,
        }
    }

    /// Add data to the context
    pub fn add_data(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    /// Get data from the context
    pub fn get_data(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Set the current page
    pub fn set_current_page(&mut self, page: String) {
        self.current_page = Some(page);
    }

    /// Clear the current page
    pub fn clear_current_page(&mut self) {
        self.current_page = None;
    }
}

/// Result of a hook execution
#[derive(Debug)]
pub enum HookResult {
    /// Continue with the next hook
    Continue,
    /// Stop propagation to other hooks
    StopPropagation,
    /// Error occurred
    Error(String),
}

impl From<String> for HookResult {
    fn from(error: String) -> Self {
        HookResult::Error(error)
    }
}

impl From<&str> for HookResult {
    fn from(error: &str) -> Self {
        HookResult::Error(error.to_string())
    }
}