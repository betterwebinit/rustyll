use liquid_core::{Runtime, Error, BlockReflection, ParseBlock, TagBlock, Renderable, TagTokenIter};
use regex::Regex;
use std::io::Write;
use log::{warn, debug, error};
use crate::config::Config;
use std::collections::HashMap;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::html::{highlighted_html_for_string, ClassStyle, ClassedHTMLGenerator};
use syntect::util::LinesWithEndings;
use once_cell::sync::Lazy;

// Load syntax and theme sets once at startup
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    SyntaxSet::load_defaults_newlines()
});

static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    ThemeSet::load_defaults()
});

/// Jekyll-compatible highlight tag block for code syntax highlighting
#[derive(Debug, Clone)]
pub struct HighlightBlock {
    config: Config,
}

impl HighlightBlock {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Parse highlight options from the input string
    fn parse_options(&self, input: &str) -> HashMap<String, String> {
        let mut options = HashMap::new();
        if input.trim().is_empty() {
            return options;
        }

        // Match options in the form key="value", key=value, or key
        let regex = Regex::new(r#"(?:\w+="[^"]*"|\w+=\w+|\w+)"#).unwrap();
        for cap in regex.captures_iter(input) {
            let opt = cap[0].to_string();
            if let Some(pos) = opt.find('=') {
                let (key, value) = opt.split_at(pos);
                let value = value[1..].trim_matches('"').to_string();
                options.insert(key.to_string(), value);
            } else {
                // Option without value (like "linenos")
                options.insert(opt, "true".to_string());
            }
        }

        options
    }

    /// Get syntax reference for a language
    fn get_syntax_for_language(lang: &str) -> Option<&'static SyntaxReference> {
        // Map Jekyll/Rouge language names to Syntect syntax names
        let mapped_lang = match lang.to_lowercase().as_str() {
            "ruby" | "rb" => "Ruby",
            "python" | "py" | "python3" => "Python",
            "javascript" | "js" => "JavaScript",
            "jsx" => "JavaScript (JSX)",
            "typescript" | "ts" => "TypeScript",
            "tsx" => "TypeScript (TSX)",
            "vue" => "Vue.js",
            "java" => "Java",
            "c" => "C",
            "cpp" | "c++" | "cxx" => "C++",
            "csharp" | "cs" | "c#" => "C#",
            "php" => "PHP",
            "html" | "htm" => "HTML",
            "xml" => "XML",
            "css" => "CSS",
            "scss" | "sass" => "Sass",
            "json" => "JSON",
            "yaml" | "yml" => "YAML",
            "markdown" | "md" => "Markdown",
            "rust" | "rs" => "Rust",
            "go" | "golang" => "Go",
            "shell" | "bash" | "sh" | "zsh" => "Bourne Again Shell (bash)",
            "powershell" | "ps1" => "PowerShell",
            "sql" => "SQL",
            "dockerfile" | "docker" => "Dockerfile",
            "makefile" | "make" => "Makefile",
            "toml" => "TOML",
            "ini" | "cfg" | "conf" => "INI",
            "diff" | "patch" => "Diff",
            "text" | "plain" | "plaintext" | "txt" => "Plain Text",
            _ => {
                // Try to find syntax by extension or name directly
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(lang) {
                    return Some(syntax);
                }

                // Try to find by name
                if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(lang) {
                    return Some(syntax);
                }

                // Try case-insensitive search
                for syntax in SYNTAX_SET.syntaxes() {
                    if syntax.name.to_lowercase() == lang.to_lowercase() {
                        return Some(syntax);
                    }
                }

                return None;
            }
        };

        SYNTAX_SET.find_syntax_by_name(mapped_lang)
    }

