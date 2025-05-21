use liquid_core::{Runtime, Error, BlockReflection, ParseBlock, TagBlock, Renderable, TagTokenIter};
use std::io::Write;

/// Jekyll-compatible raw tag block
#[derive(Debug, Clone)]
pub struct RawBlock;

impl RawBlock {
    pub fn new() -> Self {
        Self
    }
}

struct RawBlockReflection;

impl BlockReflection for RawBlockReflection {
    fn start_tag(&self) -> &str {
        "raw"
    }

    fn description(&self) -> &str {
        "Preserve content without Liquid parsing"
    }

    fn end_tag(&self) -> &str {
        "endraw"
    }
}

impl ParseBlock for RawBlock {
    fn reflection(&self) -> &dyn BlockReflection {
        &RawBlockReflection
    }
    
    fn parse(&self, _arguments: TagTokenIter, mut content: TagBlock<'_, '_>, _options: &liquid_core::parser::Language) -> Result<Box<dyn Renderable>, Error> {
        // Get all content from the block using escape_liquid
        // This allows us to get the raw content without parsing any liquid tags inside
        let content_str = content.escape_liquid(false)?;
        
        // Store the content as-is without parsing
        Ok(Box::new(RawBlockRenderer { content: content_str.to_string() }))
    }
}

/// Renderer for the raw tag
#[derive(Debug)]
struct RawBlockRenderer {
    content: String,
}

impl Renderable for RawBlockRenderer {
    fn render(&self, _runtime: &dyn Runtime) -> Result<String, Error> {
        // Return the content as-is without any processing
        Ok(self.content.clone())
    }

    fn render_to(&self, writer: &mut dyn Write, _runtime: &dyn Runtime) -> Result<(), Error> {
        writer.write_all(self.content.as_bytes())
            .map_err(|e| Error::with_msg(format!("Failed to write to output: {}", e)))?;
        Ok(())
    }
} 