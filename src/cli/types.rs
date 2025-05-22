use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Main CLI parser structure
#[derive(Parser)]
#[command(name = "rustyll")]
#[command(about = "Jekyll-compatible static site generator written in Rust", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Source directory (defaults to ./)
    #[arg(short, long, value_name = "DIR")]
    pub source: Option<PathBuf>,

    /// Destination directory (defaults to ./_site)
    #[arg(short, long, value_name = "DIR")]
    pub destination: Option<PathBuf>,

    /// Safe mode (defaults to false)
    #[arg(long, default_value_t = false)]
    pub safe: bool,

    /// Plugins directory (defaults to ./_plugins)
    #[arg(short, long, value_name = "PLUGINS_DIR")]
    pub plugins: Option<String>,

    /// Layouts directory (defaults to ./_layouts)
    #[arg(long, value_name = "DIR")]
    pub layouts: Option<PathBuf>,

    /// Show the full backtrace when an error occurs
    #[arg(short, long, default_value_t = false)]
    pub trace: bool,

    /// Enable verbose debugging
    #[arg(short = 'g', long, default_value_t = false)]
    pub debug: bool,
}

/// Subcommands for the CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Build your site
    #[command(alias = "b")]
    Build {
        /// Custom configuration file
        #[arg(long, value_name = "CONFIG_FILE")]
        config: Option<Vec<String>>,

        /// Watch for changes and rebuild
        #[arg(short = 'w', long, default_value_t = false)]
        watch: bool,

        /// Serve the website from the given base URL
        #[arg(short, long, value_name = "URL")]
        baseurl: Option<String>,

        /// Use LSI for improved related posts
        #[arg(long, default_value_t = false)]
        lsi: bool,

        /// Render posts in the _drafts folder
        #[arg(short = 'D', long, default_value_t = false)]
        drafts: bool,

        /// Render posts that were marked as unpublished
        #[arg(long, default_value_t = false)]
        unpublished: bool,

        /// Silence output
        #[arg(short, long, default_value_t = false)]
        quiet: bool,

        /// Print verbose output
        #[arg(short = 'V', long, default_value_t = false)]
        verbose: bool,

        /// Enable incremental rebuild
        #[arg(short = 'I', long, default_value_t = false)]
        incremental: bool,

        /// Source directory (defaults to ./)
        #[arg(short, long, value_name = "DIR")]
        source: Option<PathBuf>,

        /// Destination directory (defaults to ./_site)
        #[arg(short, long, value_name = "DIR")]
        destination: Option<PathBuf>,
        
        /// Enable debug logging
        #[arg(short = 'g', long, default_value_t = false)]
        debug: bool,
    },
    
    /// Serve your site locally
    #[command(alias = "s", alias = "server")]
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, value_name = "HOST", default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short = 'P', long, value_name = "PORT", default_value = "4000")]
        port: u16,

        /// Launch your site in a browser
        #[arg(short = 'o', long, default_value_t = false)]
        open_url: bool,

        /// Watch for changes and rebuild
        #[arg(short = 'w', long, default_value_t = true)]
        watch: bool,

        /// Use LiveReload to automatically refresh browsers
        #[arg(short = 'l', long, default_value_t = false)]
        livereload: bool,

        /// Print verbose output
        #[arg(short = 'V', long, default_value_t = false)]
        verbose: bool,
        
        /// Custom configuration file
        #[arg(long, value_name = "CONFIG_FILE")]
        config: Option<Vec<String>>,
        
        /// Render posts in the _drafts folder
        #[arg(short = 'D', long, default_value_t = false)]
        drafts: bool,

        /// Render posts that were marked as unpublished
        #[arg(long, default_value_t = false)]
        unpublished: bool,
        
        /// Source directory (defaults to ./)
        #[arg(short, long, value_name = "DIR")]
        source: Option<PathBuf>,

        /// Destination directory (defaults to ./_site)
        #[arg(short, long, value_name = "DIR")]
        destination: Option<PathBuf>,

        /// Serve the website from the given base URL
        #[arg(short, long, value_name = "URL")]
        baseurl: Option<String>,
    },
    
    /// Clean the site (removes site output and metadata file) without building
    Clean {},
    
    /// Generate a comprehensive report of your site
    #[command(alias = "r")]
    Report {
        /// Source directory (usually _site directory)
        #[arg(short = 's', long, value_name = "DIR")]
        source: Option<PathBuf>,
        
        /// Print verbose output with detailed analysis
        #[arg(short = 'v', long, default_value_t = false)]
        verbose: bool,
        
        /// Display report in console instead of generating an HTML file
        #[arg(short = 'c', long, default_value_t = false)]
        console: bool,
        
        /// Output file path for the HTML report
        #[arg(short = 'o', long, value_name = "FILE", default_value = "./rustyll-report.html")]
        output: PathBuf,
    },
    
    /// Migrate a site from another static site generator to Rustyll
    #[command(alias = "m")]
    Migrate {
        /// Source directory containing the site to be migrated
        #[arg(short, long, value_name = "DIR")]
        source: Option<PathBuf>,
        
        /// Destination directory for the migrated site
        #[arg(short, long, value_name = "DIR")]
        destination: Option<PathBuf>,
        
        /// Source engine to migrate from (e.g., jekyll, hugo, etc.)
        #[arg(short = 'e', long, value_name = "ENGINE")]
        engine: String,
        
        /// Print verbose output during migration
        #[arg(short = 'v', long, default_value_t = false)]
        verbose: bool,
        
        /// Clean the destination directory before migration
        #[arg(short = 'c', long, default_value_t = false)]
        clean: bool,
    },
    
    /// Creates a new Rustyll site scaffold in PATH
    #[command(alias = "n")]
    New {
        /// Path where the new site will be created
        path: PathBuf,
        
        /// Force creation even if PATH already exists
        #[arg(long, default_value_t = false)]
        force: bool,
        
        /// Creates scaffolding but with empty files
        #[arg(long, default_value_t = false)]
        blank: bool,
        
        /// Skip 'bundle install'
        #[arg(long, default_value_t = false)]
        skip_bundle: bool,
    },
} 