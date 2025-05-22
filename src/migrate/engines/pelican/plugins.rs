use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_plugins(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Pelican plugins...");
    }

    // Find plugins used in the Pelican site
    let plugins = find_used_plugins(source_dir);
    
    if plugins.is_empty() {
        if verbose {
            log::info!("No Pelican plugins detected.");
        }
        return Ok(());
    }
    
    if verbose {
        let plugin_list = plugins.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ");
        log::info!("Found Pelican plugins: {}", plugin_list);
    }
    
    // Create destination plugin directory (/_plugins for Jekyll)
    let dest_plugins_dir = dest_dir.join("_plugins");
    create_dir_if_not_exists(&dest_plugins_dir)?;
    
    // Create Gemfile additions for Jekyll plugins
    let gemfile_path = dest_dir.join("Gemfile");
    update_gemfile_with_plugins(&gemfile_path, &plugins, result)?;
    
    // Generate Jekyll plugin equivalents for Pelican plugins
    for plugin in &plugins {
        generate_jekyll_plugin_equivalent(plugin, &dest_plugins_dir, result)?;
    }
    
    // Add plugin configuration to _config.yml
    let config_path = dest_dir.join("_config.yml");
    update_config_with_plugins(&config_path, &plugins, result)?;
    
    Ok(())
}

