use std::path::Path;
use std::fs;
use walkdir::WalkDir;

use crate::migrate::{MigrationResult, copy_file};

pub fn migrate_assets(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Bridgetown assets to Jekyll...");
    }

    // Bridgetown uses frontend/javascript and frontend/styles for assets
    let frontend_dir = source_dir.join("frontend");
    if !frontend_dir.exists() || !frontend_dir.is_dir() {
        if verbose {
            println!("No frontend directory found. Skipping asset migration.");
        }
        return Ok(());
    }

    // Create assets directory in Jekyll
    let jekyll_assets_dir = dest_dir.join("assets");
    if !jekyll_assets_dir.exists() {
        fs::create_dir_all(&jekyll_assets_dir).map_err(|e| {
            format!("Failed to create assets directory: {}", e)
        })?;
    }

    // Process JavaScript files
    let js_dir = frontend_dir.join("javascript");
    if js_dir.exists() && js_dir.is_dir() {
        let target_js_dir = jekyll_assets_dir.join("js");
        if !target_js_dir.exists() {
            fs::create_dir_all(&target_js_dir).map_err(|e| {
                format!("Failed to create js directory: {}", e)
            })?;
        }
        
        if verbose {
            println!("Migrating JavaScript files from {:?} to {:?}", js_dir, target_js_dir);
        }
        
        migrate_js_files(&js_dir, &target_js_dir, verbose, result)?;
    }

    // Process CSS/SCSS files
    let styles_dir = frontend_dir.join("styles");
    if styles_dir.exists() && styles_dir.is_dir() {
        let target_css_dir = jekyll_assets_dir.join("css");
        if !target_css_dir.exists() {
            fs::create_dir_all(&target_css_dir).map_err(|e| {
                format!("Failed to create css directory: {}", e)
            })?;
        }
        
        if verbose {
            println!("Migrating CSS/SCSS files from {:?} to {:?}", styles_dir, target_css_dir);
        }
        
        migrate_css_files(&styles_dir, &target_css_dir, verbose, result)?;
    }

    // Process images if they exist in the frontend directory
    let images_dir = frontend_dir.join("images");
    if images_dir.exists() && images_dir.is_dir() {
        let target_images_dir = jekyll_assets_dir.join("images");
        if !target_images_dir.exists() {
            fs::create_dir_all(&target_images_dir).map_err(|e| {
                format!("Failed to create images directory: {}", e)
            })?;
        }
        
        if verbose {
            println!("Copying images from {:?} to {:?}", images_dir, target_images_dir);
        }
        
        copy_directory(&images_dir, &target_images_dir, verbose, result)?;
    }

    // Check for package.json and create a Jekyll-appropriate version
    let package_json_path = source_dir.join("package.json");
    if package_json_path.exists() {
        migrate_package_json(&package_json_path, dest_dir, verbose, result)?;
    }

    if verbose {
        println!("Completed Bridgetown assets migration");
    }

    Ok(())
}

fn migrate_js_files(
    source_js_dir: &Path,
    target_js_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Copy and convert JavaScript files
    for entry in WalkDir::new(source_js_dir).min_depth(1) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let relative_path = path.strip_prefix(source_js_dir).unwrap_or(path);
                let target_path = target_js_dir.join(relative_path);
                
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "js" || ext == "mjs" || ext == "jsx" || ext == "ts" || ext == "tsx" {
                            // Create parent directories if they don't exist
                            if let Some(parent) = target_path.parent() {
                                if !parent.exists() {
                                    if let Err(e) = fs::create_dir_all(parent) {
                                        result.errors.push(format!(
                                            "Failed to create directory {:?}: {}", parent, e
                                        ));
                                        continue;
                                    }
                                }
                            }
                            
                            // Check if it's an index.js file that may use Bridgetown-specific imports
                            if path.file_name().unwrap_or_default() == "index.js" {
                                match fs::read_to_string(path) {
                                    Ok(content) => {
                                        // Convert Bridgetown-specific imports
                                        let converted = convert_bridgetown_js(&content);
                                        
                                        if let Err(e) = fs::write(&target_path, converted) {
                                            result.errors.push(format!(
                                                "Failed to write JS file {:?}: {}", target_path, e
                                            ));
                                        } else if verbose {
                                            println!("Converted JS file {:?} to {:?}", path, target_path);
                                        }
                                    }
                                    Err(e) => {
                                        result.errors.push(format!(
                                            "Failed to read JS file {:?}: {}", path, e
                                        ));
                                    }
                                }
                            } else {
                                // Copy other JS files as is
                                if let Err(e) = copy_file(path, &target_path) {
                                    result.errors.push(format!(
                                        "Failed to copy JS file {:?} to {:?}: {}", path, target_path, e
                                    ));
                                } else if verbose {
                                    println!("Copied JS file {:?} to {:?}", path, target_path);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Error walking directory {:?}: {}", source_js_dir, e
                ));
            }
        }
    }
    
    Ok(())
}

