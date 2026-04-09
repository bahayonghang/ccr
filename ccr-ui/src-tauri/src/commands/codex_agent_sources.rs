use super::agents::{
    agent_file_path, ensure_agents_dir, read_agent_table, record_from_table, table_from_config,
    write_agent_file,
};
use super::{codex_agents_dir, invalidate_codex_dashboard_overview_cache};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use reqwest::header::ETAG;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;
use tempfile::NamedTempFile;
use tokio::time::{Duration, sleep};

const SOURCES_FILENAME: &str = "sources.json";
const INSTALLS_FILENAME: &str = "installs.json";
const CATALOGS_DIR: &str = "catalogs";
const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_RAW_BASE: &str = "https://raw.githubusercontent.com";
const GITHUB_USER_AGENT: &str = "CCR-Desktop";
const GITHUB_RETRY_ATTEMPTS: usize = 3;
const GITHUB_RETRY_BASE_DELAY_MS: u64 = 250;
const CATALOG_STALE_AFTER_SECS: i64 = 15 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CodexAgentSourceFile {
    #[serde(default)]
    sources: Vec<CodexAgentSourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexAgentSourceEntry {
    id: String,
    repo_url: String,
    owner: String,
    repo: String,
    default_branch: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_scanned_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
    agent_count: usize,
    diagnostics_count: usize,
    scan_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CodexAgentInstallFile {
    #[serde(default)]
    installs: Vec<CodexTrackedInstallEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexTrackedInstallEntry {
    id: String,
    source_id: String,
    repo_url: String,
    source_path: String,
    source_blob_sha: String,
    source_content_hash: String,
    installed_name: String,
    target_path: String,
    installed_content_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CodexAgentCatalogFile {
    source_id: String,
    repo_url: String,
    default_branch: String,
    status: String,
    scan_complete: bool,
    truncated: bool,
    scanned_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tree_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tree_etag: Option<String>,
    #[serde(default)]
    diagnostics: Vec<CodexAgentSourceDiagnostic>,
    #[serde(default)]
    agents: Vec<CodexRemoteAgentCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexRemoteAgentCacheEntry {
    id: String,
    source_path: String,
    file_name: String,
    blob_sha: String,
    content_hash: String,
    category: String,
    category_label: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    developer_instructions: Option<String>,
    #[serde(default)]
    nickname_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sandbox_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_servers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skills_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    other: Option<Value>,
    raw_toml: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceRecord {
    pub id: String,
    pub repo_url: String,
    pub owner: String,
    pub repo: String,
    pub default_branch: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scanned_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub agent_count: usize,
    pub diagnostics_count: usize,
    pub scan_complete: bool,
    pub is_stale: bool,
    pub cache_ttl_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceDiagnostic {
    pub path: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRemoteAgentRecord {
    pub id: String,
    pub source_id: String,
    pub source_path: String,
    pub file_name: String,
    pub blob_sha: String,
    pub content_hash: String,
    pub category: String,
    pub category_label: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,
    #[serde(default)]
    pub nickname_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
    pub raw_toml: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSourceInstallRecord {
    pub id: String,
    pub source_id: String,
    pub repo_url: String,
    pub source_path: String,
    pub installed_name: String,
    pub target_path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub has_upstream_update: bool,
    pub has_local_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceCatalogResponse {
    pub source: CodexAgentSourceRecord,
    #[serde(default)]
    pub agents: Vec<CodexRemoteAgentRecord>,
    #[serde(default)]
    pub diagnostics: Vec<CodexAgentSourceDiagnostic>,
    #[serde(default)]
    pub installs: Vec<CodexSourceInstallRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourcesResponse {
    #[serde(default)]
    pub sources: Vec<CodexAgentSourceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceInstallRequest {
    pub source_id: String,
    pub agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceSyncRequest {
    pub install_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentSourceInstallActionRequest {
    pub install_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubRepoPayload {
    full_name: String,
    default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubTreePayload {
    sha: String,
    truncated: bool,
    tree: Vec<GitHubTreeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubTreeEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: String,
    sha: String,
    size: Option<u64>,
}

#[derive(Debug, Clone)]
struct ParsedGitHubRepo {
    repo_url: String,
    owner: String,
    repo: String,
}

#[derive(Debug, Clone)]
struct ScanOutput {
    status: String,
    scan_complete: bool,
    truncated: bool,
    diagnostics: Vec<CodexAgentSourceDiagnostic>,
    agents: Vec<CodexRemoteAgentCacheEntry>,
    tree_sha: Option<String>,
    tree_etag: Option<String>,
}

fn codex_agent_sources_root() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home
        .join(".ccr")
        .join("platforms")
        .join("codex")
        .join("agent-sources"))
}

fn codex_agent_sources_path() -> Result<PathBuf, String> {
    Ok(codex_agent_sources_root()?.join(SOURCES_FILENAME))
}

fn codex_agent_installs_path() -> Result<PathBuf, String> {
    Ok(codex_agent_sources_root()?.join(INSTALLS_FILENAME))
}

fn codex_agent_catalogs_dir() -> Result<PathBuf, String> {
    Ok(codex_agent_sources_root()?.join(CATALOGS_DIR))
}

fn codex_agent_catalog_path(source_id: &str) -> Result<PathBuf, String> {
    Ok(codex_agent_catalogs_dir()?.join(format!("{source_id}.json")))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建目录 '{}' 失败: {error}", parent.to_string_lossy()))?;
    }
    Ok(())
}

fn write_string_atomic(path: &Path, payload: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("目标路径 '{}' 缺少父目录", path.to_string_lossy()))?;
    let mut temp =
        NamedTempFile::new_in(parent).map_err(|error| format!("创建临时文件失败: {error}"))?;
    temp.write_all(payload.as_bytes())
        .map_err(|error| format!("写入临时文件失败: {error}"))?;
    temp.flush()
        .map_err(|error| format!("刷新临时文件失败: {error}"))?;
    temp.persist(path).map_err(|error| {
        format!(
            "原子替换 '{}' 失败: {}",
            path.to_string_lossy(),
            error.error
        )
    })?;
    Ok(())
}

fn hash_string(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn read_json_file<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("读取 '{}' 失败: {error}", path.to_string_lossy()))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("解析 '{}' 失败: {error}", path.to_string_lossy()))
}

fn save_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let payload = serde_json::to_string_pretty(value)
        .map_err(|error| format!("序列化 '{}' 失败: {error}", path.to_string_lossy()))?;
    write_string_atomic(path, &payload)
}

fn load_sources() -> Result<CodexAgentSourceFile, String> {
    read_json_file(&codex_agent_sources_path()?)
}

fn save_sources(file: &CodexAgentSourceFile) -> Result<(), String> {
    save_json_file(&codex_agent_sources_path()?, file)
}

fn load_installs() -> Result<CodexAgentInstallFile, String> {
    read_json_file(&codex_agent_installs_path()?)
}

fn save_installs(file: &CodexAgentInstallFile) -> Result<(), String> {
    save_json_file(&codex_agent_installs_path()?, file)
}

fn load_catalog(source_id: &str) -> Result<Option<CodexAgentCatalogFile>, String> {
    let path = codex_agent_catalog_path(source_id)?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(read_json_file(&path)?))
}

fn save_catalog(source_id: &str, catalog: &CodexAgentCatalogFile) -> Result<(), String> {
    save_json_file(&codex_agent_catalog_path(source_id)?, catalog)
}

fn remove_catalog(source_id: &str) -> Result<(), String> {
    let path = codex_agent_catalog_path(source_id)?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("删除 catalog '{}' 失败: {error}", path.to_string_lossy()))?;
    }
    Ok(())
}

fn source_record(entry: &CodexAgentSourceEntry) -> CodexAgentSourceRecord {
    let is_stale = entry
        .last_scanned_at
        .as_deref()
        .map(is_stale_scan)
        .unwrap_or(true);

    CodexAgentSourceRecord {
        id: entry.id.clone(),
        repo_url: entry.repo_url.clone(),
        owner: entry.owner.clone(),
        repo: entry.repo.clone(),
        default_branch: entry.default_branch.clone(),
        status: entry.status.clone(),
        last_scanned_at: entry.last_scanned_at.clone(),
        last_error: entry.last_error.clone(),
        agent_count: entry.agent_count,
        diagnostics_count: entry.diagnostics_count,
        scan_complete: entry.scan_complete,
        is_stale,
        cache_ttl_seconds: CATALOG_STALE_AFTER_SECS,
    }
}

fn is_stale_scan(timestamp: &str) -> bool {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| Utc::now().signed_duration_since(value.with_timezone(&Utc)).num_seconds() >= CATALOG_STALE_AFTER_SECS)
        .unwrap_or(true)
}

fn parse_github_repo(url: &str) -> Result<ParsedGitHubRepo, String> {
    let trimmed = url.trim().trim_end_matches('/');
    let trimmed = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .ok_or_else(|| "仅支持 GitHub 仓库 URL".to_string())?;
    let repo_root = trimmed.split("/tree/").next().unwrap_or(trimmed);
    let mut parts = repo_root.split('/');
    let owner = parts
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "GitHub URL 缺少 owner".to_string())?;
    let repo = parts
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "GitHub URL 缺少 repo".to_string())?;

    Ok(ParsedGitHubRepo {
        repo_url: format!("https://github.com/{owner}/{repo}"),
        owner: owner.to_string(),
        repo: repo.to_string(),
    })
}

fn infer_category(path: &str) -> (String, String) {
    let segments = path.split('/').collect::<Vec<_>>();
    let raw_category = if segments.len() >= 3 && segments.first() == Some(&"categories") {
        segments[1].to_string()
    } else if segments.len() >= 2 {
        segments[segments.len() - 2].to_string()
    } else {
        "root".to_string()
    };

    let label = raw_category
        .split_once('-')
        .map(|(_, value)| value)
        .unwrap_or(raw_category.as_str())
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    (
        raw_category,
        if label.is_empty() {
            "Root".to_string()
        } else {
            label
        },
    )
}

fn build_raw_github_url(
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(GITHUB_RAW_BASE)
        .map_err(|error| format!("构造 GitHub raw URL 失败: {error}"))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| "GitHub raw URL 不支持 path segments".to_string())?;
        segments.push(owner);
        segments.push(repo);
        segments.push(branch);
        for segment in path.split('/') {
            segments.push(segment);
        }
    }
    Ok(url)
}

fn format_github_http_error(status: StatusCode, body: &str, context: &str) -> String {
    let trimmed = body.trim();
    match status {
        StatusCode::NOT_FOUND => {
            format!("{context}: repository or file not found, or access is unavailable")
        }
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            if trimmed.to_ascii_lowercase().contains("rate limit") {
                format!("{context}: GitHub rate limit or access restriction")
            } else {
                format!("{context}: GitHub access denied or authentication is required")
            }
        }
        _ => {
            if trimmed.is_empty() {
                format!("{context}: GitHub request failed with status {status}")
            } else {
                format!("{context}: GitHub request failed with status {status}: {trimmed}")
            }
        }
    }
}

