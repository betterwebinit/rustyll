use std::path::PathBuf;
use liquid::Parser;
use liquid::model::Value;
use log::debug;

use crate::config::Config;
use crate::utils::error::{BoxResult, RustyllError};
use crate::utils::fs;

/// Layout renderer for Jekyll-style layouts
pub struct LayoutRenderer {
    /// Layout directory
    layouts_dir: PathBuf,
    
    /// Liquid parser
    parser: Parser,
}

impl LayoutRenderer {
    /// Create a new layout renderer
    pub fn new(config: &Config) -> Self {
        let layouts_dir = config.layouts_dir.clone();
        
        // Create a Liquid parser with Jekyll-compatible settings
        let parser = liquid::ParserBuilder::with_stdlib()
            .build()
            .expect("Failed to create Liquid parser");
        
        LayoutRenderer {
            layouts_dir,
            parser,
        }
    }
    
    /// Render content with a layout
    pub fn render(&self, content: &str, layout_name: &str, globals: &liquid::Object) -> BoxResult<String> {
        // Find the layout file
        let layout_path = self.find_layout(layout_name)?;
        debug!("Using layout: {}", layout_path.display());
        
        // Read the layout template
        let layout_content = fs::read_file(&layout_path)?;
        
        // Parse the layout
        let template = self.parser.parse(&layout_content)
            .map_err(|e| RustyllError::Template(format!(
                "Failed to parse layout {}: {}", layout_name, e
            )))?;
        
        // Create a copy of globals to modify
        let mut render_globals = globals.clone();
        
        // Add content to globals
        render_globals.insert("content".into(), Value::Scalar(content.to_string().into()));
        
        // Render the template
        let rendered = template.render(&render_globals)
            .map_err(|e| RustyllError::Template(format!(
                "Failed to render layout {}: {}", layout_name, e
            )))?;
        
        Ok(rendered)
    }
    
    /// Find a layout file by name
    fn find_layout(&self, name: &str) -> BoxResult<PathBuf> {
        // First try with the exact name
        let layout_path = self.layouts_dir.join(name);
        
        // If the layout doesn't have an extension, try common extensions
        if !layout_path.extension().is_some() {
            for ext in &["html", "liquid", "md", "markdown"] {
                let with_ext = self.layouts_dir.join(format!("{}.{}", name, ext));
                if with_ext.exists() {
                    return Ok(with_ext);
                }
            }
        }
        
        // Check if the layout exists
        if layout_path.exists() {
            return Ok(layout_path);
        }
        
        // If layout doesn't exist, return an error
        Err(RustyllError::Template(format!(
            "Layout not found: {}", name
        )).into())
    }
} 