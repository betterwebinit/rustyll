use regex::Regex;

/// Extract title from first heading in content
pub fn extract_title_from_content(content: &str) -> Option<String> {
    // Look for a Markdown H1 heading
    let h1_regex = Regex::new(r"^#\s+(.+)$").unwrap();
    
    for line in content.lines() {
        if let Some(captures) = h1_regex.captures(line) {
            if let Some(title) = captures.get(1) {
                return Some(title.as_str().trim().to_string());
            }
        }
    }
    
    // Look for an HTML <h1> tag
    let html_h1_regex = Regex::new(r"<h1[^>]*>([^<]+)</h1>").unwrap();
    
    if let Some(captures) = html_h1_regex.captures(content) {
        if let Some(title) = captures.get(1) {
            return Some(title.as_str().trim().to_string());
        }
    }
    
    None
}

/// Extract the first paragraph as an excerpt
pub fn extract_excerpt(content: &str, excerpt_separator: &str) -> Option<String> {
    // Check for custom excerpt separator
    if excerpt_separator != "\n\n" {
        if let Some(pos) = content.find(excerpt_separator) {
            return Some(content[..pos].trim().to_string());
        }
    }
    
    // Default to first paragraph
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    if !paragraphs.is_empty() {
        // Strip any HTML tags
        let first_para = paragraphs[0].trim();
        let html_tag_regex = Regex::new(r"<[^>]+>").unwrap();
        let clean_para = html_tag_regex.replace_all(first_para, "");
        
        return Some(clean_para.to_string());
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_title_from_content() {
        let content = "# My Page Title\n\nThis is the content.";
        assert_eq!(extract_title_from_content(content), Some("My Page Title".to_string()));
        
        let html_content = "<h1>HTML Title</h1>\n<p>This is HTML content.</p>";
        assert_eq!(extract_title_from_content(html_content), Some("HTML Title".to_string()));
    }
    
    #[test]
    fn test_extract_excerpt() {
        let content = "First paragraph.\n\nSecond paragraph.";
        assert_eq!(extract_excerpt(content, "\n\n"), Some("First paragraph.".to_string()));
        
        let content_with_separator = "Before separator.\n<!-- more -->\nAfter separator.";
        assert_eq!(extract_excerpt(content_with_separator, "<!-- more -->"), Some("Before separator.".to_string()));
    }
} 