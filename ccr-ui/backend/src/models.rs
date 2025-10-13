// API Models - Request and Response structures

use serde::{Deserialize, Serialize};

// ===== Generic API Response =====

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

// ===== Command Execution Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandListResponse {
    pub commands: Vec<CommandInfo>,
}

// ===== Config Management Models =====

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
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigListResponse {
    pub current_config: String,
    pub default_config: String,
    pub configs: Vec<ConfigItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchRequest {
    pub config_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
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
}

// ===== History Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntryJson {
    pub id: String,
    pub timestamp: String,
    pub operation: String,
    pub actor: String,
    pub from_config: Option<String>,
    pub to_config: Option<String>,
    pub changes: Vec<EnvChangeJson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvChangeJson {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub entries: Vec<HistoryEntryJson>,
    pub total: usize,
}

// ===== System Info Models =====

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

// ===== Clean Backup Models =====

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanResponse {
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub total_size_mb: f64,
    pub dry_run: bool,
}

// ===== Export/Import Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    #[serde(default = "default_include_secrets")]
    pub include_secrets: bool,
}

fn default_include_secrets() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub content: String,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRequest {
    pub content: String,
    #[serde(default = "default_import_mode")]
    pub mode: String,
    #[serde(default = "default_backup")]
    pub backup: bool,
}

fn default_import_mode() -> String {
    "merge".to_string()
}

fn default_backup() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResponse {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

