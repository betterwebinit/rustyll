use crate::cli::types::{Commands, ConfigAction};
use crate::config;
use serde_yaml::Value;
use std::path::PathBuf;

pub async fn handle_config_command(command: &Commands) {
    if let Commands::Config { action } = command {
        match action {
            ConfigAction::Get { key } => {
                if let Ok(cfg) = config::load_config(PathBuf::from("."), None) {
                    let yaml = serde_yaml::to_value(&cfg).unwrap_or(Value::Null);
                    if let Some(v) = get_nested_value(&yaml, key) {
                        println!("{}", serde_yaml::to_string(v).unwrap_or_default());
                    } else {
                        println!("Key not found: {}", key);
                    }
                } else {
                    println!("Failed to load configuration");
                }
            }
            ConfigAction::Set { key, value } => {
                println!("Config set: {} = {}", key, value);
                // TODO implement writing back to config file
            }
            ConfigAction::List {} => {
                match config::load_config(PathBuf::from("."), None) {
                    Ok(cfg) => {
                        let yaml = serde_yaml::to_string(&cfg).unwrap_or_default();
                        println!("{}", yaml);
                    }
                    Err(_) => println!("Failed to load configuration"),
                }
            }
        }
    }
}

fn get_nested_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in key.split('.') {
        match current {
            Value::Mapping(map) => {
                let part_key = Value::String(part.to_string());
                current = map.get(&part_key)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

