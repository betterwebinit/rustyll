use clap_complete::{generate, Shell};
use clap::{CommandFactory};
use crate::cli::types::{Commands, Cli};

pub async fn handle_completions_command(command: &Commands) {
    if let Commands::Completions { shell } = command {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "rustyll", &mut std::io::stdout());
    }
}
