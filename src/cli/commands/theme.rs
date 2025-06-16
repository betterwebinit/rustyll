use crate::cli::types::{Commands, ThemeAction};

pub async fn handle_theme_command(command: &Commands) {
    if let Commands::Theme { action } = command {
        match action {
            ThemeAction::Install { name_or_url } => {
                println!("Theme install {}", name_or_url);
            }
            ThemeAction::List {} => {
                println!("Theme list");
            }
            ThemeAction::Apply { name } => {
                println!("Theme apply {}", name);
            }
        }
    }
}

