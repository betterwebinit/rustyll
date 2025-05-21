use liquid_core::model::{Object, Value};

/// Create default include globals used by both include and include_relative tags
pub fn create_default_include_globals() -> Object {
    let mut include = Object::new();
    
    // Add default values for commonly used include parameters
    include.insert("content".into(), Value::scalar(""));
    include.insert("path".into(), Value::scalar(""));
    include.insert("name".into(), Value::scalar(""));
    include.insert("url".into(), Value::scalar(""));
    include.insert("title".into(), Value::scalar(""));  // Ensure title is always available
    include.insert("color".into(), Value::scalar(""));
    include.insert("show_logo".into(), Value::scalar(false));
    include.insert("logo_path".into(), Value::scalar("/assets/images/logo.png"));
    
    include
} 