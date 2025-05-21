use std::path::Path;
use std::fs;

pub async fn check_accessibility(source_dir: &Path, verbose: bool) -> Result<Vec<String>, String> {
    if verbose {
        log::info!("Running accessibility check on {}...", source_dir.display());
    }
    
    let mut issues = Vec::new();
    
    // Recursively scan HTML files in the source directory
    scan_directory_for_accessibility(source_dir, &mut issues, verbose)?;
    
    if verbose {
        log::info!("Accessibility check completed, found {} issues", issues.len());
    }
    
    Ok(issues)
}

fn scan_directory_for_accessibility(dir: &Path, issues: &mut Vec<String>, verbose: bool) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_accessibility(&path, issues, verbose)?;
        } else if path.is_file() {
            // Check if it's an HTML file
            if let Some(extension) = path.extension() {
                if extension == "html" {
                    check_html_file_accessibility(&path, issues, verbose)?;
                }
            }
        }
    }
    
    Ok(())
}

fn check_html_file_accessibility(file_path: &Path, issues: &mut Vec<String>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking accessibility for {}", file_path.display());
    }
    
    // Read the HTML file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    // Simple checks (in a real implementation you would use a proper accessibility library)
    
    // Check for images without alt attributes
    if content.contains("<img") && !content.contains("<img alt=\"") && !content.contains("<img alt='") {
        issues.push(format!("Image without alt text in {}", file_path.display()));
    }
    
    // Check for empty links or links with only #
    if content.contains("<a href=\"#\">") || content.contains("<a href='#'>") {
        issues.push(format!("Empty link found in {}", file_path.display()));
    }
    
    // Check for missing language attribute in html tag
    if !content.contains("<html lang=") {
        issues.push(format!("Missing language attribute in HTML tag in {}", file_path.display()));
    }
    
    // Check for missing title
    if !content.contains("<title>") {
        issues.push(format!("Missing title tag in {}", file_path.display()));
    }
    
    // Color contrast issues would require more complex analysis
    
    // Missing form labels
    if (content.contains("<input") || content.contains("<textarea")) && !content.contains("<label") {
        issues.push(format!("Form elements without labels in {}", file_path.display()));
    }
    
    Ok(())
} 