fn find_used_plugins(source_dir: &Path) -> HashSet<String> {
    let mut plugins = HashSet::new();
    
    // Check pelicanconf.py for plugin settings
    let pelicanconf_path = source_dir.join("pelicanconf.py");
    if pelicanconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&pelicanconf_path) {
            // Look for PLUGINS list
            let plugins_regex = regex::Regex::new(r"PLUGINS\s*=\s*\[(.*?)\]").unwrap();
            if let Some(captures) = plugins_regex.captures(&content) {
                let plugins_str = &captures[1];
                
                // Extract individual plugin names
                let plugin_regex = regex::Regex::new(r#"["']([^"']+)["']"#).unwrap();
                for plugin_match in plugin_regex.captures_iter(plugins_str) {
                    plugins.insert(plugin_match[1].to_string());
                }
            }
        }
    }
    
    // Also check publishconf.py
    let publishconf_path = source_dir.join("publishconf.py");
    if publishconf_path.exists() {
        if let Ok(content) = fs::read_to_string(&publishconf_path) {
            // Look for PLUGINS list
            let plugins_regex = regex::Regex::new(r"PLUGINS\s*=\s*\[(.*?)\]").unwrap();
            if let Some(captures) = plugins_regex.captures(&content) {
                let plugins_str = &captures[1];
                
                // Extract individual plugin names
                let plugin_regex = regex::Regex::new(r#"["']([^"']+)["']"#).unwrap();
                for plugin_match in plugin_regex.captures_iter(plugins_str) {
                    plugins.insert(plugin_match[1].to_string());
                }
            }
        }
    }
    
    // Check for plugins directory in project
    let plugins_dir = source_dir.join("plugins");
    if plugins_dir.exists() && plugins_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name() {
                            plugins.insert(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    plugins
}

fn update_gemfile_with_plugins(
    gemfile_path: &Path,
    plugins: &HashSet<String>,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if !gemfile_path.exists() {
        return Ok(());
    }
    
    let mut gemfile_content = fs::read_to_string(gemfile_path)
        .map_err(|e| format!("Failed to read Gemfile: {}", e))?;
    
    // Map Pelican plugins to Jekyll gems
    let mut added_gems = Vec::new();
    
    for plugin in plugins {
        let (gem_name, gem_version) = map_plugin_to_gem(plugin);
        if !gem_name.is_empty() && !gemfile_content.contains(&gem_name) {
            gemfile_content.push_str(&format!("gem \"{}\", \"{}\"", gem_name, gem_version));
            added_gems.push(gem_name.to_string());
        }
    }
    
    if !added_gems.is_empty() {
        // Write updated Gemfile
        fs::write(gemfile_path, gemfile_content)
            .map_err(|e| format!("Failed to write updated Gemfile: {}", e))?;
        
        result.changes.push(MigrationChange {
            change_type: ChangeType::Modified,
            file_path: "Gemfile".into(),
            description: format!("Added Jekyll plugin gems: {}", added_gems.join(", ")),
        });
    }
    
    Ok(())
}

fn map_plugin_to_gem(plugin_name: &str) -> (String, String) {
    // Map Pelican plugins to Jekyll gems
    match plugin_name {
        "sitemap" => ("jekyll-sitemap".to_string(), "~> 1.4.0".to_string()),
        "feed" | "atom" => ("jekyll-feed".to_string(), "~> 0.15.1".to_string()),
        "seo" | "meta_tags" => ("jekyll-seo-tag".to_string(), "~> 2.7.1".to_string()),
        "pagination" => ("jekyll-paginate".to_string(), "~> 1.1.0".to_string()),
        "related_posts" => ("jekyll-related-posts".to_string(), "~> 1.0.0".to_string()),
        "toc" | "table_of_contents" => ("jekyll-toc".to_string(), "~> 0.17.0".to_string()),
        "archives" => ("jekyll-archives".to_string(), "~> 2.2.1".to_string()),
        "redirect" => ("jekyll-redirect-from".to_string(), "~> 0.16.0".to_string()),
        "i18n" | "i18n_subsites" => ("jekyll-polyglot".to_string(), "~> 1.5.0".to_string()),
        _ => ("".to_string(), "".to_string()), // No direct mapping
    }
}

fn generate_jekyll_plugin_equivalent(
    plugin_name: &str,
    dest_plugins_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // For plugins without a direct Jekyll gem equivalent, create a custom plugin
    
    match plugin_name {
        // Plugins with direct gem equivalents don't need custom plugins
        "sitemap" | "feed" | "atom" | "seo" | "meta_tags" | "pagination" | 
        "related_posts" | "toc" | "table_of_contents" | "archives" | "redirect" | 
        "i18n" | "i18n_subsites" => Ok(()),
        
        "tipue_search" => {
            // Create a simple search plugin for Jekyll
            let plugin_file = dest_plugins_dir.join("tipue_search.rb");
            let plugin_content = r#"# Jekyll search plugin (migrated from Pelican tipue_search)
# Creates a JSON index of site content for client-side search

Jekyll::Hooks.register :site, :post_write do |site|
  require 'json'
  
  # Index all pages and posts
  search_data = {}
  
  # Index posts
  site.posts.docs.each do |post|
    search_data[post.url] = {
      "title" => post.data["title"],
      "content" => post.content.gsub(/<.*?>/, ' ').gsub(/\s+/, ' ').strip,
      "url" => post.url,
      "date" => post.date.strftime('%Y-%m-%d')
    }
  end
  
  # Index pages
  site.pages.each do |page|
    next if page.data["layout"].nil? || page.data["title"].nil?
    
    search_data[page.url] = {
      "title" => page.data["title"],
      "content" => page.content.gsub(/<.*?>/, ' ').gsub(/\s+/, ' ').strip,
      "url" => page.url
    }
  end
  
  # Write the search index to a JSON file
  File.open(File.join(site.dest, "search.json"), 'w') do |f|
    f.write(search_data.to_json)
  end
end
"#;
            
            fs::write(&plugin_file, plugin_content)
                .map_err(|e| format!("Failed to write search plugin: {}", e))?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_plugins/{}", plugin_file.file_name().unwrap().to_string_lossy()),
                description: "Created search plugin (migrated from tipue_search)".into(),
            });
            
            Ok(())
        },
        
        "neighbors" | "related_posts" => {
            // Create a related posts plugin
            let plugin_file = dest_plugins_dir.join("related_posts.rb");
            let plugin_content = r#"# Related Posts Plugin (migrated from Pelican neighbors plugin)
# Adds prev_post and next_post to site.posts for navigation

module RelatedPosts
  class Generator < Jekyll::Generator
    def generate(site)
      site.posts.docs.each_with_index do |post, i|
        post.data["prev_post"] = site.posts.docs[i-1] if i > 0
        post.data["next_post"] = site.posts.docs[i+1] if i < site.posts.docs.size - 1
      end
    end
  end
end
"#;
            
            fs::write(&plugin_file, plugin_content)
                .map_err(|e| format!("Failed to write related posts plugin: {}", e))?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_plugins/{}", plugin_file.file_name().unwrap().to_string_lossy()),
                description: "Created related posts plugin (migrated from neighbors)".into(),
            });
            
            Ok(())
        },
        
        "series" => {
            // Create a series plugin to group related posts
            let plugin_file = dest_plugins_dir.join("series.rb");
            let plugin_content = r#"# Series Plugin (migrated from Pelican series plugin)
# Groups posts into series based on metadata

module Series
  class Generator < Jekyll::Generator
    def generate(site)
      # Group posts by series
      series = Hash.new { |h, k| h[k] = [] }
      
      site.posts.docs.each do |post|
        next unless post.data.has_key?("series")
        series_name = post.data["series"]
        series[series_name] << post
      end
      
      # Sort posts in each series by date
      series.each do |name, posts|
        sorted_posts = posts.sort_by { |p| p.date }
        
        # Add series information to each post
        sorted_posts.each_with_index do |post, i|
          post.data["series_index"] = i + 1
          post.data["series_total"] = sorted_posts.size
          post.data["series_next"] = sorted_posts[i+1] if i < sorted_posts.size - 1
          post.data["series_prev"] = sorted_posts[i-1] if i > 0
          post.data["series_all"] = sorted_posts
        end
      end
      
      # Add all series to site data
      site.data["series"] = series
    end
  end
end
"#;
            
            fs::write(&plugin_file, plugin_content)
                .map_err(|e| format!("Failed to write series plugin: {}", e))?;
            
            result.changes.push(MigrationChange {
                change_type: ChangeType::Created,
                file_path: format!("_plugins/{}", plugin_file.file_name().unwrap().to_string_lossy()),
                description: "Created series plugin (migrated from series)".into(),
            });
            
            Ok(())
        },
        
        // Add more custom plugins as needed
        
        _ => {
            // For plugins without specific implementation, add a note
            result.warnings.push(format!("Pelican plugin '{}' has no direct Jekyll equivalent. Manual implementation may be required.", plugin_name));
            Ok(())
        }
    }
}

