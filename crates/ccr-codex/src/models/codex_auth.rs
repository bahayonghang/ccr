// 🔐 Codex Auth 数据模型
// 用于管理 Codex CLI 的多账号登录状态
//
// 核心职责:
// - 📋 定义账号元数据结构
// - 📦 定义注册表结构
// - 🎨 定义 TUI 状态枚举

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

// Re-export shared types from ccr-types
pub use ccr_types::{LoginState, TokenFreshness};

/// OpenAI 认证方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenAiAuthMethod {
    Chatgpt,
    Api,
}

/// Codex profile 认证模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexProfileAuthMode {
    OpenAiChatgpt,
    OpenAiApiKey,
    ProviderEnvKey,
    NoAuth,
}

impl CodexProfileAuthMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodexProfileAuthMode::OpenAiChatgpt => "openai_chatgpt",
            CodexProfileAuthMode::OpenAiApiKey => "openai_api_key",
            CodexProfileAuthMode::ProviderEnvKey => "provider_env_key",
            CodexProfileAuthMode::NoAuth => "no_auth",
        }
    }

    pub fn uses_openai_auth(&self) -> bool {
        matches!(
            self,
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::OpenAiApiKey
        )
    }

    pub fn openai_login_method(&self) -> Option<OpenAiAuthMethod> {
        match self {
            CodexProfileAuthMode::OpenAiChatgpt => Some(OpenAiAuthMethod::Chatgpt),
            CodexProfileAuthMode::OpenAiApiKey => Some(OpenAiAuthMethod::Api),
            CodexProfileAuthMode::ProviderEnvKey | CodexProfileAuthMode::NoAuth => None,
        }
    }
}

/// Profile 级别的 secret 记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexProfileSecret {
    pub auth_mode: CodexProfileAuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    pub secret: String,
    pub updated_at: DateTime<Utc>,
}

/// CCR 管理的 Codex profile secret store
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodexProfileSecretStore {
    #[serde(default = "default_secret_store_version")]
    pub version: String,
    #[serde(default)]
    pub profiles: IndexMap<String, CodexProfileSecret>,
}

fn default_secret_store_version() -> String {
    "1.0".to_string()
}

/// 认证意图
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthIntent {
    /// OpenAI 认证（ChatGPT 登录 / API Key）
    OpenAiAuth { method: OpenAiAuthMethod },
    /// 自定义 provider 的 env_key 认证
    ProviderEnvKey { env_key: String },
    /// 无认证（本地模型等）
    NoAuth,
}

/// 按认证意图规范化 auth.json 字段
///
/// 仅保留当前认证模式需要的字段，并丢弃历史/未知字段。
pub fn normalize_auth_map_for_intent(
    intent: &AuthIntent,
    auth: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut normalized = JsonMap::new();

    match intent {
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Api,
        } => {
            if let Some(api_key) = auth
                .get("OPENAI_API_KEY")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                normalized.insert(
                    "OPENAI_API_KEY".to_string(),
                    JsonValue::String(api_key.to_string()),
                );
            }
        }
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Chatgpt,
        } => {
            if let Some(tokens) = auth.get("tokens").and_then(JsonValue::as_object) {
                let mut normalized_tokens = JsonMap::new();

                for field in ["id_token", "access_token", "refresh_token", "account_id"] {
                    if let Some(value) = tokens
                        .get(field)
                        .and_then(JsonValue::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        normalized_tokens
                            .insert(field.to_string(), JsonValue::String(value.to_string()));
                    }
                }

                if !normalized_tokens.is_empty() {
                    normalized.insert("tokens".to_string(), JsonValue::Object(normalized_tokens));
                }
            }

            if let Some(last_refresh) = auth
                .get("last_refresh")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                normalized.insert(
                    "last_refresh".to_string(),
                    JsonValue::String(last_refresh.to_string()),
                );
            }
        }
        AuthIntent::ProviderEnvKey { env_key } => {
            if let Some(value) = auth
                .get(env_key)
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                normalized.insert(env_key.clone(), JsonValue::String(value.to_string()));
            }
        }
        AuthIntent::NoAuth => {}
    }

    normalized
}

