use liquid::model::{Value as LiquidValue};
use serde_yaml::Value as YamlValue;

/// Convert YAML value to Liquid value
pub fn yaml_to_liquid(yaml: YamlValue) -> LiquidValue {
    match yaml {
        YamlValue::Null => LiquidValue::Nil,
        YamlValue::Bool(b) => LiquidValue::scalar(b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                LiquidValue::scalar(i)
            } else if let Some(f) = n.as_f64() {
                LiquidValue::scalar(f)
            } else {
                // Default to string
                LiquidValue::scalar(n.to_string())
            }
        },
        YamlValue::String(s) => LiquidValue::scalar(s),
        YamlValue::Sequence(seq) => {
            let values: Vec<LiquidValue> = seq.into_iter()
                .map(yaml_to_liquid)
                .collect();
            LiquidValue::Array(values)
        },
        YamlValue::Mapping(map) => {
            let mut obj = liquid::Object::new();
            for (k, v) in map {
                if let YamlValue::String(key) = k {
                    obj.insert(key.into(), yaml_to_liquid(v));
                } else {
                    // Use string representation of key
                    let key_str = format!("{:?}", k);
                    obj.insert(key_str.into(), yaml_to_liquid(v));
                }
            }
            LiquidValue::Object(obj)
        },
        YamlValue::Tagged(tagged) => yaml_to_liquid(tagged.value),
    }
}

/// Convert JSON value to Liquid value
pub fn json_to_liquid(json: serde_json::Value) -> LiquidValue {
    match json {
        serde_json::Value::Null => LiquidValue::Nil,
        serde_json::Value::Bool(b) => LiquidValue::scalar(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                LiquidValue::scalar(i)
            } else if let Some(f) = n.as_f64() {
                LiquidValue::scalar(f)
            } else {
                // Default to string
                LiquidValue::scalar(n.to_string())
            }
        },
        serde_json::Value::String(s) => LiquidValue::scalar(s),
        serde_json::Value::Array(arr) => {
            let values: Vec<LiquidValue> = arr.into_iter()
                .map(json_to_liquid)
                .collect();
            LiquidValue::Array(values)
        },
        serde_json::Value::Object(obj) => {
            let mut liquid_obj = liquid::Object::new();
            for (k, v) in obj {
                liquid_obj.insert(k.into(), json_to_liquid(v));
            }
            LiquidValue::Object(liquid_obj)
        },
    }
}

/// Convert YAML values to their equivalent string representations
pub fn yaml_to_any(yaml: YamlValue) -> Option<String> {
    match yaml {
        YamlValue::String(s) => Some(s),
        YamlValue::Number(n) => Some(n.to_string()),
        YamlValue::Bool(b) => Some(b.to_string()),
        YamlValue::Sequence(seq) => {
            let items: Vec<String> = seq.into_iter()
                .filter_map(yaml_to_any)
                .collect();
            Some(items.join(", "))
        },
        YamlValue::Mapping(map) => {
            let items: Vec<String> = map.into_iter()
                .filter_map(|(k, v)| {
                    if let (Some(key), Some(value)) = (yaml_to_any(k), yaml_to_any(v)) {
                        Some(format!("{}: {}", key, value))
                    } else {
                        None
                    }
                })
                .collect();
            Some(items.join(", "))
        },
        // Handle Null
        YamlValue::Null => Some("".to_string()),
        // Handle Tagged value
        YamlValue::Tagged(tagged) => yaml_to_any(tagged.value),
    }
} 