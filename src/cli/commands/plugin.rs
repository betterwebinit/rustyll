use crate::cli::types::{Commands, PluginAction};

pub async fn handle_plugin_command(command: &Commands) {
    if let Commands::Plugin { action } = command {
        match action {
            PluginAction::Install { name } => {
                println!("Plugin install {}", name);
            }
            PluginAction::List {} => {
                println!("Plugin list");
            }
            PluginAction::Enable { name } => {
                println!("Plugin enable {}", name);
            }
        }
    }
}

