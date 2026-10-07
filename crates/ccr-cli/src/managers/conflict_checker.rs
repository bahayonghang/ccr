use crate::models::Platform;
use ccr_codex::managers::codex_config::CodexConfigManager;
use ccr_config::ClaudeRuntimePaths;
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub conflicts: Vec<Conflict>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub key: String,
    pub platforms: Vec<PlatformValue>,
    pub severity: ConflictSeverity,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformValue {
    pub platform: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    Critical, // Same key, different values that will cause runtime issues
    Warning,  // Same key, different values that might cause confusion
    Info,     // Same key, same value (redundant but not harmful)
}

pub struct ConflictChecker {
    platforms: Vec<Platform>,
}

impl ConflictChecker {
    pub fn new() -> Self {
        Self {
            platforms: vec![Platform::Claude, Platform::Codex, Platform::Gemini],
        }
    }

    pub fn check_conflicts(&self) -> Result<ConflictReport> {
        let mut env_vars: IndexMap<String, Vec<PlatformValue>> = IndexMap::new();
        let mut warnings = Vec::new();

        // Collect environment variables from each platform's settings
        for &platform in &self.platforms {
            match self.collect_env_vars(platform) {
                Ok(vars) => {
                    for (key, value) in vars {
                        env_vars.entry(key).or_default().push(PlatformValue {
                            platform: platform.to_string(),
                            value,
                        });
                    }
                }
                Err(e) => {
                    warnings.push(format!(
                        "Failed to collect env vars from {}: {}",
                        platform, e
                    ));
                }
            }
        }

        // Analyze for conflicts
        let mut conflicts = Vec::new();

        for (key, values) in env_vars {
            if values.len() > 1 {
                let unique_values: HashSet<_> = values.iter().map(|v| &v.value).collect();

                let (severity, suggestion) = if unique_values.len() > 1 {
                    // Different values
                    if Self::is_critical_key(&key) {
                        (
                            ConflictSeverity::Critical,
                            format!(
                                "Unify '{}' across platforms or use platform-specific overrides",
                                key
                            ),
                        )
                    } else {
                        (
                            ConflictSeverity::Warning,
                            format!(
                                "Consider using the same value for '{}' across platforms",
                                key
                            ),
                        )
                    }
                } else {
                    // Same value
                    (
                        ConflictSeverity::Info,
                        format!("'{}' is consistently set across platforms", key),
                    )
                };

                conflicts.push(Conflict {
                    key,
                    platforms: values,
                    severity,
                    suggestion,
                });
            }
        }

        Ok(ConflictReport {
            conflicts,
            warnings,
        })
    }

    fn collect_env_vars(&self, platform: Platform) -> Result<IndexMap<String, String>> {
        let settings_path = match platform {
            Platform::Claude => ClaudeRuntimePaths::from_env()?.settings_file,
            Platform::Codex => CodexConfigManager::resolve_codex_dir()?.join("settings.json"),
            Platform::Gemini => {
                let home = std::env::var_os("HOME")
                    .filter(|value| !value.is_empty())
                    .or_else(|| std::env::var_os("USERPROFILE").filter(|value| !value.is_empty()))
                    .map(std::path::PathBuf::from)
                    .or_else(dirs::home_dir)
                    .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;
                home.join(".gemini")
                    .join("antigravity-cli")
                    .join("settings.json")
            }
            _ => {
                return Err(CcrError::PlatformNotSupported(platform.to_string()));
            }
        };

        if !settings_path.exists() {
            return Ok(IndexMap::new());
        }

        let content = std::fs::read_to_string(&settings_path).map_err(CcrError::IoError)?;
        let settings: serde_json::Value =
            serde_json::from_str(&content).map_err(CcrError::JsonError)?;

        let mut env_vars = IndexMap::new();

        // Extract relevant environment-like keys from settings
        if let Some(obj) = settings.as_object() {
            for (key, value) in obj {
                if Self::is_env_related_key(key)
                    && let Some(str_value) = value.as_str()
                {
                    env_vars.insert(key.clone(), str_value.to_string());
                }
            }
        }

        Ok(env_vars)
    }

    fn is_env_related_key(key: &str) -> bool {
        let env_keys = [
            "apiKey",
            "api_key",
            "baseUrl",
            "base_url",
            "model",
            "defaultModel",
            "temperature",
            "maxTokens",
            "max_tokens",
        ];
        env_keys.contains(&key)
    }

    fn is_critical_key(key: &str) -> bool {
        let critical_keys = ["apiKey", "api_key", "baseUrl", "base_url"];
        critical_keys.contains(&key)
    }
}

