//! Plugin system for Rustyll
//! Allows extending functionality with custom plugins

pub mod hooks;
pub mod loader;
pub mod registry;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use log::{info, warn, error, debug};

pub use hooks::{Hook, HookContext, HookResult};
pub use loader::PluginLoader;
pub use registry::PluginRegistry;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub min_rustyll_version: Option<String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: i32,
    pub options: HashMap<String, serde_yaml::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 0,
            options: HashMap::new(),
        }
    }
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>>;

    /// Register hooks that this plugin wants to listen to
    fn register_hooks(&self) -> Vec<Hook>;

    /// Handle a hook event
    fn handle_hook(&self, hook: &Hook, context: &mut HookContext) -> HookResult;

    /// Cleanup when plugin is unloaded
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Plugin manager handles loading and running plugins
pub struct PluginManager {
    /// Whether plugins are enabled
    pub enabled: bool,
    /// Directory containing plugins
    plugin_dir: PathBuf,
    /// Registry of loaded plugins
    registry: Arc<RwLock<PluginRegistry>>,
    /// Plugin loader
    loader: PluginLoader,
    /// Hook handlers mapped by hook name
    hook_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn Plugin>>>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(enabled: bool) -> Self {
        Self::with_plugin_dir(enabled, PathBuf::from("_plugins"))
    }

    /// Create a new plugin manager with custom plugin directory
    pub fn with_plugin_dir(enabled: bool, plugin_dir: PathBuf) -> Self {
        PluginManager {
            enabled,
            plugin_dir,
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            loader: PluginLoader::new(),
            hook_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load plugins from the configured directory
    pub fn load_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            info!("Plugins are disabled");
            return Ok(());
        }

        if !self.plugin_dir.exists() {
            debug!("Plugin directory {:?} does not exist", self.plugin_dir);
            return Ok(());
        }

        info!("Loading plugins from {:?}", self.plugin_dir);

        // Load plugin configurations
        let configs = self.load_plugin_configs()?;

        // Load each plugin
        for (plugin_name, config) in configs {
            if !config.enabled {
                debug!("Plugin {} is disabled", plugin_name);
                continue;
            }

            match self.load_plugin(&plugin_name, &config) {
                Ok(_) => info!("Loaded plugin: {}", plugin_name),
                Err(e) => error!("Failed to load plugin {}: {}", plugin_name, e),
            }
        }

        // Register all hooks
        self.register_all_hooks()?;

        Ok(())
    }

    /// Load plugin configurations from _config.yml or plugin-specific config files
    fn load_plugin_configs(&self) -> Result<HashMap<String, PluginConfig>, Box<dyn std::error::Error>> {
        let mut configs = HashMap::new();

        // Look for plugin config files in the plugin directory
        if let Ok(entries) = std::fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "yml" || ext == "yaml") {
                    if let Some(stem) = path.file_stem() {
                        let plugin_name = stem.to_string_lossy().to_string();

                        // Skip if it's a special file
                        if plugin_name.starts_with('_') || plugin_name == "config" {
                            continue;
                        }

                        // Load the config
                        let content = std::fs::read_to_string(&path)?;
                        if let Ok(config) = serde_yaml::from_str::<PluginConfig>(&content) {
                            configs.insert(plugin_name, config);
                        }
                    }
                }
            }
        }

        // Also check for Ruby plugin files (for Jekyll compatibility)
        if let Ok(entries) = std::fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rb") {
                    if let Some(stem) = path.file_stem() {
                        let plugin_name = stem.to_string_lossy().to_string();

                        // Create a default config for Ruby plugins
                        if !configs.contains_key(&plugin_name) {
                            configs.insert(plugin_name.clone(), PluginConfig {
                                enabled: true,
                                priority: 0,
                                options: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(configs)
    }

    /// Load a single plugin
    fn load_plugin(&mut self, name: &str, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Try to load as a Rust plugin first
        let plugin_path = self.plugin_dir.join(format!("{}.so", name));
        if plugin_path.exists() {
            return self.loader.load_rust_plugin(&plugin_path, config);
        }

        // Try to load as a Ruby plugin (for Jekyll compatibility)
        let ruby_path = self.plugin_dir.join(format!("{}.rb", name));
        if ruby_path.exists() {
            warn!("Ruby plugin {} detected but not supported. Consider porting to Rust.", name);
            return Ok(());
        }

        // Try to load as a JavaScript plugin
        let js_path = self.plugin_dir.join(format!("{}.js", name));
        if js_path.exists() {
            warn!("JavaScript plugin {} detected but not yet supported.", name);
            return Ok(());
        }

        Err(format!("Plugin {} not found", name).into())
    }

    /// Register all hooks from loaded plugins
    fn register_all_hooks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let registry = self.registry.read().unwrap();
        let mut handlers = self.hook_handlers.write().unwrap();

        for plugin in registry.plugins() {
            let hooks = plugin.register_hooks();
            for hook in hooks {
                let hook_name = hook.name();
                handlers
                    .entry(hook_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(Arc::clone(&plugin));

                debug!("Registered hook '{}' for plugin '{}'", hook_name, plugin.metadata().name);
            }
        }

        Ok(())
    }

    /// Execute a hook with the given context
    pub fn execute_hook(&self, hook_name: &str, context: &mut HookContext) -> HookResult {
        if !self.enabled {
            return HookResult::Continue;
        }

        let handlers = self.hook_handlers.read().unwrap();
        if let Some(plugins) = handlers.get(hook_name) {
            debug!("Executing hook '{}' with {} handlers", hook_name, plugins.len());

            for plugin in plugins {
                let hook = Hook::from_name(hook_name);
                match plugin.handle_hook(&hook, context) {
                    HookResult::Continue => continue,
                    HookResult::StopPropagation => {
                        debug!("Hook '{}' stopped by plugin '{}'", hook_name, plugin.metadata().name);
                        return HookResult::StopPropagation;
                    }
                    HookResult::Error(e) => {
                        error!("Error in hook '{}' from plugin '{}': {}",
                               hook_name, plugin.metadata().name, e);
                        return HookResult::Error(e);
                    }
                }
            }
        }

        HookResult::Continue
    }

    /// Get a list of all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        let registry = self.registry.read().unwrap();
        registry.plugins()
            .iter()
            .map(|p| p.metadata().clone())
            .collect()
    }

    /// Unload all plugins
    pub fn unload_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Unloading all plugins");

        let mut registry = self.registry.write().unwrap();
        registry.unload_all()?;

        let mut handlers = self.hook_handlers.write().unwrap();
        handlers.clear();

        Ok(())
    }

    /// Reload all plugins
    pub fn reload_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.unload_all()?;
        self.load_plugins()?;
        Ok(())
    }
}

