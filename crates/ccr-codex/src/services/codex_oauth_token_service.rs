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

use super::codex_auth_identity::{OAuthIdentity, identity_from_auth};
use super::codex_auth_refresh_lock::{CredentialLocks, CredentialResource};
use super::codex_registry_store::REGISTRY_READ_ONLY_PREFIX;
use crate::models::codex_auth::CodexAuthTokens;
use crate::models::{CodexAuthJson, CodexAuthRegistry};
use crate::utils::CodexPaths;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{
    VersionedWriteOutcome, WriteOptions, content_version_token,
    enforce_owner_only_permissions_versioned, write_guarded_versioned,
};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

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

/// runtime ↔ 已保存快照的新鲜度定向同步动作
#[derive(Clone)]
pub enum RuntimeSyncPlan {
    /// 无可同步对象（无 runtime / 非 OAuth / 缺完整 OAuth 身份 / 无匹配账号或快照）
    NoOp,
    /// tokens 相同，无需写入
    Unchanged { account: String },
    /// runtime 不旧于快照：写快照
    WriteSnapshot {
        account: String,
        doc: ResolvedOAuthDoc,
    },
    /// 快照较新且目标为 current_auth：写回 runtime
    WriteRuntime {
        account: String,
        auth: serde_json::Map<String, serde_json::Value>,
    },
    /// 快照较新但目标非 current_auth：跳过
    SkipStaleRuntime { account: String },
}

impl std::fmt::Debug for RuntimeSyncPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoOp => f.write_str("NoOp"),
            Self::Unchanged { account } => f
                .debug_struct("Unchanged")
                .field("account", account)
                .finish(),
            Self::WriteSnapshot { account, doc } => f
                .debug_struct("WriteSnapshot")
                .field("account", account)
                .field("doc", doc)
                .finish(),
            Self::WriteRuntime { account, .. } => f
                .debug_struct("WriteRuntime")
                .field("account", account)
                .field("auth", &"[REDACTED]")
                .finish(),
            Self::SkipStaleRuntime { account } => f
                .debug_struct("SkipStaleRuntime")
                .field("account", account)
                .finish(),
        }
    }
}

