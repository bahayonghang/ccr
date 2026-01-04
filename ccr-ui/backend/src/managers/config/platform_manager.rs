// Platform Config Manager
// Manages ~/.ccr/config.toml (platform registry in Unified mode)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// 平台注册表配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRegistryConfig {
    /// 默认平台
    pub default_platform: String,
    /// 当前激活的平台
    pub current_platform: String,
    /// 平台注册表（使用 flatten 序列化）
    #[serde(flatten)]
    pub platforms: HashMap<String, PlatformRegistryEntry>,
}

/// 平台注册表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRegistryEntry {
    /// 平台是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 当前激活的 profile 名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
    /// 平台描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 最后使用时间 (ISO 8601 格式)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for PlatformRegistryEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            current_profile: None,
            description: None,
            last_used: None,
        }
    }
}

impl Default for PlatformRegistryConfig {
    fn default() -> Self {
        let mut platforms = HashMap::new();

        // 默认启用 Claude 平台
        platforms.insert(
            "claude".to_string(),
            PlatformRegistryEntry {
                enabled: true,
                current_profile: None,
                description: Some("Claude Code AI Assistant".to_string()),
                last_used: None,
            },
        );

        Self {
            default_platform: "claude".to_string(),
            current_platform: "claude".to_string(),
            platforms,
        }
    }
}

/// Platform registry manager for ~/.ccr/config.toml
pub struct PlatformConfigManager {
    config_path: PathBuf,
}

impl PlatformConfigManager {
    /// Create manager with custom path
    #[allow(dead_code)] // 预留用于自定义配置路径
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// Create manager with default path (~/.ccr/config.toml)
    pub fn default() -> io::Result<Self> {
        let ccr_root = Self::get_ccr_root()?;
        let config_path = ccr_root.join("config.toml");
        Ok(Self { config_path })
    }

    /// Get CCR root directory
    fn get_ccr_root() -> io::Result<PathBuf> {
        if let Ok(custom_root) = std::env::var("CCR_ROOT") {
            Ok(PathBuf::from(custom_root))
        } else {
            // Support both Unix (HOME) and Windows (USERPROFILE)
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "HOME/USERPROFILE environment variable not set",
                    )
                })?;
            Ok(Path::new(&home).join(".ccr"))
        }
    }

    /// Read platform registry
    pub fn read(&self) -> io::Result<PlatformRegistryConfig> {
        if !self.config_path.exists() {
            return Ok(PlatformRegistryConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        toml::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse platform config: {}", e),
            )
        })
    }

    /// Write platform registry atomically
    pub fn write(&self, config: &PlatformRegistryConfig) -> io::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file first (atomic operation)
        let temp_file =
            NamedTempFile::new_in(self.config_path.parent().unwrap_or_else(|| Path::new("/")))?;

        let content = toml::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize platform config: {}", e),
            )
        })?;

        fs::write(temp_file.path(), content)?;

        // Atomic rename
        temp_file
            .persist(&self.config_path)
            .map_err(|e| io::Error::other(format!("Failed to persist platform config: {}", e)))?;

        Ok(())
    }

    /// Get current platform
    pub fn get_current_platform(&self) -> io::Result<String> {
        let config = self.read()?;
        Ok(config.current_platform)
    }

    /// Set current platform
    pub fn set_current_platform(&self, platform: &str) -> io::Result<()> {
        let mut config = self.read()?;

        // Verify platform exists
        if !config.platforms.contains_key(platform) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Platform '{}' not found", platform),
            ));
        }

        // Update current platform
        config.current_platform = platform.to_string();

        // Update last_used timestamp
        if let Some(entry) = config.platforms.get_mut(platform) {
            entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }

        self.write(&config)
    }

    /// Get platform entry
    pub fn get_platform(&self, name: &str) -> io::Result<PlatformRegistryEntry> {
        let config = self.read()?;
        config.platforms.get(name).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Platform '{}' not found", name),
            )
        })
    }

    /// Register or update a platform
    pub fn register_platform(&self, name: String, entry: PlatformRegistryEntry) -> io::Result<()> {
        let mut config = self.read()?;
        config.platforms.insert(name, entry);
        self.write(&config)
    }

    /// Unregister a platform
    #[allow(dead_code)] // 预留用于取消注册平台
    pub fn unregister_platform(&self, name: &str) -> io::Result<()> {
        let mut config = self.read()?;

        // Prevent unregistering current platform if it's the only one
        if name == config.current_platform && config.platforms.len() <= 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot unregister the only platform",
            ));
        }

        // If unregistering current platform, switch to default
        if name == config.current_platform {
            config.current_platform = config.default_platform.clone();
        }

        config.platforms.remove(name);
        self.write(&config)
    }

    /// Enable platform
    pub fn enable_platform(&self, name: &str) -> io::Result<()> {
        let mut config = self.read()?;
        let entry = config.platforms.get_mut(name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Platform '{}' not found", name),
            )
        })?;

        entry.enabled = true;
        self.write(&config)
    }

    /// Disable platform
    pub fn disable_platform(&self, name: &str) -> io::Result<()> {
        let mut config = self.read()?;

        // Prevent disabling current platform
        if name == config.current_platform {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot disable current platform",
            ));
        }

        let entry = config.platforms.get_mut(name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Platform '{}' not found", name),
            )
        })?;

        entry.enabled = false;
        self.write(&config)
    }

    /// List all platforms
    pub fn list_platforms(&self) -> io::Result<Vec<(String, PlatformRegistryEntry)>> {
        let config = self.read()?;
        let mut platforms: Vec<_> = config.platforms.into_iter().collect();
        platforms.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(platforms)
    }

    /// List enabled platforms
    #[allow(dead_code)] // 预留用于列出启用的平台
    pub fn list_enabled_platforms(&self) -> io::Result<Vec<String>> {
        let config = self.read()?;
        let mut platforms: Vec<_> = config
            .platforms
            .iter()
            .filter(|(_, entry)| entry.enabled)
            .map(|(name, _)| name.clone())
            .collect();
        platforms.sort();
        Ok(platforms)
    }

    /// Set platform's current profile
    pub fn set_platform_profile(&self, platform: &str, profile: &str) -> io::Result<()> {
        let mut config = self.read()?;
        let entry = config.platforms.get_mut(platform).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Platform '{}' not found", platform),
            )
        })?;

        entry.current_profile = Some(profile.to_string());
        entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        self.write(&config)
    }

    /// Get platform's current profile
    pub fn get_platform_profile(&self, platform: &str) -> io::Result<Option<String>> {
        let entry = self.get_platform(platform)?;
        Ok(entry.current_profile)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_default_platform_config() {
        let config = PlatformRegistryConfig::default();
        assert_eq!(config.default_platform, "claude");
        assert_eq!(config.current_platform, "claude");
        assert!(config.platforms.contains_key("claude"));
    }

    #[test]
    fn test_platform_entry_default() {
        let entry = PlatformRegistryEntry::default();
        assert!(entry.enabled);
        assert!(entry.current_profile.is_none());
    }
}
