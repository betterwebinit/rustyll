use std::path::Path;
use std::fs;

use crate::migrate::MigrationResult;

pub fn migrate_plugins(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Bridgetown plugins to Jekyll...");
    }

    // Check for plugins directory in Bridgetown
    let plugins_dir = source_dir.join("plugins");
    if !plugins_dir.exists() || !plugins_dir.is_dir() {
        if verbose {
            println!("No plugins directory found. Skipping plugin migration.");
        }
        return Ok(());
    }

    // Create _plugins directory in Jekyll
    let jekyll_plugins_dir = dest_dir.join("_plugins");
    if !jekyll_plugins_dir.exists() {
        fs::create_dir_all(&jekyll_plugins_dir).map_err(|e| {
            format!("Failed to create _plugins directory: {}", e)
        })?;
    }

    // Copy Ruby plugins
    copy_ruby_plugins(&plugins_dir, &jekyll_plugins_dir, verbose, result)?;

    // Check for Gemfile and create a new one for Jekyll
    let gemfile_path = source_dir.join("Gemfile");
    if gemfile_path.exists() {
        migrate_gemfile(&gemfile_path, dest_dir, verbose, result)?;
    }

    // Create a note about plugin differences
    let plugin_note_path = jekyll_plugins_dir.join("README.md");
    let note_content = "# Migrated Plugins\n\n\
        These plugins were migrated from a Bridgetown site to Jekyll. \
        Note that there may be compatibility issues as Bridgetown and Jekyll \
        have different plugin APIs. You may need to modify these plugins to work properly in Jekyll.\n\n\
        ## Common Differences\n\n\
        - Bridgetown uses Ruby 2.7+ features, Jekyll plugins may need adjustments for compatibility\n\
        - Bridgetown Builder API is different from Jekyll's plugin system\n\
        - Bridgetown components don't have direct equivalents in Jekyll\n\
        - Liquid filter and tag implementations may need adjustments\n";

    fs::write(&plugin_note_path, note_content)
        .map_err(|e| format!("Failed to write plugin note: {}", e))?;

    if verbose {
        println!("Created plugin migration note at {:?}", plugin_note_path);
        println!("Completed Bridgetown plugins migration");
    }

    Ok(())
}

fn copy_ruby_plugins(
    source_plugins_dir: &Path,
    target_plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Copying Ruby plugins...");
    }

    if let Ok(entries) = fs::read_dir(source_plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let target_path = target_plugins_dir.join(&file_name);

            if path.is_file() {
                // Check if it's a Ruby file
                if let Some(ext) = path.extension() {
                    if ext == "rb" {
                        // Read and convert the plugin
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                // Convert Bridgetown plugin syntax to Jekyll
                                let converted = convert_bridgetown_plugin(&content);
                                
                                // Write converted plugin
                                if let Err(e) = fs::write(&target_path, converted) {
                                    result.errors.push(format!(
                                        "Failed to write plugin file {:?}: {}", target_path, e
                                    ));
                                } else if verbose {
                                    println!("Converted plugin {:?} to {:?}", path, target_path);
                                }
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to read plugin file {:?}: {}", path, e
                                ));
                            }
                        }
                    } else {
                        // Copy non-Ruby files as is
                        if let Err(e) = fs::copy(&path, &target_path) {
                            result.errors.push(format!(
                                "Failed to copy plugin file {:?}: {}", path, e
                            ));
                        } else if verbose {
                            println!("Copied plugin file {:?} to {:?}", path, target_path);
                        }
                    }
                }
            } else if path.is_dir() {
                // Create and process subdirectories
                if !target_path.exists() {
                    if let Err(e) = fs::create_dir_all(&target_path) {
                        result.errors.push(format!(
                            "Failed to create plugin directory {:?}: {}", target_path, e
                        ));
                        continue;
                    }
                }
                
                copy_ruby_plugins(&path, &target_path, verbose, result)?;
            }
        }
    }

    Ok(())
}

fn convert_bridgetown_plugin(content: &str) -> String {
    let mut converted = content.to_string();
    
    // Replace Bridgetown-specific module references
    converted = converted.replace("Bridgetown::", "Jekyll::");
    converted = converted.replace("class Builder < Bridgetown::Builder", "class Builder < Jekyll::Generator");
    converted = converted.replace("Bridgetown::Component", "Jekyll::Component");
    
    // Add warning comments
    converted = format!(
        "# CONVERTED FROM BRIDGETOWN PLUGIN\n\
         # WARNING: This plugin may require manual adjustments to work with Jekyll\n\
         # See _plugins/README.md for common differences\n\n\
         {}", 
        converted
    );
    
    converted
}

fn migrate_gemfile(
    source_gemfile: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Migrating Gemfile for Jekyll compatibility...");
    }
    
    // Read the original Gemfile
    let gemfile_content = fs::read_to_string(source_gemfile)
        .map_err(|e| format!("Failed to read Gemfile: {}", e))?;
    
    // Create a new Gemfile for Jekyll
    let jekyll_gemfile_path = dest_dir.join("Gemfile");
    
    // Convert Bridgetown gems to Jekyll equivalents
    let mut jekyll_gemfile = String::from("source \"https://rubygems.org\"\n\n");
    jekyll_gemfile.push_str("# Converted from Bridgetown to Jekyll\n");
    jekyll_gemfile.push_str("gem \"jekyll\"\n\n");
    jekyll_gemfile.push_str("# Original gems that might be compatible with Jekyll\n");
    
    // Extract gems from original Gemfile
    for line in gemfile_content.lines() {
        let trimmed = line.trim();
        
        // Skip Bridgetown gem
        if trimmed.starts_with("gem \"bridgetown\"") {
            continue;
        }
        
        // Include other gems, potentially with compatibility comments
        if trimmed.starts_with("gem ") {
            if trimmed.contains("bridgetown-") {
                // Comment out Bridgetown-specific gems
                jekyll_gemfile.push_str(&format!("# {} # Bridgetown-specific, may need a Jekyll alternative\n", trimmed));
            } else {
                // Include other gems
                jekyll_gemfile.push_str(&format!("{}\n", trimmed));
            }
        }
    }
    
    // Add Jekyll plugin group
    jekyll_gemfile.push_str("\n# Jekyll plugins\n");
    jekyll_gemfile.push_str("group :jekyll_plugins do\n");
    jekyll_gemfile.push_str("  # Add Jekyll plugins here\n");
    jekyll_gemfile.push_str("end\n");
    
    // Write the new Gemfile
    fs::write(&jekyll_gemfile_path, jekyll_gemfile)
        .map_err(|e| format!("Failed to write Jekyll Gemfile: {}", e))?;
    
    if verbose {
        println!("Created Jekyll Gemfile at {:?}", jekyll_gemfile_path);
    }
    
    Ok(())
} 