impl Default for ConflictChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use std::ffi::OsStr;
    use std::path::Path;

    fn write_settings(path: &Path, model: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::json!({ "model": model }).to_string()).unwrap();
    }

    #[test]
    fn claude_conflict_settings_follow_explicit_file_then_config_dir() {
        let mut home = TestHome::new_with_home_env();
        let custom_dir = home.home().join("custom-claude");
        let explicit_file = home.home().join("explicit-settings.json");
        write_settings(&custom_dir.join("settings.json"), "config-dir-model");
        write_settings(&explicit_file, "explicit-model");
        home.set_env("CLAUDE_CONFIG_DIR", custom_dir.as_os_str());
        home.set_env("CCR_SETTINGS_PATH", explicit_file.as_os_str());
        let checker = ConflictChecker::new();

        assert_eq!(
            checker.collect_env_vars(Platform::Claude).unwrap()["model"],
            "explicit-model"
        );
        home.set_env("CCR_SETTINGS_PATH", OsStr::new(""));
        assert_eq!(
            checker.collect_env_vars(Platform::Claude).unwrap()["model"],
            "config-dir-model"
        );
        home.remove_env("CCR_SETTINGS_PATH");
        assert_eq!(
            checker.collect_env_vars(Platform::Claude).unwrap()["model"],
            "config-dir-model"
        );
    }

    #[test]
    fn codex_conflict_settings_follow_ccr_dir_then_codex_home() {
        let mut home = TestHome::new_with_home_env();
        let custom_dir = home.home().join("custom-codex");
        write_settings(&home.codex_dir().join("settings.json"), "ccr-dir-model");
        write_settings(&custom_dir.join("settings.json"), "codex-home-model");
        home.set_env("CODEX_HOME", custom_dir.as_os_str());
        let checker = ConflictChecker::new();

        assert_eq!(
            checker.collect_env_vars(Platform::Codex).unwrap()["model"],
            "ccr-dir-model"
        );
        home.set_env("CCR_CODEX_DIR", OsStr::new(""));
        assert_eq!(
            checker.collect_env_vars(Platform::Codex).unwrap()["model"],
            "codex-home-model"
        );
        home.remove_env("CCR_CODEX_DIR");
        assert_eq!(
            checker.collect_env_vars(Platform::Codex).unwrap()["model"],
            "codex-home-model"
        );
    }

    #[test]
    fn gemini_conflict_settings_follow_home_then_userprofile() {
        let mut home = TestHome::new_with_home_env();
        let other_home = home.home().join("other-home");
        write_settings(
            &home.home().join(".gemini/antigravity-cli/settings.json"),
            "home-model",
        );
        write_settings(
            &other_home.join(".gemini/antigravity-cli/settings.json"),
            "userprofile-model",
        );
        home.set_env("USERPROFILE", other_home.as_os_str());
        let checker = ConflictChecker::new();

        assert_eq!(
            checker.collect_env_vars(Platform::Gemini).unwrap()["model"],
            "home-model"
        );
        home.set_env("HOME", OsStr::new(""));
        assert_eq!(
            checker.collect_env_vars(Platform::Gemini).unwrap()["model"],
            "userprofile-model"
        );
        home.remove_env("HOME");
        assert_eq!(
            checker.collect_env_vars(Platform::Gemini).unwrap()["model"],
            "userprofile-model"
        );
    }

    #[test]
    fn conflicts_read_only_the_selected_synthetic_settings() {
        let home = TestHome::new_with_home_env();
        let paths = [
            home.settings_path().to_path_buf(),
            home.codex_dir().join("settings.json"),
            home.home().join(".gemini/antigravity-cli/settings.json"),
        ];
        for (path, model) in paths
            .iter()
            .zip(["claude-model", "other-model", "other-model"])
        {
            write_settings(path, model);
        }
        let before: Vec<_> = paths
            .iter()
            .map(|path| std::fs::read(path).unwrap())
            .collect();
        let report = ConflictChecker::new().check_conflicts().unwrap();

        assert!(report.warnings.is_empty());
        assert_eq!(report.conflicts.len(), 1);
        let conflict = &report.conflicts[0];
        assert_eq!(conflict.key, "model");
        assert_eq!(conflict.severity, ConflictSeverity::Warning);
        assert_eq!(conflict.platforms.len(), 3);
        assert_eq!(conflict.platforms[0].platform, "claude");
        assert_eq!(conflict.platforms[0].value, "claude-model");
        for (path, original) in paths.iter().zip(before) {
            assert_eq!(std::fs::read(path).unwrap(), original);
        }
        assert!(!home.root().join("logs").exists());
    }

    #[test]
    fn missing_selected_settings_are_empty_without_creation() {
        let home = TestHome::new_with_home_env();
        let report = ConflictChecker::new().check_conflicts().unwrap();
        assert!(report.conflicts.is_empty());
        assert!(report.warnings.is_empty());
        assert!(!home.settings_path().exists());
        assert!(!home.codex_dir().join("settings.json").exists());
        assert!(!home.home().join(".gemini").exists());
        assert!(!home.root().join("logs").exists());
    }
}
