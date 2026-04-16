// 🔐 Codex OAuth Token 同步/修复服务
//
// 背景:
// - CCR 会把 ~/.codex/auth.json 的 OAuth tokens 复制为命名账号快照：
//   ~/.ccr/platforms/codex/auth/<name>.json
// - OAuth refresh_token 轮换策略下，旧 refresh_token 一旦被用过就会失效；
//   若快照未及时回写，会出现 refresh_token_reused 导致配额查询失败。
//
// 该服务提供:
// - 从 runtime / backups 中解析最新 OAuth tokens（按 last_refresh 或文件 mtime）
// - 回写到 CCR 账号快照并同步 auth_registry.toml 元数据
// - CLI/TUI 可调用的 sync / repair 操作

use crate::models::codex_auth::CodexAuthTokens;
use crate::models::{CodexAuthJson, CodexAuthRegistry};
use crate::utils::CodexPaths;
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// OAuth 文档来源
#[derive(Debug, Clone)]
pub enum OAuthDocSource {
    RuntimeAuthJson,
    BackupFile(PathBuf),
}

impl OAuthDocSource {
    pub fn label(&self) -> String {
        match self {
            OAuthDocSource::RuntimeAuthJson => "~/.codex/auth.json".to_string(),
            OAuthDocSource::BackupFile(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| path.to_string_lossy().to_string()),
        }
    }
}

/// 解析得到的 OAuth tokens 文档
#[derive(Debug, Clone)]
pub struct ResolvedOAuthDoc {
    pub tokens: CodexAuthTokens,
    pub last_refresh: Option<DateTime<Utc>>,
    pub source: OAuthDocSource,
}

/// 修复结果
#[derive(Debug, Clone)]
pub struct OAuthRepairOutcome {
    pub updated: bool,
    pub source: Option<OAuthDocSource>,
    pub message: String,
}

/// Codex OAuth Token 同步/修复服务
pub struct CodexOAuthTokenService {
    /// CCR Codex 数据目录 (~/.ccr/platforms/codex/)
    ccr_codex_dir: PathBuf,
    /// Codex CLI 配置目录 (~/.codex/)
    codex_dir: PathBuf,
}

impl CodexOAuthTokenService {
    pub fn new() -> Result<Self> {
        let paths = CodexPaths::resolve()?;
        Ok(Self {
            ccr_codex_dir: paths.ccr_codex_dir,
            codex_dir: paths.codex_dir,
        })
    }

