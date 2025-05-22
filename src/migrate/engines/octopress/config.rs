use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use crate::migrate::{MigrationResult, MigrationChange, ChangeType, create_dir_if_not_exists};

pub(super) fn migrate_config(
    source_dir: &Path,
    dest_dir: &Path,
    verbose: bool,
    result: &mut MigrationResult,
) -> Result<(), String> {
    if verbose {
        log::info!("Migrating Octopress configuration...");
    }

    // Look for _config.yml in the source directory
    let config_path = source_dir.join("_config.yml");
    if !config_path.exists() {
        result.warnings.push("No _config.yml found in Octopress source.".into());
        return Ok(());
    }
    
    // Read the configuration file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read _config.yml: {}", e))?;
    
    // Convert Octopress config to Jekyll config
    let jekyll_config = convert_octopress_config(&config_content, result);
    
    // Write Jekyll config file
    let dest_config_path = dest_dir.join("_config.yml");
    fs::write(&dest_config_path, jekyll_config)
        .map_err(|e| format!("Failed to write Jekyll _config.yml: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Converted,
        file_path: "_config.yml".into(),
        description: "Converted Octopress configuration to Jekyll configuration".into(),
    });
    
    // Create Gemfile
    generate_gemfile(dest_dir, result)?;
    
    // Process Rakefile if needed
    process_rakefile(source_dir, dest_dir, result)?;
    
    Ok(())
}

fn convert_octopress_config(content: &str, result: &mut MigrationResult) -> String {
    let mut jekyll_config = String::new();
    let mut current_section = None;
    let mut in_commented_section = false;
    
    // Add header comment
    jekyll_config.push_str("# Jekyll configuration migrated from Octopress\n");
    jekyll_config.push_str("# Some settings may need manual adjustment\n\n");
    
    // Process each line
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines
        if trimmed.is_empty() {
            jekyll_config.push('\n');
            continue;
        }
        
        // Handle commented sections
        if trimmed.starts_with('#') {
            // Keep comments
            jekyll_config.push_str(line);
            jekyll_config.push('\n');
            
            // Detect commented section headers
            if trimmed.contains("####") {
                in_commented_section = !in_commented_section;
            }
            
            continue;
        }
        
        // Skip commented sections
        if in_commented_section {
            continue;
        }
        
        // Detect section headers (usually a comment followed by key settings)
        if trimmed.contains("####") || trimmed.contains("## ") {
            jekyll_config.push('\n');
            current_section = Some(trimmed.trim_start_matches('#').trim().to_lowercase());
            jekyll_config.push_str(line);
            jekyll_config.push('\n');
            continue;
        }
        
        // Process key-value pairs
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();
            
            // Skip Octopress-specific settings that don't apply to Jekyll
            if is_octopress_specific(key) {
                result.warnings.push(format!("Skipped Octopress-specific setting: {}", key));
                continue;
            }
            
            // Update deprecated settings
            if let Some(updated) = update_deprecated_setting(key, value) {
                jekyll_config.push_str(&updated);
                jekyll_config.push('\n');
                continue;
            }
            
            // Keep other settings as-is
            jekyll_config.push_str(line);
            jekyll_config.push('\n');
        } else {
            // Keep other lines as-is
            jekyll_config.push_str(line);
            jekyll_config.push('\n');
        }
    }
    
    // Add modern Jekyll settings
    jekyll_config.push_str("\n# Modern Jekyll settings\n");
    jekyll_config.push_str("plugins:\n");
    jekyll_config.push_str("  - jekyll-feed\n");
    jekyll_config.push_str("  - jekyll-sitemap\n");
    jekyll_config.push_str("  - jekyll-paginate\n");
    
    // Add default collections
    jekyll_config.push_str("\n# Collections\n");
    jekyll_config.push_str("collections:\n");
    jekyll_config.push_str("  pages:\n");
    jekyll_config.push_str("    output: true\n");
    jekyll_config.push_str("    permalink: /:path/\n");
    
    // Add default front matter
    jekyll_config.push_str("\n# Default front matter\n");
    jekyll_config.push_str("defaults:\n");
    jekyll_config.push_str("  - scope:\n");
    jekyll_config.push_str("      path: \"\"\n");
    jekyll_config.push_str("      type: \"posts\"\n");
    jekyll_config.push_str("    values:\n");
    jekyll_config.push_str("      layout: \"post\"\n");
    jekyll_config.push_str("  - scope:\n");
    jekyll_config.push_str("      path: \"_pages\"\n");
    jekyll_config.push_str("      type: \"pages\"\n");
    jekyll_config.push_str("    values:\n");
    jekyll_config.push_str("      layout: \"page\"\n");
    jekyll_config.push_str("  - scope:\n");
    jekyll_config.push_str("      path: \"\"\n");
    jekyll_config.push_str("    values:\n");
    jekyll_config.push_str("      layout: \"default\"\n");
    
    jekyll_config
}