fn classify_scan_error_status(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("rate limit") {
        "rate-limited"
    } else if lower.contains("access denied") || lower.contains("authentication is required") {
        "access-denied"
    } else if lower.contains("not found") {
        "not-found"
    } else {
        "error"
    }
}

fn should_retry_status(status: StatusCode, body: &str) -> bool {
    if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        return true;
    }

    status == StatusCode::FORBIDDEN && body.to_ascii_lowercase().contains("rate limit")
}

async fn github_send_with_retry(
    _state: &AppState,
    request: reqwest::RequestBuilder,
    context: &str,
    allow_not_modified: bool,
) -> Result<reqwest::Response, String> {
    let mut last_error = None;

    for attempt in 0..GITHUB_RETRY_ATTEMPTS {
        let builder = request
            .try_clone()
            .ok_or_else(|| format!("{context}: request clone failed"))?;

        match builder.send().await {
            Ok(response)
                if response.status().is_success()
                    || (allow_not_modified && response.status() == StatusCode::NOT_MODIFIED) =>
            {
                return Ok(response);
            }
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                let error = format_github_http_error(status, &body, context);
                let retryable = should_retry_status(status, &body);
                last_error = Some(error);

                if !retryable || attempt + 1 == GITHUB_RETRY_ATTEMPTS {
                    break;
                }
            }
            Err(error) => {
                last_error = Some(format!("{context}: network request failed: {error}"));
                if attempt + 1 == GITHUB_RETRY_ATTEMPTS {
                    break;
                }
            }
        }

        let backoff = GITHUB_RETRY_BASE_DELAY_MS * (attempt as u64 + 1);
        sleep(Duration::from_millis(backoff)).await;
    }

    Err(last_error.unwrap_or_else(|| format!("{context}: request failed")))
}