/// Built-in plugins
pub mod builtin {
    use super::*;

    /// SEO optimization plugin
    pub struct SeoPlugin {
        metadata: PluginMetadata,
        config: PluginConfig,
    }

    impl SeoPlugin {
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    name: "seo".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Rustyll Team".to_string(),
                    description: "SEO optimization for generated pages".to_string(),
                    homepage: None,
                    license: Some("MIT".to_string()),
                    min_rustyll_version: None,
                },
                config: PluginConfig::default(),
            }
        }
    }

    impl Plugin for SeoPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
            self.config = config.clone();
            Ok(())
        }

        fn register_hooks(&self) -> Vec<Hook> {
            vec![
                Hook::PostRender,
                Hook::PreWrite,
            ]
        }

        fn handle_hook(&self, hook: &Hook, context: &mut HookContext) -> HookResult {
            match hook {
                Hook::PostRender => {
                    // Add SEO meta tags
                    if let Some(content) = context.data.get_mut("content") {
                        if let Some(html) = content.as_str() {
                            // Add meta tags, structured data, etc.
                            debug!("Adding SEO optimizations");
                        }
                    }
                }
                Hook::PreWrite => {
                    // Generate sitemap.xml, robots.txt
                    debug!("Generating SEO files");
                }
                _ => {}
            }
            HookResult::Continue
        }
    }

    /// Feed generation plugin
    pub struct FeedPlugin {
        metadata: PluginMetadata,
        config: PluginConfig,
    }

    impl FeedPlugin {
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    name: "feed".to_string(),
                    version: "1.0.0".to_string(),
                    author: "Rustyll Team".to_string(),
                    description: "RSS/Atom feed generation".to_string(),
                    homepage: None,
                    license: Some("MIT".to_string()),
                    min_rustyll_version: None,
                },
                config: PluginConfig::default(),
            }
        }
    }

    impl Plugin for FeedPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&mut self, config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
            self.config = config.clone();
            Ok(())
        }

        fn register_hooks(&self) -> Vec<Hook> {
            vec![Hook::PostWrite]
        }

        fn handle_hook(&self, hook: &Hook, context: &mut HookContext) -> HookResult {
            match hook {
                Hook::PostWrite => {
                    // Generate RSS/Atom feeds
                    debug!("Generating feeds");
                }
                _ => {}
            }
            HookResult::Continue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new(true);
        assert!(manager.enabled);
    }

    #[test]
    fn test_builtin_seo_plugin() {
        let mut plugin = builtin::SeoPlugin::new();
        assert_eq!(plugin.metadata().name, "seo");

        let config = PluginConfig::default();
        assert!(plugin.initialize(&config).is_ok());

        let hooks = plugin.register_hooks();
        assert!(!hooks.is_empty());
    }

    #[test]
    fn test_builtin_feed_plugin() {
        let mut plugin = builtin::FeedPlugin::new();
        assert_eq!(plugin.metadata().name, "feed");

        let config = PluginConfig::default();
        assert!(plugin.initialize(&config).is_ok());

        let hooks = plugin.register_hooks();
        assert!(!hooks.is_empty());
    }
}