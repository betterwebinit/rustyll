use std::sync::Arc;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::parsing::SyntaxSet;
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::util::LinesWithEndings;
use regex::Regex;

use crate::markdown::types::BoxResult;

/// Component for syntax highlighting code blocks in HTML
pub struct SyntaxHighlighter {
    syntax_set: Arc<SyntaxSet>,
    theme_set: Arc<ThemeSet>,
    current_theme: String,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default settings
    pub fn new() -> Self {
        // Load syntect syntax and theme sets
        let syntax_set = Arc::new(SyntaxSet::load_defaults_newlines());
        let theme_set = Arc::new(ThemeSet::load_defaults());
        
        SyntaxHighlighter {
            syntax_set,
            theme_set,
            current_theme: "InspiredGitHub".to_string(), // Default theme
        }
    }
    
    /// Set the syntax highlighting theme
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        if self.theme_set.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
            true
        } else {
            false
        }
    }
    
    /// Get the list of available themes
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_set.themes.keys()
            .map(|k| k.to_string())
            .collect()
    }
    
    /// Process HTML content to add syntax highlighting to code blocks
    pub fn highlight_html(&self, html: &str) -> BoxResult<String> {
        let pre_regex = Regex::new(r#"<pre><code( class="language-([^"]+)")?>([^<]+)</code></pre>"#)?;
        
        let html_highlighted = pre_regex.replace_all(html, |caps: &regex::Captures| {
            let code = html_escape::decode_html_entities(&caps[3]).to_string();
            let lang = caps.get(2).map(|m| m.as_str()).unwrap_or("text");
            
            self.highlight_code(&code, lang)
        });
        
        Ok(html_highlighted.to_string())
    }
    
    /// Highlight a specific code block with specified language
    pub fn highlight_code(&self, code: &str, lang: &str) -> String {
        // Get the theme
        let theme = self.theme_set.themes.get(&self.current_theme).unwrap_or_else(|| {
            // Fallback to default theme
            self.theme_set.themes.get("InspiredGitHub").unwrap()
        });
        
        // Try to get the syntax for the language
        let syntax = self.syntax_set.find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        // Set up HTML generator
        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced
        );
        
        // Process the code
        for line in LinesWithEndings::from(code) {
            html_generator.parse_html_for_line_which_includes_newline(line);
        }
        
        // Get the highlighted HTML
        let highlighted_html = html_generator.finalize();
        
        // Wrap in pre and add language class
        format!("<div class=\"highlight\"><pre class=\"highlight {}\"><code>{}</code></pre></div>", 
                lang, highlighted_html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syntax_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    println!(\"Hello, World!\");\n}";
        let html = highlighter.highlight_code(code, "rust");
        
        assert!(html.contains("<div class=\"highlight\">"));
        assert!(html.contains("<pre class=\"highlight rust\">"));
    }
    
    #[test]
    fn test_html_processing() {
        let highlighter = SyntaxHighlighter::new();
        let html = "<pre><code class=\"language-rust\">fn main() {\n    println!(\"Hello\");\n}</code></pre>";
        let processed = highlighter.highlight_html(html).unwrap();
        
        assert!(processed.contains("<div class=\"highlight\">"));
        assert!(processed.contains("<pre class=\"highlight rust\">"));
    }
} 