async fn github_get_json<T: for<'de> Deserialize<'de>>(
    state: &AppState,
    url: &str,
) -> Result<T, String> {
    let request = state
        .http_client
        .get(url)
        .header("User-Agent", GITHUB_USER_AGENT)
        .header("Accept", "application/vnd.github+json");
    let response = github_send_with_retry(state, request, "GitHub API", false).await?;

    response
        .json::<T>()
        .await
        .map_err(|error| format!("解析 GitHub 响应失败: {error}"))
}

async fn github_get_text(state: &AppState, url: reqwest::Url) -> Result<String, String> {
    let request = state
        .http_client
        .get(url)
        .header("User-Agent", GITHUB_USER_AGENT);
    let response = github_send_with_retry(state, request, "GitHub raw download", false).await?;

    response
        .text()
        .await
        .map_err(|error| format!("读取 GitHub 文件内容失败: {error}"))
}

enum GitHubConditionalJson<T> {
    NotModified,
    Modified {
        value: T,
        etag: Option<String>,
    },
}

async fn github_get_json_conditional<T: for<'de> Deserialize<'de>>(
    state: &AppState,
    url: &str,
    etag: Option<&str>,
) -> Result<GitHubConditionalJson<T>, String> {
    let mut request = state
        .http_client
        .get(url)
        .header("User-Agent", GITHUB_USER_AGENT)
        .header("Accept", "application/vnd.github+json");

    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }

    let response = github_send_with_retry(state, request, "GitHub API", true).await?;
    if response.status() == StatusCode::NOT_MODIFIED {
        return Ok(GitHubConditionalJson::NotModified);
    }

    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let value = response
        .json::<T>()
        .await
        .map_err(|error| format!("解析 GitHub 响应失败: {error}"))?;

    Ok(GitHubConditionalJson::Modified { value, etag })
}