/// 凭据存储类型（对齐 Codex cli_auth_credentials_store）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStoreKind {
    File,
    Keyring,
    Auto,
}

impl CredentialStoreKind {
    pub fn from_config_value(value: Option<&str>) -> Self {
        match value.unwrap_or("auto").to_ascii_lowercase().as_str() {
            "file" => Self::File,
            "keyring" => Self::Keyring,
            _ => Self::Auto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialStoreKind::File => "file",
            CredentialStoreKind::Keyring => "keyring",
            CredentialStoreKind::Auto => "auto",
        }
    }
}

/// 认证状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthStateStatus {
    Valid,
    Invalid,
    Missing,
    Unsupported,
}

/// 认证状态快照
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthState {
    pub intent: AuthIntent,
    pub store: CredentialStoreKind,
    pub status: AuthStateStatus,
    pub reason: String,
}

/// Codex 账号元数据
///
/// 存储在 auth_registry.toml 中的账号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthAccount {
    /// 账号描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 账号 ID (从 auth.json 提取)
    pub account_id: String,

    /// OpenAI 登录方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<OpenAiAuthMethod>,

    /// API Key 账号绑定的 Base URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,

    /// API Key 账号绑定的供应商名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_provider_name: Option<String>,

    /// 邮箱 (脱敏后存储)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// OpenAI 账号类型 (plus / team / pro 20x)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,

    /// 保存时间
    pub saved_at: DateTime<Utc>,

    /// 最后使用时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间 (从 auth.json 提取)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,

    /// 到期时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Codex 使用量归因激活记录
///
/// CCR 在账号切换时记录当前激活账号，后续可按时间窗将本地 usage
/// 归因到对应账号。该账本是 CCR 自有能力，不依赖 Codex 原生日志格式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexUsageActivation {
    /// CCR 保存的账号名（用于展示）
    pub account_name: String,

    /// OpenAI account_id（用于稳定匹配，避免重命名影响历史归因）
    pub account_id: String,

    /// 生效时间
    pub started_at: DateTime<Utc>,
}

/// Codex 账号注册表
///
/// 存储在 ~/.ccr/platforms/codex/auth_registry.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthRegistry {
    /// 版本号
    #[serde(default = "default_version")]
    pub version: String,

    /// 当前激活的账号名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_auth: Option<String>,

    /// 所有账号
    #[serde(default)]
    pub accounts: IndexMap<String, CodexAuthAccount>,

    /// CCR 维护的 usage 归因账本
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub usage_ledger: Vec<CodexUsageActivation>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for CodexAuthRegistry {
    fn default() -> Self {
        Self {
            version: default_version(),
            current_auth: None,
            accounts: IndexMap::new(),
            usage_ledger: Vec::new(),
        }
    }
}

impl CodexAuthRegistry {
    /// 记录一次账号激活，用于后续按时间窗归因本地 usage。
    ///
    /// 连续重复激活同一账号时不追加新记录，避免账本噪音。
    pub fn record_usage_activation(
        &mut self,
        account_name: impl Into<String>,
        account_id: impl Into<String>,
        started_at: DateTime<Utc>,
    ) {
        let account_name = account_name.into();
        let account_id = account_id.into();

        if self.usage_ledger.last().is_some_and(|entry| {
            entry.account_name == account_name && entry.account_id == account_id
        }) {
            return;
        }

        self.usage_ledger.push(CodexUsageActivation {
            account_name,
            account_id,
            started_at,
        });
    }
}