    /// Highlight code with line numbers
    fn highlight_with_line_numbers(&self, code: &str, syntax: &SyntaxReference, theme: &Theme) -> String {
        let mut result = String::new();
        result.push_str(r#"<div class="highlight"><table><tbody>"#);

        let lines: Vec<&str> = code.lines().collect();
        let num_lines = lines.len();
        let line_number_width = num_lines.to_string().len();

        // Generate line numbers column
        result.push_str(r#"<tr><td class="gutter gl" style="text-align: right"><pre class="lineno">"#);
        for i in 1..=num_lines {
            result.push_str(&format!("{:width$}\n", i, width = line_number_width));
        }
        result.push_str("</pre></td>");

        // Generate highlighted code column
        result.push_str(r#"<td class="code"><pre>"#);

        // Use syntect to highlight the code
        match highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme) {
            Ok(highlighted) => {
                // Remove the outer <pre> tags that syntect adds
                let highlighted = highlighted
                    .strip_prefix("<pre style=\"")
                    .and_then(|s| s.find("\">").map(|i| &s[i+2..]))
                    .and_then(|s| s.strip_suffix("</pre>\n"))
                    .unwrap_or(&highlighted);
                result.push_str(highlighted);
            }
            Err(e) => {
                error!("Failed to highlight code: {}", e);
                result.push_str(&html_escape::encode_text(code));
            }
        }

        result.push_str("</pre></td></tr></tbody></table></div>");
        result
    }

    /// Highlight code using CSS classes instead of inline styles
    fn highlight_with_classes(&self, code: &str, syntax: &SyntaxReference, lang: &str, linenos: bool) -> String {
        let mut result = String::new();

        if linenos {
            // Generate with line numbers using table layout
            result.push_str(&format!(
                r#"<div class="highlight"><table class="highlighttable"><tbody><tr><td class="linenos"><div class="linenodiv"><pre>"#
            ));

            let lines: Vec<&str> = code.lines().collect();
            for i in 1..=lines.len() {
                result.push_str(&format!("{}\n", i));
            }

            result.push_str(r#"</pre></div></td><td class="code"><div class="highlight"><pre class="highlight">"#);
            result.push_str(&format!(r#"<code class="language-{}" data-lang="{}">"#, lang, lang));

            // Generate highlighted code with CSS classes
            let mut generator = ClassedHTMLGenerator::new_with_class_style(
                syntax,
                &SYNTAX_SET,
                ClassStyle::Spaced
            );

            for line in LinesWithEndings::from(code) {
                let _ = generator.parse_html_for_line_which_includes_newline(line);
            }

            result.push_str(&generator.finalize());
            result.push_str("</code></pre></div></td></tr></tbody></table></div>");
        } else {
            // Generate without line numbers
            result.push_str(&format!(
                r#"<figure class="highlight"><pre><code class="language-{}" data-lang="{}">"#,
                lang, lang
            ));

            // Generate highlighted code with CSS classes
            let mut generator = ClassedHTMLGenerator::new_with_class_style(
                syntax,
                &SYNTAX_SET,
                ClassStyle::Spaced
            );

            for line in LinesWithEndings::from(code) {
                let _ = generator.parse_html_for_line_which_includes_newline(line);
            }

            result.push_str(&generator.finalize());
            result.push_str("</code></pre></figure>");
        }

        result
    }

    /// Apply syntax highlighting to code
    fn highlight_code(&self, code: &str, lang: &str, options: &HashMap<String, String>) -> String {
        // Get the syntax definition
        let syntax = match Self::get_syntax_for_language(lang) {
            Some(s) => s,
            None => {
                warn!("Unknown language '{}', falling back to plain text", lang);
                SYNTAX_SET.find_syntax_plain_text()
            }
        };

        // Check for line numbers option
        let linenos = options.get("linenos").is_some()
            || options.get("linenums").is_some()
            || options.get("line_numbers").is_some();

        // Check for inline styles vs CSS classes
        let use_classes = options.get("cssclass").is_some()
            || options.get("class").is_some()
            || self.config.highlighter.as_str() == "rouge";

        if use_classes {
            // Use CSS classes (Jekyll/Rouge compatible)
            self.highlight_with_classes(code, syntax, lang, linenos)
        } else {
            // Use inline styles
            let theme_name = options.get("theme")
                .map(|s| s.as_str())
                .unwrap_or("InspiredGitHub");

            let theme = THEME_SET.themes.get(theme_name)
                .unwrap_or_else(|| {
                    warn!("Theme '{}' not found, using default", theme_name);
                    &THEME_SET.themes["InspiredGitHub"]
                });

            if linenos {
                self.highlight_with_line_numbers(code, syntax, theme)
            } else {
                // Simple highlighting without line numbers
                let mut result = format!(
                    r#"<figure class="highlight"><pre><code class="language-{}" data-lang="{}">"#,
                    lang, lang
                );

                match highlighted_html_for_string(code, &SYNTAX_SET, syntax, theme) {
                    Ok(highlighted) => {
                        // Remove the outer <pre> tags that syntect adds
                        let highlighted = highlighted
                            .strip_prefix("<pre style=\"")
                            .and_then(|s| s.find("\">").map(|i| &s[i+2..]))
                            .and_then(|s| s.strip_suffix("</pre>\n"))
                            .unwrap_or(&highlighted);
                        result.push_str(highlighted);
                    }
                    Err(e) => {
                        error!("Failed to highlight code: {}", e);
                        result.push_str(&html_escape::encode_text(code));
                    }
                }

                result.push_str("</code></pre></figure>");
                result
            }
        }
    }
}

struct HighlightBlockReflection;

impl BlockReflection for HighlightBlockReflection {
    fn start_tag(&self) -> &str {
        "highlight"
    }

    fn description(&self) -> &str {
        "Highlight code with syntax highlighting"
    }

    fn end_tag(&self) -> &str {
        "endhighlight"
    }
}

impl ParseBlock for HighlightBlock {
    fn reflection(&self) -> &dyn BlockReflection {
        &HighlightBlockReflection
    }

    fn parse(&self, arguments: TagTokenIter, mut content: TagBlock<'_, '_>, _options: &liquid_core::parser::Language) -> Result<Box<dyn Renderable>, Error> {
        // Get the language argument - we need to collect the tokens to a string
        let args: Vec<_> = arguments.collect();
        let args_str = if !args.is_empty() {
            args[0].as_str().trim()
        } else {
            ""
        };

        // Parse the language (simple approach for now)
        let lang = if args_str.is_empty() {
            warn!("No language specified in highlight tag, defaulting to text");
            "text".to_string()
        } else {
            // Extract just the language part (without options)
            if let Some(first_space) = args_str.find(' ') {
                args_str[..first_space].to_string()
            } else {
                args_str.to_string()
            }
        };

        // Get options string if present
        let options_str = if let Some(first_space) = args_str.find(' ') {
            args_str[first_space..].to_string()
        } else {
            "".to_string()
        };

        // Parse options
        let options = self.parse_options(&options_str);

        // Get the content inside the block
        let content_str = content.escape_liquid(false)?;

        debug!("Highlight block: lang={}, options={:?}", lang, options);

        // Create the renderer with the parsed values
        Ok(Box::new(HighlightBlockRenderer {
            config: self.config.clone(),
            lang,
            options,
            content: content_str.to_string(),
        }))
    }
}

/// Renderer for the highlight tag
#[derive(Debug)]
struct HighlightBlockRenderer {
    config: Config,
    lang: String,
    options: HashMap<String, String>,
    content: String,
}

impl Renderable for HighlightBlockRenderer {
    fn render(&self, _runtime: &dyn Runtime) -> Result<String, Error> {
        let highlight_block = HighlightBlock::new(self.config.clone());

        // Apply syntax highlighting
        let result = highlight_block.highlight_code(&self.content, &self.lang, &self.options);

        Ok(result)
    }

    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<(), Error> {
        let s = self.render(runtime)?;
        writer.write_all(s.as_bytes())
            .map_err(|e| Error::with_msg(format!("Failed to write to output: {}", e)))?;
        Ok(())
    }
}