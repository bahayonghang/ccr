// 🎯 Tauri Commands 模块
// 将前端请求桥接到 CCR 服务层
//
// 遵循 KISS 原则：简单、直接、优雅！(￣▽￣)／

use crate::{ConfigService, HistoryService, SettingsService};
use ccr::managers::config::{ConfigSection, ProviderType};
use serde::{Deserialize, Serialize};

// ============================================================================
// 📊 数据模型定义 - 用于前端展示
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub name: String,
    pub description: String,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigList {
    pub current_config: String,
    pub default_config: String,
    pub configs: Vec<ConfigInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub from_config: Option<String>,
    pub to_config: Option<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub config_path: String,
    pub settings_path: String,
    pub backups_path: String,
}

// ============================================================================
// 🎨 配置管理 Commands
// ============================================================================

/// 📋 列出所有配置
#[tauri::command]
pub async fn list_configs() -> Result<ConfigList, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let config_list = service.list_configs().map_err(|e| e.to_string())?;

    // 转换为前端格式
    Ok(ConfigList {
        current_config: config_list.current_config,
        default_config: config_list.default_config,
        configs: config_list
            .configs
            .into_iter()
            .map(|info| ConfigInfo {
                name: info.name,
                description: info.description,
                base_url: info.base_url,
                auth_token: info.auth_token,
                model: info.model,
                small_fast_model: info.small_fast_model,
                is_current: info.is_current,
                is_default: info.is_default,
                provider: info.provider,
                provider_type: info.provider_type,
                account: info.account,
                tags: info.tags,
            })
            .collect(),
    })
}

/// 🔍 获取当前配置
#[tauri::command]
pub async fn get_current_config() -> Result<ConfigInfo, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let info = service.get_current().map_err(|e| e.to_string())?;

    Ok(ConfigInfo {
        name: info.name,
        description: info.description,
        base_url: info.base_url,
        auth_token: info.auth_token,
        model: info.model,
        small_fast_model: info.small_fast_model,
        is_current: info.is_current,
        is_default: info.is_default,
        provider: info.provider,
        provider_type: info.provider_type,
        account: info.account,
        tags: info.tags,
    })
}

/// 🔍 获取指定配置
#[tauri::command]
pub async fn get_config(name: String) -> Result<ConfigInfo, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let info = service.get_config(&name).map_err(|e| e.to_string())?;

    Ok(ConfigInfo {
        name: info.name,
        description: info.description,
        base_url: info.base_url,
        auth_token: info.auth_token,
        model: info.model,
        small_fast_model: info.small_fast_model,
        is_current: info.is_current,
        is_default: info.is_default,
        provider: info.provider,
        provider_type: info.provider_type,
        account: info.account,
        tags: info.tags,
    })
}

/// 🔄 切换配置
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    let config_service = ConfigService::default().map_err(|e| e.to_string())?;
    let settings_service = SettingsService::default().map_err(|e| e.to_string())?;

    // 加载配置文件
    let config = config_service.load_config().map_err(|e| e.to_string())?;
    let section = config
        .sections
        .get(&name)
        .ok_or(format!("配置 {} 不存在", name))?;

    // 应用配置到 settings.json
    settings_service
        .apply_config(section)
        .map_err(|e| e.to_string())?;

    // 更新当前配置标记
    config_service
        .set_current(&name)
        .map_err(|e| e.to_string())?;

    Ok(format!("✅ 成功切换到配置: {}", name))
}

/// ➕ 创建新配置
#[tauri::command]
pub async fn create_config(
    name: String,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;

    // 解析 ProviderType
    let parsed_provider_type = provider_type.and_then(|s| {
        match s.to_lowercase().as_str() {
            "officialrelay" | "official_relay" | "official-relay" => Some(ProviderType::OfficialRelay),
            "thirdpartymodel" | "third_party_model" | "third-party-model" => Some(ProviderType::ThirdPartyModel),
            _ => None
        }
    });

    let section = ConfigSection {
        description,
        base_url,
        auth_token,
        model,
        small_fast_model,
        provider,
        provider_type: parsed_provider_type,
        account,
        tags,
    };

    service
        .add_config(name.clone(), section)
        .map_err(|e| e.to_string())?;
    Ok(format!("✅ 成功创建配置: {}", name))
}

