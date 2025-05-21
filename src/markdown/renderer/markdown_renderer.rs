use crate::config::Config;
use crate::markdown::types::BoxResult;
use crate::markdown::engine::{create_comrak_options, render_markdown};
use crate::markdown::renderer::syntax::SyntaxHighlighter;

/// Markdown renderer with syntax highlighting capabilities
pub struct MarkdownRenderer<'a> {
    engine: String,
    options: comrak::Options<'a>,
    syntax_highlighter: SyntaxHighlighter,
}

impl<'a> MarkdownRenderer<'a> {
    /// Create a new markdown renderer from config
    pub fn new(config: &Config) -> Self {
        // Get comrak options
        let options = create_comrak_options();
        
        // Create syntax highlighter
        let syntax_highlighter = SyntaxHighlighter::new();
        
        MarkdownRenderer {
            engine: "comrak".to_string(),
            options,
            syntax_highlighter,
        }
    }
    
    /// Set the syntax highlighting theme
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        self.syntax_highlighter.set_theme(theme_name)
    }
    
    /// Get available syntax highlighting themes
    pub fn available_themes(&self) -> Vec<String> {
        self.syntax_highlighter.available_themes()
    }
    
    /// Render Markdown content to HTML with syntax highlighting
    pub fn render(&self, content: &str) -> String {
        // First do basic markdown rendering
        let html = render_markdown(content, &self.options);
        
        // Then process code blocks for syntax highlighting if needed
        match self.syntax_highlighter.highlight_html(&html) {
            Ok(highlighted) => highlighted,
            Err(e) => {
                // If highlighting fails, just return the original HTML
                log::warn!("Syntax highlighting failed: {}", e);
                html
            }
        }
    }
}

/// Render markdown content to HTML - convenience function
pub fn markdownify<'a>(content: &str, renderer: &MarkdownRenderer<'a>) -> String {
    renderer.render(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_rendering() {
        let config = Config::default();
        let renderer = MarkdownRenderer::new(&config);
        
        let markdown = "# Hello, World!\n\nThis is a **bold** statement.";
        let html = renderer.render(markdown);
        
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
    
    #[test]
    fn test_syntax_highlighting() {
        let config = Config::default();
        let renderer = MarkdownRenderer::new(&config);
        
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, World!\");\n}\n```";
        let html = renderer.render(markdown);
        
        assert!(html.contains("<div class=\"highlight\">"));
        assert!(html.contains("<pre class=\"highlight rust\">"));
    }
} 