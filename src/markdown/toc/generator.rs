use regex::Regex;
use crate::markdown::types::BoxResult;

/// Generate a table of contents from Markdown content
pub fn generate_toc(markdown: &str) -> BoxResult<String> {
    let heading_regex = Regex::new(r"(?m)^(#{1,6})\s+(.+)$")?;
    
    let mut toc = String::from("<ul class=\"table-of-contents\">\n");
    let mut current_level = 0;
    
    for cap in heading_regex.captures_iter(markdown) {
        let level = cap[1].len();
        let text = cap[2].trim();
        
        // Generate slug for the heading
        let slug = slug::slugify(text);
        
        // Adjust nesting level
        while current_level < level {
            toc.push_str("<ul>\n");
            current_level += 1;
        }
        
        while current_level > level {
            toc.push_str("</ul>\n");
            current_level -= 1;
        }
        
        // Add the TOC entry
        toc.push_str(&format!("<li><a href=\"#{}\">{}</a></li>\n", slug, text));
    }
    
    // Close any remaining lists
    while current_level > 0 {
        toc.push_str("</ul>\n");
        current_level -= 1;
    }
    
    toc.push_str("</ul>");
    
    Ok(toc)
}

/// Extract headings from markdown content
pub fn extract_headings(markdown: &str) -> BoxResult<Vec<(usize, String)>> {
    let heading_regex = Regex::new(r"(?m)^(#{1,6})\s+(.+)$")?;
    let mut headings = Vec::new();
    
    for cap in heading_regex.captures_iter(markdown) {
        let level = cap[1].len();
        let text = cap[2].trim().to_string();
        headings.push((level, text));
    }
    
    Ok(headings)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toc_generation() {
        let markdown = "# Top Heading\n\nText here.\n\n## Sub Heading\n\nMore text.\n\n## Another Sub\n\n### Deep Heading";
        let toc = generate_toc(markdown).unwrap();
        
        assert!(toc.contains("<ul class=\"table-of-contents\">"));
        assert!(toc.contains("<li><a href=\"#top-heading\">Top Heading</a></li>"));
        assert!(toc.contains("<li><a href=\"#sub-heading\">Sub Heading</a></li>"));
        assert!(toc.contains("<li><a href=\"#deep-heading\">Deep Heading</a></li>"));
    }
    
    #[test]
    fn test_extract_headings() {
        let markdown = "# Top Heading\n\nText here.\n\n## Sub Heading\n\nMore text.";
        let headings = extract_headings(markdown).unwrap();
        
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0], (1, "Top Heading".to_string()));
        assert_eq!(headings[1], (2, "Sub Heading".to_string()));
    }
} 