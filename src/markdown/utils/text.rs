use regex::Regex;

/// Strip markdown formatting from text
pub fn strip_markdown(text: &str) -> String {
    // Replace headings
    let heading_regex = Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = heading_regex.replace_all(text, "$1");
    
    // Replace bold/italic
    let bold_regex = Regex::new(r"\*\*(.+?)\*\*").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = bold_regex.replace_all(&text, "$1");
    
    let italic_regex = Regex::new(r"\*(.+?)\*").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = italic_regex.replace_all(&text, "$1");
    
    // Replace links
    let link_regex = Regex::new(r"\[(.+?)\]\(.+?\)").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = link_regex.replace_all(&text, "$1");
    
    // Replace code blocks and inline code
    let code_regex = Regex::new(r"```[\s\S]*?```").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = code_regex.replace_all(&text, "");
    
    let inline_code_regex = Regex::new(r"`(.+?)`").unwrap_or_else(|_| Regex::new(r"").unwrap());
    let text = inline_code_regex.replace_all(&text, "$1");
    
    text.to_string()
}

/// Extract a summary (first n characters) from markdown content
pub fn extract_summary(markdown: &str, length: usize) -> String {
    let plain_text = strip_markdown(markdown);
    let trimmed = plain_text.trim();
    
    if trimmed.len() <= length {
        return trimmed.to_string();
    }
    
    // Try to find a sensible breakpoint (sentence or paragraph)
    let truncated = &trimmed[..length];
    
    if let Some(pos) = truncated.rfind('.') {
        return truncated[..=pos].to_string();
    }
    
    if let Some(pos) = truncated.rfind('!') {
        return truncated[..=pos].to_string();
    }
    
    if let Some(pos) = truncated.rfind('?') {
        return truncated[..=pos].to_string();
    }
    
    if let Some(pos) = truncated.rfind('\n') {
        return truncated[..=pos].to_string();
    }
    
    // Fall back to word boundary
    if let Some(pos) = truncated.rfind(' ') {
        truncated[..=pos].to_string() + "..."
    } else {
        truncated.to_string() + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strip_markdown() {
        let markdown = "# Heading\n\nThis is **bold** and *italic* text with [a link](https://example.com).\n\n```rust\nfn test() {}\n```";
        let plain = strip_markdown(markdown);
        
        assert_eq!(plain.contains("# "), false);
        assert_eq!(plain.contains("**"), false);
        assert_eq!(plain.contains("*italic*"), false);
        assert_eq!(plain.contains("[a link]"), false);
        assert_eq!(plain.contains("```"), false);
        
        assert!(plain.contains("Heading"));
        assert!(plain.contains("bold"));
        assert!(plain.contains("italic"));
        assert!(plain.contains("a link"));
    }
    
    #[test]
    fn test_extract_summary() {
        let markdown = "# Test Document\n\nThis is a sample paragraph. It has multiple sentences. We want to test the summary extraction.\n\nThis is another paragraph.";
        let summary = extract_summary(markdown, 50);
        
        assert!(summary.len() <= 50);
        assert!(summary.ends_with("."));
        assert!(summary.contains("Test Document"));
    }
} 