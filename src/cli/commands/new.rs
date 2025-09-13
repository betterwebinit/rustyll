use std::fs;
use std::io;
use std::path::Path;
use crate::cli::types::Commands;
use log::{info, error};

/// Handle the 'new' command to create a new Rustyll site
pub async fn handle_new_command(command: &Commands) {
    if let Commands::New { path, force, blank, skip_bundle } = command {
        info!("Creating new Rustyll site at {}", path.display());
        
        // Check if directory exists and is not empty
        if path.exists() {
            let is_empty = path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false);
            
            if !is_empty && !force {
                error!("Directory '{}' exists and is not empty. Use --force to overwrite.", path.display());
                return;
            }
            
            if *force {
                info!("Force option specified. Continuing despite existing directory.");
            }
        } else {
            // Create the directory if it doesn't exist
            match fs::create_dir_all(path) {
                Ok(_) => info!("Created directory: {}", path.display()),
                Err(e) => {
                    error!("Failed to create directory '{}': {}", path.display(), e);
                    return;
                }
            }
        }
        
        // Create the site scaffold
        if let Err(e) = create_site_scaffold(path, *blank) {
            error!("Failed to create site scaffold: {}", e);
            return;
        }
        
        // Skip bundle install if requested
        if !skip_bundle {
            info!("Skipping bundle install (not implemented)");
            // In a real implementation, this would run bundle install
        }
        
        info!("New Rustyll site created successfully at {}", path.display());
        info!("Run 'cd {}' and then 'rustyll serve' to start your site", path.display());
    }
}

/// Create the basic site scaffold files and directories
fn create_site_scaffold(site_path: &Path, blank: bool) -> io::Result<()> {
    // Create standard directories
    let dirs = vec![
        "_layouts", "_includes", "_posts", "_sass", "_data", "_drafts", 
        "assets/css", "assets/images", "assets/js"
    ];
    
    for dir in dirs {
        let dir_path = site_path.join(dir);
        fs::create_dir_all(&dir_path)?;
        info!("Created directory: {}", dir_path.display());
    }
    
    // Create basic config file
    let config_content = if blank {
        "# Site configuration\ntitle: \ndescription: \nbaseurl: \nurl: \n"
    } else {
        "# Site configuration\ntitle: Your awesome site\ndescription: A new Rustyll site\nbaseurl: \"\"\nurl: \"https://example.com\"\n\n# Build settings\nmarkdown: kramdown\ntheme: minima\nplugins:\n  - jekyll-feed\n"
    };
    
    let config_path = site_path.join("_config.yml");
    fs::write(&config_path, config_content)?;
    info!("Created config file: {}", config_path.display());
    
    // Create index page
    let index_content = if blank {
        "---\nlayout: home\n---\n"
    } else {
        "---\nlayout: home\n---\n\n# Welcome to Your New Rustyll Site\n\nThis is the front page of your new site. Edit this page to add your own content.\n"
    };
    
    let index_path = site_path.join("index.md");
    fs::write(&index_path, index_content)?;
    info!("Created index file: {}", index_path.display());
    
    // Create an example post if not blank
    if !blank {
        let date = chrono::Local::now().format("%Y-%m-%d");
        let post_filename = format!("{}-welcome-to-rustyll.md", date);
        let post_path = site_path.join("_posts").join(post_filename);
        
        let post_content = format!("---\nlayout: post\ntitle: \"Welcome to Rustyll!\"\ndate: {}\ncategories: jekyll update\n---\n\n# Welcome to Rustyll\n\nThis is your first post. Edit it to start blogging!\n", 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S %z"));
            
        fs::write(&post_path, post_content)?;
        info!("Created example post: {}", post_path.display());
    }
    
    // Create a basic Gemfile
    let gemfile_path = site_path.join("Gemfile");
    let gemfile_content = "source \"https://rubygems.org\"\n\n# If you want to use the same versions as GitHub Pages\n# gem \"github-pages\", group: :jekyll_plugins\n\n# If you have any plugins, put them here!\ngroup :jekyll_plugins do\n  gem \"jekyll-feed\"\nend\n";
    fs::write(&gemfile_path, gemfile_content)?;
    info!("Created Gemfile: {}", gemfile_path.display());
    
    // Create a .gitignore file
    let gitignore_path = site_path.join(".gitignore");
    let gitignore_content = "_site\n.sass-cache\n.jekyll-cache\n.jekyll-metadata\nvendor\n";
    fs::write(&gitignore_path, gitignore_content)?;
    info!("Created .gitignore: {}", gitignore_path.display());
    
    Ok(())
} 