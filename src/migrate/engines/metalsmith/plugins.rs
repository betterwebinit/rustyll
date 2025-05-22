use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists, write_readme};

pub(super) fn migrate_plugins(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Metalsmith plugins...");
    }
    
    // Check for metalsmith.json or metalsmith.js configuration
    let metalsmith_json = source_dir.join("metalsmith.json");
    let metalsmith_js = source_dir.join("metalsmith.js");
    let has_plugin_config = metalsmith_json.exists() || metalsmith_js.exists();
    
    if !has_plugin_config {
        // Check for plugins in package.json
        let package_json = source_dir.join("package.json");
        if !package_json.exists() {
            result.warnings.push("Could not find Metalsmith configuration (metalsmith.json, metalsmith.js, or package.json)".into());
            return Ok(());
        }
    }
    
    // Create Jekyll plugins directory
    let plugins_dir = dest_dir.join("_plugins");
    create_dir_if_not_exists(&plugins_dir)?;
    
    // Create configuration file for Jekyll
    let config_file = dest_dir.join("_config.yml");
    let mut config_content = if config_file.exists() {
        fs::read_to_string(&config_file)
            .map_err(|e| format!("Failed to read _config.yml: {}", e))?
    } else {
        "# Site settings\ntitle: Migrated Metalsmith Site\n".to_string()
    };
    
    // Extract plugin configuration
    let mut plugins = HashMap::new();
    
    if metalsmith_json.exists() {
        extract_plugins_from_json(&metalsmith_json, &mut plugins)?;
    } else if metalsmith_js.exists() {
        result.warnings.push("Found metalsmith.js configuration. Manual review needed to extract plugin configuration.".into());
    }
    
    // Create Ruby equivalents for common Metalsmith plugins
    migrate_common_plugins(&plugins, &plugins_dir, result)?;
    
    // Update _config.yml with plugin-related configuration
    update_config_for_plugins(&plugins, &mut config_content);
    
    fs::write(&config_file, &config_content)
        .map_err(|e| format!("Failed to write _config.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        file_path: "_config.yml".into(),
        change_type: ChangeType::Modified,
        description: "Updated configuration for Metalsmith plugins".into(),
    });
    
    // Create README for plugins
    let plugins_readme = r#"# Migrated Metalsmith Plugins

This directory contains Ruby equivalents for Metalsmith plugins used in the original site.

## Original Plugins

The following Metalsmith plugins were used in the original site and have been converted to Jekyll equivalents:

- **collections**: Handled by Jekyll's built-in collections
- **markdown**: Handled by Jekyll's built-in Markdown processing
- **layouts**: Handled by Jekyll's built-in layouts
- **permalinks**: Handled by Jekyll's permalink configuration
- **excerpts**: Handled by Jekyll's excerpt functionality

## Custom Plugins

Some custom plugins may have been created to replicate functionality that doesn't have a direct equivalent in Jekyll.
Review these plugins and modify as needed.

## Additional Configuration

The `_config.yml` file has been updated with settings related to these plugins.
"#;
    
    write_readme(&plugins_dir, plugins_readme)?;
    
    result.changes.push(MigrationChange {
        file_path: "_plugins/README.md".into(),
        change_type: ChangeType::Created,
        description: "Created README for migrated plugins".into(),
    });
    
    Ok(())
}

fn extract_plugins_from_json(
    config_file: &Path,
    plugins: &mut HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    let config_content = fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read metalsmith.json: {}", e))?;
    
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse metalsmith.json: {}", e))?;
    
    if let Some(plugins_obj) = config.get("plugins").and_then(|p| p.as_object()) {
        for (name, config) in plugins_obj {
            plugins.insert(name.clone(), config.clone());
        }
    }
    
    Ok(())
}