fn cache_entry_from_raw(
    source_path: &str,
    blob_sha: &str,
    raw_toml: String,
) -> (
    CodexRemoteAgentCacheEntry,
    Option<CodexAgentSourceDiagnostic>,
) {
    let file_path = Path::new(source_path);
    let (category, category_label) = infer_category(source_path);
    let content_hash = hash_string(&raw_toml);

    match toml::from_str::<toml::Value>(&raw_toml) {
        Ok(value) => {
            let file_name = file_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();

            if let Some(table) = value.as_table().cloned() {
                let record = record_from_table(file_path, raw_toml.clone(), table, None);
                (
                    CodexRemoteAgentCacheEntry {
                        id: hash_string(source_path),
                        source_path: source_path.to_string(),
                        file_name,
                        blob_sha: blob_sha.to_string(),
                        content_hash,
                        category,
                        category_label,
                        name: record.name,
                        description: record.description,
                        developer_instructions: record.developer_instructions,
                        nickname_candidates: record.nickname_candidates,
                        model: record.model,
                        model_reasoning_effort: record.model_reasoning_effort,
                        sandbox_mode: record.sandbox_mode,
                        mcp_servers: record.mcp_servers,
                        skills_config: record.skills_config,
                        other: record.other,
                        raw_toml,
                        parse_error: None,
                    },
                    None,
                )
            } else {
                let message = "Agent 文件顶层必须是 TOML table".to_string();
                (
                    CodexRemoteAgentCacheEntry {
                        id: hash_string(source_path),
                        source_path: source_path.to_string(),
                        file_name,
                        blob_sha: blob_sha.to_string(),
                        content_hash,
                        category,
                        category_label,
                        name: file_path
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        description: None,
                        developer_instructions: None,
                        nickname_candidates: Vec::new(),
                        model: None,
                        model_reasoning_effort: None,
                        sandbox_mode: None,
                        mcp_servers: None,
                        skills_config: None,
                        other: None,
                        raw_toml,
                        parse_error: Some(message.clone()),
                    },
                    Some(CodexAgentSourceDiagnostic {
                        path: source_path.to_string(),
                        severity: "error".to_string(),
                        message,
                    }),
                )
            }
        }
        Err(error) => {
            let message = format!("解析远程 agent TOML 失败: {error}");
            (
                CodexRemoteAgentCacheEntry {
                    id: hash_string(source_path),
                    source_path: source_path.to_string(),
                    file_name: file_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    blob_sha: blob_sha.to_string(),
                    content_hash,
                    category,
                    category_label,
                    name: file_path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    description: None,
                    developer_instructions: None,
                    nickname_candidates: Vec::new(),
                    model: None,
                    model_reasoning_effort: None,
                    sandbox_mode: None,
                    mcp_servers: None,
                    skills_config: None,
                    other: None,
                    raw_toml,
                    parse_error: Some(message.clone()),
                },
                Some(CodexAgentSourceDiagnostic {
                    path: source_path.to_string(),
                    severity: "error".to_string(),
                    message,
                }),
            )
        }
    }
}

async fn scan_source(
    state: &AppState,
    source: &CodexAgentSourceEntry,
    cached_catalog: Option<&CodexAgentCatalogFile>,
) -> Result<ScanOutput, String> {
    let tree_url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/trees/{}?recursive=1",
        source.owner, source.repo, source.default_branch
    );
    let payload = github_get_json_conditional::<GitHubTreePayload>(
        state,
        &tree_url,
        cached_catalog.and_then(|catalog| catalog.tree_etag.as_deref()),
    )
    .await?;
    let mut diagnostics = Vec::new();
    let mut agents = Vec::new();

    let (payload, tree_etag) = match payload {
        GitHubConditionalJson::NotModified => {
            if let Some(catalog) = cached_catalog {
                return Ok(ScanOutput {
                    status: catalog.status.clone(),
                    scan_complete: catalog.scan_complete,
                    truncated: catalog.truncated,
                    diagnostics: catalog.diagnostics.clone(),
                    agents: catalog.agents.clone(),
                    tree_sha: catalog.tree_sha.clone(),
                    tree_etag: catalog.tree_etag.clone(),
                });
            }

            return Err("GitHub returned 304 but no cached catalog is available".to_string());
        }
        GitHubConditionalJson::Modified { value, etag } => (value, etag),
    };

    if payload.truncated {
        diagnostics.push(CodexAgentSourceDiagnostic {
            path: source.repo_url.clone(),
            severity: "warning".to_string(),
            message: "GitHub tree response was truncated; the catalog is partial and may not include every agent.".to_string(),
        });
    }

    for entry in payload
        .tree
        .into_iter()
        .filter(|item| item.kind == "blob" && item.path.to_ascii_lowercase().ends_with(".toml"))
    {
        let raw_url = build_raw_github_url(
            &source.owner,
            &source.repo,
            &source.default_branch,
            &entry.path,
        )?;
        if let Some(cached) = cached_catalog
            .and_then(|catalog| {
                catalog
                    .agents
                    .iter()
                    .find(|agent| agent.source_path == entry.path && agent.blob_sha == entry.sha)
            })
            .cloned()
        {
            agents.push(cached);
            continue;
        }
        match github_get_text(state, raw_url).await {
            Ok(raw_toml) => {
                let (agent, diagnostic) = cache_entry_from_raw(&entry.path, &entry.sha, raw_toml);
                if let Some(diagnostic) = diagnostic {
                    diagnostics.push(diagnostic);
                }
                agents.push(agent);
            }
            Err(error) => diagnostics.push(CodexAgentSourceDiagnostic {
                path: entry.path.clone(),
                severity: "error".to_string(),
                message: error,
            }),
        }
    }

    agents.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    let has_entry_errors = diagnostics.iter().any(|item| item.severity == "error");
    let status = if payload.truncated || has_entry_errors {
        "partial".to_string()
    } else {
        "ok".to_string()
    };

    Ok(ScanOutput {
        status,
        scan_complete: !payload.truncated,
        truncated: payload.truncated,
        diagnostics,
        agents,
        tree_sha: Some(payload.sha),
        tree_etag,
    })
}

async fn refresh_source(
    state: &AppState,
    source: &mut CodexAgentSourceEntry,
) -> Result<CodexAgentCatalogFile, String> {
    let cached_catalog = load_catalog(&source.id)?;
    let scan = scan_source(state, source, cached_catalog.as_ref()).await?;
    source.status = scan.status.clone();
    source.scan_complete = scan.scan_complete;
    source.agent_count = scan.agents.len();
    source.diagnostics_count = scan.diagnostics.len();
    source.last_error = None;
    source.last_scanned_at = Some(Utc::now().to_rfc3339());

    let catalog = CodexAgentCatalogFile {
        source_id: source.id.clone(),
        repo_url: source.repo_url.clone(),
        default_branch: source.default_branch.clone(),
        status: scan.status,
        scan_complete: scan.scan_complete,
        truncated: scan.truncated,
        scanned_at: source.last_scanned_at.clone().unwrap_or_default(),
        tree_sha: scan.tree_sha,
        tree_etag: scan.tree_etag,
        diagnostics: scan.diagnostics,
        agents: scan.agents,
    };
    save_catalog(&source.id, &catalog)?;
    Ok(catalog)
}

