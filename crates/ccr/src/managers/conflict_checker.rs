use crate::core::error::{CcrError, Result};
use crate::models::Platform;
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
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;

        let settings_path = match platform {
            Platform::Claude => home.join(".claude").join("settings.json"),
            Platform::Codex => home.join(".codex").join("settings.json"),
            Platform::Gemini => home.join(".gemini").join("config.json"),
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
