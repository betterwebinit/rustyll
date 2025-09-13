use regex::Regex;
use crate::front_matter::FrontMatter;

/// Default excerpt separator
const DEFAULT_EXCERPT_SEPARATOR: &str = "<!--more-->";

/// Extract a title from the first heading in content
pub fn extract_title_from_content(content: &str) -> Option<String> {
    // Look for # Heading in Markdown
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Check for Markdown h1 heading (# Title)
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
        
        // Check for Markdown h1 heading (Title =====)
        if trimmed.starts_with("=") && trimmed.chars().all(|c| c == '=') {
            // Find the line above
            let lines: Vec<&str> = content.lines().collect();
            let line_pos = lines.iter().position(|l| l.trim() == trimmed)?;
            
            if line_pos > 0 {
                let title_line = lines[line_pos - 1].trim();
                if !title_line.is_empty() {
                    return Some(title_line.to_string());
                }
            }
        }
        
        // Check for HTML <h1> heading
        if let Some(h1_content) = extract_html_h1(trimmed) {
            return Some(h1_content);
        }
    }
    
    None
}

/// Extract content from an <h1> tag
fn extract_html_h1(line: &str) -> Option<String> {
    // Simple regex-less HTML parsing for <h1> tags
    let lower = line.to_lowercase();
    
    // Find opening and closing tags
    let start_idx = lower.find("<h1")?;
    let end_tag_idx = lower.find("</h1>")?;
    
    // Find closing '>' of the opening tag
    let content_start_idx = lower[start_idx..].find('>')? + start_idx + 1;
    
    // Extract content between the tags
    if content_start_idx < end_tag_idx {
        let h1_content = &line[content_start_idx..end_tag_idx];
        return Some(h1_content.trim().to_string());
    }
    
    None
}

/// Extract an excerpt from content
pub fn extract_excerpt(content: &str, front_matter: &FrontMatter) -> Option<String> {
    // Use custom separator from front matter or default
    let separator = front_matter.excerpt_separator.as_deref().unwrap_or(DEFAULT_EXCERPT_SEPARATOR);
    
    // If the separator exists in the content, extract everything before it
    if let Some(idx) = content.find(separator) {
        return Some(content[..idx].trim().to_string());
    }
    
    // Otherwise, extract the first paragraph (or first N chars)
    extract_first_paragraph(content)
}

/// Extract the first paragraph from content
fn extract_first_paragraph(content: &str) -> Option<String> {
    let mut paragraph = String::new();
    let mut in_paragraph = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines
        if trimmed.is_empty() {
            if in_paragraph {
                // End of paragraph
                break;
            }
            continue;
        }
        
        // Skip Markdown headings
        if trimmed.starts_with('#') || trimmed.starts_with('>') {
            continue;
        }
        
        // Start or continue paragraph
        in_paragraph = true;
        
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed);
    }
    
    // Return paragraph if not empty
    if paragraph.is_empty() {
        None
    } else {
        Some(paragraph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_title_from_markdown_heading() {
        let content = "# My Title\n\nContent goes here";
        let title = extract_title_from_content(content);
        assert_eq!(title, Some("My Title".to_string()));
    }
    
    #[test]
    fn test_extract_title_from_html() {
        let content = "<h1>My HTML Title</h1>\n\nContent goes here";
        let title = extract_title_from_content(content);
        assert_eq!(title, Some("My HTML Title".to_string()));
    }
    
    #[test]
    fn test_extract_excerpt_with_separator() {
        let content = "First paragraph\n\n<!--more-->\n\nSecond paragraph";
        let front_matter = FrontMatter::default();
        let excerpt = extract_excerpt(content, &front_matter);
        assert_eq!(excerpt, Some("First paragraph".to_string()));
    }
    
    #[test]
    fn test_extract_excerpt_first_paragraph() {
        let content = "First paragraph\n\nSecond paragraph";
        let front_matter = FrontMatter::default();
        let excerpt = extract_excerpt(content, &front_matter);
        assert_eq!(excerpt, Some("First paragraph".to_string()));
    }
    
    #[test]
    fn test_extract_excerpt_custom_separator() {
        let content = "First paragraph\n\n<!-- excerpt end -->\n\nSecond paragraph";
        let mut front_matter = FrontMatter::default();
        front_matter.excerpt_separator = Some("<!-- excerpt end -->".to_string());
        let excerpt = extract_excerpt(content, &front_matter);
        assert_eq!(excerpt, Some("First paragraph".to_string()));
    }
} 