/// 账号列表项 (用于显示)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CodexAuthItem {
    /// 账号名称
    pub name: String,

    /// 账号描述
    pub description: Option<String>,

    /// 脱敏后的邮箱
    pub email: Option<String>,

    /// OpenAI 账号类型
    pub plan_type: Option<String>,

    /// 是否为当前激活账号
    pub is_current: bool,

    /// 是否为虚拟项 (未保存的 default)
    pub is_virtual: bool,

    /// 添加时间
    pub saved_at: Option<DateTime<Utc>>,

    /// 最后使用时间
    #[allow(dead_code)]
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间
    pub last_refresh: Option<DateTime<Utc>>,

    /// Token 新鲜度
    pub freshness: TokenFreshness,

    /// 到期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// Codex 当前运行时的控制模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexRuntimeMode {
    /// 当前由 Profile 独立控制（常见于 provider env key / 无认证 profile）
    ProfileOnly,
    /// 当前由 Profile 控制路由，Auth 控制身份
    ProfileWithAuth,
    /// 当前 Profile 需要 OpenAI Auth，但运行时尚未具备有效凭据
    ProfilePendingAuth,
    /// 当前只有 runtime/auth 生效，未稳定绑定到某个 profile
    RuntimeOnly,
    /// 当前既没有可解析的 profile，也没有有效 auth
    Unresolved,
}

impl CodexRuntimeMode {
    pub fn label(&self) -> &'static str {
        match self {
            CodexRuntimeMode::ProfileOnly => "Profile 驱动",
            CodexRuntimeMode::ProfileWithAuth => "Profile 路由 + Auth 身份",
            CodexRuntimeMode::ProfilePendingAuth => "Profile 路由，等待 Auth",
            CodexRuntimeMode::RuntimeOnly => "仅 Runtime/Auth 生效",
            CodexRuntimeMode::Unresolved => "未解析",
        }
    }
}

/// Codex 运行时摘要（用于统一解释当前是谁在控制 Codex）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexRuntimeSummary {
    pub mode: CodexRuntimeMode,
    pub current_profile_name: Option<String>,
    pub current_profile_provider: Option<String>,
    pub current_profile_auth_mode: Option<CodexProfileAuthMode>,
    pub current_profile_auth_source: Option<String>,
    pub current_auth_name: Option<String>,
    pub login_state: LoginState,
    pub auth_state: AuthState,
}

impl CodexRuntimeSummary {
    pub fn profile_label(&self) -> String {
        let Some(name) = &self.current_profile_name else {
            return "未绑定".to_string();
        };

        let mut parts = vec![name.clone()];

        if let Some(provider) = self
            .current_profile_provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            parts.push(provider.to_string());
        }

        match self.current_profile_auth_mode {
            Some(CodexProfileAuthMode::ProviderEnvKey) => {
                if let Some(source) = self
                    .current_profile_auth_source
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    parts.push(source.to_string());
                }
            }
            Some(mode) => parts.push(mode.as_str().to_string()),
            None => {}
        }

        parts.join(" · ")
    }

    pub fn auth_label(&self) -> String {
        match &self.login_state {
            LoginState::LoggedInSaved(name) => {
                format!("{name} · {}", auth_intent_label(&self.auth_state.intent))
            }
            LoginState::LoggedInUnsaved => {
                format!(
                    "未保存账号 · {}",
                    auth_intent_label(&self.auth_state.intent)
                )
            }
            LoginState::ApiKeyActive => auth_intent_label(&self.auth_state.intent),
            LoginState::ProviderKeyActive { env_key } => format!("Provider / {env_key}"),
            LoginState::NotLoggedIn => {
                if matches!(
                    self.current_profile_auth_mode,
                    Some(CodexProfileAuthMode::ProviderEnvKey)
                ) {
                    return self
                        .current_profile_auth_source
                        .as_deref()
                        .map(profile_auth_source_label)
                        .unwrap_or_else(|| "Provider Key".to_string());
                }

                if matches!(
                    self.current_profile_auth_mode,
                    Some(CodexProfileAuthMode::NoAuth)
                ) {
                    return "No Auth".to_string();
                }

                self.current_profile_auth_mode
                    .map(expected_auth_label)
                    .map(|label| format!("未登录 · {label}"))
                    .unwrap_or_else(|| "未登录".to_string())
            }
            LoginState::Unknown { type_name, .. } => format!("未知状态 · {type_name}"),
        }
    }
}

