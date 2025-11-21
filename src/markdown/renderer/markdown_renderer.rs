use crate::config::Config;
use crate::markdown::engine::{create_comrak_options, render_markdown};
use crate::markdown::renderer::syntax::SyntaxHighlighter;
use crate::markdown::toc::{generate_toc, extract_headings, TocOptions};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Regex for math equations
    static ref MATH_INLINE_REGEX: Regex = Regex::new(r"\$(.+?)\$").unwrap();
    static ref MATH_BLOCK_REGEX: Regex = Regex::new(r"\$\$([\s\S]+?)\$\$").unwrap();
    
    // Regex for typographical improvements
    static ref SMART_QUOTES_REGEX: Regex = Regex::new(r#"(^|[-—/\(\[\{"""\s])[''](.+?)['']([-.,:;!?\)\]\}""\s]|$)"#).unwrap();
    static ref EM_DASH_REGEX: Regex = Regex::new(r"---").unwrap();
    static ref EN_DASH_REGEX: Regex = Regex::new(r"--").unwrap();
    
    // Regex for diagrams
    static ref MERMAID_REGEX: Regex = Regex::new(r"```mermaid\s([\s\S]+?)```").unwrap();
}

/// Markdown renderer with syntax highlighting and extended features
pub struct MarkdownRenderer<'a> {
    engine: String,
    options: comrak::Options<'a>,
    syntax_highlighter: SyntaxHighlighter,
    toc_options: TocOptions,
    enable_math: bool,
    enable_diagrams: bool,
    enable_typographic: bool,
}

impl<'a> MarkdownRenderer<'a> {
    /// Create a new markdown renderer from config
    pub fn new(config: &Config) -> Self {
        // Get comrak options
        let options = create_comrak_options();
        
        // Create syntax highlighter
        let syntax_highlighter = SyntaxHighlighter::new();
        
        // Set default TOC options
        let toc_options = TocOptions {
            min_level: 1,
            max_level: 6,
            include_title: false,
            list_class: "toc".to_string(),
        };
        
        // Check for markdown extensions in config
        let enable_math = config.markdown_extensions.as_ref()
            .map(|exts| exts.contains(&"math".to_string()))
            .unwrap_or(false);
            
        let enable_diagrams = config.markdown_extensions.as_ref()
            .map(|exts| exts.contains(&"diagrams".to_string()))
            .unwrap_or(false);
            
        let enable_typographic = config.markdown_extensions.as_ref()
            .map(|exts| exts.contains(&"typographic".to_string()))
            .unwrap_or(true); // Enable by default
        
        MarkdownRenderer {
            engine: "comrak".to_string(),
            options,
            syntax_highlighter,
            toc_options,
            enable_math,
            enable_diagrams,
            enable_typographic,
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
    
    /// Configure table of contents options
    pub fn set_toc_options(&mut self, options: TocOptions) {
        self.toc_options = options;
    }
    
    /// Enable or disable math equation support
    pub fn set_math_enabled(&mut self, enabled: bool) {
        self.enable_math = enabled;
    }
    
    /// Enable or disable diagram support
    pub fn set_diagrams_enabled(&mut self, enabled: bool) {
        self.enable_diagrams = enabled;
    }
    
    /// Enable or disable typographical improvements
    pub fn set_typographic_enabled(&mut self, enabled: bool) {
        self.enable_typographic = enabled;
    }
    
    /// Process math equations in the HTML content
    fn process_math(&self, html: &str) -> String {
        if !self.enable_math {
            return html.to_string();
        }
        
        // Process inline math
        let html = MATH_INLINE_REGEX.replace_all(html, |caps: &regex::Captures| {
            format!("<span class=\"math inline\">{}</span>", &caps[1])
        }).to_string();
        
        // Process block math
        let html = MATH_BLOCK_REGEX.replace_all(&html, |caps: &regex::Captures| {
            format!("<div class=\"math display\">{}</div>", &caps[1])
        }).to_string();
        
        html
    }
    
    /// Process diagrams in the HTML content
    fn process_diagrams(&self, html: &str) -> String {
        if !self.enable_diagrams {
            return html.to_string();
        }
        
        // Process mermaid diagrams
        let html = MERMAID_REGEX.replace_all(html, |caps: &regex::Captures| {
            format!(
                "<div class=\"mermaid\">{}</div>
                <script>
                document.addEventListener('DOMContentLoaded', function() {{
                  if (typeof mermaid !== 'undefined') {{
                    mermaid.initialize();
                  }}
                }});
                </script>",
                &caps[1]
            )
        }).to_string();
        
        html
    }
    
    /// Apply typographical improvements to the HTML content
    fn apply_typography(&self, html: &str) -> String {
        if !self.enable_typographic {
            return html.to_string();
        }
        
        // Replace smart quotes
        let html = SMART_QUOTES_REGEX.replace_all(html, |caps: &regex::Captures| {
            format!("{}\"{}\"{}",&caps[1].to_string(), &caps[2].to_string(), &caps[3].to_string())
        }).to_string();
        
        // Replace em dashes
        let html = EM_DASH_REGEX.replace_all(&html, "—").to_string();
        
        // Replace en dashes
        let html = EN_DASH_REGEX.replace_all(&html, "–").to_string();
        
        html
    }
    
    /// Generate a table of contents from the HTML content
    pub fn generate_toc(&self, html: &str) -> String {
        let headings = extract_headings(html);
        match headings {
            Ok(_h) => {
                // In the future, use TocOptions here
                match generate_toc(html) {
                    Ok(toc) => toc,
                    Err(_) => String::new()
                }
            },
            Err(_) => String::new()
        }
    }
    
    /// Render Markdown content to HTML with all enabled features
    pub fn render(&self, content: &str) -> String {
        // First do basic markdown rendering
        let mut html = render_markdown(content, &self.options);
        
        // Process math equations if enabled
        if self.enable_math {
            html = self.process_math(&html);
        }
        
        // Process diagrams if enabled
        if self.enable_diagrams {
            html = self.process_diagrams(&html);
        }
        
        // Apply typographical improvements if enabled
        if self.enable_typographic {
            html = self.apply_typography(&html);
        }
        
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
    
    /// Render Markdown content to HTML with table of contents
    pub fn render_with_toc(&self, content: &str) -> (String, String) {
        let html = self.render(content);
        let toc = self.generate_toc(&html);
        (html, toc)
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
    
    #[test]
    fn test_table_of_contents() {
        let config = Config::default();
        let renderer = MarkdownRenderer::new(&config);
        
        let markdown = "# Main Title\n\n## Section 1\n\nContent\n\n## Section 2\n\nMore content";
        let (html, toc) = renderer.render_with_toc(markdown);
        
        assert!(toc.contains("<ul class=\"toc\">"));
        assert!(toc.contains("<a href=\"#section-1\">"));
        assert!(toc.contains("<a href=\"#section-2\">"));
    }
    
    #[test]
    fn test_math_rendering() {
        let mut config = Config::default();
        if let Some(ref mut exts) = config.markdown_extensions {
            exts.push("math".to_string());
        } else {
            config.markdown_extensions = Some(vec!["math".to_string()]);
        }
        
        let renderer = MarkdownRenderer::new(&config);
        
        let markdown = "Inline equation: $E = mc^2$\n\nBlock equation:\n\n$$\nf(x) = \\int_{-\\infty}^\\infty \\hat f(\\xi)\\,e^{2 \\pi i \\xi x} \\,d\\xi\n$$";
        let html = renderer.render(markdown);
        
        assert!(html.contains("<span class=\"math inline\">"));
        assert!(html.contains("<div class=\"math display\">"));
    }
} 