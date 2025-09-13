use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{info, debug, warn};
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
    
    // Normalize all hyphenated keys to underscore keys for compatibility
    normalize_hyphenated_keys(&mut data);
    
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

/// Recursively normalize hyphenated keys to underscore keys
fn normalize_hyphenated_keys(data: &mut DataCollection) {
    // Use a two-phase approach to avoid borrowing conflicts
    let mut nested_objects_to_normalize = Vec::new();
    let mut nested_arrays_to_normalize = Vec::new();
    let mut keys_to_normalize = Vec::new();
    
    // First phase: collect all keys and nested objects that need normalization
    for (key, value) in data.iter() {
        if key.contains('-') {
            keys_to_normalize.push(key.clone());
        }
        
        // Identify nested objects and arrays for processing
        match value {
            Value::Object(obj) => {
                let key_clone = key.clone();
                let mut nested_data: DataCollection = obj.iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                
                // Convert HashMap to liquid::Object before pushing
                let liquid_obj: liquid::Object = nested_data.iter()
                    .map(|(k, v)| (k.clone().into(), v.clone()))
                    .collect();
                    
                nested_objects_to_normalize.push((key_clone, Value::Object(liquid_obj)));
            },
            Value::Array(arr) => {
                let mut has_objects = false;
                let mut updated_array = Vec::new();
                
                for item in arr {
                    if let Value::Object(obj) = item {
                        has_objects = true;
                        let mut nested_data: DataCollection = obj.iter()
                            .map(|(k, v)| (k.to_string(), v.clone()))
                            .collect();
                        
                        // Recursively normalize the nested object
                        normalize_hyphenated_keys(&mut nested_data);
                        
                        // Add the normalized object to the new array
                        let normalized_obj: liquid::Object = nested_data.into_iter()
                            .map(|(k, v)| (k.into(), v))
                            .collect();
                            
                        updated_array.push(Value::Object(normalized_obj));
                    } else {
                        // Non-object items can be added as-is
                        updated_array.push(item.clone());
                    }
                }
                
                if has_objects {
                    // Instead of updating immediately, store for later update
                    nested_arrays_to_normalize.push((key.clone(), Value::Array(updated_array)));
                }
            },
            _ => {}
        }
    }
    
    // Process all nested objects
    for (key, new_value) in nested_objects_to_normalize {
        if let Value::Object(obj) = new_value {
            let mut nested_data: DataCollection = obj.iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            
            // Recursively normalize the nested object
            normalize_hyphenated_keys(&mut nested_data);
            
            // Convert HashMap to liquid::Object for the final insert
            let normalized_obj: liquid::Object = nested_data.into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect();
                
            // Update with the normalized object
            data.insert(key, Value::Object(normalized_obj));
        }
    }
    
    // Process all array updates
    for (key, array_value) in nested_arrays_to_normalize {
        data.insert(key, array_value);
    }
    
    // Second phase: normalize the collected keys
    for key in keys_to_normalize {
        if let Some(value) = data.remove(&key) {
            let normalized_key = key.replace('-', "_");
            warn!("Normalized data key: '{}' -> '{}'", key, normalized_key);
            
            // Insert with the normalized key, but also keep original for compatibility
            data.insert(normalized_key, value.clone());
            data.insert(key, value);
        }
    }
} 