fn auth_intent_label(intent: &AuthIntent) -> String {
    match intent {
        AuthIntent::OpenAiAuth { method } => match method {
            OpenAiAuthMethod::Chatgpt => "OpenAI / ChatGPT".to_string(),
            OpenAiAuthMethod::Api => "OpenAI / API Key".to_string(),
        },
        AuthIntent::ProviderEnvKey { env_key } => format!("Provider / {env_key}"),
        AuthIntent::NoAuth => "No Auth".to_string(),
    }
}

fn expected_auth_label(mode: CodexProfileAuthMode) -> String {
    match mode {
        CodexProfileAuthMode::OpenAiChatgpt => "OpenAI / ChatGPT".to_string(),
        CodexProfileAuthMode::OpenAiApiKey => "OpenAI / API Key".to_string(),
        CodexProfileAuthMode::ProviderEnvKey => "Provider Key".to_string(),
        CodexProfileAuthMode::NoAuth => "No Auth".to_string(),
    }
}

fn profile_auth_source_label(source: &str) -> String {
    source
        .strip_prefix("provider:")
        .map(|env_key| format!("Provider / {env_key}"))
        .unwrap_or_else(|| source.to_string())
}

/// 当前 auth.json 解析信息
#[derive(Debug, Clone)]
pub struct CurrentAuthInfo {
    /// 账号 ID
    pub account_id: String,

    /// OpenAI 登录方式（如果适用）
    #[allow(dead_code)]
    pub auth_method: Option<OpenAiAuthMethod>,

    /// 邮箱 (原始)
    pub email: Option<String>,

    /// OpenAI 账号类型
    pub plan_type: Option<String>,

    /// 最后刷新时间
    pub last_refresh: Option<DateTime<Utc>>,

    /// Token 新鲜度
    pub freshness: TokenFreshness,
}

/// TUI 输入模式
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum InputMode {
    /// 正常模式
    #[default]
    Normal,

    /// 保存账号输入模式
    SaveInput,

    /// 删除确认模式
    DeleteConfirm,
}

/// Codex auth.json 文件结构
///
/// 用于解析 ~/.codex/auth.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthJson {
    /// OpenAI API Key (可选)
    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,

    /// OAuth tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<CodexAuthTokens>,

    /// 最后刷新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<String>,
}

/// Codex OAuth tokens 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthTokens {
    /// ID Token (JWT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,

    /// Access Token (JWT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    /// Refresh Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// 账号 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

// ==================== 导入/导出数据结构 ====================

/// Codex Auth 导出格式
///
/// 用于导入/导出账号数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthExport {
    /// 导出格式版本
    #[serde(default = "default_export_version")]
    pub version: String,

    /// 导出时间
    pub exported_at: DateTime<Utc>,

    /// 账号数据
    pub accounts: IndexMap<String, CodexAuthExportAccount>,
}

fn default_export_version() -> String {
    "1.0".to_string()
}

/// 导出的账号数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthExportAccount {
    /// 账号描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 账号 ID (从 auth.json 提取)
    pub account_id: String,

    /// OpenAI 登录方式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<OpenAiAuthMethod>,

    /// API Key 账号绑定的 Base URL（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,

    /// API Key 账号绑定的供应商名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_provider_name: Option<String>,

    /// 邮箱 (脱敏后存储)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// OpenAI 账号类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,

    /// 保存时间
    pub saved_at: DateTime<Utc>,

    /// 最后使用时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// 最后刷新时间 (从 auth.json 提取)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,

    /// 到期时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// 完整的 auth.json 数据（可选，根据 include_secrets 决定）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_data: Option<CodexAuthJson>,
}

