use crate::managers::PlatformConfigManager;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::fileio;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const DEFAULT_TAB_ORDER: [TuiTabId; 6] = [
    TuiTabId::CodexProfile,
    TuiTabId::ClaudeProfile,
    TuiTabId::GrokProfile,
    TuiTabId::CodexAuth,
    TuiTabId::ClaudeAuth,
    TuiTabId::OpencodeAuth,
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TuiLanguage {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh_cn")]
    SimplifiedChinese,
}

impl TuiLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::SimplifiedChinese => "zh_cn",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::English => Self::SimplifiedChinese,
            Self::SimplifiedChinese => Self::English,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TuiTheme {
    #[default]
    #[serde(rename = "mocha")]
    Mocha,
    #[serde(rename = "latte")]
    Latte,
}

impl TuiTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mocha => "mocha",
            Self::Latte => "latte",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Mocha => Self::Latte,
            Self::Latte => Self::Mocha,
        }
    }
}

fn deserialize_language_or_english<'de, D>(
    deserializer: D,
) -> std::result::Result<TuiLanguage, D::Error>
where
    D: Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    match value.as_str() {
        Some("en") => Ok(TuiLanguage::English),
        Some("zh_cn") => Ok(TuiLanguage::SimplifiedChinese),
        Some(value) => {
            tracing::warn!("Unsupported TUI language `{value}`; falling back to English");
            Ok(TuiLanguage::English)
        }
        None => {
            tracing::warn!("TUI language must be a string; falling back to English");
            Ok(TuiLanguage::English)
        }
    }
}

