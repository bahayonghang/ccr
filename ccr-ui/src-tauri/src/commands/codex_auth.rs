use super::*;
use crate::desktop_shell;
use crate::process::{ProcessDescriptor, ProcessGateway};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ccr_codex::services::CodexModelProviderStoreService;
use ccr_codex::{
    CodexAuthJson, CodexAuthService, CodexModelProviderApiKey, CodexModelProviderRecord,
    ImportMode, OpenAiAuthMethod, Platform, PlatformPaths,
};
use chrono::Utc;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter, State};
use ts_rs::TS;
use uuid::Uuid;

const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const OAUTH_AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OAUTH_SCOPE: &str = "openid profile email offline_access";
const OAUTH_ORIGINATOR: &str = "codex_vscode";
const OAUTH_CALLBACK_PORT: u16 = 1455;
const OAUTH_TIMEOUT_SECONDS: i64 = 300;
const OAUTH_PENDING_FILE: &str = "oauth_pending.json";

static OAUTH_PENDING_STATE: LazyLock<Mutex<Option<CodexOAuthPendingState>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexOAuthPendingState {
    login_id: String,
    auth_url: String,
    redirect_uri: String,
    code_verifier: String,
    state: String,
    port: u16,
    expires_at: i64,
    callback_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CodexOAuthTokenResponse {
    id_token: String,
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthImportPayload {
    content: String,
    #[ts(optional)]
    switch_after_import: Option<bool>,
    #[ts(optional)]
    preferred_account_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexApiKeyAddPayload {
    api_key: String,
    #[ts(optional)]
    api_base_url: Option<String>,
    #[ts(optional)]
    provider_name: Option<String>,
    #[ts(optional)]
    save_provider: Option<bool>,
    #[ts(optional)]
    switch_after_add: Option<bool>,
    #[ts(optional)]
    preferred_account_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProviderUpsertPayload {
    #[ts(optional)]
    id: Option<String>,
    name: String,
    base_url: String,
    #[ts(optional)]
    website_url: Option<String>,
    #[ts(optional)]
    api_key_url: Option<String>,
    #[ts(optional)]
    api_key_name: Option<String>,
    #[ts(optional)]
    api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub enum CodexLoginState {
    NotLoggedIn,
    LoggedInUnsaved,
    LoggedInSaved {
        account_name: String,
    },
    ApiKeyActive,
    ProviderKeyActive {
        env_key: String,
    },
    Unknown {
        raw_type: String,
        raw: CodexJsonValue,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(untagged)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub enum CodexJsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<CodexJsonValue>),
    Object(BTreeMap<String, CodexJsonValue>),
}

impl From<JsonValue> for CodexJsonValue {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(value) => Self::Bool(value),
            JsonValue::Number(value) => Self::Number(value.as_f64().unwrap_or_default()),
            JsonValue::String(value) => Self::String(value),
            JsonValue::Array(values) => Self::Array(values.into_iter().map(Into::into).collect()),
            JsonValue::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
            ),
        }
    }
}

impl From<ccr_codex::LoginState> for CodexLoginState {
    fn from(value: ccr_codex::LoginState) -> Self {
        match value {
            ccr_codex::LoginState::NotLoggedIn => Self::NotLoggedIn,
            ccr_codex::LoginState::LoggedInUnsaved => Self::LoggedInUnsaved,
            ccr_codex::LoginState::LoggedInSaved(account_name) => {
                Self::LoggedInSaved { account_name }
            }
            ccr_codex::LoginState::ApiKeyActive => Self::ApiKeyActive,
            ccr_codex::LoginState::ProviderKeyActive { env_key } => {
                Self::ProviderKeyActive { env_key }
            }
            ccr_codex::LoginState::Unknown { type_name, raw } => Self::Unknown {
                raw_type: type_name,
                raw: raw.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthAccountItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub email: Option<String>,
    pub is_current: bool,
    pub is_virtual: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_provider_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plan_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthCurrentInfo {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plan_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_refresh: Option<String>,
}

impl From<ccr_codex::CurrentAuthInfo> for CodexAuthCurrentInfo {
    fn from(value: ccr_codex::CurrentAuthInfo) -> Self {
        Self {
            account_id: value.account_id,
            auth_method: value.auth_method.map(auth_method_name),
            email: value.email,
            plan_type: value.plan_type,
            last_refresh: value.last_refresh.map(|date| date.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthListResponse {
    pub accounts: Vec<CodexAuthAccountItem>,
    pub login_state: CodexLoginState,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthCurrentResponse {
    pub logged_in: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub info: Option<CodexAuthCurrentInfo>,
    pub login_state: CodexLoginState,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthMutationResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub account_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub switched: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub imported: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub results: Option<Vec<CodexAuthMutationResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub message: Option<String>,
}

impl CodexAuthMutationResponse {
    fn account(account_name: String, switched: bool) -> Self {
        Self {
            success: true,
            account_name: Some(account_name),
            switched: Some(switched),
            imported: None,
            results: None,
            message: None,
        }
    }

    fn import(results: Vec<Self>, switched: bool) -> Self {
        Self {
            success: true,
            account_name: None,
            switched: Some(switched),
            imported: Some(results.len()),
            results: (!results.is_empty()).then_some(results),
            message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexOAuthStartResponse {
    pub login_id: String,
    pub auth_url: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthAccountMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_provider_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plan_type: Option<String>,
    pub saved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub last_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub expires_at: Option<String>,
}

impl From<ccr_codex::CodexAuthAccount> for CodexAuthAccountMetadata {
    fn from(value: ccr_codex::CodexAuthAccount) -> Self {
        Self {
            description: value.description,
            account_id: value.account_id,
            auth_method: value.auth_method.map(auth_method_name),
            api_base_url: value.api_base_url,
            api_provider_name: value.api_provider_name,
            email: value.email,
            plan_type: value.plan_type,
            saved_at: value.saved_at.to_rfc3339(),
            last_used: value.last_used.map(|date| date.to_rfc3339()),
            last_refresh: value.last_refresh.map(|date| date.to_rfc3339()),
            expires_at: value.expires_at.map(|date| date.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthRenameResponse {
    pub success: bool,
    pub old_name: String,
    pub new_name: String,
    pub account: CodexAuthAccountMetadata,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexAuthProcessResponse {
    pub has_running_process: bool,
    pub pids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProviderApiKeyDto {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CodexModelProviderApiKey> for CodexModelProviderApiKeyDto {
    fn from(value: CodexModelProviderApiKey) -> Self {
        Self {
            id: value.id,
            name: value.name,
            api_key: value.api_key,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProviderRecordDto {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub website_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub api_key_url: Option<String>,
    pub api_keys: Vec<CodexModelProviderApiKeyDto>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CodexModelProviderRecord> for CodexModelProviderRecordDto {
    fn from(value: CodexModelProviderRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            base_url: value.base_url,
            website_url: value.website_url,
            api_key_url: value.api_key_url,
            api_keys: value.api_keys.into_iter().map(Into::into).collect(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProvidersResponse {
    pub providers: Vec<CodexModelProviderRecordDto>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProviderSaveResponse {
    pub provider: CodexModelProviderRecordDto,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct CodexModelProviderDeleteResponse {
    pub success: bool,
}

fn auth_method_name(method: OpenAiAuthMethod) -> String {
    match method {
        OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
        OpenAiAuthMethod::Api => "api".to_string(),
    }
}

fn now_ts() -> i64 {
    Utc::now().timestamp()
}

fn oauth_pending_path() -> Result<PathBuf, String> {
    let paths =
        PlatformPaths::new(Platform::Codex).map_err(|e| format!("解析 Codex 平台路径失败: {e}"))?;
    Ok(paths.platform_dir.join(OAUTH_PENDING_FILE))
}

fn load_oauth_pending_from_disk() -> Result<Option<CodexOAuthPendingState>, String> {
    let path = oauth_pending_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 OAuth pending 状态失败: {e}"))?;
    let state: CodexOAuthPendingState =
        serde_json::from_str(&content).map_err(|e| format!("解析 OAuth pending 状态失败: {e}"))?;
    if state.expires_at <= now_ts() {
        let _ = std::fs::remove_file(path);
        return Ok(None);
    }
    Ok(Some(state))
}

fn persist_oauth_pending(state: Option<&CodexOAuthPendingState>) -> Result<(), String> {
    let path = oauth_pending_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建 OAuth 状态目录失败: {e}"))?;
    }

    match state {
        Some(value) => {
            let content = serde_json::to_string_pretty(value)
                .map_err(|e| format!("序列化 OAuth pending 状态失败: {e}"))?;
            ccr_core::core::AtomicWriter::new(&path)
                .write_string(&content)
                .map_err(|e| format!("写入 OAuth pending 状态失败: {e}"))?;
            ccr_codex::utils::ensure_private_permissions(&path);
        }
        None => {
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("清理 OAuth pending 状态失败: {e}"))?;
            }
        }
    }

    Ok(())
}

fn hydrate_oauth_pending_if_needed() -> Result<(), String> {
    let mut guard = OAUTH_PENDING_STATE
        .lock()
        .map_err(|_| "OAuth pending 状态锁定失败".to_string())?;
    if guard.is_none() {
        *guard = load_oauth_pending_from_disk()?;
    }
    Ok(())
}

fn set_oauth_pending(state: Option<CodexOAuthPendingState>) -> Result<(), String> {
    {
        let mut guard = OAUTH_PENDING_STATE
            .lock()
            .map_err(|_| "OAuth pending 状态锁定失败".to_string())?;
        *guard = state.clone();
    }
    persist_oauth_pending(state.as_ref())
}

fn current_pending_state() -> Result<Option<CodexOAuthPendingState>, String> {
    hydrate_oauth_pending_if_needed()?;
    let guard = OAUTH_PENDING_STATE
        .lock()
        .map_err(|_| "OAuth pending 状态锁定失败".to_string())?;
    Ok(guard.clone())
}

fn generate_code_verifier() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn generate_code_challenge(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn build_oauth_authorize_url(
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> Result<String, String> {
    let url = Url::parse_with_params(
        OAUTH_AUTHORIZE_URL,
        &[
            ("response_type", "code"),
            ("client_id", OAUTH_CLIENT_ID),
            ("redirect_uri", redirect_uri),
            ("scope", OAUTH_SCOPE),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("id_token_add_organizations", "true"),
            ("codex_cli_simplified_flow", "true"),
            ("state", state),
            ("originator", OAUTH_ORIGINATOR),
        ],
    )
    .map_err(|e| format!("构造 OAuth 授权链接失败: {e}"))?;
    Ok(url.to_string())
}

fn find_available_oauth_port() -> Result<u16, String> {
    match std::net::TcpListener::bind(("127.0.0.1", OAUTH_CALLBACK_PORT)) {
        Ok(listener) => {
            drop(listener);
            Ok(OAUTH_CALLBACK_PORT)
        }
        Err(error) if error.kind() == ErrorKind::AddrInUse => {
            Err(format!("无法绑定端口 {OAUTH_CALLBACK_PORT}: {}", error))
        }
        Err(error) => Err(format!("无法绑定端口 {OAUTH_CALLBACK_PORT}: {error}")),
    }
}

fn callback_url_from_path(path: &str, port: u16) -> Result<Url, String> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err("回调地址不能为空".to_string());
    }

    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Url::parse(raw).map_err(|e| format!("回调地址格式无效: {e}"));
    }

    if raw.starts_with('/') {
        return Url::parse(&format!("http://localhost:{port}{raw}"))
            .map_err(|e| format!("回调地址格式无效: {e}"));
    }

    Url::parse(&format!(
        "http://localhost:{port}/auth/callback?{}",
        raw.trim_start_matches('?')
    ))
    .map_err(|e| format!("回调地址格式无效: {e}"))
}

fn html_response(title: &str, description: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head><body style=\"font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; padding: 24px; background: #0f172a; color: #e2e8f0;\"><h2 style=\"margin:0 0 8px;\">{title}</h2><p style=\"margin:0; color:#cbd5e1;\">{description}</p><p style=\"margin-top:16px; font-size:12px; color:#94a3b8;\">You can return to CCR now.</p></body></html>"
    )
}

fn parse_http_request_path(request: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    if method.eq_ignore_ascii_case("GET") {
        Some(path.to_string())
    } else {
        None
    }
}

async fn start_oauth_callback_listener(
    app: AppHandle,
    state: CodexOAuthPendingState,
) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", state.port))
        .await
        .map_err(|e| format!("启动 OAuth 回调监听失败: {e}"))?;

    while let Some(current) = current_pending_state()? {
        if current.login_id != state.login_id || current.state != state.state {
            break;
        }
        if current.expires_at <= now_ts() {
            set_oauth_pending(None)?;
            let payload = json!({
                "loginId": state.login_id,
                "callbackUrl": state.redirect_uri,
                "timeoutSeconds": OAUTH_TIMEOUT_SECONDS,
            });
            let _ = app.emit("codex-oauth-login-timeout", payload);
            break;
        }

        let accepted =
            tokio::time::timeout(std::time::Duration::from_secs(1), listener.accept()).await;

        let Ok(Ok((mut stream, _))) = accepted else {
            continue;
        };

        let mut buffer = vec![0_u8; 8192];
        let size = tokio::io::AsyncReadExt::read(&mut stream, &mut buffer)
            .await
            .unwrap_or(0);
        let request_text = String::from_utf8_lossy(&buffer[..size]).to_string();
        let Some(path) = parse_http_request_path(&request_text) else {
            let _ = tokio::io::AsyncWriteExt::write_all(
                &mut stream,
                format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html_response("Invalid request", "CCR expected an OAuth callback request.").len(),
                    html_response("Invalid request", "CCR expected an OAuth callback request.")
                )
                .as_bytes(),
            )
            .await;
            continue;
        };

        let parsed_url = callback_url_from_path(&path, state.port)?;
        let code = parsed_url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
            .unwrap_or_default();
        let callback_state = parsed_url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.to_string())
            .unwrap_or_default();

        let (status_line, body) = if callback_state != state.state {
            (
                "HTTP/1.1 400 Bad Request",
                html_response(
                    "OAuth state mismatch",
                    "The callback state does not match the current CCR login request.",
                ),
            )
        } else if code.is_empty() {
            (
                "HTTP/1.1 400 Bad Request",
                html_response(
                    "OAuth code missing",
                    "OpenAI returned without an authorization code.",
                ),
            )
        } else {
            let callback_url = parsed_url.to_string();
            let next = CodexOAuthPendingState {
                callback_url: Some(callback_url.clone()),
                ..state.clone()
            };
            set_oauth_pending(Some(next))?;
            let payload = json!({ "loginId": state.login_id });
            let _ = app.emit("codex-oauth-login-completed", payload);
            (
                "HTTP/1.1 200 OK",
                html_response(
                    "Authorization received",
                    "CCR captured the OpenAI callback. Finish the login back in the app.",
                ),
            )
        };

        let response = format!(
            "{status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes()).await;
    }

    Ok(())
}

pub fn restore_pending_oauth_listener(app: AppHandle) {
    let Ok(Some(state)) = load_oauth_pending_from_disk() else {
        return;
    };
    if state.expires_at <= now_ts() {
        let _ = persist_oauth_pending(None);
        return;
    }
    let _ = set_oauth_pending(Some(state.clone()));
    tauri::async_runtime::spawn(async move {
        let _ = start_oauth_callback_listener(app, state).await;
    });
}

async fn exchange_oauth_tokens(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<CodexOAuthTokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", OAUTH_CLIENT_ID),
        ("code_verifier", code_verifier),
    ];

    let response = client
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token 请求失败: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("读取 Token 响应失败: {e}"))?;

    if !status.is_success() {
        return Err(format!("Token 交换失败: status={status}, body={body}"));
    }

    serde_json::from_str::<CodexOAuthTokenResponse>(&body)
        .map_err(|e| format!("解析 Token 响应失败: {e}"))
}

fn oauth_state_from_login_id(login_id: &str) -> Result<CodexOAuthPendingState, String> {
    let state =
        current_pending_state()?.ok_or_else(|| "当前没有进行中的 Codex OAuth 流程".to_string())?;
    if state.login_id != login_id {
        return Err("OAuth 登录流程已变化，请刷新授权链接后重试".to_string());
    }
    Ok(state)
}

fn json_value_to_auth(value: JsonValue) -> Result<CodexAuthJson, String> {
    serde_json::from_value(value).map_err(|e| format!("解析认证数据失败: {e}"))
}

fn derive_email_from_auth(auth: &CodexAuthJson) -> Option<String> {
    let id_token = auth.tokens.as_ref()?.id_token.as_ref()?;
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let decoded = ccr_codex::utils::decode_base64url(parts[1])?;
    let payload: JsonValue = serde_json::from_slice(&decoded).ok()?;
    payload
        .get("email")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn derive_name_hint(auth: &CodexAuthJson, provider_name: Option<&str>) -> Option<String> {
    if let Some(email) = derive_email_from_auth(auth) {
        return email.split('@').next().map(|value| value.to_string());
    }

    if let Some(provider_name) = provider_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(provider_name.to_string());
    }

    if auth
        .openai_api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        return Some("openai-api".to_string());
    }

    auth.tokens
        .as_ref()
        .and_then(|tokens| tokens.account_id.as_ref())
        .map(|value| value.to_string())
}

fn single_account_import_payload(
    account_name: &str,
    export_account: ccr_codex::CodexAuthExportAccount,
) -> Result<String, String> {
    let mut accounts = JsonMap::new();
    accounts.insert(
        account_name.to_string(),
        serde_json::to_value(export_account).map_err(|e| format!("序列化导入账号失败: {e}"))?,
    );

    serde_json::to_string(&json!({
        "version": "1.0",
        "exported_at": Utc::now().to_rfc3339(),
        "accounts": accounts,
    }))
    .map_err(|e| format!("构造导入数据失败: {e}"))
}

fn normalize_url(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().trim_end_matches('/').to_string())
        .filter(|raw| !raw.is_empty())
}

fn normalize_account_name(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportPayloadShape {
    Single,
    Multiple,
    Bundle,
}

fn detect_import_payload_shape(content: &str) -> Result<ImportPayloadShape, String> {
    let raw: JsonValue =
        serde_json::from_str(content).map_err(|e| format!("解析 JSON 输入失败: {e}"))?;

    match raw {
        JsonValue::Object(map) => {
            if map.get("accounts").is_some() {
                Ok(ImportPayloadShape::Bundle)
            } else {
                Ok(ImportPayloadShape::Single)
            }
        }
        JsonValue::Array(items) => {
            if items.len() == 1 {
                Ok(ImportPayloadShape::Single)
            } else {
                Ok(ImportPayloadShape::Multiple)
            }
        }
        _ => Err("仅支持对象或数组格式的认证 JSON".to_string()),
    }
}

fn provider_record_from_payload(
    payload: CodexModelProviderUpsertPayload,
) -> Result<CodexModelProviderRecord, String> {
    let now = Utc::now();
    let provider_id = payload
        .id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let base_url =
        normalize_url(Some(payload.base_url)).ok_or_else(|| "Base URL 不能为空".to_string())?;
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err("供应商名称不能为空".to_string());
    }

    let api_keys = payload
        .api_key
        .map(|api_key| api_key.trim().to_string())
        .filter(|api_key| !api_key.is_empty())
        .map(|api_key| {
            vec![CodexModelProviderApiKey {
                id: Uuid::new_v4().to_string(),
                name: payload
                    .api_key_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Primary")
                    .to_string(),
                api_key,
                created_at: now,
                updated_at: now,
            }]
        })
        .unwrap_or_default();

    Ok(CodexModelProviderRecord {
        id: provider_id,
        name,
        base_url,
        website_url: normalize_url(payload.website_url),
        api_key_url: normalize_url(payload.api_key_url),
        api_keys,
        created_at: now,
        updated_at: now,
    })
}

#[cfg(any(target_os = "windows", test))]
fn parse_netstat_listeners(text: &str, port: u16) -> Vec<u32> {
    let mut pids = Vec::new();
    for line in text.lines() {
        if !(line.contains(&format!(":{port}")) && line.contains("LISTENING")) {
            continue;
        }
        if let Some(pid) = line
            .split_whitespace()
            .last()
            .and_then(|value| value.parse::<u32>().ok())
        {
            pids.push(pid);
        }
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

#[cfg(any(not(target_os = "windows"), test))]
fn parse_lsof_listeners(text: &str) -> Vec<u32> {
    let mut pids = text
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect::<Vec<_>>();
    pids.sort_unstable();
    pids.dedup();
    pids
}

async fn discover_port_processes(port: u16) -> Result<Vec<u32>, String> {
    let descriptor = ProcessDescriptor::port_discovery();
    #[cfg(target_os = "windows")]
    {
        let output = ProcessGateway::execute(
            &descriptor,
            &[
                OsString::from("-ano"),
                OsString::from("-p"),
                OsString::from("tcp"),
            ],
        )
        .await?;
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_netstat_listeners(&text, port))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = ProcessGateway::execute(
            &descriptor,
            &[
                OsString::from("-nP"),
                OsString::from(format!("-iTCP:{port}")),
                OsString::from("-sTCP:LISTEN"),
                OsString::from("-t"),
            ],
        )
        .await?;
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lsof_listeners(&text))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/codex_auth/")]
pub struct OAuthPortReleaseReport {
    pub discovered_pids: Vec<u32>,
    pub owned_pids: Vec<u32>,
    pub unknown_pids: Vec<u32>,
    pub cancel_requested: u32,
}

fn build_port_release_report(
    mut discovered_pids: Vec<u32>,
    mut owned_pids: Vec<u32>,
) -> OAuthPortReleaseReport {
    discovered_pids.sort_unstable();
    discovered_pids.dedup();
    owned_pids.sort_unstable();
    owned_pids.dedup();
    let unknown_pids = discovered_pids
        .iter()
        .copied()
        .filter(|pid| !owned_pids.contains(pid))
        .collect();
    OAuthPortReleaseReport {
        discovered_pids,
        cancel_requested: owned_pids.len() as u32,
        owned_pids,
        unknown_pids,
    }
}

fn account_item_to_dto(
    item: ccr_codex::CodexAuthItem,
    snapshot: &ccr_codex::AuthReadSnapshot,
) -> CodexAuthAccountItem {
    let registry_account = snapshot.registry.accounts.get(&item.name);
    let auth_method = registry_account
        .and_then(|account| account.auth_method)
        .map(auth_method_name);

    CodexAuthAccountItem {
        name: item.name,
        description: item.description,
        email: item.email,
        is_current: item.is_current,
        is_virtual: item.is_virtual,
        saved_at: item.saved_at.map(|date| date.to_rfc3339()),
        last_used: item.last_used.map(|date| date.to_rfc3339()),
        last_refresh: item.last_refresh.map(|date| date.to_rfc3339()),
        plan_type: item.plan_type,
        auth_method,
        api_base_url: registry_account.and_then(|account| account.api_base_url.clone()),
        api_provider_name: registry_account.and_then(|account| account.api_provider_name.clone()),
    }
}

fn finalize_account_mutation(
    service: &CodexAuthService,
    export_account: ccr_codex::CodexAuthExportAccount,
    preferred_name: Option<String>,
    switch_after_import: bool,
) -> Result<CodexAuthMutationResponse, String> {
    let explicit_name = normalize_account_name(preferred_name);
    let name_hint = explicit_name.clone().or_else(|| {
        derive_name_hint(
            &export_account.auth_data.clone().unwrap_or(CodexAuthJson {
                openai_api_key: None,
                tokens: None,
                last_refresh: None,
            }),
            export_account.api_provider_name.as_deref(),
        )
    });

    let account_name = if let Some(name) = explicit_name {
        service
            .reserve_explicit_account_name(&name)
            .map_err(|e| format!("校验账号名称失败: {e}"))?
    } else {
        service
            .suggest_account_name(name_hint.as_deref())
            .map_err(|e| format!("生成账号名称失败: {e}"))?
    };
    let payload = single_account_import_payload(&account_name, export_account)?;
    service
        .import_accounts(&payload, ImportMode::Merge, false)
        .map_err(|e| format!("导入账号失败: {e}"))?;

    if switch_after_import {
        service
            .switch_account(&account_name)
            .map_err(|e| format!("切换账号失败: {e}"))?;
    }

    Ok(CodexAuthMutationResponse::account(
        account_name,
        switch_after_import,
    ))
}

fn parse_import_entries(content: &str) -> Result<Vec<(Option<String>, CodexAuthJson)>, String> {
    let raw: JsonValue =
        serde_json::from_str(content).map_err(|e| format!("解析 JSON 输入失败: {e}"))?;

    if raw.get("accounts").is_some() {
        let auth_service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        auth_service
            .import_accounts(content, ImportMode::Merge, false)
            .map_err(|e| format!("导入账号失败: {e}"))?;
        return Ok(Vec::new());
    }

    match raw {
        JsonValue::Object(_) => {
            let auth = json_value_to_auth(raw)?;
            Ok(vec![(None, auth)])
        }
        JsonValue::Array(items) => items
            .into_iter()
            .map(|item| {
                let hint = item
                    .get("name")
                    .or_else(|| item.get("email"))
                    .or_else(|| item.get("id"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string);
                let auth = json_value_to_auth(item)?;
                Ok((hint, auth))
            })
            .collect(),
        _ => Err("仅支持对象或数组格式的认证 JSON".to_string()),
    }
}

/// 列出所有 Codex Auth 账号
#[tauri::command]
pub async fn codex_list_auth_accounts() -> Result<CodexAuthListResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot)
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let accounts = accounts
            .into_iter()
            .map(|item| account_item_to_dto(item, &snapshot))
            .collect::<Vec<_>>();

        Ok(CodexAuthListResponse {
            accounts,
            login_state: snapshot.login_state.into(),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取当前 Codex Auth 信息
#[tauri::command]
pub async fn codex_get_auth_current() -> Result<CodexAuthCurrentResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;

        let info = snapshot.current_info.map(Into::into);

        Ok(CodexAuthCurrentResponse {
            logged_in: info.is_some(),
            info,
            login_state: snapshot.login_state.into(),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存当前登录到命名账号
#[tauri::command]
pub async fn codex_save_auth(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    force: Option<bool>,
) -> Result<CodexAuthActionResponse, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service
            .save_current(&name, description, force.unwrap_or(false))
            .map_err(|e| format!("{e}"))?;

        Ok(CodexAuthActionResponse {
            success: true,
            message: format!("Codex Auth 账号 '{name}' 已成功保存"),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(response)
}

/// 切换到指定账号
#[tauri::command]
pub async fn codex_switch_auth(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<CodexAuthActionResponse, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        service.switch_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(CodexAuthActionResponse {
        success: true,
        message: format!("已切换到 Codex Auth 账号 '{name_resp}'"),
    })
}

/// 删除指定账号
#[tauri::command]
pub async fn codex_delete_auth(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<CodexAuthActionResponse, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        service.delete_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(CodexAuthActionResponse {
        success: true,
        message: format!("Codex Auth 账号 '{name_resp}' 已成功删除"),
    })
}

/// 重命名指定账号
///
/// 同步迁移 auth 文件、registry 键顺序以及 usage_ledger 归因记录。
/// 默认拒绝同名冲突，传入 `force=true` 时备份并覆盖占位账号。
#[tauri::command]
pub async fn codex_rename_auth(
    app: AppHandle,
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
    force: Option<bool>,
) -> Result<CodexAuthRenameResponse, String> {
    let old_resp = old_name.clone();
    let new_resp = new_name.clone();
    let force_flag = force.unwrap_or(false);

    let account = tokio::task::spawn_blocking(move || -> Result<_, String> {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let updated = service
            .rename_account(&old_name, &new_name, force_flag)
            .map_err(|e| format!("{e}"))?;
        Ok(CodexAuthAccountMetadata::from(updated))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(CodexAuthRenameResponse {
        success: true,
        old_name: old_resp.clone(),
        new_name: new_resp.clone(),
        account,
        message: format!("已重命名 Codex Auth '{old_resp}' -> '{new_resp}'"),
    })
}

/// 检测运行中的 Codex 进程
#[tauri::command]
pub async fn codex_detect_process() -> Result<CodexAuthProcessResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let pids = service.detect_codex_process();
        let has_running_process = !pids.is_empty();

        let warning = has_running_process.then(|| {
            format!(
                "检测到 {} 个运行中的 Codex 进程 (PID: {})，切换账号前请先关闭这些进程",
                pids.len(),
                pids.iter()
                    .map(|pid| pid.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

        Ok(CodexAuthProcessResponse {
            has_running_process,
            pids,
            warning,
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_oauth_login_start(app: AppHandle) -> Result<CodexOAuthStartResponse, String> {
    hydrate_oauth_pending_if_needed()?;
    if let Some(existing) = current_pending_state()? {
        if existing.expires_at > now_ts() {
            return Ok(CodexOAuthStartResponse {
                login_id: existing.login_id,
                auth_url: existing.auth_url,
            });
        }
        set_oauth_pending(None)?;
    }

    let port = find_available_oauth_port()?;
    let login_id = Uuid::new_v4().to_string();
    let state = Uuid::new_v4().to_string();
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let redirect_uri = format!("http://localhost:{port}/auth/callback");
    let auth_url = build_oauth_authorize_url(&redirect_uri, &code_challenge, &state)?;

    let pending = CodexOAuthPendingState {
        login_id: login_id.clone(),
        auth_url: auth_url.clone(),
        redirect_uri,
        code_verifier,
        state,
        port,
        expires_at: now_ts() + OAUTH_TIMEOUT_SECONDS,
        callback_url: None,
    };
    set_oauth_pending(Some(pending.clone()))?;

    tauri::async_runtime::spawn(async move {
        let _ = start_oauth_callback_listener(app, pending).await;
    });

    Ok(CodexOAuthStartResponse { login_id, auth_url })
}

#[tauri::command]
pub async fn codex_oauth_login_completed(
    app: AppHandle,
    state: State<'_, AppState>,
    login_id: String,
    preferred_account_name: Option<String>,
) -> Result<CodexAuthMutationResponse, String> {
    let pending = oauth_state_from_login_id(&login_id)?;
    let callback_url = pending
        .callback_url
        .clone()
        .ok_or_else(|| "尚未收到 OAuth 回调，请在浏览器授权后重试".to_string())?;
    let parsed = Url::parse(&callback_url).map_err(|e| format!("解析回调地址失败: {e}"))?;
    let callback_state = parsed
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    if callback_state != pending.state {
        return Err("OAuth state 校验失败，请重新发起授权".to_string());
    }

    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    if code.is_empty() {
        return Err("OAuth 回调中缺少 code 参数".to_string());
    }

    let tokens =
        exchange_oauth_tokens(&code, &pending.code_verifier, &pending.redirect_uri).await?;
    let auth = CodexAuthJson {
        openai_api_key: None,
        tokens: Some(ccr_codex::CodexAuthTokens {
            id_token: Some(tokens.id_token),
            access_token: Some(tokens.access_token),
            refresh_token: tokens.refresh_token,
            account_id: None,
        }),
        last_refresh: Some(Utc::now().to_rfc3339()),
    };

    let response = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let export = service
            .build_export_account_from_auth_json(auth, None, None, None, None)
            .map_err(|e| format!("构建 OAuth 账号失败: {e}"))?;
        finalize_account_mutation(&service, export, preferred_account_name, true)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    set_oauth_pending(None)?;
    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(response)
}

#[tauri::command]
pub fn codex_oauth_login_cancel(login_id: Option<String>) -> Result<(), String> {
    let Some(current) = current_pending_state()? else {
        return Ok(());
    };
    if let Some(login_id) = login_id
        && current.login_id != login_id
    {
        return Err("指定的 OAuth 登录会话已变化".to_string());
    }
    set_oauth_pending(None)
}

#[tauri::command]
pub fn codex_oauth_submit_callback_url(
    app: AppHandle,
    login_id: String,
    callback_url: String,
) -> Result<(), String> {
    let current = oauth_state_from_login_id(&login_id)?;
    let parsed = Url::parse(&callback_url).map_err(|e| format!("解析回调地址失败: {e}"))?;
    let callback_state = parsed
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();

    if callback_state != current.state {
        return Err("OAuth state 校验失败，请确认复制的是当前授权链接完成后的回调地址".to_string());
    }
    if code.is_empty() {
        return Err("回调地址中缺少 code 参数".to_string());
    }

    set_oauth_pending(Some(CodexOAuthPendingState {
        callback_url: Some(callback_url),
        ..current.clone()
    }))?;
    let _ = app.emit(
        "codex-oauth-login-completed",
        json!({ "loginId": login_id }),
    );
    Ok(())
}

#[tauri::command]
pub fn codex_is_oauth_port_in_use() -> Result<bool, String> {
    Ok(std::net::TcpListener::bind(("127.0.0.1", OAUTH_CALLBACK_PORT)).is_err())
}

#[tauri::command]
pub async fn codex_release_oauth_port() -> Result<OAuthPortReleaseReport, String> {
    let discovered = discover_port_processes(OAUTH_CALLBACK_PORT).await?;
    let owned = ProcessGateway::owned_processes_for_port(&discovered, OAUTH_CALLBACK_PORT);
    for process in &owned {
        process.request_cancel();
    }
    Ok(build_port_release_report(
        discovered,
        owned.iter().map(|process| process.pid).collect(),
    ))
}

#[tauri::command]
pub async fn codex_open_external_url(url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || ProcessGateway::open_oauth_url(&url))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_import_auth_payload(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: CodexAuthImportPayload,
) -> Result<CodexAuthMutationResponse, String> {
    let switch_after_import = payload.switch_after_import.unwrap_or(false);
    let content = payload.content;
    let preferred_account_name = normalize_account_name(payload.preferred_account_name);

    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let payload_shape = detect_import_payload_shape(&content)?;
        if preferred_account_name.is_some() && payload_shape != ImportPayloadShape::Single {
            return Err(
                "自定义名称仅支持单账号导入；批量/导出包导入请保留原始命名策略".to_string(),
            );
        }

        match parse_import_entries(&content) {
            Ok(entries) if entries.is_empty() => {
                Ok(CodexAuthMutationResponse::import(Vec::new(), false))
            }
            Ok(entries) => {
                let mut imported = Vec::new();
                for (index, (hint, auth)) in entries.into_iter().enumerate() {
                    let export = service
                        .build_export_account_from_auth_json(auth, None, None, None, None)
                        .map_err(|e| format!("构建导入账号失败: {e}"))?;
                    let response = finalize_account_mutation(
                        &service,
                        export,
                        if index == 0 {
                            preferred_account_name.clone().or(hint)
                        } else {
                            hint
                        },
                        switch_after_import && index == 0,
                    )?;
                    imported.push(response);
                }
                let switched = switch_after_import && !imported.is_empty();
                Ok(CodexAuthMutationResponse::import(imported, switched))
            }
            Err(error) => Err(error),
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(result)
}

#[tauri::command]
pub async fn codex_import_auth_from_local(
    app: AppHandle,
    state: State<'_, AppState>,
    preferred_account_name: Option<String>,
) -> Result<CodexAuthMutationResponse, String> {
    let result = tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let auth = service
            .load_current_auth_json()
            .map_err(|e| format!("读取本地 Codex 认证失败: {e}"))?;
        let export = service
            .build_export_account_from_auth_json(auth, None, None, None, None)
            .map_err(|e| format!("构建本地导入账号失败: {e}"))?;
        finalize_account_mutation(
            &service,
            export,
            normalize_account_name(preferred_account_name),
            false,
        )
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(result)
}

#[tauri::command]
pub async fn codex_add_auth_with_api_key(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: CodexApiKeyAddPayload,
) -> Result<CodexAuthMutationResponse, String> {
    let response = tokio::task::spawn_blocking(move || {
        let api_key = payload.api_key.trim().to_string();
        if api_key.is_empty() {
            return Err("API Key 不能为空".to_string());
        }

        let api_base_url = normalize_url(payload.api_base_url);
        let provider_name = payload
            .provider_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        let auth = CodexAuthJson {
            openai_api_key: Some(api_key.clone()),
            tokens: None,
            last_refresh: None,
        };
        let export = service
            .build_export_account_from_auth_json(
                auth,
                None,
                None,
                api_base_url.clone(),
                provider_name.clone(),
            )
            .map_err(|e| format!("构建 API Key 账号失败: {e}"))?;

        if payload.save_provider.unwrap_or(false)
            && let Some(base_url) = api_base_url.clone()
        {
            let provider_store = CodexModelProviderStoreService::new()
                .map_err(|e| format!("初始化供应商存储失败: {e}"))?;
            let now = Utc::now();
            let provider = CodexModelProviderRecord {
                id: Uuid::new_v4().to_string(),
                name: provider_name
                    .clone()
                    .unwrap_or_else(|| "Custom Provider".to_string()),
                base_url,
                website_url: None,
                api_key_url: None,
                api_keys: vec![CodexModelProviderApiKey {
                    id: Uuid::new_v4().to_string(),
                    name: "Primary".to_string(),
                    api_key,
                    created_at: now,
                    updated_at: now,
                }],
                created_at: now,
                updated_at: now,
            };
            let _ = provider_store.upsert_provider(provider);
        }

        finalize_account_mutation(
            &service,
            export,
            normalize_account_name(payload.preferred_account_name)
                .or(provider_name.or_else(|| Some("openai-api".to_string()))),
            payload.switch_after_add.unwrap_or(false),
        )
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    let _ = desktop_shell::refresh_codex_tray(&app, true).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_list_model_providers() -> Result<CodexModelProvidersResponse, String> {
    tokio::task::spawn_blocking(|| {
        let service = CodexModelProviderStoreService::new()
            .map_err(|e| format!("初始化供应商存储失败: {e}"))?;
        let store = service
            .load()
            .map_err(|e| format!("读取供应商列表失败: {e}"))?;
        Ok(CodexModelProvidersResponse {
            providers: store.providers.into_iter().map(Into::into).collect(),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_save_model_provider(
    payload: CodexModelProviderUpsertPayload,
) -> Result<CodexModelProviderSaveResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service = CodexModelProviderStoreService::new()
            .map_err(|e| format!("初始化供应商存储失败: {e}"))?;
        let existing = service
            .load()
            .map_err(|e| format!("读取供应商列表失败: {e}"))?;
        let mut provider = provider_record_from_payload(payload)?;
        if let Some(previous) = existing
            .providers
            .iter()
            .find(|item| item.id == provider.id)
        {
            provider.created_at = previous.created_at;
            if provider.api_keys.is_empty() {
                provider.api_keys = previous.api_keys.clone();
            }
        }
        provider.updated_at = Utc::now();

        let saved = service
            .upsert_provider(provider)
            .map_err(|e| format!("保存供应商失败: {e}"))?;
        Ok(CodexModelProviderSaveResponse {
            provider: saved.into(),
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_delete_model_provider(
    provider_id: String,
) -> Result<CodexModelProviderDeleteResponse, String> {
    tokio::task::spawn_blocking(move || {
        let service = CodexModelProviderStoreService::new()
            .map_err(|e| format!("初始化供应商存储失败: {e}"))?;
        service
            .delete_provider(&provider_id)
            .map_err(|e| format!("删除供应商失败: {e}"))?;
        Ok(CodexModelProviderDeleteResponse { success: true })
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_release_report_never_claims_unknown_processes() {
        let report = build_port_release_report(vec![41, 7, 41, 9], vec![9]);

        assert_eq!(report.discovered_pids, vec![7, 9, 41]);
        assert_eq!(report.owned_pids, vec![9]);
        assert_eq!(report.unknown_pids, vec![7, 41]);
        assert_eq!(report.cancel_requested, 1);
    }

    #[test]
    fn port_release_report_is_noop_when_no_owned_process_matches() {
        let report = build_port_release_report(vec![23], Vec::new());

        assert_eq!(report.unknown_pids, vec![23]);
        assert_eq!(report.cancel_requested, 0);
    }

    #[test]
    fn port_discovery_parsers_deduplicate_listener_pids() {
        let netstat = "TCP 127.0.0.1:1455 0.0.0.0:0 LISTENING 41\n\
                       TCP [::1]:1455 [::]:0 LISTENING 41\n\
                       TCP 127.0.0.1:1456 0.0.0.0:0 LISTENING 99";
        assert_eq!(parse_netstat_listeners(netstat, 1455), vec![41]);
        assert_eq!(parse_lsof_listeners("41\n7\n41\ninvalid\n"), vec![7, 41]);
    }
}