fn migrate_common_plugins(
    plugins: &HashMap<String, serde_json::Value>,
    plugins_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Handle markdown plugin
    if plugins.contains_key("metalsmith-markdown") {
        // Jekyll handles markdown natively, but we can create a plugin for extra options
        let markdown_plugin = plugins_dir.join("markdown.rb");
        let markdown_content = r#"# Migrated from metalsmith-markdown
# Jekyll handles Markdown processing natively, this just adds extra options

module CustomMarkdown
  class CustomMarkdownConverter < Jekyll::Converters::Markdown
    def convert(content)
      # Add any custom markdown processing here
      super(content)
    end
  end
end

# Register the converter
Jekyll::Converters::Markdown.prepend CustomMarkdown
"#;
        
        fs::write(&markdown_plugin, markdown_content)
            .map_err(|e| format!("Failed to write markdown plugin: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_plugins/markdown.rb".into(),
            change_type: ChangeType::Created,
            description: "Created markdown plugin equivalent".into(),
        });
    }
    
    // Handle collections plugin
    if plugins.contains_key("metalsmith-collections") {
        // Collections are handled by Jekyll natively, but we can create helper methods
        let collections_plugin = plugins_dir.join("collections.rb");
        let collections_content = r#"# Migrated from metalsmith-collections
# Jekyll handles collections natively, this just adds helper methods

module MetalsmithCollectionsHelper
  # Add helper methods for collection manipulation similar to Metalsmith
  def sort_by_date(collection)
    collection.sort_by { |item| item.data['date'] || Time.now }.reverse
  end
  
  def limit(collection, count)
    collection.take(count)
  end
end

Liquid::Template.register_filter(MetalsmithCollectionsHelper)
"#;
        
        fs::write(&collections_plugin, collections_content)
            .map_err(|e| format!("Failed to write collections plugin: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_plugins/collections.rb".into(),
            change_type: ChangeType::Created,
            description: "Created collections plugin equivalent".into(),
        });
    }
    
    // Handle permalinks plugin
    if plugins.contains_key("metalsmith-permalinks") {
        // Jekyll handles permalinks natively via _config.yml
        // So nothing to do here but note it in the changes
        
        result.changes.push(MigrationChange {
            file_path: "_config.yml".into(),
            change_type: ChangeType::Modified,
            description: "Added permalink configuration equivalent to metalsmith-permalinks".into(),
        });
    }
    
    // Handle excerpts plugin
    if plugins.contains_key("metalsmith-excerpts") {
        // Jekyll handles excerpts natively, but we can create a plugin for custom behavior
        let excerpts_plugin = plugins_dir.join("excerpts.rb");
        let excerpts_content = r#"# Migrated from metalsmith-excerpts
# Jekyll has built-in excerpt functionality, this adds custom behavior

module CustomExcerpts
  class ExcerptGenerator < Jekyll::Generator
    def generate(site)
      site.posts.docs.each do |post|
        # Only generate excerpt if it doesn't already exist
        if post.data['excerpt'].nil? && !post.content.nil?
          # Default to first paragraph
          # This mimics metalsmith-excerpts behavior
          first_para = post.content.split(/\r?\n\r?\n/).first
          post.data['excerpt'] = first_para
        end
      end
    end
  end
end
"#;
        
        fs::write(&excerpts_plugin, excerpts_content)
            .map_err(|e| format!("Failed to write excerpts plugin: {}", e))?;
        
        result.changes.push(MigrationChange {
            file_path: "_plugins/excerpts.rb".into(),
            change_type: ChangeType::Created,
            description: "Created excerpts plugin equivalent".into(),
        });
    }
    
    // Handle drafts plugin
    if plugins.contains_key("metalsmith-drafts") {
        result.warnings.push("Metalsmith drafts plugin detected. Jekyll handles drafts natively in the _drafts folder.".into());
    }
    
    Ok(())
}

fn update_config_for_plugins(
    plugins: &HashMap<String, serde_json::Value>,
    config_content: &mut String,
) {
    // Add standard Jekyll configuration that corresponds to common Metalsmith plugins
    
    // Handle collections from metalsmith-collections
    if plugins.contains_key("metalsmith-collections") && !config_content.contains("collections:") {
        config_content.push_str("\n# Collections (migrated from metalsmith-collections)\ncollections:\n");
        
        if let Some(collections_config) = plugins.get("metalsmith-collections") {
            if let Some(obj) = collections_config.as_object() {
                for (name, _) in obj {
                    if name != "posts" { // posts are handled specially in Jekyll
                        config_content.push_str(&format!("  {}:\n    output: true\n", name));
                    }
                }
            }
        } else {
            // Default collections
            config_content.push_str("  posts:\n    output: true\n");
        }
    }
    
    // Handle permalinks from metalsmith-permalinks
    if plugins.contains_key("metalsmith-permalinks") && !config_content.contains("permalink:") {
        // Check for pattern in the permalinks plugin config
        if let Some(permalinks_config) = plugins.get("metalsmith-permalinks") {
            if let Some(pattern) = permalinks_config.get("pattern").and_then(|p| p.as_str()) {
                // Convert metalsmith pattern to Jekyll
                let jekyll_pattern = pattern
                    .replace(":title", ":title")
                    .replace(":date", ":year/:month/:day");
                
                config_content.push_str(&format!("\n# Permalinks (migrated from metalsmith-permalinks)\npermalink: {}\n", jekyll_pattern));
            } else {
                // Default permalink pattern
                config_content.push_str("\n# Permalinks (migrated from metalsmith-permalinks)\npermalink: /:categories/:year/:month/:day/:title/\n");
            }
        }
    }
    
    // Handle excerpts from metalsmith-excerpts
    if plugins.contains_key("metalsmith-excerpts") && !config_content.contains("excerpt_separator:") {
        config_content.push_str("\n# Excerpts (migrated from metalsmith-excerpts)\nexcerpt_separator: <!--more-->\n");
    }
    
    // Handle markdown from metalsmith-markdown
    if plugins.contains_key("metalsmith-markdown") {
        // Check if there's already a markdown configuration
        if !config_content.contains("markdown:") {
            config_content.push_str("\n# Markdown processing (migrated from metalsmith-markdown)\nmarkdown: kramdown\n");
            config_content.push_str("kramdown:\n  input: GFM\n  hard_wrap: false\n");
        }
    }
} 