fn update_config_with_plugins(
    config_path: &Path,
    plugins: &HashSet<String>,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }
    
    let mut config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read _config.yml: {}", e))?;
    
    // Check if plugins section already exists
    let has_plugins_section = config_content.contains("\nplugins:") || config_content.contains("\nplugins:");
    
    // Add Jekyll plugin configuration
    let mut jekyll_plugins = Vec::new();
    
    for plugin in plugins {
        let gem_name = map_plugin_to_gem(plugin).0;
        if !gem_name.is_empty() && !config_content.contains(&gem_name) {
            jekyll_plugins.push(gem_name);
        }
    }
    
    if !jekyll_plugins.is_empty() {
        if has_plugins_section {
            // Add to existing plugins section
            for plugin in &jekyll_plugins {
                if !config_content.contains(&format!("  - {}", plugin)) {
                    // Find the end of the plugins section
                    if let Some(pos) = config_content.find("\nplugins:") {
                        // Find where to insert the new plugin
                        let mut insert_pos = pos + "\nplugins:".len();
                        while insert_pos < config_content.len() {
                            if config_content[insert_pos..].starts_with("\n") &&
                               !config_content[insert_pos+1..].starts_with("  -") {
                                break;
                            }
                            insert_pos += 1;
                        }
                        
                        let plugin_line = format!("\n  - {}", plugin);
                        config_content.insert_str(insert_pos, &plugin_line);
                    }
                }
            }
        } else {
            // Add new plugins section
            config_content.push_str("\n\n# Plugins\nplugins:\n");
            for plugin in &jekyll_plugins {
                config_content.push_str(&format!("  - {}\n", plugin));
            }
        }
        
        // Add plugin-specific configuration
        for plugin in &jekyll_plugins {
            match plugin.as_str() {
                "pagination" => {
                    if !config_content.contains("paginate:") {
                        config_content.push_str("\n# Pagination settings\npaginate: 10\npaginate_path: \"/page:num/\"\n");
                    }
                },
                "i18n" | "i18n_subsites" => {
                    if !config_content.contains("languages:") {
                        config_content.push_str("\n# Multilingual settings\nlanguages: [\"en\"]\ndefault_lang: \"en\"\nexclude_from_localization: [\"assets\", \"images\"]\nparallel_localization: true\n");
                    }
                },
                "archives" => {
                    if !config_content.contains("jekyll-archives:") {
                        config_content.push_str("\n# Archives settings\njekyll-archives:\n  enabled:\n    - year\n    - month\n    - tags\n    - categories\n  layouts:\n    year: archive-year\n    month: archive-month\n    tag: archive-tag\n    category: archive-category\n  permalinks:\n    year: '/archives/:year/'\n    month: '/archives/:year/:month/'\n    tag: '/tags/:name/'\n    category: '/categories/:name/'\n");
                    }
                },
                _ => {}
            }
        }
        
        // Write updated config
        fs::write(config_path, config_content)
            .map_err(|e| format!("Failed to write updated _config.yml: {}", e))?;
        
        let joined_plugins = jekyll_plugins.join(", ");
        result.changes.push(MigrationChange {
            change_type: ChangeType::Modified,
            file_path: "_config.yml".into(),
            description: format!("Added Jekyll plugins configuration: {}", joined_plugins),
        });
    }
    
    Ok(())
} 