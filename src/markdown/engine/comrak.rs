use comrak::Options;

/// Create default ComrakOptions with GitHub Flavored Markdown settings
pub fn create_comrak_options<'a>() -> Options<'a> {
    let mut options = Options::default();
    
    // Extension options - GitHub Flavored Markdown
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.header_ids = Some("header-".to_string());
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    
    // Render options
    options.render.hardbreaks = false;
    options.render.github_pre_lang = true;
    options.render.unsafe_ = true; // Allow HTML (careful with this!)
    
    // Parse options
    options.parse.smart = true;
    options.parse.default_info_string = Some("text".to_string());
    
    options
}

/// Render markdown to HTML using Comrak
pub fn render_markdown<'a>(content: &str, options: &Options<'a>) -> String {
    comrak::markdown_to_html(content, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comrak_rendering() {
        let options = create_comrak_options();
        let markdown = "# Hello, World!\n\nThis is a **bold** statement.";
        let html = render_markdown(markdown, &options);
        
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
} 