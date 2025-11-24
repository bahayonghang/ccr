use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// MCP Server configuration (stored in ~/.claude.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Root structure of ~/.claude.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Manager for ~/.claude.json file
pub struct ClaudeConfigManager {
    config_path: PathBuf,
}

impl ClaudeConfigManager {
    pub fn default() -> io::Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "HOME environment variable not set")
        })?;
        let config_path = Path::new(&home).join(".claude.json");
        Ok(Self { config_path })
    }

    /// Read the config file
    pub fn read(&self) -> io::Result<ClaudeConfig> {
        if !self.config_path.exists() {
            return Ok(ClaudeConfig {
                mcp_servers: HashMap::new(),
                other: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.config_path)?;
        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse .claude.json: {}", e),
            )
        })
    }

    /// Write the config file atomically
    pub fn write(&self, config: &ClaudeConfig) -> io::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file first (atomic operation)
        let temp_file =
            NamedTempFile::new_in(self.config_path.parent().unwrap_or_else(|| Path::new("/")))?;

        let content = serde_json::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {}", e),
            )
        })?;

        fs::write(temp_file.path(), content)?;

        // Atomic rename
        temp_file
            .persist(&self.config_path)
            .map_err(|e| io::Error::other(format!("Failed to persist config: {}", e)))?;

        Ok(())
    }

    /// Get MCP servers
    pub fn get_mcp_servers(&self) -> io::Result<HashMap<String, McpServerConfig>> {
        let config = self.read()?;
        Ok(config.mcp_servers)
    }

    /// Add MCP server
    pub fn add_mcp_server(&self, name: String, server: McpServerConfig) -> io::Result<()> {
        let mut config = self.read()?;
        config.mcp_servers.insert(name, server);
        self.write(&config)
    }

    /// Update MCP server
    pub fn update_mcp_server(&self, name: &str, server: McpServerConfig) -> io::Result<()> {
        let mut config = self.read()?;
        if !config.mcp_servers.contains_key(name) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("MCP server '{}' not found", name),
            ));
        }
        config.mcp_servers.insert(name.to_string(), server);
        self.write(&config)
    }

    /// Delete MCP server
    pub fn delete_mcp_server(&self, name: &str) -> io::Result<()> {
        let mut config = self.read()?;
        if config.mcp_servers.remove(name).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("MCP server '{}' not found", name),
            ));
        }
        self.write(&config)
    }
}
