// Claude Code Settings Handler
// 读写 ~/.claude/settings.json 和 ~/.claude.json 中的用户配置

use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::managers::config::claude_manager::ClaudeConfigManager;

// ===== Response / Request =====

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettingsData {
    // Tab 1: 模型与推理
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_thinking_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_thinking_tokens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_level: Option<String>,

    // Tab 2: 权限
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_dangerous_mode_permission_prompt: Option<bool>,

    // Tab 3: 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,

    // Tab 4: UI 体验
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_turn_duration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinner_tips_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_progress_bar_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_spinner_tree: Option<bool>,

    // Tab 5: 沙箱安全
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxData>,

    // Tab 6: Git 归属
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<AttributionData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_co_authored_by: Option<bool>,

    // 其他
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_updates: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_updates_channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_gitignore: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsData {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_directories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SandboxData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_allow_bash_if_sandboxed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<SandboxNetworkData>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SandboxNetworkData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_local_binding: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AttributionData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<String>,
}

// ===== Helpers =====

fn get_str(other: &HashMap<String, Value>, key: &str) -> Option<String> {
    other.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn get_bool(other: &HashMap<String, Value>, key: &str) -> Option<bool> {
    other.get(key).and_then(|v| v.as_bool())
}

fn get_u64(other: &HashMap<String, Value>, key: &str) -> Option<u64> {
    other.get(key).and_then(|v| v.as_u64())
}

fn get_string_array(other: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
    other.get(key).and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
}

// ===== Handlers =====

/// GET /api/claude-settings
pub async fn get_settings() -> ApiResult<ApiSuccess<ClaudeSettingsData>> {
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {e}")))?;

    let other = &settings.other;

    // 从 env 提取特殊字段
    let env = &settings.env;
    let max_thinking_tokens = env.get("MAX_THINKING_TOKENS").cloned();
    let max_output_tokens = env.get("CLAUDE_CODE_MAX_OUTPUT_TOKENS").cloned();
    let effort_level = env.get("CLAUDE_CODE_EFFORT_LEVEL").cloned();

    // 解析 permissions (存储为 serde_json::Value)
    let permissions = settings.permissions.as_ref().map(|v| PermissionsData {
        allow: v
            .get("allow")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        deny: v
            .get("deny")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        default_mode: v
            .get("defaultMode")
            .and_then(|v| v.as_str())
            .map(String::from),
        additional_directories: v
            .get("additionalDirectories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            }),
    });

    // 解析 sandbox
    let sandbox =
        other.get("sandbox").and_then(|v| {
            let obj = v.as_object()?;
            let network = obj.get("network").and_then(|n| {
                let nobj = n.as_object()?;
                Some(SandboxNetworkData {
                    allow_local_binding: nobj.get("allowLocalBinding").and_then(|v| v.as_bool()),
                    allowed_domains: nobj.get("allowedDomains").and_then(|v| v.as_array()).map(
                        |arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        },
                    ),
                })
            });
            Some(SandboxData {
                enabled: obj.get("enabled").and_then(|v| v.as_bool()),
                auto_allow_bash_if_sandboxed: obj
                    .get("autoAllowBashIfSandboxed")
                    .and_then(|v| v.as_bool()),
                excluded_commands: obj.get("excludedCommands").and_then(|v| v.as_array()).map(
                    |arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    },
                ),
                network,
            })
        });

    // 解析 attribution
    let attribution = other.get("attribution").and_then(|v| {
        let obj = v.as_object()?;
        Some(AttributionData {
            commit: obj.get("commit").and_then(|v| v.as_str()).map(String::from),
            pr: obj.get("pr").and_then(|v| v.as_str()).map(String::from),
        })
    });

    // 读取 .claude.json
    let claude_config = ClaudeConfigManager::default()
        .and_then(|mgr| mgr.read())
        .ok();
    let claude_other = claude_config.as_ref().map(|c| &c.other);

    // 构建用户可见的 env（排除已提取为独立字段的变量）
    let visible_env: HashMap<String, String> = env
        .iter()
        .filter(|(k, _)| {
            !matches!(
                k.as_str(),
                "MAX_THINKING_TOKENS"
                    | "CLAUDE_CODE_MAX_OUTPUT_TOKENS"
                    | "CLAUDE_CODE_EFFORT_LEVEL"
            )
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let data = ClaudeSettingsData {
        model: get_str(other, "model"),
        available_models: get_string_array(other, "availableModels"),
        always_thinking_enabled: get_bool(other, "alwaysThinkingEnabled"),
        max_thinking_tokens,
        max_output_tokens,
        effort_level,
        permissions,
        skip_dangerous_mode_permission_prompt: get_bool(other, "skipDangerousModePermissionPrompt"),
        env: visible_env,
        theme: claude_other.and_then(|o| get_str(o, "theme")),
        language: get_str(other, "language"),
        show_turn_duration: get_bool(other, "showTurnDuration"),
        prefers_reduced_motion: get_bool(other, "prefersReducedMotion"),
        spinner_tips_enabled: get_bool(other, "spinnerTipsEnabled"),
        terminal_progress_bar_enabled: get_bool(other, "terminalProgressBarEnabled"),
        show_spinner_tree: claude_other.and_then(|o| get_bool(o, "showSpinnerTree")),
        sandbox,
        attribution,
        include_co_authored_by: get_bool(other, "includeCoAuthoredBy"),
        auto_updates: claude_other.and_then(|o| get_bool(o, "autoUpdates")),
        auto_updates_channel: get_str(other, "autoUpdatesChannel"),
        cleanup_period_days: get_u64(other, "cleanupPeriodDays"),
        respect_gitignore: get_bool(other, "respectGitignore"),
    };

    Ok(ApiSuccess(data))
}

/// PUT /api/claude-settings
pub async fn update_settings(
    Json(req): Json<ClaudeSettingsData>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {e}")))?;

    // --- 写入 settings.json ---

    // env: 合并用户 env + 特殊字段
    let mut env = req.env;
    if let Some(v) = &req.max_thinking_tokens {
        env.insert("MAX_THINKING_TOKENS".into(), v.clone());
    }
    if let Some(v) = &req.max_output_tokens {
        env.insert("CLAUDE_CODE_MAX_OUTPUT_TOKENS".into(), v.clone());
    }
    if let Some(v) = &req.effort_level {
        env.insert("CLAUDE_CODE_EFFORT_LEVEL".into(), v.clone());
    }
    settings.env = env;

    // permissions
    if let Some(p) = &req.permissions {
        let mut perm = serde_json::Map::new();
        perm.insert(
            "allow".into(),
            serde_json::to_value(&p.allow).unwrap_or_default(),
        );
        perm.insert(
            "deny".into(),
            serde_json::to_value(&p.deny).unwrap_or_default(),
        );
        if let Some(dm) = &p.default_mode {
            perm.insert("defaultMode".into(), Value::String(dm.clone()));
        }
        if let Some(ad) = &p.additional_directories {
            perm.insert(
                "additionalDirectories".into(),
                serde_json::to_value(ad).unwrap_or_default(),
            );
        }
        settings.permissions = Some(Value::Object(perm));
    }

    // other 字段的 set/remove helper
    let other = &mut settings.other;

    fn set_opt_str(m: &mut HashMap<String, Value>, k: &str, v: &Option<String>) {
        match v {
            Some(s) => {
                m.insert(k.into(), Value::String(s.clone()));
            }
            None => {
                m.remove(k);
            }
        }
    }
    fn set_opt_bool(m: &mut HashMap<String, Value>, k: &str, v: &Option<bool>) {
        match v {
            Some(b) => {
                m.insert(k.into(), Value::Bool(*b));
            }
            None => {
                m.remove(k);
            }
        }
    }
    fn set_opt_u64(m: &mut HashMap<String, Value>, k: &str, v: &Option<u64>) {
        match v {
            Some(n) => {
                m.insert(k.into(), serde_json::json!(*n));
            }
            None => {
                m.remove(k);
            }
        }
    }

    set_opt_str(other, "model", &req.model);
    if let Some(am) = &req.available_models {
        other.insert(
            "availableModels".into(),
            serde_json::to_value(am).unwrap_or_default(),
        );
    } else {
        other.remove("availableModels");
    }
    set_opt_bool(other, "alwaysThinkingEnabled", &req.always_thinking_enabled);
    set_opt_bool(
        other,
        "skipDangerousModePermissionPrompt",
        &req.skip_dangerous_mode_permission_prompt,
    );
    set_opt_str(other, "language", &req.language);
    set_opt_bool(other, "showTurnDuration", &req.show_turn_duration);
    set_opt_bool(other, "prefersReducedMotion", &req.prefers_reduced_motion);
    set_opt_bool(other, "spinnerTipsEnabled", &req.spinner_tips_enabled);
    set_opt_bool(
        other,
        "terminalProgressBarEnabled",
        &req.terminal_progress_bar_enabled,
    );
    set_opt_str(other, "autoUpdatesChannel", &req.auto_updates_channel);
    set_opt_u64(other, "cleanupPeriodDays", &req.cleanup_period_days);
    set_opt_bool(other, "respectGitignore", &req.respect_gitignore);
    set_opt_bool(other, "includeCoAuthoredBy", &req.include_co_authored_by);

    // sandbox
    if let Some(sb) = &req.sandbox {
        other.insert(
            "sandbox".into(),
            serde_json::to_value(sb).unwrap_or_default(),
        );
    } else {
        other.remove("sandbox");
    }

    // attribution
    if let Some(attr) = &req.attribution {
        other.insert(
            "attribution".into(),
            serde_json::to_value(attr).unwrap_or_default(),
        );
    } else {
        other.remove("attribution");
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {e}")))?;

    // --- 写入 .claude.json ---
    let mgr = ClaudeConfigManager::default()
        .map_err(|e| ApiError::internal(format!("Failed to init ClaudeConfigManager: {e}")))?;
    let mut claude_config = mgr
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read .claude.json: {e}")))?;

    let co = &mut claude_config.other;
    set_opt_str(co, "theme", &req.theme);
    set_opt_bool(co, "autoUpdates", &req.auto_updates);
    set_opt_bool(co, "showSpinnerTree", &req.show_spinner_tree);

    mgr.write(&claude_config)
        .map_err(|e| ApiError::internal(format!("Failed to save .claude.json: {e}")))?;

    Ok(ApiSuccess("Settings updated successfully"))
}