    /// 从显式路径构造（用于测试注入，避免 unsafe set_var）
    pub fn from_dirs(ccr_codex_dir: PathBuf, codex_dir: PathBuf) -> Self {
        Self {
            ccr_codex_dir,
            codex_dir,
        }
    }

    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_codex_dir.join("auth")
    }

    fn account_auth_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{}.json", name))
    }

    fn runtime_auth_json_path(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    fn codex_backups_dir(&self) -> PathBuf {
        self.codex_dir.join("backups")
    }

    fn registry_store(&self) -> super::codex_registry_store::CodexRegistryStore {
        super::codex_registry_store::CodexRegistryStore::new(&self.ccr_codex_dir)
    }

    fn load_registry(&self) -> Result<CodexAuthRegistry> {
        self.registry_store().load()
    }

    fn save_registry(&self, registry: &CodexAuthRegistry) -> Result<()> {
        self.registry_store().save(registry)
    }

    fn ensure_private_permissions(&self, path: &Path) {
        crate::utils::ensure_private_permissions(path);
    }

    fn parse_rfc3339(value: Option<&str>) -> Option<DateTime<Utc>> {
        value
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn system_time_secs(value: Option<SystemTime>) -> i64 {
        value
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    fn effective_ts(last_refresh: Option<DateTime<Utc>>, mtime: Option<SystemTime>) -> i64 {
        last_refresh
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|| Self::system_time_secs(mtime))
    }

    fn parse_oauth_doc_from_path(
        &self,
        path: &Path,
        source: OAuthDocSource,
        expected_account_id: &str,
    ) -> Result<Option<ResolvedOAuthDoc>> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };
        let auth: CodexAuthJson = match serde_json::from_str(&content) {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };
        let Some(tokens) = auth.tokens else {
            return Ok(None);
        };

        // 至少需要 access_token / refresh_token 才有修复意义
        if tokens
            .access_token
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_none()
        {
            return Ok(None);
        }
        if tokens
            .refresh_token
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_none()
        {
            return Ok(None);
        }

        let account_id = tokens
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                tokens
                    .access_token
                    .as_deref()
                    .and_then(Self::extract_account_id_from_jwt)
            });

        let Some(account_id) = account_id else {
            return Ok(None);
        };

        if account_id != expected_account_id {
            return Ok(None);
        }

        let last_refresh = Self::parse_rfc3339(auth.last_refresh.as_deref());
        Ok(Some(ResolvedOAuthDoc {
            tokens,
            last_refresh,
            source,
        }))
    }

    /// 解析最新的 OAuth tokens 文档
    ///
    /// 候选源:
    /// 1) runtime ~/.codex/auth.json
    /// 2) ~/.codex/backups/auth.*.json.bak（重点为 auth.runtime_switch.*）
    pub fn resolve_latest_oauth_doc(&self, account_id: &str) -> Result<Option<ResolvedOAuthDoc>> {
        let mut best: Option<(i64, ResolvedOAuthDoc)> = None;

        let runtime_path = self.runtime_auth_json_path();
        if runtime_path.exists()
            && let Some(doc) = self.parse_oauth_doc_from_path(
                &runtime_path,
                OAuthDocSource::RuntimeAuthJson,
                account_id,
            )?
        {
            let mtime = fs::metadata(&runtime_path)
                .ok()
                .and_then(|m| m.modified().ok());
            let ts = Self::effective_ts(doc.last_refresh, mtime);
            best = Some((ts, doc));
        }

        let backups_dir = self.codex_backups_dir();
        if backups_dir.exists() {
            let mut entries: Vec<(bool, PathBuf, Option<SystemTime>)> = Vec::new();
            for entry in fs::read_dir(&backups_dir)
                .map_err(|e| CcrError::ConfigError(format!("读取 Codex backups 目录失败: {}", e)))?
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if !file_name.starts_with("auth.") || !file_name.ends_with(".json.bak") {
                    continue;
                }
                // 优先扫描 runtime_switch（仍允许其他前缀作为补充）
                let preferred = file_name.contains("runtime_switch");
                let mtime = entry.metadata().ok().and_then(|m| m.modified().ok());
                entries.push((preferred, path, mtime));
            }

            // 先解析 runtime_switch，再解析其他；各自再按 mtime 倒序
            entries.sort_by_key(|(preferred, _path, mtime)| {
                (
                    std::cmp::Reverse(*preferred),
                    std::cmp::Reverse(Self::system_time_secs(*mtime)),
                )
            });
            for (_preferred, path, mtime) in entries.into_iter().take(120) {
                if let Some(doc) = self.parse_oauth_doc_from_path(
                    &path,
                    OAuthDocSource::BackupFile(path.clone()),
                    account_id,
                )? {
                    let ts = Self::effective_ts(doc.last_refresh, mtime);
                    match &best {
                        Some((best_ts, _)) if *best_ts >= ts => {}
                        _ => best = Some((ts, doc)),
                    }
                }
            }
        }

        Ok(best.map(|(_, doc)| doc))
    }

    /// 将 OAuth tokens 回写到 CCR 账号快照（.ccr/platforms/codex/auth/<name>.json）
    pub fn sync_account_auth_file(&self, name: &str, doc: &ResolvedOAuthDoc) -> Result<()> {
        let path = self.account_auth_path(name);
        if !path.exists() {
            return Err(CcrError::ConfigError(format!(
                "账号 '{}' 的 auth 快照不存在: {:?}",
                name, path
            )));
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取账号 auth 快照失败: {}", e)))?;

        let mut value: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        // tokens
        let tokens_value = serde_json::to_value(&doc.tokens)
            .map_err(|e| CcrError::ConfigError(format!("序列化 tokens 失败: {}", e)))?;
        value["tokens"] = tokens_value;

        // last_refresh
        let now = Utc::now();
        let ts = doc
            .last_refresh
            .unwrap_or(now)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        value["last_refresh"] = serde_json::Value::String(ts);

        let content = serde_json::to_string_pretty(&value)
            .map_err(|e| CcrError::ConfigError(format!("序列化账号 auth 快照失败: {}", e)))?;

        AtomicWriter::new(&path).write_string(&content)?;
        self.ensure_private_permissions(&path);
        Ok(())
    }

    /// 更新 auth_registry.toml 中该账号的 last_refresh 元数据
    pub fn update_registry_metadata(&self, name: &str, doc: &ResolvedOAuthDoc) -> Result<()> {
        let mut registry = self.load_registry()?;
        let Some(account) = registry.accounts.get_mut(name) else {
            return Ok(());
        };
        account.last_refresh = Some(doc.last_refresh.unwrap_or_else(Utc::now));
        self.save_registry(&registry)
    }

    /// 将当前 runtime OAuth tokens 回写到匹配的已保存账号
    ///
    /// 返回: Ok(Some(account_name)) 表示已回写；Ok(None) 表示无可回写对象
    pub fn sync_runtime_tokens_to_saved_account(&self) -> Result<Option<String>> {
        let runtime_path = self.runtime_auth_json_path();
        if !runtime_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&runtime_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 runtime auth.json 失败: {}", e)))?;
        let auth: CodexAuthJson = serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 runtime auth.json 失败: {}", e)))?;
        let Some(tokens) = auth.tokens else {
            return Ok(None);
        };

        let account_id = tokens
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                tokens
                    .access_token
                    .as_deref()
                    .and_then(Self::extract_account_id_from_jwt)
            });
        let Some(account_id) = account_id else {
            return Ok(None);
        };

        let last_refresh = Self::parse_rfc3339(auth.last_refresh.as_deref());
        let doc = ResolvedOAuthDoc {
            tokens,
            last_refresh,
            source: OAuthDocSource::RuntimeAuthJson,
        };

        let registry = self.load_registry()?;
        let matched = registry
            .accounts
            .iter()
            .find_map(|(name, account)| (account.account_id == account_id).then(|| name.clone()));

        let Some(name) = matched else {
            return Ok(None);
        };

        debug!(
            "Sync runtime OAuth tokens to saved account '{}' (account_id: {})",
            name, account_id
        );
        self.sync_account_auth_file(&name, &doc)?;
        self.update_registry_metadata(&name, &doc)?;
        Ok(Some(name))
    }

    /// 修复指定账号快照中的 OAuth tokens（从 runtime/backups 中找最新副本）
    pub fn repair_saved_account(&self, name: &str) -> Result<OAuthRepairOutcome> {
        let path = self.account_auth_path(name);
        if !path.exists() {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: format!("账号 '{}' 的 auth 快照不存在", name),
            });
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取账号 auth 快照失败: {}", e)))?;
        let auth: CodexAuthJson = serde_json::from_str(&raw)
            .map_err(|e| CcrError::ConfigError(format!("解析账号 auth 快照失败: {}", e)))?;
        let Some(tokens) = auth.tokens else {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "账号缺少 OAuth tokens（可能是 API Key 模式）".to_string(),
            });
        };

        let account_id = tokens
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                tokens
                    .access_token
                    .as_deref()
                    .and_then(Self::extract_account_id_from_jwt)
            });
        let Some(account_id) = account_id else {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "账号 OAuth tokens 缺少 account_id，无法修复".to_string(),
            });
        };

        let current_last_refresh = Self::parse_rfc3339(auth.last_refresh.as_deref());
        let current_mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        let current_ts = Self::effective_ts(current_last_refresh, current_mtime);

        let Some(latest) = self.resolve_latest_oauth_doc(&account_id)? else {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "未在 runtime/backups 中找到可用的 OAuth tokens".to_string(),
            });
        };

        let latest_path = match &latest.source {
            OAuthDocSource::RuntimeAuthJson => self.runtime_auth_json_path(),
            OAuthDocSource::BackupFile(p) => p.clone(),
        };
        let latest_mtime = fs::metadata(&latest_path)
            .ok()
            .and_then(|m| m.modified().ok());
        let latest_ts = Self::effective_ts(latest.last_refresh, latest_mtime);

        let latest_refresh = latest
            .tokens
            .refresh_token
            .as_deref()
            .map(|s: &str| s.trim())
            .filter(|s| !s.is_empty());
        let current_refresh = tokens
            .refresh_token
            .as_deref()
            .map(|s: &str| s.trim())
            .filter(|s| !s.is_empty());
        let refresh_changed = latest_refresh != current_refresh;

        let should_update = refresh_changed || latest_ts > current_ts;
        if !should_update {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: Some(latest.source),
                message: "已是最新 tokens，无需修复".to_string(),
            });
        }

        self.sync_account_auth_file(name, &latest)?;
        self.update_registry_metadata(name, &latest)?;

        let source_label = latest.source.label();
        let source = latest.source;
        Ok(OAuthRepairOutcome {
            updated: true,
            source: Some(source),
            message: format!("已从 {} 修复 OAuth tokens", source_label),
        })
    }

    /// 从 JWT access_token 中提取 chatgpt_account_id
    fn extract_account_id_from_jwt(access_token: &str) -> Option<String> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload = Self::decode_base64_url(parts[1])?;
        let value: serde_json::Value = serde_json::from_slice(&payload).ok()?;

        value
            .get("chatgpt_account_id")
            .or_else(|| value.get("account_id"))
            .or_else(|| {
                value
                    .get("https://api.openai.com/auth")
                    .and_then(|v| v.get("account_id"))
            })
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    fn decode_base64_url(input: &str) -> Option<Vec<u8>> {
        crate::utils::decode_base64url(input)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn write_json(path: &Path, value: &serde_json::Value) {
        let content = serde_json::to_string_pretty(value).unwrap();
        AtomicWriter::new(path).write_string(&content).unwrap();
        crate::utils::ensure_private_permissions(path);
    }

    fn setup_dirs() -> (TempDir, PathBuf, TempDir, PathBuf) {
        let ccr_root = tempfile::tempdir().unwrap();
        let codex_root = tempfile::tempdir().unwrap();
        let ccr_codex_dir = ccr_root.path().join("platforms/codex");
        let codex_dir = codex_root.path().to_path_buf();
        fs::create_dir_all(ccr_codex_dir.join("auth")).unwrap();
        fs::create_dir_all(codex_dir.join("backups")).unwrap();
        (ccr_root, ccr_codex_dir, codex_root, codex_dir)
    }

    #[test]
    fn test_resolve_latest_oauth_doc_prefers_newer_last_refresh_or_mtime() {
        let (_ccr_root, ccr_codex_dir, _codex_root, codex_dir) = setup_dirs();

        let service = CodexOAuthTokenService::from_dirs(ccr_codex_dir.clone(), codex_dir.clone());
        assert_eq!(service.ccr_codex_dir, ccr_codex_dir);

        let acc_id = "acc-123";
        let runtime = service.runtime_auth_json_path();
        write_json(
            &runtime,
            &json!({
                "tokens": {
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_old",
                    "account_id": acc_id
                },
                "last_refresh": "2026-03-01T00:00:00Z"
            }),
        );

        let backup_new = service
            .codex_backups_dir()
            .join("auth.runtime_switch.20260326_000000.json.bak");
        write_json(
            &backup_new,
            &json!({
                "tokens": {
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_new",
                    "account_id": acc_id
                },
                "last_refresh": "2026-03-26T00:00:00Z"
            }),
        );

        let best = service.resolve_latest_oauth_doc(acc_id).unwrap().unwrap();
        assert_eq!(
            best.tokens.refresh_token.as_deref(),
            Some("rt_new"),
            "should pick latest by last_refresh"
        );
    }

    #[test]
    fn test_repair_saved_account_updates_snapshot_and_registry() {
        let (_ccr_root, ccr_codex_dir, _codex_root, codex_dir) = setup_dirs();

        let service = CodexOAuthTokenService::from_dirs(ccr_codex_dir.clone(), codex_dir.clone());

        // registry with one account
        let mut registry = CodexAuthRegistry::default();
        registry.accounts.insert(
            "team".to_string(),
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-1".to_string(),
                auth_method: Some(crate::models::OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );
        service.save_registry(&registry).unwrap();

        // saved snapshot contains old token
        let saved = service.account_auth_path("team");
        write_json(
            &saved,
            &json!({
                "tokens": {
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_old",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-01T00:00:00Z"
            }),
        );

        // backup contains new token
        let backup = service
            .codex_backups_dir()
            .join("auth.runtime_switch.20260326_000000.json.bak");
        write_json(
            &backup,
            &json!({
                "tokens": {
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_new",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-26T00:00:00Z"
            }),
        );

        let outcome = service.repair_saved_account("team").unwrap();
        assert!(outcome.updated);

        // snapshot updated
        let updated: CodexAuthJson =
            serde_json::from_str(&fs::read_to_string(&saved).unwrap()).unwrap();
        assert_eq!(
            updated.tokens.unwrap().refresh_token.as_deref().unwrap(),
            "rt_new"
        );

        // registry last_refresh updated
        let registry2 = service.load_registry().unwrap();
        let acc = registry2.accounts.get("team").unwrap();
        assert!(acc.last_refresh.is_some());
    }
}
