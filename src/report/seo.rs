use std::path::Path;
use std::fs;
use regex::Regex;

pub async fn check_seo(source_dir: &Path, verbose: bool) -> Result<Vec<String>, String> {
    if verbose {
        log::info!("Checking SEO in {}...", source_dir.display());
    }
    
    let mut issues = Vec::new();
    
    // Recursively scan HTML files in the source directory
    scan_directory_for_seo(source_dir, &mut issues, verbose)?;
    
    if verbose {
        log::info!("SEO check completed, found {} issues", issues.len());
    }
    
    Ok(issues)
}

fn scan_directory_for_seo(dir: &Path, issues: &mut Vec<String>, verbose: bool) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_seo(&path, issues, verbose)?;
        } else if path.is_file() {
            // Check if it's an HTML file
            if let Some(extension) = path.extension() {
                if extension == "html" {
                    check_html_file_seo(&path, issues, verbose)?;
                }
            }
        }
    }
    
    Ok(())
}

fn check_html_file_seo(file_path: &Path, issues: &mut Vec<String>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking SEO for {}", file_path.display());
    }
    
    // Read the HTML file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    // Check for essential SEO tags
    
    // Check meta description
    if !content.contains("<meta name=\"description\"") && !content.contains("<meta name='description'") {
        issues.push(format!("Missing meta description in {}", file_path.display()));
    }
    
    // Check title
    if !content.contains("<title>") {
        issues.push(format!("Missing title tag in {}", file_path.display()));
    } else {
        // Check if title is too long (>60 chars is generally not recommended for SEO)
        let title_regex = Regex::new(r"<title>(.*?)</title>")
            .map_err(|e| format!("Failed to create regex: {}", e))?;
        
        if let Some(cap) = title_regex.captures(&content) {
            if let Some(title_match) = cap.get(1) {
                let title = title_match.as_str();
                if title.len() > 60 {
                    issues.push(format!("Title too long ({} chars) in {}", title.len(), file_path.display()));
                } else if title.len() < 10 {
                    issues.push(format!("Title too short ({} chars) in {}", title.len(), file_path.display()));
                }
            }
        }
    }
    
    // Check headings structure
    if !content.contains("<h1") {
        issues.push(format!("Missing H1 heading in {}", file_path.display()));
    } else {
        // Count H1 tags to ensure there's only one
        let h1_count = content.matches("<h1").count();
        if h1_count > 1 {
            issues.push(format!("Multiple H1 headings ({}) in {}", h1_count, file_path.display()));
        }
    }
    
    // Check for canonical URL
    if !content.contains("<link rel=\"canonical\"") && !content.contains("<link rel='canonical'") {
        issues.push(format!("Missing canonical link in {}", file_path.display()));
    }
    
    // Check for viewport meta tag (mobile-friendliness)
    if !content.contains("<meta name=\"viewport\"") && !content.contains("<meta name='viewport'") {
        issues.push(format!("Missing viewport meta tag in {}", file_path.display()));
    }
    
    // Check for image alt text
    if content.contains("<img") && (!content.contains("alt=\"") && !content.contains("alt='")) {
        issues.push(format!("Images missing alt text in {}", file_path.display()));
    }
    
    Ok(())
} 