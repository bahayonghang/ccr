// API Models - Request and Response structures

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

// Implement IntoResponse for Axum
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(self)).into_response()
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
    // 📊 使用统计和状态字段
    #[serde(default)]
    pub usage_count: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
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
    #[serde(default, deserialize_with = "deserialize_optional_trimmed_string")]
    pub model: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_trimmed_string")]
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
}

fn deserialize_optional_trimmed_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::UpdateConfigRequest;

    #[test]
    fn update_config_request_blank_model_deserializes_to_none() {
        let input = serde_json::json!({
            "name": "test",
            "base_url": "https://api.example.com",
            "auth_token": "sk-test",
            "model": ""
        });

        let req: UpdateConfigRequest = serde_json::from_value(input).unwrap();
        assert_eq!(req.model, None);
    }

    #[test]
    fn update_config_request_whitespace_model_deserializes_to_none() {
        let input = serde_json::json!({
            "name": "test",
            "base_url": "https://api.example.com",
            "auth_token": "sk-test",
            "model": "   "
        });

        let req: UpdateConfigRequest = serde_json::from_value(input).unwrap();
        assert_eq!(req.model, None);
    }

    #[test]
    fn update_config_request_model_is_trimmed() {
        let input = serde_json::json!({
            "name": "test",
            "base_url": "https://api.example.com",
            "auth_token": "sk-test",
            "model": "  gpt-5  "
        });

        let req: UpdateConfigRequest = serde_json::from_value(input).unwrap();
        assert_eq!(req.model.as_deref(), Some("gpt-5"));
    }
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

// ===== Version Management Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current_version: String,
    pub build_time: String,
    pub git_commit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateExecutionResponse {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub exit_code: i32,
}

// ===== MCP Server Management Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerWithName {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServersResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: Option<String>,
}

// ===== Slash Command Management Models =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub disabled: bool,
    /// Folder path (empty for root, "subfolder" for subfolder, etc.)
    #[serde(default)]
    pub folder: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlashCommandRequest {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub disabled: Option<bool>,
}

// ===== Agent Management Models =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub model: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    /// Folder path (empty for root, "kfc" for subfolder, etc.)
    #[serde(default)]
    pub folder: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub name: String,
    pub model: String,
    pub tools: Option<Vec<String>>,
    pub system_prompt: Option<String>,
    pub disabled: Option<bool>,
}

// ===== Plugin Management Models =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRequest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

// ===== Sync Models =====

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub success: bool,
    pub output: String,
    pub configured: bool,
    pub config: Option<SyncConfigDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfigDetails {
    pub enabled: bool,
    pub webdav_url: String,
    pub username: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub remote_file_exists: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncOperationRequest {
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncOperationResponse {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncInfoResponse {
    pub feature_name: String,
    pub description: String,
    pub supported_services: Vec<String>,
    pub setup_steps: Vec<String>,
    pub security_notes: Vec<String>,
}

// ===== Sync Folder Management Models (Multi-folder sync v2.5+) =====
// These structured models are reserved for future CLI output parsing
// Current implementation returns raw CLI output (see handlers/sync.rs:200)

/// Sync folder configuration
#[allow(dead_code)] // Reserved for structured CLI output parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolder {
    pub name: String,
    pub description: String,
    pub local_path: String,
    pub remote_path: String,
    pub enabled: bool,
    #[serde(default)]
    pub auto_sync: bool,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Request to add a new sync folder
#[derive(Debug, Serialize, Deserialize)]
pub struct AddSyncFolderRequest {
    pub name: String,
    pub local_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for folder list GET /api/sync/folders
#[allow(dead_code)] // Reserved for structured CLI output parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolderListResponse {
    pub success: bool,
    pub folders: Vec<SyncFolder>,
    pub total: usize,
    pub enabled_count: usize,
    pub disabled_count: usize,
}

/// Response for single folder GET /api/sync/folders/:name
#[allow(dead_code)] // Reserved for structured CLI output parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolderInfoResponse {
    pub success: bool,
    pub folder: SyncFolder,
    pub local_exists: bool,
    pub remote_exists: Option<bool>,
}

/// Response for folder operation (add/remove/enable/disable)
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolderOperationResponse {
    pub success: bool,
    pub message: String,
}

/// Response for folder sync operation (push/pull)
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolderSyncResponse {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration_ms: u64,
}

/// Response for batch operations (all push/pull/status)
#[allow(dead_code)] // Reserved for structured CLI output parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncBatchOperationResponse {
    pub success: bool,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<SyncFolderResult>,
}

/// Individual folder result in batch operation
#[allow(dead_code)] // Reserved for structured CLI output parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFolderResult {
    pub folder_name: String,
    pub success: bool,
    pub message: String,
}

// ===== Platform Management Models (Unified Mode) =====

/// Platform list item for GET /api/platforms
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformListItem {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    pub is_current: bool,
}

/// Current platform response for GET /api/platforms/current
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentPlatformResponse {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// Switch platform request for POST /api/platforms/switch
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchPlatformRequest {
    pub platform_name: String,
}

/// Platform detail response for GET /api/platforms/:name
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformDetailResponse {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    pub is_current: bool,
}

/// Update platform request for PUT /api/platforms/:name
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePlatformRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
}

/// Platform profile response for GET /api/platforms/:name/profile
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformProfileResponse {
    pub platform_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
}

/// Set platform profile request for POST /api/platforms/:name/profile
#[derive(Debug, Serialize, Deserialize)]
pub struct SetPlatformProfileRequest {
    pub profile_name: String,
}