fn migrate_css_files(
    source_css_dir: &Path,
    target_css_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Copy and convert CSS/SCSS files
    for entry in WalkDir::new(source_css_dir).min_depth(1) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let relative_path = path.strip_prefix(source_css_dir).unwrap_or(path);
                let target_path = target_css_dir.join(relative_path);
                
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "css" || ext == "scss" || ext == "sass" {
                            // Create parent directories if they don't exist
                            if let Some(parent) = target_path.parent() {
                                if !parent.exists() {
                                    if let Err(e) = fs::create_dir_all(parent) {
                                        result.errors.push(format!(
                                            "Failed to create directory {:?}: {}", parent, e
                                        ));
                                        continue;
                                    }
                                }
                            }
                            
                            // Copy the file
                            if let Err(e) = copy_file(path, &target_path) {
                                result.errors.push(format!(
                                    "Failed to copy CSS file {:?} to {:?}: {}", path, target_path, e
                                ));
                            } else if verbose {
                                println!("Copied CSS file {:?} to {:?}", path, target_path);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Error walking directory {:?}: {}", source_css_dir, e
                ));
            }
        }
    }
    
    // Create a main.scss file for Jekyll if not already present
    let main_scss = target_css_dir.join("main.scss");
    if !main_scss.exists() {
        let jekyll_scss_content = "---\n---\n\n\
            // Jekyll front matter required for SCSS processing\n\
            // Importing migrated styles\n\
            @import \"styles\";\n";
        
        if let Err(e) = fs::write(&main_scss, jekyll_scss_content) {
            result.errors.push(format!(
                "Failed to create main.scss for Jekyll: {}", e
            ));
        } else if verbose {
            println!("Created main.scss for Jekyll at {:?}", main_scss);
        }
    }
    
    Ok(())
}

fn copy_directory(
    source_dir: &Path,
    target_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    for entry in WalkDir::new(source_dir).min_depth(1) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let relative_path = path.strip_prefix(source_dir).unwrap_or(path);
                let target_path = target_dir.join(relative_path);
                
                if path.is_file() {
                    // Create parent directories if they don't exist
                    if let Some(parent) = target_path.parent() {
                        if !parent.exists() {
                            if let Err(e) = fs::create_dir_all(parent) {
                                result.errors.push(format!(
                                    "Failed to create directory {:?}: {}", parent, e
                                ));
                                continue;
                            }
                        }
                    }
                    
                    // Copy the file
                    if let Err(e) = copy_file(path, &target_path) {
                        result.errors.push(format!(
                            "Failed to copy file {:?} to {:?}: {}", path, target_path, e
                        ));
                    } else if verbose {
                        println!("Copied file {:?} to {:?}", path, target_path);
                    }
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Error walking directory {:?}: {}", source_dir, e
                ));
            }
        }
    }
    
    Ok(())
}

fn convert_bridgetown_js(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Replace Bridgetown-specific imports
    converted = converted.replace("import \"@bridgetown/core\"", "// import \"@bridgetown/core\" - removed for Jekyll");
    
    // Add warning comment
    converted = format!(
        "// CONVERTED FROM BRIDGETOWN JAVASCRIPT\n\
         // WARNING: This JS may require manual adjustments to work with Jekyll\n\n\
         {}", 
        converted
    );
    
    converted
}

fn migrate_package_json(
    source_file: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating package.json for Jekyll compatibility...");
    }
    
    // Create a simple package.json for Jekyll
    let jekyll_package_json = dest_dir.join("package.json");
    
    let package_json_content = r#"{
  "name": "jekyll-site",
  "version": "1.0.0",
  "description": "Migrated from Bridgetown to Jekyll",
  "main": "assets/js/index.js",
  "scripts": {
    "build": "webpack --mode production",
    "dev": "webpack --mode development --watch"
  },
  "dependencies": {},
  "devDependencies": {
    "webpack": "^5.0.0",
    "webpack-cli": "^4.0.0",
    "sass": "^1.0.0"
  }
}"#;
    
    fs::write(&jekyll_package_json, package_json_content)
        .map_err(|e| format!("Failed to write Jekyll package.json: {}", e))?;
    
    if verbose {
        println!("Created Jekyll package.json at {:?}", jekyll_package_json);
    }
    
    // Create a webpack.config.js file for Jekyll
    let webpack_config = dest_dir.join("webpack.config.js");
    
    let webpack_content = r#"const path = require("path");

module.exports = {
  entry: "./assets/js/index.js",
  output: {
    filename: "[name].js",
    path: path.resolve(__dirname, "assets/dist"),
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
      },
    ],
  },
};"#;
    
    fs::write(&webpack_config, webpack_content)
        .map_err(|e| format!("Failed to write webpack.config.js: {}", e))?;
    
    if verbose {
        println!("Created webpack.config.js at {:?}", webpack_config);
    }
    
    Ok(())
} 