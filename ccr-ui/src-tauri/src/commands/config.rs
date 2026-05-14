//! 配置管理命令 — list / switch / add / delete / rename / duplicate / validate / import / export / history。

use serde::{Deserialize, Serialize};
use tauri::State;

use ccr_config::{ConfigManager, ConfigSection, ConfigService, ImportMode, ProviderType};
use ccr_core::LockManager;
use ccr_store::HistoryService;

use crate::state::AppState;

/// 配置项详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_token: String, // masked
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    pub usage_count: u64,
    pub enabled: bool,
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub content: String,
    pub filename: String,
}

/// 屏蔽敏感 token（显示前 4 位和后 4 位）
fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        format!("{}...{}", &token[..4], &token[token.len() - 4..])
    }
}

// ── 配置管理 ──

#[tauri::command]
pub async fn list_configs(_state: State<'_, AppState>) -> Result<Vec<ConfigInfo>, String> {
    let result = tokio::task::spawn_blocking(move || {
        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        let configs: Vec<ConfigInfo> = config
            .sections
            .iter()
            .map(|(name, section)| ConfigInfo {
                name: name.clone(),
                description: section.description.clone().unwrap_or_default(),
                base_url: section.base_url.clone().unwrap_or_default(),
                auth_token: mask_token(&section.auth_token.clone().unwrap_or_default()),
                model: section.model.clone(),
                small_fast_model: section.small_fast_model.clone(),
                is_current: name == &config.current_config,
                is_default: name == &config.default_config,
                provider: section.provider.clone(),
                provider_type: section
                    .provider_type
                    .as_ref()
                    .map(|pt| pt.to_string_value().to_string()),
                account: section.account.clone(),
                tags: section.tags.clone(),
                usage_count: u64::from(section.usage_count()),
                enabled: section.is_enabled(),
            })
            .collect();

        Ok::<_, String>(configs)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    ccr::commands::switch_command(&name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Switched to config: {name}"))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_config(
    name: String,
    description: Option<String>,
    base_url: String,
    auth_token: String,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // 使用锁管理器确保并发安全
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {e}"))?;
        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;

        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        if config.sections.contains_key(&name) {
            return Err(format!("Config '{name}' already exists"));
        }

        let section = ConfigSection {
            description,
            base_url: Some(base_url),
            auth_token: Some(auth_token),
            model,
            small_fast_model,
            provider,
            provider_type: provider_type.as_deref().and_then(|s| match s {
                "official_relay" => Some(ProviderType::OfficialRelay),
                "third_party_model" => Some(ProviderType::ThirdPartyModel),
                _ => None,
            }),
            account,
            tags,
            usage_count: Some(0),
            enabled: Some(true),
            other: Default::default(),
            ..Default::default()
        };

        config.set_section(name.clone(), section);
        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {e}"))?;

        Ok(format!("Configuration '{name}' added successfully"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn delete_config(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {e}"))?;
        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;

        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        if config.current_config == name {
            return Err(format!("Cannot delete current config '{name}'"));
        }

        config
            .remove_section(&name)
            .map_err(|e| format!("Failed to remove config: {e}"))?;

        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {e}"))?;

        Ok(format!("Configuration '{name}' deleted successfully"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn rename_config(old_name: String, new_name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {e}"))?;
        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;

        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        if !config.sections.contains_key(&old_name) {
            return Err(format!("Config '{old_name}' not found"));
        }

        if config.sections.contains_key(&new_name) {
            return Err(format!("Config '{new_name}' already exists"));
        }

        // 取出旧配置节
        let section = config
            .sections
            .shift_remove(&old_name)
            .ok_or_else(|| format!("Config '{old_name}' not found"))?;

        // 用新名称插入
        config.set_section(new_name.clone(), section);

        // 更新 current/default 引用
        if config.current_config == old_name {
            config.current_config = new_name.clone();
        }
        if config.default_config == old_name {
            config.default_config = new_name.clone();
        }

        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {e}"))?;

        Ok(format!(
            "Configuration '{old_name}' renamed to '{new_name}'"
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn duplicate_config(source: String, target: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {e}"))?;
        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;

        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        if !config.sections.contains_key(&source) {
            return Err(format!("Config '{source}' not found"));
        }

        if config.sections.contains_key(&target) {
            return Err(format!("Config '{target}' already exists"));
        }

        let section = config
            .sections
            .get(&source)
            .ok_or_else(|| format!("Config '{source}' not found"))?
            .clone();

        config.set_section(target.clone(), section);

        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {e}"))?;

        Ok(format!("Configuration '{source}' duplicated as '{target}'"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn validate_configs() -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;
        service
            .validate_all()
            .map_err(|e| format!("Validation failed: {e}"))?;
        Ok::<_, String>("All configurations are valid".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn import_config(
    content: String,
    mode: String,
    backup: bool,
) -> Result<ImportResult, String> {
    tokio::task::spawn_blocking(move || {
        let import_mode = match mode.to_lowercase().as_str() {
            "merge" => ImportMode::Merge,
            "replace" => ImportMode::Replace,
            _ => return Err(format!("Invalid import mode: {mode}")),
        };

        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;

        let result = service
            .import_config(&content, import_mode, backup)
            .map_err(|e| format!("Import failed: {e}"))?;

        Ok(ImportResult {
            added: result.added,
            updated: result.updated,
            skipped: result.skipped,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn export_config(include_secrets: bool) -> Result<ExportResult, String> {
    tokio::task::spawn_blocking(move || {
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {e}"))?;

        let content = service
            .export_config(include_secrets)
            .map_err(|e| format!("Failed to export config: {e}"))?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ccr_config_export_{timestamp}.toml");

        Ok(ExportResult { content, filename })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 历史记录 ──

#[tauri::command]
pub async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntry>, String> {
    let limit = limit.unwrap_or(100);

    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {e}"))?;

    let entries = service
        .get_recent_async(limit)
        .await
        .map_err(|e| format!("Failed to get history: {e}"))?;

    let json_entries: Vec<HistoryEntry> = entries
        .into_iter()
        .map(|e| HistoryEntry {
            id: e.id.to_string(),
            timestamp: e.timestamp.to_rfc3339(),
            operation: format!("{:?}", e.operation),
            actor: e.actor,
        })
        .collect();

    Ok(json_entries)
}

#[tauri::command]
pub async fn clear_history() -> Result<String, String> {
    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {e}"))?;
    service
        .clear_async()
        .await
        .map_err(|e| format!("Failed to clear history: {e}"))?;
    Ok("History cleared successfully".to_string())
}

#[tauri::command]
pub async fn update_config(name: String, data: serde_json::Value) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let lock_manager = LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {e}"))?;
        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;

        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {e}"))?;
        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {e}"))?;

        let section = config
            .sections
            .get_mut(&name)
            .ok_or_else(|| format!("Config '{name}' not found"))?;

        // 按字段名称更新对应字段
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    "description" => {
                        section.description = value.as_str().map(str::to_string);
                    }
                    "base_url" => {
                        section.base_url = value.as_str().map(str::to_string);
                    }
                    "auth_token" => {
                        section.auth_token = value.as_str().map(str::to_string);
                    }
                    "model" => {
                        section.model = value.as_str().map(str::to_string);
                    }
                    "small_fast_model" => {
                        section.small_fast_model = value.as_str().map(str::to_string);
                    }
                    "provider" => {
                        section.provider = value.as_str().map(str::to_string);
                    }
                    "account" => {
                        section.account = value.as_str().map(str::to_string);
                    }
                    "enabled" => {
                        if let Some(b) = value.as_bool() {
                            section.enabled = Some(b);
                        }
                    }
                    _ => {}
                }
            }
        }

        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {e}"))?;

        Ok(format!("Configuration '{name}' updated successfully"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── 退出确认设置 ──

#[tauri::command]
pub async fn get_skip_exit_confirm(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(!state.desktop_shell_preferences().confirm_before_exit)
}

#[tauri::command]
pub async fn set_skip_exit_confirm(skip: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.update_desktop_shell_preferences(|settings| {
        settings.confirm_before_exit = !skip;
    })?;
    Ok(())
}

// ── 备份清理 ──

#[tauri::command]
pub async fn clean_backups() -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // CCR 配置文件位于 ~/.ccr/ 目录下
        let home =
            dirs::home_dir().ok_or_else(|| "Unable to determine home directory".to_string())?;
        let ccr_dir = home.join(".ccr");

        if !ccr_dir.exists() {
            return Ok("No CCR config directory found".to_string());
        }

        let mut deleted = 0usize;
        // 递归扫描 .bak / .backup 文件
        for entry in walkdir::WalkDir::new(&ccr_dir)
            .max_depth(3)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if (ext == "bak" || ext == "backup") && std::fs::remove_file(path).is_ok() {
                deleted += 1;
            }
        }

        Ok(format!("Deleted {} backup file(s)", deleted))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