async fn refresh_source_with_error_capture(
    state: &AppState,
    source: &mut CodexAgentSourceEntry,
) -> Result<CodexAgentCatalogFile, String> {
    match refresh_source(state, source).await {
        Ok(catalog) => Ok(catalog),
        Err(error) => {
            let scanned_at = Utc::now().to_rfc3339();
            source.status = classify_scan_error_status(&error).to_string();
            source.scan_complete = false;
            source.agent_count = 0;
            source.diagnostics_count = 1;
            source.last_error = Some(error.clone());
            source.last_scanned_at = Some(scanned_at.clone());

            let catalog = CodexAgentCatalogFile {
                source_id: source.id.clone(),
                repo_url: source.repo_url.clone(),
                default_branch: source.default_branch.clone(),
                status: source.status.clone(),
                scan_complete: false,
                truncated: false,
                scanned_at,
                tree_sha: None,
                tree_etag: None,
                diagnostics: vec![CodexAgentSourceDiagnostic {
                    path: source.repo_url.clone(),
                    severity: "error".to_string(),
                    message: error,
                }],
                agents: Vec::new(),
            };
            save_catalog(&source.id, &catalog)?;
            Ok(catalog)
        }
    }
}

fn remote_agent_record(
    source_id: &str,
    entry: &CodexRemoteAgentCacheEntry,
) -> CodexRemoteAgentRecord {
    CodexRemoteAgentRecord {
        id: entry.id.clone(),
        source_id: source_id.to_string(),
        source_path: entry.source_path.clone(),
        file_name: entry.file_name.clone(),
        blob_sha: entry.blob_sha.clone(),
        content_hash: entry.content_hash.clone(),
        category: entry.category.clone(),
        category_label: entry.category_label.clone(),
        name: entry.name.clone(),
        description: entry.description.clone(),
        developer_instructions: entry.developer_instructions.clone(),
        nickname_candidates: entry.nickname_candidates.clone(),
        model: entry.model.clone(),
        model_reasoning_effort: entry.model_reasoning_effort.clone(),
        sandbox_mode: entry.sandbox_mode.clone(),
        mcp_servers: entry.mcp_servers.clone(),
        skills_config: entry.skills_config.clone(),
        other: entry.other.clone(),
        raw_toml: entry.raw_toml.clone(),
        parse_error: entry.parse_error.clone(),
    }
}

fn evaluate_install_status(
    install: &CodexTrackedInstallEntry,
    catalog: Option<&CodexAgentCatalogFile>,
) -> CodexSourceInstallRecord {
    let mut status = "ok".to_string();
    let mut last_error = None;
    let mut has_upstream_update = false;
    let mut has_local_changes = false;

    let target_path = PathBuf::from(&install.target_path);
    let local_content = match fs::read_to_string(&target_path) {
        Ok(content) => content,
        Err(error) => {
            status = "broken".to_string();
            last_error = Some(format!("本地文件缺失或不可读: {error}"));
            return CodexSourceInstallRecord {
                id: install.id.clone(),
                source_id: install.source_id.clone(),
                repo_url: install.repo_url.clone(),
                source_path: install.source_path.clone(),
                installed_name: install.installed_name.clone(),
                target_path: install.target_path.clone(),
                status,
                last_synced_at: install.last_synced_at.clone(),
                last_error,
                has_upstream_update,
                has_local_changes,
            };
        }
    };

    let local_hash = hash_string(&local_content);
    has_local_changes = local_hash != install.installed_content_hash;

    if let Some(catalog) = catalog {
        if catalog.status == "partial" {
            status = "partial".to_string();
        }

        match catalog
            .agents
            .iter()
            .find(|agent| agent.source_path == install.source_path)
        {
            Some(remote) => {
                has_upstream_update = remote.content_hash != install.source_content_hash;
                status = match (has_upstream_update, has_local_changes) {
                    (false, false) => status,
                    (true, false) => "update-available".to_string(),
                    (false, true) => "local-modified".to_string(),
                    (true, true) => "conflict".to_string(),
                };
            }
            None => {
                status = "broken".to_string();
                last_error = Some("上游源中已找不到该 agent".to_string());
            }
        }
    } else {
        status = "unknown".to_string();
        last_error = Some("尚未扫描对应源，无法确认上游状态".to_string());
    }

    CodexSourceInstallRecord {
        id: install.id.clone(),
        source_id: install.source_id.clone(),
        repo_url: install.repo_url.clone(),
        source_path: install.source_path.clone(),
        installed_name: install.installed_name.clone(),
        target_path: install.target_path.clone(),
        status,
        last_synced_at: install.last_synced_at.clone(),
        last_error,
        has_upstream_update,
        has_local_changes,
    }
}

