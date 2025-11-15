// 📦 Web API 数据模型
// 定义所有 API 请求和响应的数据结构
// 部分模型为将来的 Web API 扩展准备

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 📦 API 响应结构
///
/// 统一的 API 响应格式,包含成功状态、数据和错误消息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// ✅ 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    /// ❌ 创建错误响应
    pub fn error_without_data(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

/// 配置列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigListResponse {
    pub current_config: String,
    pub default_config: String,
    pub configs: Vec<ConfigItem>,
}

/// 配置项
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigItem {
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_token: String,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    // === 🆕 分类字段 ===
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    // === 📊 使用统计和状态字段 ===
    pub usage_count: u32,
    pub enabled: bool,
}

/// 切换配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchRequest {
    pub config_name: String,
}

/// 清理备份请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanRequest {
    #[serde(default = "default_days")]
    pub days: u64,
    #[serde(default)]
    pub dry_run: bool,
}

fn default_days() -> u64 {
    7
}

/// 清理备份响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanResponse {
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub total_size_mb: f64,
    pub dry_run: bool,
}

/// 更新配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    // === 🆕 分类字段 ===
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Codex profiles 列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexProfilesResponse {
    pub current_profile: Option<String>,
    pub profiles: Vec<CodexProfileItem>,
}

/// Codex profile 项
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexProfileItem {
    pub name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    pub api_mode: Option<String>,
    pub wire_api: Option<String>,
    pub env_key: Option<String>,
    pub requires_openai_auth: Option<bool>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub model_reasoning_effort: Option<String>,
    pub organization: Option<String>,
    pub is_current: bool,
}

/// Codex profile 创建/更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexProfileRequest {
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    pub api_mode: Option<String>,
    pub wire_api: Option<String>,
    pub env_key: Option<String>,
    pub requires_openai_auth: Option<bool>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub model_reasoning_effort: Option<String>,
    pub organization: Option<String>,
}

/// 历史记录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntryJson>,
    pub total: usize,
}

/// 历史记录条目 JSON 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntryJson {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
    pub from_config: Option<String>,
    pub to_config: Option<String>,
    pub changes: Vec<EnvChangeJson>,
}

/// 环境变量更改 JSON 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvChangeJson {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Settings 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub settings: serde_json::Value,
}

/// Settings 备份响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsBackupsResponse {
    pub backups: Vec<BackupItem>,
}

/// 备份项
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupItem {
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
}

/// 恢复 Settings 请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreSettingsRequest {
    pub backup_path: String,
}

/// 导出配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    #[serde(default = "default_include_secrets")]
    pub include_secrets: bool,
}

fn default_include_secrets() -> bool {
    true
}

/// 导出配置响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub content: String,
    pub filename: String,
}

/// 导入配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRequest {
    pub content: String,
    #[serde(default = "default_import_mode")]
    pub mode: String, // "merge" or "replace"
    #[serde(default = "default_backup")]
    pub backup: bool,
}

fn default_import_mode() -> String {
    "merge".to_string()
}

fn default_backup() -> bool {
    true
}

/// 导入配置响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResponse {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

/// 系统信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_usage: f32,
    pub total_memory_gb: f64,
    pub used_memory_gb: f64,
    pub memory_usage_percent: f32,
    pub total_swap_gb: f64,
    pub used_swap_gb: f64,
    pub uptime_seconds: u64,
}

// ===== 🆕 平台管理模型 (Unified Mode) =====

/// 平台信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfoResponse {
    pub mode: String, // "legacy" or "unified"
    pub current_platform: Option<String>,
    pub available_platforms: Option<Vec<PlatformItem>>,
}

/// 平台项
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformItem {
    pub name: String,
    pub enabled: bool,
    pub current_profile: Option<String>,
    pub description: Option<String>,
    pub last_used: Option<String>,
    pub is_current: bool,
}

/// 切换平台请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchPlatformRequest {
    pub platform_name: String,
}

// ===== ☁️ 同步相关模型 =====

/// 同步状态响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub configured: bool,
    pub enabled: bool,
    pub webdav_url: Option<String>,
    pub username: Option<String>,
    pub remote_path: Option<String>,
    pub auto_sync: Option<bool>,
    pub sync_type: Option<String>, // "directory" or "file"
    pub local_path: Option<String>,
    pub remote_exists: Option<bool>,
}

/// 设置/更新同步配置请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfigRequest {
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_remote_path")]
    pub remote_path: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_sync: bool,
}

fn default_remote_path() -> String {
    "/ccr/".to_string()
}

/// 同步操作请求（push/pull）
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncOperationRequest {
    #[serde(default)]
    pub force: bool,
}

/// 同步操作响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncOperationResponse {
    pub message: String,
}