/// 导入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// 合并模式：保留现有账号，只添加新的
    Merge,
    /// 替换模式：覆盖同名账号
    Replace,
}

/// 导入结果
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    /// 新增账号数
    pub added: usize,
    /// 更新账号数
    pub updated: usize,
    /// 跳过账号数
    pub skipped: usize,
    /// 被覆盖的账号列表
    pub overwritten: Vec<String>,
}

// ==================== 加密导出数据结构 ====================

/// Argon2id KDF 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// 内存成本 (KiB), 默认 65536 (64MB)
    pub m_cost: u32,
    /// 时间成本 (迭代次数), 默认 3
    pub t_cost: u32,
    /// 并行度, 默认 1
    pub p_cost: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

/// 加密头信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionHeader {
    /// 加密算法 ("aes-256-gcm")
    pub algorithm: String,
    /// 密钥派生函数 ("argon2id")
    pub kdf: String,
    /// KDF 参数
    pub kdf_params: KdfParams,
    /// 盐值 (base64)
    pub salt: String,
    /// Nonce (base64)
    pub nonce: String,
}

/// 加密导出格式 (version 2.0)
///
/// 信封结构：可读的元数据头 + 加密的 payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthEncryptedExport {
    /// 导出格式版本 ("2.0")
    pub version: String,
    /// 格式标识 ("encrypted")
    pub format: String,
    /// 导出时间
    pub exported_at: DateTime<Utc>,
    /// 账号数量（无需解密即可预览）
    pub account_count: usize,
    /// 加密参数
    pub encryption: EncryptionHeader,
    /// 加密后的 payload (base64)
    ///
    /// 内容为 accounts JSON 的 AES-256-GCM 密文，
    /// 信封头字段作为 AAD 绑定到 GCM 认证标签。
    pub encrypted_payload: String,
}

/// 导入文件格式检测结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportFormat {
    /// 明文 JSON (version 1.0)
    PlaintextV1,
    /// 加密信封 (version 2.0)
    EncryptedV2,
    /// 未知格式
    Unknown,
}

// ==================== 配额查询数据结构 ====================

/// Codex 配额信息（从 wham/usage API 解析）
///
/// 记录 5 小时窗口和周限额的使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexQuota {
    /// 5h 窗口剩余百分比 (0-100)
    pub hourly_percentage: i32,
    /// 5h 窗口重置时间 (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly_reset_time: Option<i64>,
    /// 5h 窗口时长（分钟）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly_window_minutes: Option<i64>,
    /// 是否存在 5h 窗口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly_window_present: Option<bool>,
    /// 周限剩余百分比 (0-100)
    pub weekly_percentage: i32,
    /// 周限重置时间 (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_reset_time: Option<i64>,
    /// 周限窗口时长（分钟）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_window_minutes: Option<i64>,
    /// 是否存在周限窗口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_window_present: Option<bool>,
    /// 订阅类型 (FREE/PLUS/PRO/TEAM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    /// 原始 API 响应（用于调试）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
}

/// 单个账号的配额查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountQuota {
    /// 账号名称
    pub account_name: String,
    /// 邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// 配额信息（成功时存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<CodexQuota>,
    /// 错误信息（失败时存在）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 查询时间
    pub fetched_at: DateTime<Utc>,
}

