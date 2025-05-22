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
        println!("Migrating Cobalt plugins to Jekyll...");
    }

    // Cobalt doesn't have a standard plugins directory structure like Jekyll
    // Instead, plugins are typically specified in _cobalt.yml
    let cobalt_config = source_dir.join("_cobalt.yml");
    
    // Create _plugins directory in Jekyll
    let jekyll_plugins_dir = dest_dir.join("_plugins");
    if !jekyll_plugins_dir.exists() {
        fs::create_dir_all(&jekyll_plugins_dir).map_err(|e| {
            format!("Failed to create _plugins directory: {}", e)
        })?;
    }
    
    // Create a readme to document plugin conversion
    let readme_path = jekyll_plugins_dir.join("README.md");
    let readme_content = "# Cobalt to Jekyll Plugin Migration\n\n\
        Cobalt uses a different plugin system than Jekyll. \
        This directory is prepared for you to add equivalent Jekyll plugins.\n\n\
        ## Common Differences\n\n\
        - Cobalt plugins are typically specified in _cobalt.yml\n\
        - Jekyll plugins go in _plugins/ directory as Ruby files\n\
        - You'll need to reimplement Cobalt plugin functionality using Jekyll's plugin system\n\n\
        ## Syntax Highlighting\n\n\
        If you were using syntax highlighting in Cobalt, consider using Jekyll's built-in Rouge highlighter:\n\n\
        ```yaml\n\
        # In _config.yml\n\
        markdown: kramdown\n\
        highlighter: rouge\n\
        ```\n";
    
    fs::write(&readme_path, readme_content)
        .map_err(|e| format!("Failed to write plugin readme: {}", e))?;
    
    if verbose {
        println!("Created plugin migration guide at {:?}", readme_path);
    }
    
    // If we have a Cobalt config file, extract plugin information
    if cobalt_config.exists() {
        extract_plugin_info(&cobalt_config, &jekyll_plugins_dir, verbose, result)?;
    }
    
    // Look for potential custom plugin directories
    let possible_plugin_dirs = [
        source_dir.join("plugins"),
        source_dir.join("_plugins"),
        source_dir.join("_cobalt/plugins"),
    ];
    
    for dir_path in possible_plugin_dirs.iter() {
        if dir_path.exists() && dir_path.is_dir() {
            if verbose {
                println!("Found potential plugin directory at {:?}", dir_path);
            }
            
            // Create a document describing what was found
            let plugin_info = jekyll_plugins_dir.join("cobalt_plugins_found.md");
            let info_content = format!(
                "# Cobalt Plugins Found\n\n\
                 Potential Cobalt plugins were found in: `{}`\n\n\
                 You'll need to manually convert these to Jekyll plugins.\n", 
                dir_path.display()
            );
            
            fs::write(&plugin_info, info_content)
                .map_err(|e| format!("Failed to write plugin info file: {}", e))?;
            
            if verbose {
                println!("Created plugin information file at {:?}", plugin_info);
            }
            
            // Create a sample Jekyll plugin for syntax highlighting
            create_sample_plugin(&jekyll_plugins_dir, verbose, result)?;
            
            break;
        }
    }
    
    if verbose {
        println!("Completed Cobalt plugins migration");
    }
    
    Ok(())
}

fn extract_plugin_info(
    config_file: &Path,
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        println!("Extracting plugin information from {:?}", config_file);
    }
    
    // Read the Cobalt config file
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read Cobalt config file: {}", e))?;
    
    // Look for plugin-related entries
    let mut plugin_info = String::from("# Cobalt Plugin Configuration\n\n");
    plugin_info.push_str("The following plugin configuration was found in _cobalt.yml:\n\n```yaml\n");
    
    let mut found_plugins = false;
    let mut in_plugin_section = false;
    
    for line in config_content.lines() {
        let trimmed = line.trim();
        
        if trimmed == "plugins:" {
            in_plugin_section = true;
            found_plugins = true;
            plugin_info.push_str(line);
            plugin_info.push('\n');
        } else if in_plugin_section {
            if !trimmed.is_empty() && (trimmed.starts_with('-') || trimmed.starts_with(' ')) {
                plugin_info.push_str(line);
                plugin_info.push('\n');
            } else {
                in_plugin_section = false;
            }
        } else if trimmed.contains("syntax-highlight") || trimmed.contains("highlighter") {
            // Also capture syntax highlighting configuration
            found_plugins = true;
            plugin_info.push_str(line);
            plugin_info.push('\n');
        }
    }
    
    plugin_info.push_str("```\n\n");
    
    // Add Jekyll equivalents information
    plugin_info.push_str("## Jekyll Equivalents\n\n");
    plugin_info.push_str("To enable similar functionality in Jekyll, add the following to your `_config.yml`:\n\n");
    plugin_info.push_str("```yaml\n");
    plugin_info.push_str("markdown: kramdown\n");
    plugin_info.push_str("highlighter: rouge\n");
    plugin_info.push_str("\n");
    plugin_info.push_str("# If you were using Sass in Cobalt:\n");
    plugin_info.push_str("sass:\n");
    plugin_info.push_str("  style: compressed\n");
    plugin_info.push_str("```\n");
    
    if found_plugins {
        // Write the plugin info file
        let plugin_info_file = plugins_dir.join("cobalt_plugin_info.md");
        fs::write(&plugin_info_file, plugin_info)
            .map_err(|e| format!("Failed to write plugin info file: {}", e))?;
        
        if verbose {
            println!("Created plugin information file at {:?}", plugin_info_file);
        }
    } else if verbose {
        println!("No plugin configuration found in Cobalt config");
    }
    
    Ok(())
}

fn create_sample_plugin(
    plugins_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Create a sample Jekyll plugin for syntax highlighting
    let sample_plugin_path = plugins_dir.join("highlight.rb");
    let sample_plugin = "# Sample Jekyll plugin for syntax highlighting\n\
        # This is a basic example of how to create a Jekyll plugin\n\
        \n\
        module Jekyll\n\
        \n\
        # Example Liquid tag for custom syntax highlighting\n\
        # Usage: {% highlight_code ruby %}\n\
        #   puts 'Hello, world!'\n\
        # {% endhighlight_code %}\n\
        class HighlightCodeTag < Liquid::Block\n\
        \n\
        def initialize(tag_name, markup, tokens)\n\
        super\n\
        @language = markup.strip\n\
        end\n\
        \n\
        def render(context)\n\
        code = super.to_s.strip\n\
        \n\
        output = \"<div class=\\\"highlight\\\"><pre><code class=\\\"language-#{@language}\\\">\"\n\
        output += code\n\
        output += \"</code></pre></div>\"\n\
        output\n\
        end\n\
        end\n\
        \n\
        end\n\
        \n\
        # Register the tag\n\
        Liquid::Template.register_tag('highlight_code', Jekyll::HighlightCodeTag)\n";
    
    fs::write(&sample_plugin_path, sample_plugin)
        .map_err(|e| format!("Failed to write sample plugin: {}", e))?;
    
    if verbose {
        println!("Created sample Jekyll plugin at {:?}", sample_plugin_path);
    }
    
    Ok(())
} 