/// 观测点同步的执行结果（账号名均为已保存账号）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSyncOutcome {
    NoOp,
    Unchanged(String),
    SnapshotUpdated(String),
    RuntimeUpdated(String),
    SkippedStaleRuntime(String),
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

    fn parse_rfc3339(value: Option<&str>) -> Option<DateTime<Utc>> {
        value
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn effective_ts(last_refresh: Option<DateTime<Utc>>, mtime: Option<SystemTime>) -> i128 {
        last_refresh
            .map(|dt| {
                i128::from(dt.timestamp()) * 1_000_000_000 + i128::from(dt.timestamp_subsec_nanos())
            })
            .unwrap_or_else(|| {
                mtime
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i128)
                    .unwrap_or(0)
            })
    }

    fn parse_oauth_doc_from_path(
        &self,
        path: &Path,
        source: OAuthDocSource,
        expected_identity_key: &str,
    ) -> Result<Option<ResolvedOAuthDoc>> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };
        let auth: CodexAuthJson = match serde_json::from_str(&content) {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };
        if identity_from_auth(&auth).is_none_or(|identity| identity.key() != expected_identity_key)
        {
            return Ok(None);
        }
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
    pub fn resolve_latest_oauth_doc(&self, identity_key: &str) -> Result<Option<ResolvedOAuthDoc>> {
        let mut best: Option<(i128, ResolvedOAuthDoc)> = None;

        let runtime_path = self.runtime_auth_json_path();
        if runtime_path.exists()
            && let Some(doc) = self.parse_oauth_doc_from_path(
                &runtime_path,
                OAuthDocSource::RuntimeAuthJson,
                identity_key,
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
                (std::cmp::Reverse(*preferred), std::cmp::Reverse(*mtime))
            });
            for (_preferred, path, mtime) in entries.into_iter().take(120) {
                if let Some(doc) = self.parse_oauth_doc_from_path(
                    &path,
                    OAuthDocSource::BackupFile(path.clone()),
                    identity_key,
                )? {
                    let ts = Self::effective_ts(doc.last_refresh, mtime);
                    match &best {
                        Some((best_ts, _)) if *best_ts >= ts => {}
                        _ => best = Some((ts, doc)),
                    }
                }
            }
        }

        // Registered aliases can hold a rotation completed by another query.
        // Validate names and registry/snapshot identity before using each source.
        let registry = self.load_registry()?;
        let auth_service = super::codex_auth_service::CodexAuthService::from_dirs_with_env_lock(
            self.ccr_codex_dir.clone(),
            self.codex_dir.clone(),
        );
        for name in registry.accounts.keys() {
            if auth_service.validate_account_name(name).is_err()
                || self
                    .saved_identity(&registry, name)
                    .is_none_or(|identity| identity.key() != identity_key)
            {
                continue;
            }
            let path = self.account_auth_path(name);
            if let Some(doc) = self.parse_oauth_doc_from_path(
                &path,
                OAuthDocSource::BackupFile(path.clone()),
                identity_key,
            )? {
                let ts = Self::effective_ts(doc.last_refresh, Self::mtime(&path));
                if best.as_ref().is_none_or(|(best_ts, _)| ts > *best_ts) {
                    best = Some((ts, doc));
                }
            }
        }
        Ok(best.map(|(_, doc)| doc))
    }

    /// Caller holds the complete identity resource; unknown identities never associate aliases.
    pub(crate) fn sync_registered_aliases_locked(&self, source_path: &Path) -> Result<()> {
        let auth: CodexAuthJson = serde_json::from_slice(&fs::read(source_path)?)
            .map_err(|_| CcrError::ConfigError("auth 来源无法解析，跳过别名同步".into()))?;
        let Some(identity) = identity_from_auth(&auth) else {
            return Ok(());
        };
        let Some(doc) = self.parse_oauth_doc_from_path(
            source_path,
            if source_path == self.runtime_auth_json_path() {
                OAuthDocSource::RuntimeAuthJson
            } else {
                OAuthDocSource::BackupFile(source_path.to_path_buf())
            },
            &identity.key(),
        )?
        else {
            return Ok(());
        };
        let registry = self.load_registry()?;
        let auth_service = super::codex_auth_service::CodexAuthService::from_dirs_with_env_lock(
            self.ccr_codex_dir.clone(),
            self.codex_dir.clone(),
        );
        for name in registry.accounts.keys() {
            if auth_service.validate_account_name(name).is_err()
                || self.saved_identity(&registry, name).as_ref() != Some(&identity)
            {
                continue;
            }
            let path = self.account_auth_path(name);
            if path == source_path {
                continue;
            }
            let Some(current) = self.parse_oauth_doc_from_path(
                &path,
                OAuthDocSource::BackupFile(path.clone()),
                &identity.key(),
            )?
            else {
                continue;
            };
            if Self::tokens_equal(&current.tokens, &doc.tokens)
                || Self::effective_ts(current.last_refresh, Self::mtime(&path))
                    > Self::effective_ts(doc.last_refresh, Self::mtime(source_path))
            {
                continue;
            }
            self.apply_snapshot_write_locked(name, &doc)?;
        }
        Ok(())
    }

    /// 将 OAuth tokens 回写到 CCR 账号快照（.ccr/platforms/codex/auth/<name>.json）
    pub fn sync_account_auth_file(&self, name: &str, doc: &ResolvedOAuthDoc) -> Result<()> {
        let path = self.account_auth_path(name);
        let locks = CredentialLocks::acquire_sources(vec![(
            path.clone(),
            CredentialResource::from_path(&path),
        )])?;
        locks.verify_path(&path)?;
        self.sync_account_auth_file_locked(name, doc)
    }

    pub(crate) fn sync_account_auth_file_locked(
        &self,
        name: &str,
        doc: &ResolvedOAuthDoc,
    ) -> Result<()> {
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

        let snapshot: CodexAuthJson = serde_json::from_value(value.clone())
            .map_err(|_| CcrError::ConfigError("账号 auth 快照无法解析，跳过同步".into()))?;
        let identity = identity_from_auth(&snapshot);
        if identity.is_none() || identity != OAuthIdentity::from_tokens(&doc.tokens) {
            return Err(CcrError::ConfigError(
                "OAuth 身份不完整或不匹配，跳过同步".into(),
            ));
        }
        let registry = self.load_registry()?;
        if registry
            .accounts
            .get(name)
            .and_then(|account| account.identity_key.as_deref())
            .is_some_and(|key| {
                identity
                    .as_ref()
                    .is_none_or(|identity| identity.key() != key)
            })
        {
            return Err(CcrError::ConfigError(
                "OAuth 快照身份与注册表不匹配，跳过同步".into(),
            ));
        }

        let source_path = match &doc.source {
            OAuthDocSource::RuntimeAuthJson => self.runtime_auth_json_path(),
            OAuthDocSource::BackupFile(path) => path.clone(),
        };
        let source_ts = Self::effective_ts(doc.last_refresh, Self::mtime(&source_path));
        let snapshot_ts = Self::effective_ts(
            Self::parse_rfc3339(snapshot.last_refresh.as_deref()),
            Self::mtime(&path),
        );
        if source_ts < snapshot_ts {
            return Err(CcrError::ConfigError("账号 auth 快照较新，跳过同步".into()));
        }

        // tokens
        let tokens_value = serde_json::to_value(&doc.tokens)
            .map_err(|e| CcrError::ConfigError(format!("序列化 tokens 失败: {}", e)))?;
        value["tokens"] = tokens_value;

        // last_refresh
        let now = Utc::now();
        let ts = doc
            .last_refresh
            .unwrap_or(now)
            .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
        value["last_refresh"] = serde_json::Value::String(ts);

        let content = serde_json::to_string_pretty(&value)
            .map_err(|e| CcrError::ConfigError(format!("序列化账号 auth 快照失败: {}", e)))?;

        let expected_version = content_version_token(raw.as_bytes());
        if !enforce_owner_only_permissions_versioned(
            &path,
            &expected_version,
            Duration::from_secs(10),
        )? {
            return Err(CcrError::ConfigError(
                "账号 auth 快照已变化，跳过同步".into(),
            ));
        }
        if write_guarded_versioned(
            &path,
            content.as_bytes(),
            &expected_version,
            &WriteOptions {
                secret: true,
                ..Default::default()
            },
        )? == VersionedWriteOutcome::Conflict
        {
            return Err(CcrError::ConfigError(
                "账号 auth 快照已变化，跳过同步".into(),
            ));
        }
        Ok(())
    }

    /// 更新 auth_registry.toml 中该账号的 last_refresh 元数据
    pub fn update_registry_metadata(&self, name: &str, doc: &ResolvedOAuthDoc) -> Result<()> {
        let mut registry = self.load_registry()?;
        let Some(account) = registry.accounts.get_mut(name) else {
            return Ok(());
        };
        account.last_refresh = Some(doc.last_refresh.unwrap_or_else(Utc::now));
        match self.save_registry(&registry) {
            Err(CcrError::ConfigError(message))
                if message.starts_with(REGISTRY_READ_ONLY_PREFIX) =>
            {
                warn!("Skipped auth registry metadata update: {}", message);
                Ok(())
            }
            result => result,
        }
    }

    /// 将当前 runtime OAuth tokens 回写到匹配的已保存账号
    ///
    /// 仅在 runtime 不旧于快照时写入（新鲜度定向，见 [`Self::plan_runtime_sync`]）。
    /// 返回: Ok(Some(account_name)) 表示快照已与 runtime 一致；Ok(None) 表示无可回写对象或快照较新
    pub fn sync_runtime_tokens_to_saved_account(&self) -> Result<Option<String>> {
        let path = self.runtime_auth_json_path();
        let locks = CredentialLocks::acquire_sources(vec![(
            path.clone(),
            CredentialResource::from_path(&path),
        )])?;
        locks.verify_path(&path)?;
        self.backfill_identity_keys()?;
        match self.plan_runtime_sync()? {
            RuntimeSyncPlan::WriteSnapshot { account, doc } => {
                self.apply_snapshot_write_locked(&account, &doc)?;
                Ok(Some(account))
            }
            RuntimeSyncPlan::Unchanged { account } => Ok(self
                .harden_unchanged_auth_locked(&account)?
                .then_some(account)),
            RuntimeSyncPlan::WriteRuntime { account, .. }
            | RuntimeSyncPlan::SkipStaleRuntime { account } => {
                debug!(
                    "Saved snapshot for '{}' is newer than runtime; runtime tokens not copied",
                    account
                );
                Ok(None)
            }
            RuntimeSyncPlan::NoOp => Ok(None),
        }
    }

    /// Caller holds the runtime credential resources; the planner stays read-only.
    pub(crate) fn harden_unchanged_auth_locked(&self, name: &str) -> Result<bool> {
        let runtime = fs::read(self.runtime_auth_json_path())?;
        let snapshot = fs::read(self.account_auth_path(name))?;
        self.harden_unchanged_pair(name, &runtime, &snapshot)
    }

    fn harden_unchanged_pair(&self, name: &str, runtime: &[u8], snapshot: &[u8]) -> Result<bool> {
        let (Ok(runtime_auth), Ok(snapshot_auth)) = (
            serde_json::from_slice::<CodexAuthJson>(runtime),
            serde_json::from_slice::<CodexAuthJson>(snapshot),
        ) else {
            return Ok(false);
        };
        let Some(identity) = identity_from_auth(&runtime_auth) else {
            return Ok(false);
        };
        if identity_from_auth(&snapshot_auth).as_ref() != Some(&identity)
            || self
                .select_sync_target(&self.load_registry()?, &identity)
                .as_deref()
                != Some(name)
            || !runtime_auth
                .tokens
                .as_ref()
                .zip(snapshot_auth.tokens.as_ref())
                .is_some_and(|(runtime, snapshot)| Self::tokens_equal(runtime, snapshot))
        {
            return Ok(false);
        }
        if !enforce_owner_only_permissions_versioned(
            &self.runtime_auth_json_path(),
            &content_version_token(runtime),
            Duration::from_secs(10),
        )? {
            return Ok(false);
        }
        enforce_owner_only_permissions_versioned(
            &self.account_auth_path(name),
            &content_version_token(snapshot),
            Duration::from_secs(10),
        )
    }

    pub(crate) fn harden_quota_runtime_locked(
        &self,
        expected_identity: &OAuthIdentity,
    ) -> Result<()> {
        let runtime = fs::read(self.runtime_auth_json_path())?;
        let auth: CodexAuthJson = serde_json::from_slice(&runtime)
            .map_err(|_| CcrError::ConfigError("runtime auth 身份无法解析，跳过配额准备".into()))?;
        if identity_from_auth(&auth).as_ref() != Some(expected_identity) {
            return Err(CcrError::ConfigError(
                "auth 来源身份已变化，跳过查询".into(),
            ));
        }
        if let Some(name) = self.select_sync_target(&self.load_registry()?, expected_identity)
            && let Ok(snapshot) = fs::read(self.account_auth_path(&name))
            && let Ok(snapshot_auth) = serde_json::from_slice::<CodexAuthJson>(&snapshot)
            && identity_from_auth(&snapshot_auth).as_ref() == Some(expected_identity)
            && auth
                .tokens
                .as_ref()
                .zip(snapshot_auth.tokens.as_ref())
                .is_some_and(|(runtime, saved)| Self::tokens_equal(runtime, saved))
        {
            if self.harden_unchanged_pair(&name, &runtime, &snapshot)? {
                return Ok(());
            }
        } else if enforce_owner_only_permissions_versioned(
            &self.runtime_auth_json_path(),
            &content_version_token(&runtime),
            Duration::from_secs(10),
        )? {
            return Ok(());
        }
        Err(CcrError::ConfigError(
            "auth 文件已变化，跳过配额准备".into(),
        ))
    }

    /// 执行快照方向的回写（快照 + 注册表 last_refresh）
    pub fn apply_snapshot_write(&self, name: &str, doc: &ResolvedOAuthDoc) -> Result<()> {
        let path = self.account_auth_path(name);
        let locks = CredentialLocks::acquire_sources(vec![(
            path.clone(),
            CredentialResource::from_path(&path),
        )])?;
        locks.verify_path(&path)?;
        self.apply_snapshot_write_locked(name, doc)
    }

    pub(crate) fn apply_snapshot_write_locked(
        &self,
        name: &str,
        doc: &ResolvedOAuthDoc,
    ) -> Result<()> {
        debug!("Sync runtime OAuth tokens to saved account '{}'", name);
        self.sync_account_auth_file_locked(name, doc)?;
        self.update_registry_metadata(name, doc)
    }

    /// 计算 runtime auth.json 与匹配的已保存快照之间的新鲜度定向同步动作（只读）
    ///
    /// 规则:
    /// - runtime 不存在 / 无 tokens / 无完整 OAuth 身份 / 无匹配账号 → NoOp
    /// - 同完整 OAuth 身份多个账号 → 目标取 current_auth，其次 last_used 最新者，其余不写
    /// - tokens 相同 → Unchanged（不写文件）
    /// - runtime 不旧于快照 → WriteSnapshot
    /// - 快照较新且目标为 current_auth → WriteRuntime（保留 runtime 其他字段，仅替换 tokens/last_refresh）
    /// - 快照较新且非 current_auth → SkipStaleRuntime
    pub fn plan_runtime_sync(&self) -> Result<RuntimeSyncPlan> {
        let runtime_path = self.runtime_auth_json_path();
        if !runtime_path.exists() {
            return Ok(RuntimeSyncPlan::NoOp);
        }

        let content = fs::read_to_string(&runtime_path)
            .map_err(|e| CcrError::ConfigError(format!("读取 runtime auth.json 失败: {}", e)))?;
        let runtime_raw: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&content).map_err(|e| {
                CcrError::ConfigError(format!("解析 runtime auth.json 失败: {}", e))
            })?;
        let auth: CodexAuthJson = serde_json::from_value(serde_json::Value::Object(
            runtime_raw.clone(),
        ))
        .map_err(|e| CcrError::ConfigError(format!("解析 runtime auth.json 失败: {}", e)))?;
        let Some(identity) = identity_from_auth(&auth) else {
            debug!("Runtime OAuth identity is incomplete; skip snapshot sync");
            return Ok(RuntimeSyncPlan::NoOp);
        };
        let Some(tokens) = auth.tokens else {
            return Ok(RuntimeSyncPlan::NoOp);
        };

        let registry = self.load_registry()?;
        let Some(name) = self.select_sync_target(&registry, &identity) else {
            return Ok(RuntimeSyncPlan::NoOp);
        };

        let snapshot_path = self.account_auth_path(&name);
        let Ok(snapshot_content) = fs::read_to_string(&snapshot_path) else {
            debug!(
                "Saved snapshot for '{}' is missing; skip snapshot sync",
                name
            );
            return Ok(RuntimeSyncPlan::NoOp);
        };
        let Ok(snapshot_raw) =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&snapshot_content)
        else {
            debug!(
                "Saved snapshot for '{}' is unreadable; skip snapshot sync",
                name
            );
            return Ok(RuntimeSyncPlan::NoOp);
        };
        let Ok(snapshot) = serde_json::from_value::<CodexAuthJson>(serde_json::Value::Object(
            snapshot_raw.clone(),
        )) else {
            return Ok(RuntimeSyncPlan::NoOp);
        };
        let Some(snapshot_tokens) = snapshot.tokens else {
            return Ok(RuntimeSyncPlan::NoOp);
        };

        if OAuthIdentity::from_tokens(&snapshot_tokens).as_ref() != Some(&identity) {
            return Ok(RuntimeSyncPlan::NoOp);
        }

        if Self::tokens_equal(&tokens, &snapshot_tokens) {
            return Ok(RuntimeSyncPlan::Unchanged { account: name });
        }

        let runtime_last_refresh = Self::parse_rfc3339(auth.last_refresh.as_deref());
        let runtime_ts = Self::effective_ts(runtime_last_refresh, Self::mtime(&runtime_path));
        let snapshot_ts = Self::effective_ts(
            Self::parse_rfc3339(snapshot.last_refresh.as_deref()),
            Self::mtime(&snapshot_path),
        );

        if runtime_ts >= snapshot_ts {
            return Ok(RuntimeSyncPlan::WriteSnapshot {
                account: name,
                doc: ResolvedOAuthDoc {
                    tokens,
                    last_refresh: runtime_last_refresh,
                    source: OAuthDocSource::RuntimeAuthJson,
                },
            });
        }

        if registry.current_auth.as_deref() != Some(name.as_str()) {
            return Ok(RuntimeSyncPlan::SkipStaleRuntime { account: name });
        }

        let mut merged = runtime_raw;
        if let Some(snapshot_tokens) = snapshot_raw.get("tokens") {
            merged.insert("tokens".to_string(), snapshot_tokens.clone());
        }
        match snapshot_raw.get("last_refresh") {
            Some(value) => {
                merged.insert("last_refresh".to_string(), value.clone());
            }
            None => {
                merged.remove("last_refresh");
            }
        }
        Ok(RuntimeSyncPlan::WriteRuntime {
            account: name,
            auth: merged,
        })
    }

    /// Prepare only the current quota source using existing freshness rules.
    pub(crate) fn plan_quota_runtime_preparation(
        &self,
        expected_identity: &OAuthIdentity,
    ) -> Result<Option<(serde_json::Map<String, serde_json::Value>, String)>> {
        let path = self.runtime_auth_json_path();
        let content = fs::read(&path)?;
        let mut raw: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&content)
            .map_err(|_| CcrError::ConfigError("runtime auth 无法解析，跳过配额准备".into()))?;
        let auth: CodexAuthJson = serde_json::from_value(serde_json::Value::Object(raw.clone()))
            .map_err(|_| CcrError::ConfigError("runtime auth 身份无法解析，跳过配额准备".into()))?;
        if identity_from_auth(&auth).as_ref() != Some(expected_identity) {
            return Err(CcrError::ConfigError(
                "auth 来源身份已变化，跳过查询".into(),
            ));
        }
        let Some(tokens) = auth.tokens else {
            return Ok(None);
        };
        let Some(latest) = self.resolve_latest_oauth_doc(&expected_identity.key())? else {
            return Ok(None);
        };
        let latest_path = match &latest.source {
            OAuthDocSource::RuntimeAuthJson => path.clone(),
            OAuthDocSource::BackupFile(path) => path.clone(),
        };
        if Self::tokens_equal(&tokens, &latest.tokens)
            || Self::effective_ts(latest.last_refresh, Self::mtime(&latest_path))
                <= Self::effective_ts(
                    Self::parse_rfc3339(auth.last_refresh.as_deref()),
                    Self::mtime(&path),
                )
        {
            return Ok(None);
        }
        let token_values = serde_json::to_value(&latest.tokens)
            .map_err(|_| CcrError::ConfigError("OAuth tokens 无法序列化".into()))?;
        if let Some(target) = raw
            .get_mut("tokens")
            .and_then(serde_json::Value::as_object_mut)
            && let Some(source) = token_values.as_object()
        {
            target.extend(source.clone());
        }
        match latest.last_refresh {
            Some(timestamp) => {
                raw.insert("last_refresh".into(), timestamp.to_rfc3339().into());
            }
            None => {
                raw.remove("last_refresh");
            }
        }
        Ok(Some((raw, content_version_token(&content))))
    }

    /// Reads runtime identity without modifying files.
    pub fn runtime_account_id(&self) -> Option<String> {
        let content = fs::read_to_string(self.runtime_auth_json_path()).ok()?;
        let auth: CodexAuthJson = serde_json::from_str(&content).ok()?;
        let tokens = auth.tokens?;
        tokens
            .account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                tokens
                    .access_token
                    .as_deref()
                    .and_then(super::openai_quota_core::OpenAiQuotaCore::extract_account_id)
            })
    }

    /// Reads the complete runtime identity without modifying files.
    pub(crate) fn runtime_identity(&self) -> Option<OAuthIdentity> {
        let content = fs::read_to_string(self.runtime_auth_json_path()).ok()?;
        let auth: CodexAuthJson = serde_json::from_str(&content).ok()?;
        identity_from_auth(&auth)
    }

    /// Derives identity only from the named account's own snapshot.
    pub(crate) fn saved_identity(
        &self,
        registry: &CodexAuthRegistry,
        name: &str,
    ) -> Option<OAuthIdentity> {
        let account = registry.accounts.get(name)?;
        if account.auth_method == Some(crate::models::OpenAiAuthMethod::Api) {
            return None;
        }
        let content = fs::read_to_string(self.account_auth_path(name)).ok()?;
        let auth: CodexAuthJson = serde_json::from_str(&content).ok()?;
        let identity = identity_from_auth(&auth)?;
        if account
            .identity_key
            .as_deref()
            .is_some_and(|key| key != identity.key())
        {
            warn!("Saved OAuth identity conflicts with its snapshot; skip association");
            return None;
        }
        Some(identity)
    }

    /// Persists safe legacy metadata at execution observation points only.
    pub(crate) fn backfill_identity_keys(&self) -> Result<()> {
        let mut registry = self.load_registry()?;
        if registry.is_read_only() {
            warn!("Skipped OAuth identity backfill for read-only auth registry");
            return Ok(());
        }
        let updates: Vec<_> = registry
            .accounts
            .iter()
            .filter(|(_, account)| account.identity_key.is_none())
            .filter_map(|(name, _)| {
                self.saved_identity(&registry, name)
                    .map(|identity| (name.clone(), identity.key()))
            })
            .collect();
        if updates.is_empty() {
            return Ok(());
        }
        for (name, key) in updates {
            if let Some(account) = registry.accounts.get_mut(&name) {
                account.identity_key = Some(key);
            }
        }
        self.save_registry(&registry)
    }

    /// Selects an alias by current_auth, last_used, then insertion order.
    pub(crate) fn select_sync_target(
        &self,
        registry: &CodexAuthRegistry,
        identity: &OAuthIdentity,
    ) -> Option<String> {
        let mut matches = registry
            .accounts
            .iter()
            .filter(|(name, _)| self.saved_identity(registry, name).as_ref() == Some(identity));
        if let Some(current) = registry.current_auth.as_deref()
            && self.saved_identity(registry, current).as_ref() == Some(identity)
        {
            return Some(current.to_string());
        }
        let first = matches.next()?;
        let best = matches.fold(first, |best, candidate| {
            if candidate.1.last_used > best.1.last_used {
                candidate
            } else {
                best
            }
        });
        Some(best.0.clone())
    }

    fn tokens_equal(left: &CodexAuthTokens, right: &CodexAuthTokens) -> bool {
        fn norm(value: &Option<String>) -> Option<&str> {
            value.as_deref().map(str::trim).filter(|s| !s.is_empty())
        }
        norm(&left.refresh_token) == norm(&right.refresh_token)
            && norm(&left.access_token) == norm(&right.access_token)
            && norm(&left.id_token) == norm(&right.id_token)
            && norm(&left.account_id) == norm(&right.account_id)
    }

    fn mtime(path: &Path) -> Option<SystemTime> {
        fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    /// 修复指定账号快照中的 OAuth tokens（从 runtime/backups 中找最新副本）
    pub fn repair_saved_account(&self, name: &str) -> Result<OAuthRepairOutcome> {
        let path = self.account_auth_path(name);
        let locks = CredentialLocks::acquire_sources(vec![(
            path.clone(),
            CredentialResource::from_path(&path),
        )])?;
        locks.verify_path(&path)?;
        self.repair_saved_account_locked(name)
    }

    pub(crate) fn repair_saved_account_locked(&self, name: &str) -> Result<OAuthRepairOutcome> {
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

        let Some(identity) = OAuthIdentity::from_tokens(&tokens) else {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "账号缺少完整 OAuth 身份，无法修复".to_string(),
            });
        };
        let registry = self.load_registry()?;
        if registry
            .accounts
            .get(name)
            .and_then(|account| account.identity_key.as_deref())
            .is_some_and(|key| key != identity.key())
        {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "OAuth 快照身份与注册表不匹配，跳过修复".to_string(),
            });
        }
        self.backfill_identity_keys()?;

        let current_last_refresh = Self::parse_rfc3339(auth.last_refresh.as_deref());
        let current_mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        let current_ts = Self::effective_ts(current_last_refresh, current_mtime);

        let Some(latest) = self.resolve_latest_oauth_doc(&identity.key())? else {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: None,
                message: "未在 runtime/backups 中找到可用的 OAuth tokens".to_string(),
            });
        };

        if Self::tokens_equal(&tokens, &latest.tokens) {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: Some(latest.source),
                message: "tokens 相同，无需修复".into(),
            });
        }

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

        // 新鲜度决定胜者：来源旧于快照时不回写，避免用已消费的旧 refresh_token 覆盖较新快照
        let should_update = (refresh_changed && latest_ts >= current_ts) || latest_ts > current_ts;
        if !should_update {
            return Ok(OAuthRepairOutcome {
                updated: false,
                source: Some(latest.source),
                message: "已是最新 tokens，无需修复".to_string(),
            });
        }

        self.sync_account_auth_file_locked(name, &latest)?;
        self.update_registry_metadata(name, &latest)?;

        let source_label = latest.source.label();
        let source = latest.source;
        Ok(OAuthRepairOutcome {
            updated: true,
            source: Some(source),
            message: format!("已从 {} 修复 OAuth tokens", source_label),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::core::atomic_writer::AtomicWriter;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn p4_sync_plan_debug_hides_raw_auth_and_token_documents() {
        let auth = serde_json::from_str(r#"{"tokens":{"refresh_token":"p4-private-plan"},"provider_key":"p4-private-provider"}"#).unwrap();
        let plan = RuntimeSyncPlan::WriteRuntime {
            account: "safe-alias".into(),
            auth,
        };
        assert!(!format!("{plan:?}").contains("p4-private-"));
        let doc = ResolvedOAuthDoc {
            tokens: CodexAuthTokens {
                id_token: None,
                access_token: Some("p4-private-access".into()),
                refresh_token: Some("p4-private-refresh".into()),
                account_id: None,
            },
            last_refresh: None,
            source: OAuthDocSource::RuntimeAuthJson,
        };
        assert!(!format!("{doc:?}").contains("p4-private-"));
    }

    fn write_json(path: &Path, value: &serde_json::Value) {
        let content = serde_json::to_string_pretty(value).unwrap();
        AtomicWriter::new(path)
            .secret(true)
            .write_string(&content)
            .unwrap();
        crate::utils::ensure_private_permissions(path).unwrap();
    }

    fn setup_dirs() -> (crate::test_support::TestCodexEnv, PathBuf, TempDir, PathBuf) {
        let ccr_root = crate::test_support::TestCodexEnv::new();
        let codex_root = tempfile::tempdir().unwrap();
        let ccr_codex_dir = ccr_root.ccr_codex_dir().to_path_buf();
        let codex_dir = codex_root.path().to_path_buf();
        fs::create_dir_all(ccr_codex_dir.join("auth")).unwrap();
        fs::create_dir_all(codex_dir.join("backups")).unwrap();
        (ccr_root, ccr_codex_dir, codex_root, codex_dir)
    }

    #[test]
    fn p4_unchanged_hardening_rejects_stale_observed_runtime_or_snapshot_bytes() {
        let (_env, ccr, _dir, codex) = setup_dirs();
        let service = CodexOAuthTokenService::from_dirs(ccr, codex);
        let doc = |refresh: &str| json!({"tokens": {"id_token":super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"p4-user"})), "access_token":"synthetic-access", "refresh_token":refresh, "account_id":"p4-workspace"}});
        write_json(&service.runtime_auth_json_path(), &doc("old-refresh"));
        write_json(&service.account_auth_path("p4"), &doc("old-refresh"));
        let mut registry = CodexAuthRegistry::default();
        registry.accounts.insert(
            "p4".into(),
            serde_json::from_value(
                json!({"account_id":"p4-workspace", "saved_at":"2026-10-01T00:00:00Z"}),
            )
            .unwrap(),
        );
        service.save_registry(&registry).unwrap();
        let runtime = fs::read(service.runtime_auth_json_path()).unwrap();
        let snapshot = fs::read(service.account_auth_path("p4")).unwrap();
        write_json(&service.runtime_auth_json_path(), &doc("new-refresh"));
        assert!(
            !service
                .harden_unchanged_pair("p4", &runtime, &snapshot)
                .unwrap()
        );
        assert_eq!(fs::read(service.account_auth_path("p4")).unwrap(), snapshot);
        fs::write(service.runtime_auth_json_path(), &runtime).unwrap();
        write_json(&service.account_auth_path("p4"), &doc("new-refresh"));
        assert!(
            !service
                .harden_unchanged_pair("p4", &runtime, &snapshot)
                .unwrap()
        );
        assert_eq!(fs::read(service.runtime_auth_json_path()).unwrap(), runtime);
    }

    #[test]
    fn snapshot_write_rejects_a_stale_plan_for_the_same_identity() {
        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexOAuthTokenService::from_dirs(
            env.ccr_codex_dir().to_path_buf(),
            env.codex_dir().to_path_buf(),
        );
        fs::create_dir_all(service.auth_storage_dir()).unwrap();
        let doc = |refresh: &str, date: &str| json!({"tokens":{"id_token":super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"stale-user"})),"access_token":"synthetic-access","refresh_token":refresh,"account_id":"stale-workspace"},"last_refresh":date});
        write_json(
            &service.account_auth_path("a"),
            &doc("rt-a1", "2026-10-01T00:00:00Z"),
        );
        write_json(
            &service.runtime_auth_json_path(),
            &doc("rt-a2", "2026-10-02T00:00:00Z"),
        );
        let mut registry = CodexAuthRegistry::default();
        registry.accounts.insert(
            "a".into(),
            serde_json::from_value(
                json!({"account_id":"stale-workspace","saved_at":"2026-10-01T00:00:00Z"}),
            )
            .unwrap(),
        );
        service.save_registry(&registry).unwrap();
        let RuntimeSyncPlan::WriteSnapshot { account, doc } = service.plan_runtime_sync().unwrap()
        else {
            panic!("expected a runtime-to-snapshot plan");
        };
        write_json(
            &service.account_auth_path("a"),
            &json!({"tokens":{"id_token":super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"stale-user"})),"access_token":"synthetic-access","refresh_token":"rt-a3","account_id":"stale-workspace"},"last_refresh":"2026-10-03T00:00:00Z"}),
        );
        let before = fs::read(service.account_auth_path("a")).unwrap();
        assert!(service.apply_snapshot_write(&account, &doc).is_err());
        assert_eq!(fs::read(service.account_auth_path("a")).unwrap(), before);
    }

    #[test]
    fn update_registry_metadata_skips_read_only_versions() {
        for version in ["2.0", "abc"] {
            let env = crate::test_support::TestCodexEnv::new();
            let service = CodexOAuthTokenService::from_dirs(
                env.ccr_codex_dir().to_path_buf(),
                env.codex_dir().to_path_buf(),
            );
            let path = env.ccr_codex_dir().join("auth_registry.toml");
            let content = format!(
                "version = {version:?}\n[accounts.team]\naccount_id = \"acc-1\"\nsaved_at = \"2026-10-05T00:00:00Z\"\n"
            );
            fs::write(&path, &content).unwrap();
            let doc = ResolvedOAuthDoc {
                tokens: CodexAuthTokens {
                    id_token: None,
                    access_token: Some("synthetic-access".into()),
                    refresh_token: Some("synthetic-refresh".into()),
                    account_id: Some("acc-1".into()),
                },
                last_refresh: Some(Utc::now()),
                source: OAuthDocSource::RuntimeAuthJson,
            };
            service.update_registry_metadata("team", &doc).unwrap();
            assert_eq!(fs::read(&path).unwrap(), content.as_bytes());
            assert!(
                service.load_registry().unwrap().accounts["team"]
                    .last_refresh
                    .is_none()
            );
            assert!(!env.ccr_codex_dir().join("auth/backups").exists());
        }
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
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
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
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_new",
                    "account_id": acc_id
                },
                "last_refresh": "2026-03-26T00:00:00Z"
            }),
        );

        let best = service
            .resolve_latest_oauth_doc(&format!("test-user::{acc_id}"))
            .unwrap()
            .unwrap();
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
                identity_key: None,
                auth_method: Some(crate::models::OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
                extra: toml::Table::new(),
            },
        );
        service.save_registry(&registry).unwrap();

        // saved snapshot contains old token
        let saved = service.account_auth_path("team");
        write_json(
            &saved,
            &json!({
                "tokens": {
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
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
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
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

    #[test]
    fn test_repair_does_not_overwrite_newer_snapshot_with_older_source() {
        let (_ccr_root, ccr_codex_dir, _codex_root, codex_dir) = setup_dirs();
        let service = CodexOAuthTokenService::from_dirs(ccr_codex_dir, codex_dir);

        // 快照持有 CCR 自身轮换得到的较新 token
        let saved = service.account_auth_path("team");
        write_json(
            &saved,
            &json!({
                "tokens": {
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_newer",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-26T00:00:00Z"
            }),
        );
        // 备份中只有较旧（已被消费）的 token
        let backup = service
            .codex_backups_dir()
            .join("auth.runtime_switch.20260301_000000.json.bak");
        write_json(
            &backup,
            &json!({
                "tokens": {
                    "id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":"test-user"})),
                    "access_token": "header.payload.sig",
                    "refresh_token": "rt_consumed",
                    "account_id": "acc-1"
                },
                "last_refresh": "2026-03-01T00:00:00Z"
            }),
        );
        let before = fs::read(&saved).unwrap();

        let outcome = service.repair_saved_account("team").unwrap();

        assert!(!outcome.updated);
        assert_eq!(fs::read(&saved).unwrap(), before);
    }

    #[test]
    fn repair_ignores_newer_runtime_and_backups_from_another_workspace_user() {
        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexOAuthTokenService::from_dirs(
            env.ccr_codex_dir().to_path_buf(),
            env.codex_dir().to_path_buf(),
        );
        fs::create_dir_all(service.auth_storage_dir()).unwrap();
        fs::create_dir_all(service.codex_backups_dir()).unwrap();
        let doc = |user: &str, refresh: &str, date: &str| {
            json!({
                "tokens": {"id_token": super::super::codex_auth_identity::test_jwt(json!({"chatgpt_user_id":user})),"access_token":"synthetic-access","refresh_token":refresh,"account_id":"workspace"},
                "last_refresh":date
            })
        };
        let saved = service.account_auth_path("a");
        write_json(&saved, &doc("user-a", "rt-a1", "2026-10-01T00:00:00Z"));
        let mut registry = CodexAuthRegistry::default();
        registry.accounts.insert(
            "a".into(),
            serde_json::from_value(
                json!({"account_id":"workspace","saved_at":"2026-10-01T00:00:00Z"}),
            )
            .unwrap(),
        );
        registry.current_auth = Some("a".into());
        service.save_registry(&registry).unwrap();
        write_json(
            &service.runtime_auth_json_path(),
            &doc("user-b", "rt-b9", "2026-10-09T00:00:00Z"),
        );
        let wrong_backup = service
            .codex_backups_dir()
            .join("auth.runtime_switch.other.json.bak");
        write_json(
            &wrong_backup,
            &doc("user-b", "rt-b10", "2026-10-10T00:00:00Z"),
        );
        let before = fs::read(&saved).unwrap();
        assert!(!service.repair_saved_account("a").unwrap().updated);
        assert_eq!(fs::read(&saved).unwrap(), before);
        let correct_backup = service
            .codex_backups_dir()
            .join("auth.runtime_switch.correct.json.bak");
        write_json(
            &correct_backup,
            &doc("user-a", "rt-a2", "2026-10-02T00:00:00Z"),
        );
        assert!(service.repair_saved_account("a").unwrap().updated);
        let updated: CodexAuthJson =
            serde_json::from_str(&fs::read_to_string(&saved).unwrap()).unwrap();
        assert_eq!(
            updated.tokens.unwrap().refresh_token.as_deref(),
            Some("rt-a2")
        );
        assert_eq!(
            service.load_registry().unwrap().accounts["a"]
                .identity_key
                .as_deref(),
            Some("user-a::workspace")
        );

        let wrong: CodexAuthJson =
            serde_json::from_value(doc("user-b", "rt-b11", "2026-10-11T00:00:00Z")).unwrap();
        let wrong = ResolvedOAuthDoc {
            tokens: wrong.tokens.unwrap(),
            last_refresh: Some(Utc::now()),
            source: OAuthDocSource::RuntimeAuthJson,
        };
        let before = fs::read(&saved).unwrap();
        let error = service
            .sync_account_auth_file("a", &wrong)
            .unwrap_err()
            .to_string();
        assert!(
            !error.contains("user-a") && !error.contains("user-b") && !error.contains("rt-b11")
        );
        assert_eq!(fs::read(&saved).unwrap(), before);

        for id_token in [None, Some("invalid-jwt"), Some("header.e30.signature")] {
            let mut unknown = doc("user-a", "rt-a0", "2026-10-01T00:00:00Z");
            unknown["tokens"]["id_token"] = id_token
                .map(|value| value.into())
                .unwrap_or(serde_json::Value::Null);
            write_json(&saved, &unknown);
            let before = fs::read(&saved).unwrap();
            assert!(!service.repair_saved_account("a").unwrap().updated);
            assert_eq!(fs::read(&saved).unwrap(), before);
        }
    }
}
