//! Plugin registry for managing loaded plugins

use std::collections::HashMap;
use std::sync::Arc;
use log::{info, debug};

use super::Plugin;

/// Registry for managing loaded plugins
pub struct PluginRegistry {
    /// Loaded plugins indexed by name
    plugins: HashMap<String, Arc<dyn Plugin>>,
    /// Plugin load order (for deterministic execution)
    load_order: Vec<String>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) -> Result<(), String> {
        let name = plugin.metadata().name.clone();

        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' is already registered", name));
        }

        info!("Registering plugin: {}", name);
        self.load_order.push(name.clone());
        self.plugins.insert(name, plugin);

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, name: &str) -> Result<(), String> {
        if !self.plugins.contains_key(name) {
            return Err(format!("Plugin '{}' is not registered", name));
        }

        info!("Unregistering plugin: {}", name);
        self.plugins.remove(name);
        self.load_order.retain(|n| n != name);

        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }

    /// Get all plugins in load order
    pub fn plugins(&self) -> Vec<Arc<dyn Plugin>> {
        self.load_order
            .iter()
            .filter_map(|name| self.plugins.get(name).cloned())
            .collect()
    }

    /// Get the number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Clear all plugins
    pub fn clear(&mut self) {
        self.plugins.clear();
        self.load_order.clear();
    }

    /// Unload all plugins (with cleanup)
    pub fn unload_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Unloading {} plugins", self.plugins.len());

        // Note: In a real implementation, we would:
        // 1. Call cleanup() on each plugin
        // 2. Properly unload any dynamic libraries
        // 3. Clean up any resources

        self.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{PluginMetadata, PluginConfig, Hook, HookContext, HookResult};

    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&mut self, _config: &PluginConfig) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn register_hooks(&self) -> Vec<Hook> {
            vec![]
        }

        fn handle_hook(&self, _hook: &Hook, _context: &mut HookContext) -> HookResult {
            HookResult::Continue
        }
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = PluginRegistry::new();
        assert!(registry.is_empty());

        let plugin = Arc::new(TestPlugin {
            metadata: PluginMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "Test plugin".to_string(),
                homepage: None,
                license: None,
                min_rustyll_version: None,
            },
        });

        // Register plugin
        assert!(registry.register(plugin.clone()).is_ok());
        assert_eq!(registry.len(), 1);

        // Get plugin
        assert!(registry.get("test").is_some());

        // Try to register duplicate
        assert!(registry.register(plugin).is_err());

        // Unregister plugin
        assert!(registry.unregister("test").is_ok());
        assert!(registry.is_empty());

        // Try to unregister non-existent
        assert!(registry.unregister("test").is_err());
    }
}