/// 配额查询错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CodexQuotaError {
    /// 错误代码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// 错误消息
    pub message: String,
    /// 错误时间戳
    pub timestamp: i64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_auth_map_for_openai_api_keeps_only_api_key() {
        let auth = serde_json::json!({
            "OPENAI_API_KEY": " sk-api ",
            "tokens": {
                "id_token": "id",
                "access_token": "access",
                "refresh_token": "refresh",
                "account_id": "acc"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z",
            "MISTRAL_API_KEY": "mistral",
            "custom_meta": "stale"
        })
        .as_object()
        .unwrap()
        .clone();

        let normalized = normalize_auth_map_for_intent(
            &AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Api,
            },
            &auth,
        );

        let expected = serde_json::json!({
            "OPENAI_API_KEY": "sk-api"
        })
        .as_object()
        .unwrap()
        .clone();
        assert_eq!(normalized, expected);
    }

    #[test]
    fn test_normalize_auth_map_for_chatgpt_keeps_only_tokens_and_last_refresh() {
        let auth = serde_json::json!({
            "OPENAI_API_KEY": "sk-api",
            "tokens": {
                "id_token": " id ",
                "access_token": "access",
                "refresh_token": " ",
                "account_id": "acc"
            },
            "last_refresh": " 2026-01-08T03:09:53.894843900Z ",
            "MISTRAL_API_KEY": "mistral",
            "custom_meta": "stale"
        })
        .as_object()
        .unwrap()
        .clone();

        let normalized = normalize_auth_map_for_intent(
            &AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Chatgpt,
            },
            &auth,
        );

        let expected = serde_json::json!({
            "tokens": {
                "id_token": "id",
                "access_token": "access",
                "account_id": "acc"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z"
        })
        .as_object()
        .unwrap()
        .clone();
        assert_eq!(normalized, expected);
    }

    #[test]
    fn test_normalize_auth_map_for_provider_and_no_auth_drops_unrelated_fields() {
        let auth = serde_json::json!({
            "OPENAI_API_KEY": "sk-api",
            "tokens": {
                "id_token": "id"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z",
            "MISTRAL_API_KEY": " mistral ",
            "custom_meta": "stale"
        })
        .as_object()
        .unwrap()
        .clone();

        let provider_normalized = normalize_auth_map_for_intent(
            &AuthIntent::ProviderEnvKey {
                env_key: "MISTRAL_API_KEY".to_string(),
            },
            &auth,
        );
        let provider_expected = serde_json::json!({
            "MISTRAL_API_KEY": "mistral"
        })
        .as_object()
        .unwrap()
        .clone();
        assert_eq!(provider_normalized, provider_expected);

        let no_auth_normalized = normalize_auth_map_for_intent(&AuthIntent::NoAuth, &auth);
        assert!(no_auth_normalized.is_empty());
    }

    #[test]
    fn test_token_freshness_icon() {
        // Icons are now emoji from ccr-types
        assert_eq!(TokenFreshness::Fresh.icon(), "🟢");
        assert_eq!(TokenFreshness::Stale.icon(), "🟡");
        assert_eq!(TokenFreshness::Old.icon(), "🔴");
        assert_eq!(TokenFreshness::unknown().icon(), "⚪");
    }

    #[test]
    fn test_registry_default() {
        let registry = CodexAuthRegistry::default();
        assert_eq!(registry.version, "1.0");
        assert!(registry.current_auth.is_none());
        assert!(registry.accounts.is_empty());
        assert!(registry.usage_ledger.is_empty());
    }

    #[test]
    fn test_input_mode_default() {
        let mode = InputMode::default();
        assert_eq!(mode, InputMode::Normal);
    }

    #[test]
    fn test_codex_auth_json_deserialize() {
        let json = r#"{
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": "eyJ...",
                "access_token": "eyJ...",
                "refresh_token": "rt_...",
                "account_id": "test-id"
            },
            "last_refresh": "2026-01-08T03:09:53.894843900Z"
        }"#;

        let auth: CodexAuthJson = serde_json::from_str(json).unwrap();
        assert!(auth.openai_api_key.is_none());
        assert!(auth.tokens.is_some());
        let tokens = auth.tokens.unwrap();
        assert_eq!(tokens.account_id, Some("test-id".to_string()));
    }

    #[test]
    fn test_codex_auth_account_with_expiry() {
        let account = CodexAuthAccount {
            description: Some("Test account".to_string()),
            account_id: "acc-123".to_string(),
            auth_method: Some(OpenAiAuthMethod::Api),
            api_base_url: None,
            api_provider_name: None,
            email: Some("t***@example.com".to_string()),
            plan_type: None,
            saved_at: Utc::now(),
            last_used: None,
            last_refresh: Some(Utc::now()),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        };

        // Test serialization
        let toml_str = toml::to_string(&account).unwrap();
        assert!(toml_str.contains("expires_at"));
        assert!(toml_str.contains("account_id"));

        // Test deserialization
        let parsed: CodexAuthAccount = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.account_id, "acc-123");
        assert!(parsed.expires_at.is_some());
    }

    #[test]
    fn test_codex_auth_account_without_expiry() {
        let account = CodexAuthAccount {
            description: None,
            account_id: "acc-456".to_string(),
            auth_method: None,
            api_base_url: None,
            api_provider_name: None,
            email: None,
            plan_type: None,
            saved_at: Utc::now(),
            last_used: None,
            last_refresh: None,
            expires_at: None,
        };

        // Test serialization - expires_at should be omitted
        let toml_str = toml::to_string(&account).unwrap();
        assert!(!toml_str.contains("expires_at"));

        // Test deserialization
        let parsed: CodexAuthAccount = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.account_id, "acc-456");
        assert!(parsed.expires_at.is_none());
    }

    #[test]
    fn test_codex_auth_registry_serialization() {
        let mut registry = CodexAuthRegistry {
            current_auth: Some("main".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "main".to_string(),
            CodexAuthAccount {
                description: Some("Main account".to_string()),
                account_id: "acc-main".to_string(),
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: Some("m***@example.com".to_string()),
                plan_type: None,
                saved_at: Utc::now(),
                last_used: Some(Utc::now()),
                last_refresh: Some(Utc::now()),
                expires_at: Some(Utc::now() + chrono::Duration::days(7)),
            },
        );
        registry.record_usage_activation("main", "acc-main", Utc::now());

        // Test serialization
        let toml_str = toml::to_string(&registry).unwrap();
        assert!(toml_str.contains("version"));
        assert!(toml_str.contains("current_auth"));
        assert!(toml_str.contains("[accounts.main]"));
        assert!(toml_str.contains("[[usage_ledger]]"));
        assert!(toml_str.contains("expires_at"));

        // Test deserialization
        let parsed: CodexAuthRegistry = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.current_auth, Some("main".to_string()));
        assert!(parsed.accounts.contains_key("main"));
        assert!(parsed.accounts["main"].expires_at.is_some());
        assert_eq!(parsed.usage_ledger.len(), 1);
    }

    #[test]
    fn test_codex_auth_registry_backward_compatibility() {
        // Old format without expires_at
        let old_toml = r#"
version = "1.0"
current_auth = "legacy"

[accounts.legacy]
account_id = "acc-legacy"
saved_at = "2026-01-01T00:00:00Z"
"#;

        let parsed: CodexAuthRegistry = toml::from_str(old_toml).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.current_auth, Some("legacy".to_string()));
        assert!(parsed.accounts.contains_key("legacy"));
        assert!(parsed.accounts["legacy"].expires_at.is_none());
        assert!(parsed.usage_ledger.is_empty());
    }

    #[test]
    fn test_record_usage_activation_deduplicates_consecutive_entries() {
        let mut registry = CodexAuthRegistry::default();
        let now = Utc::now();

        registry.record_usage_activation("main", "acc-main", now);
        registry.record_usage_activation("main", "acc-main", now + chrono::Duration::minutes(5));
        registry.record_usage_activation(
            "backup",
            "acc-backup",
            now + chrono::Duration::minutes(10),
        );

        assert_eq!(registry.usage_ledger.len(), 2);
        assert_eq!(registry.usage_ledger[0].account_name, "main");
        assert_eq!(registry.usage_ledger[1].account_name, "backup");
    }
}
