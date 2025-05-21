use std::path::Path;
use std::fs;

pub async fn check_performance(source_dir: &Path, verbose: bool) -> Result<Vec<(String, String)>, String> {
    if verbose {
        log::info!("Checking performance in {}...", source_dir.display());
    }
    
    let mut issues = Vec::new();
    
    // Recursively scan files in the source directory
    scan_directory_for_performance(source_dir, &mut issues, verbose)?;
    
    if verbose {
        log::info!("Performance check completed, found {} issues", issues.len());
    }
    
    Ok(issues)
}

fn scan_directory_for_performance(dir: &Path, issues: &mut Vec<(String, String)>, verbose: bool) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_for_performance(&path, issues, verbose)?;
        } else if path.is_file() {
            // Get file extension
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                
                match ext.as_str() {
                    "html" => check_html_performance(&path, issues, verbose)?,
                    "css" => check_css_performance(&path, issues, verbose)?,
                    "js" => check_js_performance(&path, issues, verbose)?,
                    "jpg" | "jpeg" | "png" | "gif" | "webp" => check_image_performance(&path, issues, verbose)?,
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

fn check_html_performance(file_path: &Path, issues: &mut Vec<(String, String)>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking HTML performance for {}", file_path.display());
    }
    
    // Read the file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check if HTML is minified (simple heuristic)
    if !is_minified(&content) {
        issues.push((file_path_str.clone(), "HTML file is not minified".to_string()));
    }
    
    // Check for large inline scripts
    if content.contains("<script>") {
        let script_size = content
            .split("<script>")
            .skip(1)
            .map(|s| s.split("</script>").next().unwrap_or("").len())
            .sum::<usize>();
        
        if script_size > 1024 * 5 { // More than 5KB of inline scripts
            issues.push((file_path_str.clone(), format!("Large inline scripts ({} KB)", script_size / 1024)));
        }
    }
    
    // Check file size
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata for {}: {}", file_path.display(), e))?;
    let file_size = metadata.len();
    
    if file_size > 1024 * 100 { // 100 KB
        issues.push((file_path_str, format!("HTML file is large: {} KB", file_size / 1024)));
    }
    
    Ok(())
}

fn check_css_performance(file_path: &Path, issues: &mut Vec<(String, String)>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking CSS performance for {}", file_path.display());
    }
    
    // Read the file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check if CSS is minified (simple heuristic)
    if !is_minified(&content) {
        issues.push((file_path_str.clone(), "CSS file is not minified".to_string()));
    }
    
    // Check file size
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata for {}: {}", file_path.display(), e))?;
    let file_size = metadata.len();
    
    if file_size > 1024 * 50 { // 50 KB
        issues.push((file_path_str, format!("CSS file is large: {} KB", file_size / 1024)));
    }
    
    Ok(())
}

fn check_js_performance(file_path: &Path, issues: &mut Vec<(String, String)>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking JS performance for {}", file_path.display());
    }
    
    // Read the file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check if JS is minified (simple heuristic)
    if !is_minified(&content) {
        issues.push((file_path_str.clone(), "JavaScript file is not minified".to_string()));
    }
    
    // Check file size
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata for {}: {}", file_path.display(), e))?;
    let file_size = metadata.len();
    
    if file_size > 1024 * 100 { // 100 KB
        issues.push((file_path_str, format!("JavaScript file is large: {} KB", file_size / 1024)));
    }
    
    Ok(())
}

fn check_image_performance(file_path: &Path, issues: &mut Vec<(String, String)>, verbose: bool) -> Result<(), String> {
    if verbose {
        log::info!("Checking image performance for {}", file_path.display());
    }
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check file size
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to get metadata for {}: {}", file_path.display(), e))?;
    let file_size = metadata.len();
    
    if file_size > 1024 * 200 { // 200 KB
        issues.push((file_path_str, format!("Image is large: {} KB", file_size / 1024)));
    }
    
    // For a real implementation, you might want to check image dimensions
    // This would require an image processing library like 'image'
    
    Ok(())
}

// Helper to check if a file appears to be minified
fn is_minified(content: &str) -> bool {
    let content = content.trim();
    
    // Count newlines
    let newline_count = content.matches('\n').count();
    
    // If there are few newlines relative to content length, it's probably minified
    let newline_ratio = newline_count as f64 / content.len() as f64;
    
    // Arbitrary threshold based on typical minified files
    newline_ratio < 0.01
} 