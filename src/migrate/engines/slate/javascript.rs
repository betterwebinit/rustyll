use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_javascript(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Slate JavaScript...");
    }

    // Create destination JavaScript directory
    let dest_js_dir = dest_dir.join("assets/js");
    create_dir_if_not_exists(&dest_js_dir)?;

    // In Slate, JavaScript is typically in source/javascripts
    let source_js_dir = source_dir.join("source/javascripts");
    if source_js_dir.exists() && source_js_dir.is_dir() {
        // Process JavaScript files
        for entry in WalkDir::new(&source_js_dir)
            .into_iter()
            .filter_map(Result::ok) {
            
            if entry.file_type().is_file() {
                let file_path = entry.path();
                
                // Process JavaScript files
                if is_javascript_file(file_path) {
                    migrate_js_file(file_path, &source_js_dir, &dest_js_dir, result)?;
                }
            }
        }
    } else {
        // Try alternative locations
        let alt_js_dirs = [
            source_dir.join("javascripts"),
            source_dir.join("source/js"),
            source_dir.join("js"),
        ];
        
        let mut found = false;
        for alt_dir in alt_js_dirs.iter() {
            if alt_dir.exists() && alt_dir.is_dir() {
                for entry in WalkDir::new(alt_dir)
                    .into_iter()
                    .filter_map(Result::ok) {
                    
                    if entry.file_type().is_file() {
                        let file_path = entry.path();
                        
                        if is_javascript_file(file_path) {
                            migrate_js_file(file_path, alt_dir, &dest_js_dir, result)?;
                            found = true;
                        }
                    }
                }
            }
        }
        
        if !found {
            // Create default script
            create_default_scripts(&dest_js_dir, result)?;
        }
    }

    Ok(())
}

fn migrate_js_file(
    file_path: &Path,
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Get the relative path from the source directory
    let rel_path = file_path.strip_prefix(source_dir)
        .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
    
    // Create destination path
    let dest_path = dest_dir.join(rel_path);
    
    // Create parent directory if needed
    if let Some(parent) = dest_path.parent() {
        create_dir_if_not_exists(parent)?;
    }
    
    // Read the JavaScript file
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read JavaScript file {}: {}", file_path.display(), e))?;
    
    // Write to destination
    fs::write(&dest_path, content)
        .map_err(|e| format!("Failed to write JavaScript file {}: {}", dest_path.display(), e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Copied,
        file_path: format!("assets/js/{}", rel_path.display()),
        description: format!("Copied JavaScript file from {}", file_path.display()),
    });
    
    Ok(())
}

fn create_default_scripts(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a minimal script.js file
    let script_js_content = r#"// Main JavaScript file for the Jekyll site
// This is a minimal script based on the Slate API documentation site

document.addEventListener('DOMContentLoaded', function() {
  // Language selector functionality for code examples
  const languageTabs = document.querySelectorAll('.lang-selector a');
  
  languageTabs.forEach(tab => {
    tab.addEventListener('click', function(e) {
      e.preventDefault();
      
      // Set active class on the selected language tab
      languageTabs.forEach(t => t.classList.remove('active'));
      this.classList.add('active');
      
      // Show code blocks for the selected language
      const language = this.getAttribute('data-language-name');
      const codeBlocks = document.querySelectorAll('pre code');
      
      codeBlocks.forEach(block => {
        const parent = block.parentElement;
        if (block.classList.contains(language)) {
          parent.style.display = 'block';
        } else {
          parent.style.display = 'none';
        }
      });
    });
  });
  
  // Activate the first language tab by default
  if (languageTabs.length > 0) {
    languageTabs[0].click();
  }
});
"#;

    let script_path = dest_dir.join("script.js");
    fs::write(&script_path, script_js_content)
        .map_err(|e| format!("Failed to create script.js: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "assets/js/script.js".to_string(),
        description: "Created default JavaScript file".to_string(),
    });

    Ok(())
}

fn is_javascript_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "js" || ext_str == "mjs" || ext_str == "jsx"
    } else {
        false
    }
} 