fn is_octopress_specific(key: &str) -> bool {
    let octopress_keys = [
        "subscribe_rss",
        "subscribe_email",
        "email",
        "category_dir",
        "category_title_prefix",
        "simple_search",
        "default_asides",
        "sidebar",
        "github_user",
        "github_repo_count",
        "twitter_user",
        "twitter_tweet_count",
        "twitter_show_follower_count",
        "delicious_user",
        "disqus_short_name",
        "facebook_like",
        "google_plus_one",
        "octopress_version",
    ];
    
    octopress_keys.contains(&key)
}

fn update_deprecated_setting(key: &str, value: &str) -> Option<String> {
    match key {
        "pygments" => Some(format!("highlighter: rouge  # Updated from pygments: {}", value)),
        "markdown" if value.contains("rdiscount") => Some("markdown: kramdown  # Updated from rdiscount".to_string()),
        "markdown" if value.contains("maruku") => Some("markdown: kramdown  # Updated from maruku".to_string()),
        "paginate_path" if !value.contains(":num") => Some(format!("paginate_path: /page:num/  # Updated from {}", value)),
        _ => None,
    }
}

fn generate_gemfile(
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    let gemfile_content = r#"source "https://rubygems.org"

# Jekyll and plugins
gem "jekyll", "~> 4.2.0"
gem "jekyll-feed", "~> 0.15.1"
gem "jekyll-sitemap", "~> 1.4.0"
gem "jekyll-paginate", "~> 1.1.0"

# Theme - you can replace this with another theme
gem "minima", "~> 2.5.1"

# Markdown processor
gem "kramdown", "~> 2.3.1"
gem "kramdown-parser-gfm", "~> 1.1.0"

# Syntax highlighting
gem "rouge", "~> 3.26.0"

# Windows and JRuby does not include zoneinfo files, so bundle the tzinfo-data gem
platforms :mingw, :x64_mingw, :mswin, :jruby do
  gem "tzinfo", "~> 1.2"
  gem "tzinfo-data"
end

# Performance-booster for watching directories on Windows
gem "wdm", "~> 0.1.1", :platforms => [:mingw, :x64_mingw, :mswin]
"#;

    let gemfile_path = dest_dir.join("Gemfile");
    fs::write(&gemfile_path, gemfile_content)
        .map_err(|e| format!("Failed to write Gemfile: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "Gemfile".into(),
        description: "Created modern Gemfile for Jekyll site".into(),
    });
    
    Ok(())
}

fn process_rakefile(
    source_dir: &Path,
    dest_dir: &Path,
    result: &mut MigrationResult,
) -> Result<(), String> {
    // Look for Rakefile in the source directory
    let rakefile_path = source_dir.join("Rakefile");
    if !rakefile_path.exists() {
        return Ok(());
    }
    
    // Create a simplified Rakefile for Jekyll
    let jekyll_rakefile = r##"require 'rake'
require 'yaml'

desc "Build the site"
task :build do
  system "bundle exec jekyll build"
end

desc "Build and serve the site locally"
task :serve do
  system "bundle exec jekyll serve"
end

desc "Build the site for production"
task :production do
  system "JEKYLL_ENV=production bundle exec jekyll build"
end

desc "Create a new post"
task :post, [:title] do |t, args|
  title = args[:title] || "New Post"
  slug = title.downcase.strip.gsub(' ', '-').gsub(/[^\w-]/, '')
  filename = "_posts/#{Time.now.strftime('%Y-%m-%d')}-#{slug}.md"
  
  if File.exist?(filename)
    abort("#{filename} already exists!")
  end
  
  # Create post file
  File.open(filename, "w") do |f|
    f.puts "---"
    f.puts "layout: post"
    f.puts "title: \"#{title}\""
    f.puts "date: #{Time.now.strftime('%Y-%m-%d %H:%M:%S %z')}"
    f.puts "categories: "
    f.puts "tags: "
    f.puts "---"
    f.puts
  end
  
  puts "Created new post: #{filename}"
end

desc "Create a new page"
task :page, [:title, :path] do |t, args|
  title = args[:title] || "New Page"
  path = args[:path] || title.downcase.strip.gsub(' ', '-').gsub(/[^\w-]/, '')
  filename = "_pages/#{path}.md"
  
  if File.exist?(filename)
    abort("#{filename} already exists!")
  end
  
  # Create directory if it doesn't exist
  FileUtils.mkdir_p(File.dirname(filename))
  
  # Create page file
  File.open(filename, "w") do |f|
    f.puts "---"
    f.puts "layout: page"
    f.puts "title: \"#{title}\""
    f.puts "permalink: /#{path}/"
    f.puts "---"
    f.puts
  end
  
  puts "Created new page: #{filename}"
end
"##;

    let dest_rakefile_path = dest_dir.join("Rakefile");
    fs::write(&dest_rakefile_path, jekyll_rakefile)
        .map_err(|e| format!("Failed to write Rakefile: {}", e))?;
    
    result.changes.push(MigrationChange {
        change_type: ChangeType::Created,
        file_path: "Rakefile".into(),
        description: "Created simplified Rakefile for Jekyll site".into(),
    });
    
    Ok(())
} 