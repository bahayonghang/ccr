use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Plugin configuration in config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default)]
    pub repositories: HashMap<String, Value>,
}

/// Manager for ~/.claude/plugins/config.json
pub struct PluginsManager {
    config_path: PathBuf,
}

impl PluginsManager {
    pub fn default() -> io::Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "HOME environment variable not set")
        })?;
        let config_path = Path::new(&home).join(".claude/plugins/config.json");
        Ok(Self { config_path })
    }

    /// Read the plugins config file
    pub fn read(&self) -> io::Result<PluginConfig> {
        if !self.config_path.exists() {
            return Ok(PluginConfig {
                repositories: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.config_path)?;
        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse plugins config: {}", e),
            )
        })
    }

    /// Write the plugins config file atomically
    pub fn write(&self, config: &PluginConfig) -> io::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file first (atomic operation)
        let temp_file = NamedTempFile::new_in(
            self.config_path
                .parent()
                .unwrap_or_else(|| Path::new("/")),
        )?;

        let content = serde_json::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize plugins config: {}", e),
            )
        })?;

        fs::write(temp_file.path(), content)?;

        // Atomic rename
        temp_file.persist(&self.config_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to persist plugins config: {}", e),
            )
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

