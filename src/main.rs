use tokio;

// Module declarations
mod builder;
mod server;
mod config;
mod markdown;
mod directory;
mod collections;
mod front_matter;
mod liquid;
mod cli;
mod utils;
mod layout;

#[tokio::main]
async fn main() {
    // Run the CLI
    cli::run().await;
}
