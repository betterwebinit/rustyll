// Rustyll - A blazing fast, Jekyll-compatible static site generator written in Rust
// Main entry point for the Rustyll application
use tokio;

// Core modules
mod builder;     // Site building and processing
mod server;      // Development server
mod config;      // Configuration handling
mod markdown;    // Markdown processing
mod directory;   // Directory and file operations
mod collections; // Content collections
mod front_matter; // Front matter parsing and handling
mod liquid;      // Liquid templating
mod cli;         // Command-line interface
mod utils;       // Utility functions
mod layout;      // Layout handling
mod report;      // Site reporting and analytics
mod migrate;     // Migration tools
mod plugins;     // Plugin system (extensibility)

#[tokio::main]
async fn main() {
    // Print startup banner in debug mode
    #[cfg(debug_assertions)]
    print_banner();
    
    // Run the CLI and handle all commands
    cli::run().await;
}

/// Prints a welcome banner for debug builds
#[cfg(debug_assertions)]
fn print_banner() {
    println!("
    ╦═╗┬ ┬┌─┐┌┬┐┬ ┬┬  ┬  
    ╠╦╝│ │└─┐ │ └┬┘│  │  
    ╩╚═└─┘└─┘ ┴  ┴ ┴─┘┴─┘
    
    A blazing fast, Jekyll-compatible static site generator
    ");
} 