fn deserialize_theme_or_mocha<'de, D>(deserializer: D) -> std::result::Result<TuiTheme, D::Error>
where
    D: Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    match value.as_str() {
        Some("mocha") => Ok(TuiTheme::Mocha),
        Some("latte") => Ok(TuiTheme::Latte),
        Some(value) => {
            tracing::warn!("Unsupported TUI theme `{value}`; falling back to Mocha");
            Ok(TuiTheme::Mocha)
        }
        None => {
            tracing::warn!("TUI theme must be a string; falling back to Mocha");
            Ok(TuiTheme::Mocha)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuiTabId {
    CodexProfile,
    GrokProfile,
    ClaudeProfile,
    /// Deprecated: 独立 Usage tab 已下线(用量内嵌到 profile 详情面板)。
    /// 仅为解析旧版 `tui.toml` 保留;`load()` 会过滤该项并记录 warn,
    /// 不得因它出现而丢弃用户的自定义排序。
    #[doc(hidden)]
    Usage,
    CodexAuth,
    ClaudeAuth,
    OpencodeAuth,
}

impl TuiTabId {
    pub fn as_str(self) -> &'static str {
        match self {
            TuiTabId::CodexProfile => "codex_profile",
            TuiTabId::GrokProfile => "grok_profile",
            TuiTabId::ClaudeProfile => "claude_profile",
            TuiTabId::Usage => "usage",
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
    #[serde(default, deserialize_with = "deserialize_language_or_english")]
    pub language: TuiLanguage,
    #[serde(default, deserialize_with = "deserialize_theme_or_mocha")]
    pub theme: TuiTheme,
    #[serde(default = "default_tab_order")]
    pub tab_order: Vec<TuiTabId>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            language: TuiLanguage::default(),
            theme: TuiTheme::default(),
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

        let mut config: TuiConfig = fileio::read_toml(&self.config_path)?;
        // 兼容旧版配置:曾存在独立 usage tab,现已下线。过滤而非报错,
        // 保住用户对其余 tab 的自定义排序。
        if config.tab_order.contains(&TuiTabId::Usage) {
            tracing::warn!(
                "TUI config {} contains deprecated `usage` tab; ignoring it (usage now lives in profile details)",
                self.config_path.display()
            );
            config.tab_order.retain(|tab_id| *tab_id != TuiTabId::Usage);
        }

        let missing = DEFAULT_TAB_ORDER
            .iter()
            .copied()
            .filter(|tab_id| !config.tab_order.contains(tab_id))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            tracing::warn!(
                "TUI config {} is missing tab(s) {}; appending them in default order",
                self.config_path.display(),
                missing
                    .iter()
                    .map(|tab_id| format!("`{}`", tab_id.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            config.tab_order.extend(missing);
        }
        validate_tab_order(&config.tab_order)?;
        Ok(config)
    }

    pub fn load_or_default(&self) -> TuiConfig {
        match self.load() {
            Ok(config) => config,
            Err(error) => {
                tracing::warn!(
                    "Failed to load TUI config from {}: {}. Falling back to default config.",
                    self.config_path.display(),
                    error
                );
                TuiConfig::default()
            }
        }
    }

    pub fn save(&self, config: &TuiConfig) -> Result<()> {
        validate_tab_order(&config.tab_order)?;
        fileio::write_toml(&self.config_path, config)
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
    use super::{TuiConfig, TuiConfigManager, TuiLanguage, TuiTabId, TuiTheme};
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
    fn default_language_is_english() {
        assert_eq!(TuiConfig::default().language, TuiLanguage::English);
        assert_eq!(TuiLanguage::English.as_str(), "en");
        assert_eq!(TuiLanguage::SimplifiedChinese.as_str(), "zh_cn");
        assert_eq!(
            TuiLanguage::English.toggled(),
            TuiLanguage::SimplifiedChinese
        );
    }

    #[test]
    fn default_theme_is_mocha() {
        assert_eq!(TuiConfig::default().theme, TuiTheme::Mocha);
        assert_eq!(TuiTheme::Mocha.as_str(), "mocha");
        assert_eq!(TuiTheme::Latte.as_str(), "latte");
        assert_eq!(TuiTheme::Mocha.toggled(), TuiTheme::Latte);
    }

    #[test]
    fn default_order_excludes_deprecated_usage_tab() {
        let order = TuiTabId::default_order();

        assert_eq!(
            order,
            vec![
                TuiTabId::CodexProfile,
                TuiTabId::ClaudeProfile,
                TuiTabId::GrokProfile,
                TuiTabId::CodexAuth,
                TuiTabId::ClaudeAuth,
                TuiTabId::OpencodeAuth,
            ]
        );
        assert!(!order.contains(&TuiTabId::Usage));
    }

    #[test]
    fn load_migrates_five_item_tab_order_without_losing_custom_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "claude_profile",
  "codex_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        let config = manager.load().unwrap();
        assert_eq!(config.language, TuiLanguage::English);
        assert_eq!(
            config.tab_order,
            vec![
                TuiTabId::ClaudeProfile,
                TuiTabId::CodexProfile,
                TuiTabId::CodexAuth,
                TuiTabId::ClaudeAuth,
                TuiTabId::OpencodeAuth,
                TuiTabId::GrokProfile,
            ]
        );
    }

    #[test]
    fn load_accepts_simplified_chinese_language() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"language = "zh_cn"
tab_order = [
  "codex_profile",
  "claude_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        assert_eq!(
            manager.load().unwrap().language,
            TuiLanguage::SimplifiedChinese
        );
    }

    #[test]
    fn unsupported_language_falls_back_without_discarding_tab_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"language = "fr"
tab_order = [
  "claude_profile",
  "codex_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        let config = manager.load().unwrap();
        assert_eq!(config.language, TuiLanguage::English);
        assert_eq!(config.tab_order[0], TuiTabId::ClaudeProfile);
        assert_eq!(config.tab_order[1], TuiTabId::CodexProfile);
    }

    #[test]
    fn non_string_language_falls_back_without_discarding_tab_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"language = 42
tab_order = [
  "claude_profile",
  "codex_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        let config = manager.load().unwrap();
        assert_eq!(config.language, TuiLanguage::English);
        assert_eq!(config.tab_order[0], TuiTabId::ClaudeProfile);
    }

    #[test]
    fn save_round_trips_language_and_tab_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));
        let config = TuiConfig {
            language: TuiLanguage::SimplifiedChinese,
            theme: TuiTheme::Latte,
            tab_order: vec![
                TuiTabId::ClaudeProfile,
                TuiTabId::CodexProfile,
                TuiTabId::GrokProfile,
                TuiTabId::CodexAuth,
                TuiTabId::ClaudeAuth,
                TuiTabId::OpencodeAuth,
            ],
        };

        manager.save(&config).unwrap();

        assert_eq!(manager.load().unwrap(), config);
        let saved = std::fs::read_to_string(manager.config_path()).unwrap();
        assert!(saved.contains("language = \"zh_cn\""));
        assert!(saved.contains("theme = \"latte\""));
    }

    #[test]
    fn save_rejects_invalid_tab_order_without_overwriting_existing_config() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));
        let original = TuiConfig::default();
        manager.save(&original).unwrap();

        let invalid = TuiConfig {
            language: TuiLanguage::SimplifiedChinese,
            theme: TuiTheme::Latte,
            tab_order: vec![TuiTabId::CodexProfile],
        };

        assert!(manager.save(&invalid).is_err());
        assert_eq!(manager.load().unwrap(), original);
    }

    // 旧版 tui.toml 含已下线的 usage tab:自定义顺序必须原样保留,仅剔除 usage
    #[test]
    fn load_preserves_legacy_custom_order_and_ignores_usage_tab() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "claude_auth",
  "codex_profile",
  "claude_profile",
  "usage",
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
                TuiTabId::GrokProfile,
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
  "usage",
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
    fn load_appends_multiple_missing_tabs_in_default_relative_order() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"tab_order = [
  "claude_auth",
  "codex_profile",
]
"#,
        )
        .unwrap();

        assert_eq!(
            manager.load().unwrap().tab_order,
            vec![
                TuiTabId::ClaudeAuth,
                TuiTabId::CodexProfile,
                TuiTabId::ClaudeProfile,
                TuiTabId::GrokProfile,
                TuiTabId::CodexAuth,
                TuiTabId::OpencodeAuth,
            ]
        );
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
  "usage",
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

    #[test]
    fn unsupported_theme_falls_back_without_discarding_other_preferences() {
        let env = TestCcrEnv::new();
        let manager = TuiConfigManager::new(env.root().join("tui.toml"));

        std::fs::write(
            manager.config_path(),
            r#"language = "zh_cn"
theme = "solarized"
tab_order = [
  "claude_profile",
  "codex_profile",
  "codex_auth",
  "claude_auth",
  "opencode_auth",
]
"#,
        )
        .unwrap();

        let config = manager.load().unwrap();
        assert_eq!(config.theme, TuiTheme::Mocha);
        assert_eq!(config.language, TuiLanguage::SimplifiedChinese);
        assert_eq!(config.tab_order[0], TuiTabId::ClaudeProfile);
    }
}