fn catalog_with_installs(
    source: &CodexAgentSourceEntry,
    catalog: &CodexAgentCatalogFile,
    installs: &[CodexTrackedInstallEntry],
) -> CodexAgentSourceCatalogResponse {
    let installs = installs
        .iter()
        .filter(|install| install.source_id == source.id)
        .map(|install| evaluate_install_status(install, Some(catalog)))
        .collect::<Vec<_>>();

    CodexAgentSourceCatalogResponse {
        source: source_record(source),
        agents: catalog
            .agents
            .iter()
            .map(|entry| remote_agent_record(&source.id, entry))
            .collect(),
        diagnostics: catalog.diagnostics.clone(),
        installs,
    }
}

async fn fetch_repo_metadata(
    state: &AppState,
    repo: &ParsedGitHubRepo,
) -> Result<GitHubRepoPayload, String> {
    let repo_url = format!("{GITHUB_API_BASE}/repos/{}/{}", repo.owner, repo.repo);
    github_get_json::<GitHubRepoPayload>(state, &repo_url).await
}

#[tauri::command]
pub async fn codex_list_agent_sources() -> Result<CodexAgentSourcesResponse, String> {
    let sources = load_sources()?;
    Ok(CodexAgentSourcesResponse {
        sources: sources.sources.iter().map(source_record).collect(),
    })
}

#[tauri::command]
pub async fn codex_add_agent_source(
    state: State<'_, AppState>,
    request: CodexAgentSourceRequest,
) -> Result<CodexAgentSourceRecord, String> {
    let repo = parse_github_repo(&request.url)?;
    let mut sources = load_sources()?;

    if let Some(existing) = sources
        .sources
        .iter_mut()
        .find(|entry| entry.repo_url == repo.repo_url)
    {
        existing.owner = repo.owner.clone();
        existing.repo = repo.repo.clone();
        if let Ok(metadata) = fetch_repo_metadata(&state, &repo).await {
            existing.default_branch = metadata.default_branch;
        }
        let _ = refresh_source_with_error_capture(&state, existing).await?;
        let response = source_record(existing);
        save_sources(&sources)?;
        return Ok(response);
    }

    let default_branch = fetch_repo_metadata(&state, &repo)
        .await
        .map(|metadata| metadata.default_branch)
        .unwrap_or_else(|_| "main".to_string());

    let mut entry = CodexAgentSourceEntry {
        id: format!("src_{}", hash_string(&repo.repo_url)),
        repo_url: repo.repo_url,
        owner: repo.owner,
        repo: repo.repo,
        default_branch,
        status: "pending".to_string(),
        last_scanned_at: None,
        last_error: None,
        agent_count: 0,
        diagnostics_count: 0,
        scan_complete: false,
    };
    let _ = refresh_source_with_error_capture(&state, &mut entry).await?;
    sources.sources.push(entry.clone());
    sources
        .sources
        .sort_by(|left, right| left.repo_url.cmp(&right.repo_url));
    save_sources(&sources)?;
    Ok(source_record(&entry))
}

#[tauri::command]
pub async fn codex_remove_agent_source(source_id: String) -> Result<(), String> {
    let mut sources = load_sources()?;
    let initial_len = sources.sources.len();
    sources.sources.retain(|entry| entry.id != source_id);
    if sources.sources.len() == initial_len {
        return Err(format!("Source '{source_id}' 不存在"));
    }
    save_sources(&sources)?;
    remove_catalog(&source_id)?;
    Ok(())
}

#[tauri::command]
pub async fn codex_sync_agent_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<CodexAgentSourceRecord, String> {
    let mut sources = load_sources()?;
    let source = sources
        .sources
        .iter_mut()
        .find(|entry| entry.id == source_id)
        .ok_or_else(|| format!("Source '{source_id}' 不存在"))?;
    let parsed = ParsedGitHubRepo {
        repo_url: source.repo_url.clone(),
        owner: source.owner.clone(),
        repo: source.repo.clone(),
    };
    if let Ok(metadata) = fetch_repo_metadata(&state, &parsed).await {
        source.default_branch = metadata.default_branch;
    }
    let _ = refresh_source_with_error_capture(&state, source).await?;
    let response = source_record(source);
    save_sources(&sources)?;
    Ok(response)
}

