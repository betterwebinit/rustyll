use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, copy_file};

impl super::GitbookMigrator {
    pub(super) fn migrate_styles_and_plugins(&self, source_dir: &Path, dest_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // In GitBook, styles may be in styles/, and plugins in plugins/
        // We need to migrate these to Jekyll/Rustyll structure
        
        // Create destination asset directories
        let dest_assets_dir = dest_dir.join("assets");
        create_dir_if_not_exists(&dest_assets_dir)?;
        
        let dest_css_dir = dest_assets_dir.join("css");
        create_dir_if_not_exists(&dest_css_dir)?;
        
        let dest_js_dir = dest_assets_dir.join("js");
        create_dir_if_not_exists(&dest_js_dir)?;
        
        // Migrate styles
        self.migrate_styles(source_dir, &dest_css_dir, verbose, result)?;
        
        // Migrate plugins
        self.migrate_plugins(source_dir, &dest_assets_dir, &dest_js_dir, verbose, result)?;
        
        Ok(())
    }
    
    fn migrate_styles(&self, source_dir: &Path, dest_css_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for styles directory
        let styles_dir = source_dir.join("styles");
        
        if styles_dir.exists() && styles_dir.is_dir() {
            if verbose {
                log::info!("Found styles directory, migrating styles");
            }
            
            // Copy CSS files
            for entry in WalkDir::new(&styles_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let extension = file_path.extension().map(|ext| ext.to_string_lossy().to_lowercase());
                    
                    if let Some(ext) = extension {
                        if ext == "css" || ext == "scss" || ext == "less" {
                            // Process CSS file
                            self.migrate_style_file(file_path, &styles_dir, dest_css_dir, result)?;
                        }
                    }
                }
            }
        } else {
            // Check for possible theme assets
            let theme_dir = source_dir.join("theme");
            if theme_dir.exists() && theme_dir.is_dir() {
                let theme_assets = theme_dir.join("assets");
                
                if theme_assets.exists() && theme_assets.is_dir() {
                    // Check for theme CSS
                    let theme_css = theme_assets.join("css");
                    if theme_css.exists() && theme_css.is_dir() {
                        if verbose {
                            log::info!("Found theme CSS directory, migrating styles");
                        }
                        
                        for entry in WalkDir::new(&theme_css)
                            .into_iter()
                            .filter_map(Result::ok) {
                            
                            if entry.file_type().is_file() {
                                let file_path = entry.path();
                                // Copy CSS file directly
                                let dest_file = dest_css_dir.join(file_path.file_name().unwrap());
                                copy_file(file_path, &dest_file)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/css/{}", file_path.file_name().unwrap().to_string_lossy()),
                                    change_type: ChangeType::Copied,
                                    description: "CSS file migrated from theme".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Create basic CSS if none was found
        if dest_css_dir.read_dir().map_or(true, |mut r| r.next().is_none()) {
            if verbose {
                log::info!("No CSS files found, creating basic styles");
            }
            
            self.create_basic_styles(dest_css_dir, result)?;
        }
        
        Ok(())
    }
    
    fn migrate_style_file(&self, file_path: &Path, styles_dir: &Path, dest_css_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Get the relative path from styles directory
        let rel_path = file_path.strip_prefix(styles_dir)
            .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
            
        // Determine the destination file name
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        let dest_file_name = if file_name == "website.css" {
            // Main GitBook style, rename to style.css
            "style.css".to_string()
        } else {
            file_name.to_string()
        };
        
        // Create parent directories if needed
        if rel_path.parent().map_or(false, |p| !p.as_os_str().is_empty()) {
            let dest_subdir = dest_css_dir.join(rel_path.parent().unwrap());
            create_dir_if_not_exists(&dest_subdir)?;
            
            // Copy to subdirectory preserving structure
            let dest_file = dest_subdir.join(&dest_file_name);
            copy_file(file_path, &dest_file)?;
            
            let rel_asset_path = format!("assets/css/{}/{}", rel_path.parent().unwrap().to_string_lossy(), dest_file_name);
            result.changes.push(MigrationChange {
                file_path: rel_asset_path,
                change_type: ChangeType::Copied,
                description: "CSS file migrated from GitBook styles".to_string(),
            });
        } else {
            // Copy directly to css directory
            let dest_file = dest_css_dir.join(&dest_file_name);
            copy_file(file_path, &dest_file)?;
            
            result.changes.push(MigrationChange {
                file_path: format!("assets/css/{}", dest_file_name),
                change_type: ChangeType::Copied,
                description: "CSS file migrated from GitBook styles".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn migrate_plugins(&self, source_dir: &Path, dest_assets_dir: &Path, dest_js_dir: &Path, verbose: bool, result: &mut MigrationResult) -> Result<(), String> {
        // Check for plugins directory
        let plugins_dir = source_dir.join("plugins");
        
        if plugins_dir.exists() && plugins_dir.is_dir() {
            if verbose {
                log::info!("Found plugins directory");
            }
            
            // Create plugins directory in assets
            let dest_plugins_dir = dest_assets_dir.join("plugins");
            create_dir_if_not_exists(&dest_plugins_dir)?;
            
            // Copy plugin assets (we only care about client-side assets)
            for entry in WalkDir::new(&plugins_dir)
                .into_iter()
                .filter_map(Result::ok) {
                
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let extension = file_path.extension().map(|ext| ext.to_string_lossy().to_lowercase());
                    
                    if let Some(ext) = extension {
                        // Only copy assets that could be used client-side
                        if ext == "js" || ext == "css" || 
                           ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif" || ext == "svg" ||
                           ext == "woff" || ext == "woff2" || ext == "ttf" || ext == "eot" {
                            
                            // Determine the destination path preserving plugin structure
                            let rel_path = file_path.strip_prefix(&plugins_dir)
                                .map_err(|_| format!("Failed to get relative path for {}", file_path.display()))?;
                                
                            let dest_file = dest_plugins_dir.join(rel_path);
                            
                            // Create parent directories if needed
                            if let Some(parent) = dest_file.parent() {
                                create_dir_if_not_exists(parent)?;
                            }
                            
                            // Copy the file
                            copy_file(file_path, &dest_file)?;
                            
                            result.changes.push(MigrationChange {
                                file_path: format!("assets/plugins/{}", rel_path.to_string_lossy()),
                                change_type: ChangeType::Copied,
                                description: "Plugin asset migrated from GitBook".to_string(),
                            });
                        }
                    }
                }
            }
            
            // Create the plugins loader JavaScript file
            self.create_plugins_loader(&dest_js_dir, result)?;
        } else {
            // Check for theme JavaScript
            let theme_dir = source_dir.join("theme");
            if theme_dir.exists() && theme_dir.is_dir() {
                let theme_assets = theme_dir.join("assets");
                
                if theme_assets.exists() && theme_assets.is_dir() {
                    // Check for theme JS
                    let theme_js = theme_assets.join("js");
                    if theme_js.exists() && theme_js.is_dir() {
                        if verbose {
                            log::info!("Found theme JS directory, migrating scripts");
                        }
                        
                        for entry in WalkDir::new(&theme_js)
                            .into_iter()
                            .filter_map(Result::ok) {
                            
                            if entry.file_type().is_file() {
                                let file_path = entry.path();
                                // Copy JS file directly
                                let dest_file = dest_js_dir.join(file_path.file_name().unwrap());
                                copy_file(file_path, &dest_file)?;
                                
                                result.changes.push(MigrationChange {
                                    file_path: format!("assets/js/{}", file_path.file_name().unwrap().to_string_lossy()),
                                    change_type: ChangeType::Copied,
                                    description: "JavaScript file migrated from theme".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Create basic JS if none was found
        if dest_js_dir.read_dir().map_or(true, |mut r| r.next().is_none()) {
            if verbose {
                log::info!("No JavaScript files found, creating basic scripts");
            }
            
            self.create_basic_scripts(dest_js_dir, result)?;
        }
        
        Ok(())
    }
    
    fn create_plugins_loader(&self, dest_js_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a simple JavaScript file that loads plugins from _config.yml
        let js_content = r#"/**
 * GitBook Plugins Loader
 * This script loads plugins that were migrated from GitBook
 */

document.addEventListener('DOMContentLoaded', function() {
  // You can configure plugins in _config.yml under gitbook.plugins
  console.log('GitBook plugins loader initialized');
  
  // Add your custom plugin loading code here
  // In a full implementation, this would read from site.gitbook.plugins
  // and dynamically load the appropriate scripts
});
"#;

        fs::write(dest_js_dir.join("plugins.js"), js_content)
            .map_err(|e| format!("Failed to write plugins loader: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/js/plugins.js".to_string(),
            change_type: ChangeType::Created,
            description: "GitBook plugins loader created".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_styles(&self, dest_css_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a basic stylesheet with GitBook-like styles
        let css_content = r#"/**
 * Basic CSS for GitBook-style site migrated to Rustyll
 */

/* Reset & base styles */
* {
  box-sizing: border-box;
}

html, body {
  height: 100%;
}

body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #333;
  background-color: #fff;
}

a {
  color: #4183c4;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

/* GitBook layout */
.book {
  position: relative;
  width: 100%;
  height: 100%;
}

.book-summary {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  width: 300px;
  overflow-y: auto;
  background: #fafafa;
  border-right: 1px solid rgba(0,0,0,.07);
  z-index: 1;
  padding: 10px;
}

.book-body {
  position: absolute;
  top: 0;
  right: 0;
  left: 300px;
  bottom: 0;
  overflow-y: auto;
}

.page-wrapper {
  position: relative;
  outline: none;
}

.page-inner {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px 15px 40px 15px;
}

/* Navigation */
.book-summary ul.summary {
  list-style: none;
  margin: 0;
  padding: 0;
}

.book-summary ul.summary li {
  list-style: none;
}

.book-summary ul.summary li a {
  display: block;
  padding: 5px 10px;
  border-bottom: none;
  color: #333;
  background: 0 0;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
}

.book-summary ul.summary li.active>a {
  color: #008cff;
  background: 0 0;
  text-decoration: none;
}

.book-summary ul.summary li a:hover {
  color: #008cff;
  background: 0 0;
  text-decoration: none;
}

.book-summary ul.summary li.divider {
  height: 1px;
  margin: 7px 0;
  overflow: hidden;
  background: rgba(0,0,0,.07);
}

.book-summary ul.summary li.header {
  padding: 10px 15px;
  text-transform: uppercase;
  color: #939da3;
}

/* Content styling */
.markdown-section {
  padding: 10px 0;
}

.markdown-section h1, 
.markdown-section h2, 
.markdown-section h3, 
.markdown-section h4, 
.markdown-section h5, 
.markdown-section h6 {
  margin-top: 1.5em;
  margin-bottom: 0.8em;
  font-weight: 500;
}

.markdown-section h1 {
  font-size: 2em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
}

.markdown-section h2 {
  font-size: 1.75em;
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
}

.markdown-section h3 {
  font-size: 1.5em;
}

.markdown-section h4 {
  font-size: 1.25em;
}

.markdown-section h5 {
  font-size: 1em;
}

.markdown-section h6 {
  font-size: 0.85em;
  color: #6a737d;
}

.markdown-section p {
  margin-top: 0;
  margin-bottom: 16px;
}

.markdown-section code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(27,31,35,0.05);
  border-radius: 3px;
  font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
}

.markdown-section pre {
  padding: 16px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: #f6f8fa;
  border-radius: 3px;
}

.markdown-section pre code {
  display: block;
  padding: 0;
  margin: 0;
  overflow: visible;
  background-color: transparent;
  border: 0;
}

/* Responsive styles */
@media (max-width: 600px) {
  .book-summary {
    width: 100%;
    left: -100%;
    transition: left 250ms ease;
  }
  
  .book-body {
    left: 0;
    transition: left 250ms ease;
  }
  
  .book.with-summary .book-summary {
    left: 0;
  }
  
  .book.with-summary .book-body {
    left: 100%;
  }
}

@media (min-width: 600px) and (max-width: 1240px) {
  .book-summary {
    width: 200px;
  }
  
  .book-body {
    left: 200px;
  }
}
"#;

        fs::write(dest_css_dir.join("style.css"), css_content)
            .map_err(|e| format!("Failed to write basic stylesheet: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/css/style.css".to_string(),
            change_type: ChangeType::Created,
            description: "Basic GitBook-style CSS created".to_string(),
        });
        
        // Create a GitBook website-specific stylesheet
        let website_css_content = r#"/**
 * GitBook website-specific styles
 */

/* Header & footer */
.book-header {
  overflow: visible;
  height: 50px;
  padding: 0 8px;
  z-index: 2;
  font-size: 0.85em;
  color: #7e888b;
  background: 0 0;
}

.book-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 200;
  text-align: center;
  line-height: 50px;
  opacity: 0;
  padding-left: 200px;
  padding-right: 200px;
  transition: opacity .2s ease;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.book-header h1 a {
  color: inherit;
}

.book-header h1 i {
  display: none;
}

.book-header:hover h1 {
  opacity: 1;
}

.book-footer {
  margin-top: 40px;
  font-size: 0.85em;
  color: #999;
  text-align: center;
}

/* Home page specific */
.home-content {
  max-width: 800px;
  margin: 0 auto;
}

.pages-list {
  margin-top: 30px;
}

.pages-list h2 {
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
}

.pages-list ul {
  list-style: none;
  margin: 0;
  padding: 0;
}

.pages-list li {
  margin-bottom: 15px;
}

.pages-list li a {
  display: block;
  font-size: 1.2em;
  font-weight: 500;
}

.pages-list li p {
  margin-top: 5px;
  color: #666;
}
"#;

        fs::write(dest_css_dir.join("website.css"), website_css_content)
            .map_err(|e| format!("Failed to write website stylesheet: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/css/website.css".to_string(),
            change_type: ChangeType::Created,
            description: "GitBook website-specific CSS created".to_string(),
        });
        
        Ok(())
    }
    
    fn create_basic_scripts(&self, dest_js_dir: &Path, result: &mut MigrationResult) -> Result<(), String> {
        // Create a basic JavaScript file with GitBook-like functionality
        let js_content = r#"/**
 * Main JavaScript for GitBook-style site migrated to Rustyll
 */

document.addEventListener('DOMContentLoaded', function() {
  // Toggle sidebar on mobile
  var book = document.querySelector('.book');
  var toggleBtn = document.createElement('button');
  toggleBtn.classList.add('toggle-sidebar');
  toggleBtn.textContent = 'Menu';
  toggleBtn.style.position = 'fixed';
  toggleBtn.style.top = '10px';
  toggleBtn.style.left = '10px';
  toggleBtn.style.zIndex = '10';
  toggleBtn.style.display = 'none';
  
  toggleBtn.addEventListener('click', function() {
    book.classList.toggle('with-summary');
  });
  
  document.body.appendChild(toggleBtn);
  
  // Responsive behavior
  function checkWidth() {
    if (window.innerWidth <= 600) {
      toggleBtn.style.display = 'block';
    } else {
      toggleBtn.style.display = 'none';
      book.classList.remove('with-summary');
    }
  }
  
  window.addEventListener('resize', checkWidth);
  checkWidth();
  
  // Highlight current page in navigation
  var currentPath = window.location.pathname;
  var navLinks = document.querySelectorAll('.book-summary a');
  
  navLinks.forEach(function(link) {
    if (link.getAttribute('href') === currentPath) {
      link.parentElement.classList.add('active');
    }
  });
});
"#;

        fs::write(dest_js_dir.join("main.js"), js_content)
            .map_err(|e| format!("Failed to write main JavaScript: {}", e))?;
            
        result.changes.push(MigrationChange {
            file_path: "assets/js/main.js".to_string(),
            change_type: ChangeType::Created,
            description: "Basic GitBook-like JavaScript created".to_string(),
        });
        
        Ok(())
    }
} 