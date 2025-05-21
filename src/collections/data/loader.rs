use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::error::Error;
use log::{info, debug};
use liquid::model::Value;

use crate::config::Config;
use crate::builder::processor::{yaml_to_liquid, json_to_liquid};
use crate::collections::types::BoxResult;
use crate::collections::types::DataCollection;

/// Load data files from the _data directory
pub fn load_data_files(config: &Config) -> BoxResult<DataCollection> {
    info!("Loading data files...");
    let mut data = HashMap::new();
    
    let data_dir = config.source.join(&config.data_dir);
    if !data_dir.exists() {
        return Ok(data);
    }
    
    process_data_directory(&data_dir, &mut data, &data_dir)?;
    
    debug!("Loaded {} data files", data.len());
    
    Ok(data)
}

/// Process a directory of data files
fn process_data_directory(
    dir: &Path,
    data: &mut DataCollection,
    base_dir: &Path
) -> BoxResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Create nested data for subdirectory
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            let mut subdir_data = HashMap::new();
            process_data_directory(&path, &mut subdir_data, base_dir)?;
            
            // Add subdirectory data to parent
            let subdir_value = Value::Object(subdir_data.into_iter().map(|(k, v)| (k.into(), v)).collect());
            data.insert(dir_name, subdir_value);
        } else {
            // Process data file
            let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let extension = path.extension().unwrap_or_default().to_string_lossy();
            
            let file_data = match extension.as_ref() {
                "yml" | "yaml" => {
                    let content = fs::read_to_string(&path)?;
                    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
                    yaml_to_liquid(yaml)
                },
                "json" => {
                    let content = fs::read_to_string(&path)?;
                    let json: serde_json::Value = serde_json::from_str(&content)?;
                    json_to_liquid(json)
                },
                "csv" => {
                    let file = fs::File::open(&path)?;
                    let mut reader = csv::Reader::from_reader(file);
                    let records: Result<Vec<HashMap<String, String>>, _> = reader.deserialize().collect();
                    let records = records?;
                    
                    let array_values = records.into_iter()
                        .map(|record| {
                            let obj: liquid::Object = record.into_iter()
                                .map(|(k, v)| (k.into(), Value::scalar(v)))
                                .collect();
                            Value::Object(obj)
                        })
                        .collect();
                    
                    Value::Array(array_values)
                },
                "tsv" => {
                    let file = fs::File::open(&path)?;
                    let mut reader = csv::ReaderBuilder::new()
                        .delimiter(b'\t')
                        .from_reader(file);
                    let records: Result<Vec<HashMap<String, String>>, _> = reader.deserialize().collect();
                    let records = records?;
                    
                    let array_values = records.into_iter()
                        .map(|record| {
                            let obj: liquid::Object = record.into_iter()
                                .map(|(k, v)| (k.into(), Value::scalar(v)))
                                .collect();
                            Value::Object(obj)
                        })
                        .collect();
                    
                    Value::Array(array_values)
                },
                _ => continue, // Skip unknown formats
            };
            
            data.insert(file_stem, file_data);
        }
    }
    
    Ok(())
} 