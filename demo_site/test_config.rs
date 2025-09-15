use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiteData {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<serde_yaml::Value>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub site_data: SiteData,
}

fn main() {
    let yaml = r#"
title: Rustyll Demo Site
description: A demonstration
author:
  name: Rustyll Team
  email: hello@rustyll.dev
"#;
    
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    println!("Config: {:#?}", config);
    println!("Author field: {:?}", config.site_data.author);
}