#[tauri::command]
pub async fn codex_get_agent_source_catalog(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<CodexAgentSourceCatalogResponse, String> {
    let mut sources = load_sources()?;
    let source_index = sources
        .sources
        .iter()
        .position(|entry| entry.id == source_id)
        .ok_or_else(|| format!("Source '{source_id}' 不存在"))?;

    let source_id = sources.sources[source_index].id.clone();
    let catalog = match load_catalog(&source_id)? {
        Some(catalog) => catalog,
        None => {
            let source = sources
                .sources
                .get_mut(source_index)
                .ok_or_else(|| format!("Source '{}' 不存在", source_id))?;
            let catalog = refresh_source_with_error_capture(&state, source).await?;
            save_sources(&sources)?;
            catalog
        }
    };

    let installs = load_installs()?;
    let source = sources
        .sources
        .get(source_index)
        .ok_or_else(|| format!("Source '{}' 不存在", source_id))?;
    let response = catalog_with_installs(source, &catalog, &installs.installs);
    Ok(response)
}

#[tauri::command]
pub async fn codex_install_source_agent(
    state: State<'_, AppState>,
    request: CodexAgentSourceInstallRequest,
) -> Result<Value, String> {
    let sources = load_sources()?;
    let source = sources
        .sources
        .iter()
        .find(|entry| entry.id == request.source_id)
        .ok_or_else(|| format!("Source '{}' 不存在", request.source_id))?;
    let catalog = load_catalog(&source.id)?
        .ok_or_else(|| format!("Source '{}' 还没有可用的扫描结果", source.id))?;
    let agent = catalog
        .agents
        .iter()
        .find(|entry| entry.id == request.agent_id)
        .ok_or_else(|| format!("Agent '{}' 不存在", request.agent_id))?;

    if let Some(parse_error) = &agent.parse_error {
        return Err(format!("远程 agent 无法安装: {parse_error}"));
    }

    let target_name = request
        .target_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| agent.name.clone());

    let agents_dir = codex_agents_dir()?;
    ensure_agents_dir(&agents_dir)?;
    let target_path = agent_file_path(&agents_dir, &target_name);
    let mut installs = load_installs()?;
    let existing_install = installs
        .installs
        .iter_mut()
        .find(|entry| entry.target_path == target_path.to_string_lossy());

    if target_path.exists() {
        match request.conflict_mode.as_deref() {
            Some("replace") => {
                if existing_install.is_none() {
                    return Err("同名本地 agent 已存在；仅跟踪中的远程安装允许 replace".to_string());
                }
            }
            _ => {
                return Err(format!(
                    "Agent '{}' 已存在；请改名安装或显式选择 replace",
                    target_name
                ));
            }
        }
    }

    let table = table_from_config(
        &json!({
            "name": target_name,
            "rawToml": agent.raw_toml,
        }),
        &target_name,
    )?;
    write_agent_file(&target_path, &table)?;

    let (_raw, written_table) = read_agent_table(&target_path)?;
    let installed_raw = fs::read_to_string(&target_path)
        .map_err(|error| format!("读取已安装 agent 失败: {error}"))?;
    let record = record_from_table(&target_path, installed_raw.clone(), written_table, None);
    let installed_content_hash = hash_string(&installed_raw);
    let now = Utc::now().to_rfc3339();

    match existing_install {
        Some(existing) => {
            existing.source_id = source.id.clone();
            existing.repo_url = source.repo_url.clone();
            existing.source_path = agent.source_path.clone();
            existing.source_blob_sha = agent.blob_sha.clone();
            existing.source_content_hash = agent.content_hash.clone();
            existing.installed_name = record.name.clone();
            existing.target_path = target_path.to_string_lossy().to_string();
            existing.installed_content_hash = installed_content_hash;
            existing.last_synced_at = Some(now.clone());
        }
        None => installs.installs.push(CodexTrackedInstallEntry {
            id: format!(
                "install_{}",
                hash_string(&format!("{}:{}", source.id, target_path.to_string_lossy()))
            ),
            source_id: source.id.clone(),
            repo_url: source.repo_url.clone(),
            source_path: agent.source_path.clone(),
            source_blob_sha: agent.blob_sha.clone(),
            source_content_hash: agent.content_hash.clone(),
            installed_name: record.name.clone(),
            target_path: target_path.to_string_lossy().to_string(),
            installed_content_hash,
            last_synced_at: Some(now.clone()),
        }),
    }
    save_installs(&installs)?;
    invalidate_codex_dashboard_overview_cache(&state).await;

    Ok(json!({
        "message": format!("Installed '{}' from {}", record.name, source.repo_url),
        "agent": record,
        "sourceId": source.id,
        "targetPath": target_path.to_string_lossy().to_string(),
    }))
}

#[tauri::command]
pub async fn codex_sync_source_install(
    state: State<'_, AppState>,
    request: CodexAgentSourceSyncRequest,
) -> Result<Value, String> {
    let mut installs = load_installs()?;
    let install = installs
        .installs
        .iter_mut()
        .find(|entry| entry.id == request.install_id)
        .ok_or_else(|| format!("Install '{}' 不存在", request.install_id))?;
    let sources = load_sources()?;
    let source = sources
        .sources
        .iter()
        .find(|entry| entry.id == install.source_id)
        .ok_or_else(|| format!("Source '{}' 不存在", install.source_id))?;

    let raw_url = build_raw_github_url(
        &source.owner,
        &source.repo,
        &source.default_branch,
        &install.source_path,
    )?;
    let remote_raw = github_get_text(&state, raw_url).await?;
    let remote_hash = hash_string(&remote_raw);
    let target_path = PathBuf::from(&install.target_path);
    let local_raw = fs::read_to_string(&target_path)
        .map_err(|error| format!("读取本地安装文件失败: {error}"))?;
    let local_hash = hash_string(&local_raw);

    let local_changed = local_hash != install.installed_content_hash;
    let remote_changed = remote_hash != install.source_content_hash;
    let force = request.force.unwrap_or(false);

    match (remote_changed, local_changed) {
        (false, false) => {
            return Ok(json!({
                "message": "No upstream or local changes detected",
                "status": "ok",
            }));
        }
        (false, true) => {
            return Err(
                "本地文件已被修改，但上游没有变化；无需 sync，请先决定是否保留本地修改".to_string(),
            );
        }
        (true, true) => {
            if !force {
                return Err(
                    "本地文件与上游都发生了变化，CCR 不会静默覆盖，请先手动解决冲突".to_string(),
                );
            }
        }
        (true, false) => {}
    }

    let table = table_from_config(
        &json!({
            "name": install.installed_name,
            "rawToml": remote_raw,
        }),
        &install.installed_name,
    )?;
    write_agent_file(&target_path, &table)?;
    let updated_raw = fs::read_to_string(&target_path)
        .map_err(|error| format!("读取同步后的安装文件失败: {error}"))?;
    let updated_hash = hash_string(&updated_raw);
    install.source_content_hash = remote_hash;
    install.installed_content_hash = updated_hash;
    install.last_synced_at = Some(Utc::now().to_rfc3339());
    let installed_name = install.installed_name.clone();
    let target_path_text = install.target_path.clone();
    save_installs(&installs)?;
    invalidate_codex_dashboard_overview_cache(&state).await;

    Ok(json!({
        "message": format!("Synced '{}' from upstream", installed_name),
        "status": "updated",
        "targetPath": target_path_text,
    }))
}

