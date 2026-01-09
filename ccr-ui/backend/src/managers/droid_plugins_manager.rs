// Droid Plugins Manager
// Manages .factory/plugins/config.json file

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Plugin configuration in config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroidPluginConfig {
    #[serde(default)]
    pub repositories: HashMap<String, Value>,
}

/// Manager for .factory/plugins/config.json
pub struct DroidPluginsManager {
    config_path: PathBuf,
}

impl DroidPluginsManager {
    pub fn default() -> io::Result<Self> {
        let current_dir = std::env::current_dir().map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to get current directory: {}", e),
            )
        })?;
        let config_path = current_dir.join(".factory/plugins/config.json");
        Ok(Self { config_path })
    }

    /// Create manager with custom path (for testing)
    #[allow(dead_code)]
    pub fn with_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Read the plugins config file
    pub fn read(&self) -> io::Result<DroidPluginConfig> {
        if !self.config_path.exists() {
            return Ok(DroidPluginConfig {
                repositories: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.config_path)?;
        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse Droid plugins config: {}", e),
            )
        })
    }

    /// Write the plugins config file atomically
    pub fn write(&self, config: &DroidPluginConfig) -> io::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file first (atomic operation)
        let temp_file =
            NamedTempFile::new_in(self.config_path.parent().unwrap_or_else(|| Path::new(".")))?;

        let content = serde_json::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize Droid plugins config: {}", e),
            )
        })?;

        fs::write(temp_file.path(), content)?;

        // Atomic rename
        temp_file.persist(&self.config_path).map_err(|e| {
            io::Error::other(format!("Failed to persist Droid plugins config: {}", e))
        })?;

        Ok(())
    }

    /// Add or update a plugin
    pub fn add_plugin(&self, id: String, plugin_data: Value) -> io::Result<()> {
        let mut config = self.read()?;
        config.repositories.insert(id, plugin_data);
        self.write(&config)
    }

    /// Update a plugin
    pub fn update_plugin(&self, id: &str, plugin_data: Value) -> io::Result<()> {
        let mut config = self.read()?;
        if !config.repositories.contains_key(id) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Plugin '{}' not found", id),
            ));
        }
        config.repositories.insert(id.to_string(), plugin_data);
        self.write(&config)
    }

    /// Delete a plugin
    pub fn delete_plugin(&self, id: &str) -> io::Result<()> {
        let mut config = self.read()?;
        if config.repositories.remove(id).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Plugin '{}' not found", id),
            ));
        }
        self.write(&config)
    }

    /// Get all plugins
    pub fn get_plugins(&self) -> io::Result<HashMap<String, Value>> {
        let config = self.read()?;
        Ok(config.repositories)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let manager = DroidPluginsManager::with_path(config_path);

        let mut config = DroidPluginConfig {
            repositories: HashMap::new(),
        };
        config
            .repositories
            .insert("test-plugin".to_string(), json!({"enabled": true}));

        manager.write(&config).unwrap();
        let read_config = manager.read().unwrap();

        assert!(read_config.repositories.contains_key("test-plugin"));
    }

    #[test]
    fn test_plugin_crud() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let manager = DroidPluginsManager::with_path(config_path);

        // Add
        manager
            .add_plugin("plugin1".to_string(), json!({"version": "1.0"}))
            .unwrap();

        // Get
        let plugins = manager.get_plugins().unwrap();
        assert_eq!(plugins.len(), 1);

        // Update
        manager
            .update_plugin("plugin1", json!({"version": "2.0"}))
            .unwrap();

        let plugins = manager.get_plugins().unwrap();
        assert_eq!(plugins.get("plugin1").unwrap()["version"], "2.0");

        // Delete
        manager.delete_plugin("plugin1").unwrap();
        let plugins = manager.get_plugins().unwrap();
        assert_eq!(plugins.len(), 0);
    }
}
