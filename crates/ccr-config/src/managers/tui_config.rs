use crate::managers::PlatformConfigManager;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const DEFAULT_TAB_ORDER: [TuiTabId; 5] = [
    TuiTabId::CodexProfile,
    TuiTabId::ClaudeProfile,
    TuiTabId::CodexAuth,
    TuiTabId::ClaudeAuth,
    TuiTabId::OpencodeAuth,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuiTabId {
    CodexProfile,
    ClaudeProfile,
    CodexAuth,
    ClaudeAuth,
    OpencodeAuth,
}

impl TuiTabId {
    pub fn as_str(self) -> &'static str {
        match self {
            TuiTabId::CodexProfile => "codex_profile",
            TuiTabId::ClaudeProfile => "claude_profile",
            TuiTabId::CodexAuth => "codex_auth",
            TuiTabId::ClaudeAuth => "claude_auth",
            TuiTabId::OpencodeAuth => "opencode_auth",
        }
    }

    pub fn default_order() -> Vec<Self> {
        DEFAULT_TAB_ORDER.to_vec()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_tab_order")]
    pub tab_order: Vec<TuiTabId>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            tab_order: default_tab_order(),
        }
    }
}

fn default_tab_order() -> Vec<TuiTabId> {
    TuiTabId::default_order()
}

pub struct TuiConfigManager {
    config_path: PathBuf,
}

impl TuiConfigManager {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    pub fn with_default() -> Result<Self> {
        let platform_manager = PlatformConfigManager::with_default()?;
        let root_dir = platform_manager.root_dir()?.to_path_buf();
        Ok(Self::new(root_dir.join("tui.toml")))
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load(&self) -> Result<TuiConfig> {
        if !self.config_path.exists() {
            tracing::debug!(
                "TUI config file not found at {}; using defaults",
                self.config_path.display()
            );
            return Ok(TuiConfig::default());
        }

        let config: TuiConfig = fileio::read_toml(&self.config_path)?;
        validate_tab_order(&config.tab_order)?;
        Ok(config)
    }

    pub fn load_or_default(&self) -> TuiConfig {
        match self.load() {
            Ok(config) => config,
            Err(error) => {
                tracing::warn!(
                    "Failed to load TUI config from {}: {}. Falling back to default tab order.",
                    self.config_path.display(),
                    error
                );
                TuiConfig::default()
            }
        }
    }
}

fn validate_tab_order(tab_order: &[TuiTabId]) -> Result<()> {
    if tab_order.len() != DEFAULT_TAB_ORDER.len() {
        return Err(CcrError::ConfigFormatInvalid(format!(
            "`tab_order` must list all {} tabs in order: {}",
            DEFAULT_TAB_ORDER.len(),
            supported_tab_names()
        )));
    }

    let mut seen = HashSet::with_capacity(DEFAULT_TAB_ORDER.len());
    for tab_id in tab_order {
        if !seen.insert(*tab_id) {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "`tab_order` contains duplicate value `{}`",
                tab_id.as_str()
            )));
        }
    }

    let missing = DEFAULT_TAB_ORDER
        .iter()
        .filter(|tab_id| !seen.contains(tab_id))
        .map(|tab_id| tab_id.as_str())
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        return Err(CcrError::ConfigFormatInvalid(format!(
            "`tab_order` is missing value(s): {}",
            missing.join(", ")
        )));
    }

    Ok(())
}

fn supported_tab_names() -> String {
    DEFAULT_TAB_ORDER
        .iter()
        .map(|tab_id| format!("`{}`", tab_id.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{TuiConfig, TuiConfigManager, TuiTabId};
    use crate::test_support::TestCcrEnv;

    #[test]
    fn with_default_resolves_tui_toml_under_ccr_root() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::with_default().unwrap();

        assert_eq!(manager.config_path(), env.root().join("tui.toml").as_path());
    }

    #[test]
    fn load_returns_default_when_tui_config_is_missing() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        assert_eq!(manager.load().unwrap(), TuiConfig::default());
    }

    #[test]
    fn load_preserves_valid_custom_tab_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "claude_auth",
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        let config = manager.load().unwrap();
        assert_eq!(
            config.tab_order,
            vec![
                TuiTabId::ClaudeAuth,
                TuiTabId::CodexProfile,
                TuiTabId::ClaudeProfile,
                TuiTabId::CodexAuth,
                TuiTabId::OpencodeAuth,
            ]
        );
    }

    #[test]
    fn load_or_default_falls_back_for_duplicate_tab_ids() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "claude_auth",
  "claude_auth",
]
"#,
        )
        .unwrap();

        assert!(manager.load().is_err());
        assert_eq!(manager.load_or_default(), TuiConfig::default());
    }

    #[test]
    fn load_or_default_falls_back_for_missing_tab_ids() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "claude_auth",
]
"#,
        )
        .unwrap();

        assert!(manager.load().is_err());
        assert_eq!(manager.load_or_default(), TuiConfig::default());
    }

    #[test]
    fn load_or_default_falls_back_for_unknown_tab_ids() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "claude_auth",
  "claude_runtime"
]
"#,
        )
        .unwrap();

        assert!(manager.load().is_err());
        assert_eq!(manager.load_or_default(), TuiConfig::default());
    }
}