#[tauri::command]
pub async fn codex_accept_local_source_install(
    request: CodexAgentSourceInstallActionRequest,
) -> Result<Value, String> {
    let mut installs = load_installs()?;
    let install = installs
        .installs
        .iter_mut()
        .find(|entry| entry.id == request.install_id)
        .ok_or_else(|| format!("Install '{}' 不存在", request.install_id))?;

    let local_raw = fs::read_to_string(&install.target_path)
        .map_err(|error| format!("读取本地安装文件失败: {error}"))?;
    install.installed_content_hash = hash_string(&local_raw);
    install.last_synced_at = Some(Utc::now().to_rfc3339());
    let installed_name = install.installed_name.clone();
    save_installs(&installs)?;

    Ok(json!({
        "message": format!("Accepted local changes for '{}'", installed_name),
        "status": "local-accepted",
    }))
}

#[tauri::command]
pub async fn codex_untrack_source_install(
    request: CodexAgentSourceInstallActionRequest,
) -> Result<Value, String> {
    let mut installs = load_installs()?;
    let initial_len = installs.installs.len();
    let removed = installs
        .installs
        .iter()
        .find(|entry| entry.id == request.install_id)
        .map(|entry| entry.installed_name.clone())
        .ok_or_else(|| format!("Install '{}' 不存在", request.install_id))?;

    installs.installs.retain(|entry| entry.id != request.install_id);
    if installs.installs.len() == initial_len {
        return Err(format!("Install '{}' 不存在", request.install_id));
    }
    save_installs(&installs)?;

    Ok(json!({
        "message": format!("Stopped tracking '{}'", removed),
        "status": "untracked",
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn github_repo_parser_accepts_basic_repo_url() {
        let parsed =
            parse_github_repo("https://github.com/VoltAgent/awesome-codex-subagents").unwrap();
        assert_eq!(parsed.owner, "VoltAgent");
        assert_eq!(parsed.repo, "awesome-codex-subagents");
    }

    #[test]
    fn github_repo_parser_strips_tree_suffix() {
        let parsed = parse_github_repo(
            "https://github.com/VoltAgent/awesome-codex-subagents/tree/main/categories",
        )
        .unwrap();
        assert_eq!(parsed.owner, "VoltAgent");
        assert_eq!(parsed.repo, "awesome-codex-subagents");
    }

    #[test]
    fn infer_category_prefers_categories_segment() {
        let (raw, label) = infer_category("categories/04-quality-security/reviewer.toml");
        assert_eq!(raw, "04-quality-security");
        assert_eq!(label, "Quality Security");
    }

    #[test]
    fn evaluate_install_status_marks_conflict_when_both_changed() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("reviewer.toml");
        fs::write(
            &target,
            "name='reviewer'\ndescription='local'\ndeveloper_instructions='x'\n",
        )
        .unwrap();

        let install = CodexTrackedInstallEntry {
            id: "install_1".into(),
            source_id: "src_1".into(),
            repo_url: "https://github.com/example/repo".into(),
            source_path: "agents/reviewer.toml".into(),
            source_blob_sha: "blob".into(),
            source_content_hash: "old-remote".into(),
            installed_name: "reviewer".into(),
            target_path: target.to_string_lossy().to_string(),
            installed_content_hash: "old-local".into(),
            last_synced_at: None,
        };
        let catalog = CodexAgentCatalogFile {
            source_id: "src_1".into(),
            repo_url: "https://github.com/example/repo".into(),
            default_branch: "main".into(),
            status: "ok".into(),
            scan_complete: true,
            truncated: false,
            scanned_at: Utc::now().to_rfc3339(),
            tree_sha: Some("tree".into()),
            tree_etag: Some("\"etag\"".into()),
            diagnostics: Vec::new(),
            agents: vec![CodexRemoteAgentCacheEntry {
                id: "agent_1".into(),
                source_path: "agents/reviewer.toml".into(),
                file_name: "reviewer.toml".into(),
                blob_sha: "blob-2".into(),
                content_hash: "new-remote".into(),
                category: "agents".into(),
                category_label: "Agents".into(),
                name: "reviewer".into(),
                description: Some("desc".into()),
                developer_instructions: Some("do work".into()),
                nickname_candidates: Vec::new(),
                model: None,
                model_reasoning_effort: None,
                sandbox_mode: None,
                mcp_servers: None,
                skills_config: None,
                other: None,
                raw_toml: "name='reviewer'\ndescription='desc'\ndeveloper_instructions='do work'\n"
                    .into(),
                parse_error: None,
            }],
        };

        let status = evaluate_install_status(&install, Some(&catalog));
        assert_eq!(status.status, "conflict");
        assert!(status.has_upstream_update);
        assert!(status.has_local_changes);
    }
}