/// ✏️ 更新配置
#[tauri::command]
pub async fn update_config(
    old_name: String,
    new_name: String,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;

    // 解析 ProviderType
    let parsed_provider_type = provider_type.and_then(|s| {
        match s.to_lowercase().as_str() {
            "officialrelay" | "official_relay" | "official-relay" => Some(ProviderType::OfficialRelay),
            "thirdpartymodel" | "third_party_model" | "third-party-model" => Some(ProviderType::ThirdPartyModel),
            _ => None
        }
    });

    let section = ConfigSection {
        description,
        base_url,
        auth_token,
        model,
        small_fast_model,
        provider,
        provider_type: parsed_provider_type,
        account,
        tags,
    };

    service
        .update_config(&old_name, new_name.clone(), section)
        .map_err(|e| e.to_string())?;
    Ok(format!("✅ 成功更新配置: {}", new_name))
}

/// 🗑️ 删除配置
#[tauri::command]
pub async fn delete_config(name: String) -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    service
        .delete_config(&name)
        .map_err(|e| e.to_string())?;
    Ok(format!("✅ 成功删除配置: {}", name))
}

/// 📥 导入配置
#[tauri::command]
pub async fn import_config(content: String, merge: bool, backup: bool) -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let mode = if merge {
        ccr::services::config_service::ImportMode::Merge
    } else {
        ccr::services::config_service::ImportMode::Replace
    };

    let result = service
        .import_config(&content, mode, backup)
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "✅ 导入完成！新增: {}, 更新: {}, 跳过: {}",
        result.added, result.updated, result.skipped
    ))
}

/// 📤 导出配置
#[tauri::command]
pub async fn export_config(include_secrets: bool) -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let content = service
        .export_config(include_secrets)
        .map_err(|e| e.to_string())?;
    Ok(content)
}

/// ✅ 验证所有配置
#[tauri::command]
pub async fn validate_all() -> Result<String, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    let report = service.validate_all().map_err(|e| e.to_string())?;

    Ok(format!(
        "验证完成！有效: {}, 无效: {}",
        report.valid_count, report.invalid_count
    ))
}

// ============================================================================
// 📜 历史记录 Commands
// ============================================================================

/// 📚 获取操作历史
#[tauri::command]
pub async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntry>, String> {
    let service = HistoryService::default().map_err(|e| e.to_string())?;
    let entries = service
        .get_recent(limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    Ok(entries
        .into_iter()
        .map(|e| HistoryEntry {
            id: e.id,
            timestamp: e.timestamp.to_rfc3339(),
            operation: e.operation.as_str().to_string(),
            from_config: e.details.from_config,
            to_config: e.details.to_config,
            actor: e.actor,
        })
        .collect())
}

// ============================================================================
// 💾 备份管理 Commands
// ============================================================================

/// 📦 列出所有备份
#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let service = SettingsService::default().map_err(|e| e.to_string())?;
    let backups = service.list_backups().map_err(|e| e.to_string())?;

    Ok(backups
        .into_iter()
        .map(|b| BackupInfo {
            filename: b.filename,
            path: b.path.display().to_string(),
            created_at: humantime::format_rfc3339(b.created_at).to_string(),
            size: b.size_bytes,
        })
        .collect())
}

/// ♻️ 恢复备份
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<String, String> {
    let service = SettingsService::default().map_err(|e| e.to_string())?;
    service
        .restore_settings(std::path::Path::new(&backup_path))
        .map_err(|e| e.to_string())?;
    Ok(format!("✅ 成功从备份恢复: {}", backup_path))
}

// ============================================================================
// 🖥️ 系统信息 Commands
// ============================================================================

/// 💻 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    use std::env;

    let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户目录".to_string())?;
    let config_path = home_dir.join(".ccs_config.toml");
    let settings_path = home_dir.join(".claude/settings.json");
    let backups_path = home_dir.join(".claude/backups");

    Ok(SystemInfo {
        hostname: whoami::fallible::hostname().unwrap_or_else(|_| "Unknown".to_string()),
        username: whoami::username(),
        os: format!("{} {}", whoami::platform(), env::consts::ARCH),
        config_path: config_path.display().to_string(),
        settings_path: settings_path.display().to_string(),
        backups_path: backups_path.display().to_string(),
    })
}
