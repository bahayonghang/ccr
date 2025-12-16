// Settings Manager for ~/.claude/settings.json
// Reads and writes Claude Code settings with atomic operations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Settings file not found: {0}")]
    NotFound(String),
    #[error("Failed to read settings: {0}")]
    ReadError(String),
    #[error("Failed to parse settings: {0}")]
    ParseError(String),
    #[error("Failed to write settings: {0}")]
    WriteError(String),
}

pub type Result<T> = std::result::Result<T, SettingsError>;

/// Claude Code settings structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeSettings {
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Output style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,

    /// Permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Value>,

    /// MCP Servers
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub mcp_servers: HashMap<String, McpServer>,

    /// Slash Commands
    #[serde(
        default,
        rename = "slashCommands",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub slash_commands: Vec<SlashCommand>,

    /// Agents
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<Agent>,

    /// Plugins
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<Plugin>,

    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn default_true() -> bool {
    true
}

pub struct SettingsManager {
    settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new(settings_path: PathBuf) -> Self {
        Self { settings_path }
    }

    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| SettingsError::NotFound("Cannot get home directory".to_string()))?;
        let settings_path = home.join(".claude").join("settings.json");
        Ok(Self::new(settings_path))
    }

    /// Load settings from file
    pub fn load(&self) -> Result<ClaudeSettings> {
        if !self.settings_path.exists() {
            tracing::warn!(
                "Settings file not found, returning default: {:?}",
                self.settings_path
            );
            return Ok(ClaudeSettings::default());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| SettingsError::ReadError(format!("{}", e)))?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| SettingsError::ParseError(format!("{}", e)))?;

        tracing::debug!("Loaded settings from {:?}", self.settings_path);
        Ok(settings)
    }

    /// Save settings to file atomically
    pub fn save(&self, settings: &ClaudeSettings) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SettingsError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Serialize to JSON with pretty formatting
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| SettingsError::WriteError(format!("Failed to serialize: {}", e)))?;

        // Write to temporary file
        let temp_dir = self
            .settings_path
            .parent()
            .unwrap_or_else(|| Path::new("/tmp"));
        let mut temp_file = NamedTempFile::new_in(temp_dir)
            .map_err(|e| SettingsError::WriteError(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| SettingsError::WriteError(format!("Failed to write temp file: {}", e)))?;

        // Atomic rename
        temp_file
            .persist(&self.settings_path)
            .map_err(|e| SettingsError::WriteError(format!("Failed to persist file: {}", e)))?;

        tracing::info!("Settings saved to {:?}", self.settings_path);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_settings() {
        let settings = ClaudeSettings {
            env: HashMap::from([(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.anthropic.com".to_string(),
            )]),
            output_style: Some("nekomata-engineer".to_string()),
            permissions: None,
            mcp_servers: HashMap::new(),
            slash_commands: Vec::new(),
            agents: Vec::new(),
            plugins: Vec::new(),
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("env"));
        assert!(json.contains("ANTHROPIC_BASE_URL"));
    }
}
