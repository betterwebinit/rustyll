use std::path::Path;
use std::fs;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};
use walkdir::WalkDir;

pub(super) fn migrate_helpers(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Jigsaw helpers...");
    }
    
    // In Jigsaw, helpers are typically PHP functions defined in bootstrap.php
    // or in helper files in the source directory
    
    // Create the _plugins directory for Jekyll
    let dest_plugins_dir = dest_dir.join("_plugins");
    create_dir_if_not_exists(&dest_plugins_dir)?;
    
    // Check for bootstrap.php
    let bootstrap_path = source_dir.join("bootstrap.php");
    if bootstrap_path.exists() {
        migrate_bootstrap_helpers(&bootstrap_path, &dest_plugins_dir, result)?;
    }
    
    // Check for other helper files
    let helper_dirs = [
        source_dir.join("source").join("_helpers"),
        source_dir.join("helpers"),
        source_dir.join("source").join("_functions"),
    ];
    
    for helper_dir in helper_dirs.iter() {
        if !helper_dir.exists() {
            continue;
        }
        
        let helper_files = WalkDir::new(helper_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "php"));
        
        for entry in helper_files {
            migrate_helper_file(entry.path(), &dest_plugins_dir, result)?;
        }
    }
    
    // Create a Liquid tag to replace common Jigsaw helpers
    create_common_helpers_plugin(&dest_plugins_dir, result)?;
    
    Ok(())
}

fn migrate_bootstrap_helpers(
    bootstrap_path: &Path,
    dest_plugins_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let bootstrap_content = fs::read_to_string(bootstrap_path)
        .map_err(|e| format!("Failed to read bootstrap.php: {}", e))?;
    
    // Extract helper functions from bootstrap.php
    let helpers = extract_php_functions(&bootstrap_content);
    
    if !helpers.is_empty() {
        // Create a Ruby plugin file with the extracted helpers
        let plugin_path = dest_plugins_dir.join("jigsaw_helpers.rb");
        let mut plugin_content = String::from("# Converted helpers from Jigsaw bootstrap.php\n\n");
        plugin_content.push_str("module JigsawHelpers\n");
        
        for (fn_name, _) in helpers {
            // Convert PHP function name to Ruby method
            let ruby_method = fn_name.replace("_", "_").to_lowercase();
            
            plugin_content.push_str(&format!("  # Placeholder for PHP function '{}'\n", fn_name));
            plugin_content.push_str(&format!("  def {}_helper(input)\n", ruby_method));
            plugin_content.push_str("    # This is a placeholder for a PHP helper function\n");
            plugin_content.push_str("    # Manual implementation may be required\n");
            plugin_content.push_str("    input\n");
            plugin_content.push_str("  end\n\n");
        }
        
        plugin_content.push_str("end\n\n");
        plugin_content.push_str("Liquid::Template.register_filter(JigsawHelpers)\n");
        
        fs::write(&plugin_path, plugin_content)
            .map_err(|e| format!("Failed to write helper plugin: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_plugins/jigsaw_helpers.rb".into(),
            change_type: ChangeType::Created,
            description: "Created Ruby plugin for Jigsaw helpers".into(),
        });
        
        // Add a note in the warnings
        result.warnings.push(
            "Helper functions from bootstrap.php have been converted to placeholder Ruby methods. Manual implementation required.".into()
        );
    }
    
    Ok(())
}

fn migrate_helper_file(
    helper_path: &Path,
    dest_plugins_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let helper_content = fs::read_to_string(helper_path)
        .map_err(|e| format!("Failed to read helper file {}: {}", helper_path.display(), e))?;
    
    // Extract the filename without extension
    let file_name = helper_path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Create a Ruby plugin file
    let plugin_path = dest_plugins_dir.join(format!("{}.rb", file_name));
    let mut plugin_content = format!("# Converted from Jigsaw helper: {}\n\n", helper_path.display());
    plugin_content.push_str(&format!("module {}\n", to_pascal_case(&file_name)));
    
    // Extract helper functions
    let helpers = extract_php_functions(&helper_content);
    
    for (fn_name, _) in helpers {
        let ruby_method = fn_name.replace("_", "_").to_lowercase();
        
        plugin_content.push_str(&format!("  # Placeholder for PHP function '{}'\n", fn_name));
        plugin_content.push_str(&format!("  def {}_helper(input)\n", ruby_method));
        plugin_content.push_str("    # This is a placeholder for a PHP helper function\n");
        plugin_content.push_str("    # Manual implementation may be required\n");
        plugin_content.push_str("    input\n");
        plugin_content.push_str("  end\n\n");
    }
    
    plugin_content.push_str("end\n\n");
    plugin_content.push_str(&format!("Liquid::Template.register_filter({})\n", to_pascal_case(&file_name)));
    
    fs::write(&plugin_path, plugin_content)
        .map_err(|e| format!("Failed to write helper plugin: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: format!("_plugins/{}.rb", file_name),
        change_type: ChangeType::Created,
        description: format!("Created Ruby plugin for Jigsaw helper '{}'", file_name),
    });
    
    Ok(())
}

fn create_common_helpers_plugin(
    dest_plugins_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a plugin for common Jigsaw helpers like url(), mix(), etc.
    let plugin_path = dest_plugins_dir.join("common_jigsaw_helpers.rb");
    let plugin_content = r#"# Common Jigsaw helper functions converted to Liquid
module CommonJigsawHelpers
  # Replacement for Jigsaw's url() helper
  def url(input)
    input = input.to_s
    input = "/#{input}" unless input.start_with?('/')
    input
  end

  # Replacement for Jigsaw's mix() helper used for asset versioning
  def mix(input)
    # In Jekyll, we could use `relative_url` filter
    # This is a simple implementation
    input = input.to_s
    input = "/#{input}" unless input.start_with?('/')
    input
  end
  
  # Add more common helpers as needed
end

Liquid::Template.register_filter(CommonJigsawHelpers)

# Create custom tags for more complex helpers
class JigsawUrlTag < Liquid::Tag
  def initialize(tag_name, text, tokens)
    super
    @text = text.strip
  end

  def render(context)
    # Simple implementation of the url() Jigsaw helper
    url = @text.gsub(/["']/, '')
    url = "/#{url}" unless url.start_with?('/')
    url
  end
end

Liquid::Template.register_tag('jigsaw_url', JigsawUrlTag)
"#;
    
    fs::write(&plugin_path, plugin_content)
        .map_err(|e| format!("Failed to write common helpers plugin: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_plugins/common_jigsaw_helpers.rb".into(),
        change_type: ChangeType::Created,
        description: "Created Ruby plugin for common Jigsaw helpers".into(),
    });
    
    Ok(())
}

// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

// Extract function names and bodies from PHP code
fn extract_php_functions(php_content: &str) -> Vec<(String, String)> {
    let mut functions = Vec::new();
    let fn_regex = regex::Regex::new(r"function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*\{((?:[^{}]|(?:\{(?:[^{}]|(?:\{[^{}]*\}))*\}))*)\}").ok();
    
    if let Some(regex) = fn_regex {
        for cap in regex.captures_iter(php_content) {
            if let (Some(name), Some(params), Some(body)) = (cap.get(1), cap.get(2), cap.get(3)) {
                functions.push((name.as_str().to_string(), body.as_str().to_string()));
            }
        }
    }
    
    functions
} 