// Settings Manager for ~/.claude/settings.json
// Reads and writes Claude Code settings with atomic operations

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

// Re-export shared types from ccr-types
pub use ccr_types::{Agent, ClaudeSettings, Hook, Plugin, SlashCommand};
// Note: McpServer is also available from ccr_types but not re-exported here
// as it's not directly used by this module's consumers

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

pub struct SettingsManager {
    settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new(settings_path: PathBuf) -> Self {
        Self { settings_path }
    }

    #[deprecated(
        since = "3.15.0",
        note = "使用 `crate::cache::GLOBAL_SETTINGS_CACHE.load()` 代替，可减少 80% 文件 I/O"
    )]
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| SettingsError::NotFound("Cannot get home directory".to_string()))?;
        let settings_path = home.join(".claude").join("settings.json");
        Ok(Self::new(settings_path))
    }

    /// Load settings from file
    #[deprecated(
        since = "3.15.0",
        note = "使用 `crate::cache::GLOBAL_SETTINGS_CACHE.load()` 代替，可减少 80% 文件 I/O"
    )]
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
    #[deprecated(
        since = "3.15.0",
        note = "使用 `crate::cache::GLOBAL_SETTINGS_CACHE.save_atomic()` 代替，可自动失效缓存"
    )]
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
    use std::collections::HashMap;

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
            hooks: Vec::new(),
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("env"));
        assert!(json.contains("ANTHROPIC_BASE_URL"));
    }
}
