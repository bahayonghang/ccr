//! Codex 历史同步服务
//!
//! 修复 Codex 在不同 provider namespace 下的历史会话可见性问题。

use crate::managers::CodexConfigManager;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use chrono::{DateTime, Utc};
use filetime::FileTime;
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, params_from_iter};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
#[cfg(windows)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

const DEFAULT_PROVIDER: &str = "openai";
const THIRD_PARTY_PROVIDER: &str = "custom";
const BRIDGE_OFFICIAL_CUSTOM: &str = "official-custom";
const DEFAULT_MAX_AGE_DAYS: u64 = 7;
const BACKUP_NAMESPACE: &str = "codex_sync_history";
const BACKUP_DIR_NAME: &str = "sync-history";
const GLOBAL_STATE_FILE_NAME: &str = ".codex-global-state.json";
const SQLITE_FILE_NAME: &str = "state_5.sqlite";
const SESSION_INDEX_FILE: &str = "session_index.jsonl";
const SESSION_MANIFEST_FILE_NAME: &str = "rollout-manifest.json";
const METADATA_FILE_NAME: &str = "metadata.json";
const DEFAULT_KEEP_COUNT: usize = 5;
const LOCK_RESOURCE: &str = "codex_sync_history";
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CodexHistoryProviderBuckets {
    pub sessions: BTreeMap<String, usize>,
    pub archived_sessions: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistoryBackupSummary {
    pub count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistoryBackupPruneResult {
    pub backup_root: PathBuf,
    pub deleted_count: usize,
    pub remaining_count: usize,
    pub freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistorySyncStatus {
    pub codex_home: PathBuf,
    pub current_provider: String,
    pub current_provider_implicit: bool,
    pub session_index_present: bool,
    pub missing_session_index_entries: usize,
    pub rollout_counts: CodexHistoryProviderBuckets,
    pub recent_rollout_counts: CodexHistoryProviderBuckets,
    pub encrypted_counts: CodexHistoryProviderBuckets,
    pub sqlite_counts: Option<CodexHistoryProviderBuckets>,
    pub visibility: Option<CodexHistoryVisibilityDiagnostics>,
    pub backup_root: PathBuf,
    pub backup_summary: CodexHistoryBackupSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistoryVisibilityDiagnostics {
    pub current_visibility_provider: String,
    pub rollout_threads: usize,
    pub sqlite_rows_for_rollouts: usize,
    pub sqlite_missing_rows: usize,
    pub provider_mismatch_rows: usize,
    pub preview_column_present: bool,
    pub preview_empty_rows: usize,
    pub has_user_event_column_present: bool,
    pub has_user_event_missing_rows: usize,
    pub cwd_mismatch_rows: usize,
    pub exact_cwd_matches: usize,
    pub normalized_cwd_matches: usize,
    pub verbatim_extended_cwd_rows: usize,
    pub workspace_root_candidates: usize,
    pub workspace_root_matches: usize,
    pub desktop_first_page_limit: usize,
    pub desktop_first_page_size: usize,
    pub desktop_first_page_matching_rows: usize,
    pub desktop_first_page_limited: bool,
    pub rank_min: Option<usize>,
    pub rank_max: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistorySyncResult {
    pub codex_home: PathBuf,
    pub target_provider: String,
    pub previous_provider: String,
    pub bridge: Option<String>,
    pub bridge_all_history_recommended: bool,
    pub backup_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub all_history: bool,
    pub max_age_days: u64,
    pub full_rollout_counts: CodexHistoryProviderBuckets,
    pub rollout_counts: CodexHistoryProviderBuckets,
    pub encrypted_counts: CodexHistoryProviderBuckets,
    pub sqlite_counts: Option<CodexHistoryProviderBuckets>,
    pub changed_rollout_files: usize,
    pub added_sidebar_projects: usize,
    pub session_index_present: bool,
    pub missing_session_index_entries: usize,
    pub added_session_index_entries: usize,
    pub skipped_locked_rollout_files: Vec<PathBuf>,
    pub sqlite_rows_updated: usize,
    pub sqlite_present: bool,
    pub backup_cleanup: Option<CodexHistoryBackupPruneResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistoryRestoreResult {
    pub codex_home: PathBuf,
    pub backup_dir: PathBuf,
    pub target_provider: String,
    pub restored_session_index: bool,
    pub restored_state: bool,
}

#[derive(Debug, Clone)]
pub struct CodexHistorySyncOptions {
    pub provider: Option<String>,
    pub bridge: Option<String>,
    pub keep_count: usize,
    pub max_age_days: u64,
    pub all_history: bool,
    pub include_providers: Vec<String>,
    pub sqlite_busy_timeout: Duration,
    pub dry_run: bool,
}

impl Default for CodexHistorySyncOptions {
    fn default() -> Self {
        Self {
            provider: None,
            bridge: None,
            keep_count: DEFAULT_KEEP_COUNT,
            max_age_days: DEFAULT_MAX_AGE_DAYS,
            all_history: false,
            include_providers: Vec::new(),
            sqlite_busy_timeout: SQLITE_BUSY_TIMEOUT,
            dry_run: false,
        }
    }
}

pub struct CodexHistorySyncService {
    codex_home: PathBuf,
    config_manager: CodexConfigManager,
    lock_manager: LockManager,
}

#[derive(Debug, Clone)]
struct CurrentProvider {
    provider: String,
    implicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionScope {
    Sessions,
    ArchivedSessions,
}

impl SessionScope {
    fn as_key(self) -> &'static str {
        match self {
            SessionScope::Sessions => "sessions",
            SessionScope::ArchivedSessions => "archived_sessions",
        }
    }
}

#[derive(Debug, Clone)]
struct FirstLineRecord {
    first_line: String,
    separator: String,
    offset: u64,
}

#[derive(Debug, Clone)]
struct SessionChange {
    path: PathBuf,
    original_first_line: String,
    original_separator: String,
    original_offset: u64,
    original_size: u64,
    original_modified_ms: u128,
    original_modified_time: FileTime,
    updated_first_line: String,
}

#[derive(Debug, Clone)]
struct SessionScan {
    changes: Vec<SessionChange>,
    provider_counts: CodexHistoryProviderBuckets,
    encrypted_counts: CodexHistoryProviderBuckets,
    thread_candidates: Vec<RolloutThreadCandidate>,
}

#[derive(Debug, Clone)]
struct SessionApplyResult {
    applied_changes: Vec<AppliedSessionBackupEntry>,
    skipped_locked_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct SqliteUpdateResult {
    updated_rows: usize,
    database_present: bool,
}

#[derive(Debug, Clone)]
struct RolloutThreadCandidate {
    id: String,
    path: PathBuf,
    archived: bool,
    updated_at: i64,
    cwd: String,
    provider_key: String,
}

#[derive(Debug, Clone)]
struct RolloutThreadInsert {
    id: String,
    rollout_path: String,
    created_at: i64,
    updated_at: i64,
    source: String,
    model_provider: String,
    cwd: String,
    title: String,
    sandbox_policy: String,
    approval_mode: String,
    tokens_used: i64,
    has_user_event: bool,
    archived: bool,
    archived_at: Option<i64>,
    cli_version: String,
    first_user_message: String,
    preview: String,
    agent_nickname: Option<String>,
    agent_role: Option<String>,
    memory_mode: String,
    model: Option<String>,
    reasoning_effort: Option<String>,
    agent_path: Option<String>,
}

#[derive(Debug, Clone)]
struct ThreadDbRow {
    id: String,
    model_provider: String,
    archived: bool,
    cwd: String,
    title: String,
    preview: String,
    first_user_message: String,
    has_user_event: bool,
    updated_at: i64,
}

#[derive(Debug, Clone)]
struct SqliteRepairPolicy {
    broad_provider_repair: bool,
    allowed_provider_keys: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct SqliteRepairPlan {
    provider_update_ids: BTreeSet<String>,
    preview_updates: BTreeMap<String, String>,
    has_user_event_update_ids: BTreeSet<String>,
    cwd_updates: BTreeMap<String, String>,
    missing_inserts: Vec<RolloutThreadInsert>,
    affected_ids: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct SidebarSyncPlan {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
    next_text: Option<String>,
    added_projects: Vec<String>,
}

#[derive(Debug, Clone)]
struct SessionIndexPlan {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
    entries_to_add: Vec<JsonValue>,
}

#[derive(Debug, Clone)]
struct SessionIndexSnapshot {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
}

#[derive(Debug, Clone)]
struct SessionIndexUpdateResult {
    snapshot: Option<SessionIndexSnapshot>,
    added_entries: usize,
}

#[derive(Debug, Clone)]
struct SessionIndexState {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
    entries_by_id: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupMetadata {
    version: u32,
    namespace: String,
    codex_home: String,
    target_provider: String,
    created_at: DateTime<Utc>,
    state_db_present: bool,
    global_state_present: bool,
    #[serde(default)]
    session_index_present: Option<bool>,
    #[serde(default)]
    session_index_backed_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppliedSessionBackupEntry {
    path: PathBuf,
    original_first_line: String,
    original_separator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionBackupManifest {
    entries: Vec<AppliedSessionBackupEntry>,
}

#[derive(Debug, Clone)]
struct ManagedBackupDir {
    path: PathBuf,
    sort_key: String,
    total_bytes: u64,
}

#[derive(Debug, Clone)]
struct GlobalStateSnapshot {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
}

impl CodexHistorySyncService {
    pub fn new() -> Result<Self> {
        Self::with_codex_home::<PathBuf>(None)
    }

    pub fn with_codex_home<P>(codex_home: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        if let Some(explicit) = codex_home {
            let codex_home = explicit.as_ref().to_path_buf();
            let lock_dir = codex_home.join(".locks");
            let config_manager = CodexConfigManager::new(
                codex_home.join("config.toml"),
                codex_home.join("auth.json"),
                codex_home.join("backups"),
                LockManager::new(&lock_dir),
            );
            return Ok(Self {
                codex_home,
                config_manager,
                lock_manager: LockManager::new(lock_dir),
            });
        }

        let config_manager = CodexConfigManager::with_default()?;
        let codex_home = config_manager
            .config_path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CcrError::ConfigError("无法解析 Codex 目录".into()))?;
        let lock_manager = LockManager::new(codex_home.join(".locks"));
        Ok(Self {
            codex_home,
            config_manager,
            lock_manager,
        })
    }

    pub fn status(&self) -> Result<CodexHistorySyncStatus> {
        let current = self.current_provider()?;
        let scan = self.scan_session_changes("__status_only__", None, None)?;
        let rollout_counts = scan.provider_counts.clone();
        let recent_cutoff = Utc::now() - chrono::Duration::days(DEFAULT_MAX_AGE_DAYS as i64);
        let recent_rollout_counts = self
            .scan_session_changes("__status_only__", Some(recent_cutoff), None)?
            .provider_counts;
        let sqlite_rows = self.read_sqlite_thread_rows()?;
        let sqlite_counts = self.read_sqlite_provider_counts()?;
        let visibility = self.read_visibility_diagnostics(
            &canonical_runtime_provider(&current),
            &scan.thread_candidates,
            self.load_workspace_root_keys_for_diagnostics(),
        )?;
        let session_index_plan = self.prepare_session_index_repair(
            &scan.thread_candidates,
            sqlite_rows.as_deref(),
            None,
        )?;
        let backup_summary = self.backup_summary()?;

        Ok(CodexHistorySyncStatus {
            codex_home: self.codex_home.clone(),
            current_provider: current.provider,
            current_provider_implicit: current.implicit,
            session_index_present: session_index_plan.existed,
            missing_session_index_entries: session_index_plan.entries_to_add.len(),
            rollout_counts,
            recent_rollout_counts,
            encrypted_counts: scan.encrypted_counts,
            sqlite_counts,
            visibility,
            backup_root: self.backup_root(),
            backup_summary,
        })
    }

    pub fn sync(&self, options: CodexHistorySyncOptions) -> Result<CodexHistorySyncResult> {
        if options.keep_count == 0 {
            return Err(CcrError::ValidationError(
                "--keep 必须大于或等于 1".to_string(),
            ));
        }
        if options.max_age_days == 0 {
            return Err(CcrError::ValidationError(
                "--max-age-days 必须大于或等于 1".to_string(),
            ));
        }

        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        if options.provider.is_some() && options.bridge.is_some() {
            return Err(CcrError::ValidationError(
                "--provider and --bridge cannot be used together".to_string(),
            ));
        }

        let current = self.current_provider()?;
        let mut bridge = normalize_bridge_option(options.bridge.as_deref())?;
        let explicit_provider = options
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        // 官方 auth 登录会有意清空 config.toml 的根级 model_provider（隐式 openai）。
        // 此时若既未显式 --provider 也未显式 --bridge，则默认走 official-custom bridge，
        // 把 openai/custom/缺失 provider 的历史桥接到当前 runtime provider，
        // 使 url/key 登录产生的历史在当前 auth 会话中可见。
        if explicit_provider.is_none() && bridge.is_none() && current.implicit {
            bridge = Some(BRIDGE_OFFICIAL_CUSTOM.to_string());
        }

        let target_provider = if bridge.is_some() {
            canonical_runtime_provider(&current)
        } else {
            explicit_provider
                .map(str::to_string)
                .unwrap_or_else(|| current.provider.clone())
        };
        let cutoff = if options.all_history {
            None
        } else {
            Some(Utc::now() - chrono::Duration::days(options.max_age_days as i64))
        };
        let include_provider_keys = normalize_include_providers(&options.include_providers);
        let rewrite_provider_keys = (bridge.is_some() || options.all_history)
            .then(|| bridge_provider_keys(&include_provider_keys));
        let sqlite_policy = SqliteRepairPolicy {
            broad_provider_repair: options.all_history || bridge.is_some(),
            allowed_provider_keys: bridge_provider_keys(&include_provider_keys),
        };

        let full_scan = self.scan_session_changes("__status_only__", None, None)?;
        let scan =
            self.scan_session_changes(&target_provider, cutoff, rewrite_provider_keys.as_ref())?;
        let sqlite_rows = self.read_sqlite_thread_rows()?;
        let sidebar_plan = self.prepare_sidebar_sync(
            self.read_sqlite_project_paths(options.sqlite_busy_timeout, &scan.thread_candidates)?,
        )?;
        let session_index_plan = self.prepare_session_index_repair(
            &scan.thread_candidates,
            sqlite_rows.as_deref(),
            Some(&sqlite_policy),
        )?;
        let sqlite_preview = self.preview_sqlite_provider_update(
            &target_provider,
            options.sqlite_busy_timeout,
            &scan.thread_candidates,
            &sqlite_policy,
        )?;
        let sqlite_counts = self.read_sqlite_provider_counts()?;
        if options.dry_run {
            return Ok(CodexHistorySyncResult {
                codex_home: self.codex_home.clone(),
                target_provider,
                previous_provider: current.provider,
                bridge,
                bridge_all_history_recommended: rewrite_provider_keys.is_some()
                    && !options.all_history,
                backup_dir: None,
                dry_run: true,
                all_history: options.all_history,
                max_age_days: options.max_age_days,
                full_rollout_counts: full_scan.provider_counts,
                rollout_counts: scan.provider_counts,
                encrypted_counts: full_scan.encrypted_counts,
                sqlite_counts,
                changed_rollout_files: scan.changes.len(),
                added_sidebar_projects: sidebar_plan.added_projects.len(),
                session_index_present: session_index_plan.existed,
                missing_session_index_entries: session_index_plan.entries_to_add.len(),
                added_session_index_entries: 0,
                skipped_locked_rollout_files: Vec::new(),
                sqlite_rows_updated: sqlite_preview.updated_rows,
                sqlite_present: sqlite_preview.database_present,
                backup_cleanup: None,
            });
        }

        self.assert_sqlite_writable(options.sqlite_busy_timeout)?;

        let backup_dir =
            self.create_backup(&target_provider, &sidebar_plan, &session_index_plan)?;
        let mut applied_rollout_entries = Vec::new();
        let mut global_state_snapshot = None;
        let mut session_index_snapshot = None;

        let sync_result = (|| -> Result<(SessionApplyResult, SessionIndexUpdateResult, SqliteUpdateResult)> {
            let apply_result = self.apply_session_changes(&scan.changes)?;
            applied_rollout_entries = apply_result.applied_changes.clone();
            self.write_session_manifest(&backup_dir, &applied_rollout_entries)?;

            if let Some(snapshot) = self.apply_sidebar_sync(&sidebar_plan)? {
                global_state_snapshot = Some(snapshot);
            }

            let session_index_result = self.apply_session_index_repair(&session_index_plan)?;
            if let Some(snapshot) = session_index_result.snapshot.clone() {
                session_index_snapshot = Some(snapshot);
            }

            let sqlite_result = self.update_sqlite_provider(
                &target_provider,
                options.sqlite_busy_timeout,
                &scan.thread_candidates,
                &sqlite_policy,
            )?;
            Ok((apply_result, session_index_result, sqlite_result))
        })();

        match sync_result {
            Ok((apply_result, session_index_result, sqlite_result)) => {
                let backup_cleanup = Some(self.prune_backups_inner(options.keep_count)?);
                Ok(CodexHistorySyncResult {
                    codex_home: self.codex_home.clone(),
                    target_provider,
                    previous_provider: current.provider,
                    bridge,
                    bridge_all_history_recommended: rewrite_provider_keys.is_some()
                        && !options.all_history,
                    backup_dir: Some(backup_dir),
                    dry_run: false,
                    all_history: options.all_history,
                    max_age_days: options.max_age_days,
                    full_rollout_counts: full_scan.provider_counts,
                    rollout_counts: scan.provider_counts,
                    encrypted_counts: full_scan.encrypted_counts,
                    sqlite_counts,
                    changed_rollout_files: apply_result.applied_changes.len(),
                    added_sidebar_projects: sidebar_plan.added_projects.len(),
                    session_index_present: session_index_plan.existed,
                    missing_session_index_entries: session_index_plan.entries_to_add.len(),
                    added_session_index_entries: session_index_result.added_entries,
                    skipped_locked_rollout_files: apply_result.skipped_locked_paths,
                    sqlite_rows_updated: sqlite_result.updated_rows,
                    sqlite_present: sqlite_result.database_present,
                    backup_cleanup,
                })
            }
            Err(err) => {
                if !applied_rollout_entries.is_empty() {
                    self.restore_session_entries(&applied_rollout_entries)?;
                }
                if let Some(snapshot) = &global_state_snapshot {
                    self.restore_global_state_snapshot(snapshot)?;
                }
                if let Some(snapshot) = &session_index_snapshot {
                    self.restore_session_index_snapshot(snapshot)?;
                }
                Err(err)
            }
        }
    }

    pub fn restore<P: AsRef<Path>>(&self, backup_dir: P) -> Result<CodexHistoryRestoreResult> {
        self.restore_inner(backup_dir, false)
    }

    pub fn restore_with_state<P: AsRef<Path>>(
        &self,
        backup_dir: P,
    ) -> Result<CodexHistoryRestoreResult> {
        self.restore_inner(backup_dir, true)
    }

    fn restore_inner<P: AsRef<Path>>(
        &self,
        backup_dir: P,
        restore_state: bool,
    ) -> Result<CodexHistoryRestoreResult> {
        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        let backup_dir = backup_dir.as_ref().to_path_buf();
        let metadata = self.read_backup_metadata(&backup_dir)?;
        let session_entries = self.read_session_manifest(&backup_dir)?;
        self.restore_session_entries(&session_entries)?;
        let restored_session_index =
            self.restore_session_index_from_backup(&backup_dir, &metadata)?;
        if restore_state {
            self.restore_global_state_from_backup(&backup_dir, &metadata)?;
            self.restore_sqlite_from_backup(&backup_dir, &metadata)?;
        }

        Ok(CodexHistoryRestoreResult {
            codex_home: self.codex_home.clone(),
            backup_dir,
            target_provider: metadata.target_provider,
            restored_session_index,
            restored_state: restore_state,
        })
    }

    pub fn prune_backups(&self, keep_count: usize) -> Result<CodexHistoryBackupPruneResult> {
        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;
        self.prune_backups_inner(keep_count)
    }

    fn prune_backups_inner(&self, keep_count: usize) -> Result<CodexHistoryBackupPruneResult> {
        let backup_root = self.backup_root();
        if !backup_root.exists() {
            return Ok(CodexHistoryBackupPruneResult {
                backup_root,
                deleted_count: 0,
                remaining_count: 0,
                freed_bytes: 0,
            });
        }

        let mut backups = self.list_managed_backups()?;
        backups.sort_by(|left, right| right.sort_key.cmp(&left.sort_key));

        let mut deleted_count = 0usize;
        let mut freed_bytes = 0u64;
        for backup in backups.iter().skip(keep_count) {
            fs::remove_dir_all(&backup.path)?;
            deleted_count += 1;
            freed_bytes += backup.total_bytes;
        }

        Ok(CodexHistoryBackupPruneResult {
            backup_root,
            deleted_count,
            remaining_count: backups.len().saturating_sub(deleted_count),
            freed_bytes,
        })
    }

    fn backup_root(&self) -> PathBuf {
        self.codex_home.join("backups_state").join(BACKUP_DIR_NAME)
    }

    fn backup_summary(&self) -> Result<CodexHistoryBackupSummary> {
        let backups = self.list_managed_backups()?;
        Ok(CodexHistoryBackupSummary {
            count: backups.len(),
            total_bytes: backups.iter().map(|entry| entry.total_bytes).sum(),
        })
    }

    fn current_provider(&self) -> Result<CurrentProvider> {
        let config = self.config_manager.load_config()?;
        let provider = config
            .as_table()
            .and_then(|root| root.get("model_provider"))
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        Ok(match provider {
            Some(provider) => CurrentProvider {
                provider,
                implicit: false,
            },
            None => CurrentProvider {
                provider: DEFAULT_PROVIDER.to_string(),
                implicit: true,
            },
        })
    }

    fn scan_session_changes(
        &self,
        target_provider: &str,
        cutoff: Option<DateTime<Utc>>,
        rewrite_provider_keys: Option<&BTreeSet<String>>,
    ) -> Result<SessionScan> {
        let mut changes = Vec::new();
        let mut provider_counts = CodexHistoryProviderBuckets::default();
        let mut encrypted_counts = CodexHistoryProviderBuckets::default();
        let mut thread_candidates = Vec::new();

        for scope in [SessionScope::Sessions, SessionScope::ArchivedSessions] {
            let root = self.codex_home.join(scope.as_key());
            if !root.exists() {
                continue;
            }

            for path in collect_rollout_files(&root) {
                let first_line = match read_first_line_record(&path) {
                    Ok(record) => record,
                    Err(err) if is_locked_error(&err) => continue,
                    Err(err) => return Err(err),
                };

                let Some((parsed, payload)) = parse_session_meta_record(&first_line.first_line)?
                else {
                    continue;
                };

                let (_, updated_at) = extract_rollout_time_bounds(&path)?;
                if cutoff
                    .map(|value| updated_at < value.timestamp())
                    .unwrap_or(false)
                {
                    continue;
                }

                let current_provider_key = payload
                    .get("model_provider")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or_default()
                    .to_string();
                let current_provider = provider_display_name(&current_provider_key);
                increment_provider_count(&mut provider_counts, scope, &current_provider);
                let has_encrypted_content = rollout_has_encrypted_content(&path)?;
                if has_encrypted_content {
                    increment_provider_count(&mut encrypted_counts, scope, &current_provider);
                }

                let cwd = payload
                    .get("cwd")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();

                if let Some(id) = payload
                    .get("id")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.trim().is_empty())
                {
                    thread_candidates.push(RolloutThreadCandidate {
                        id: id.to_string(),
                        path: path.clone(),
                        archived: matches!(scope, SessionScope::ArchivedSessions),
                        updated_at,
                        cwd,
                        provider_key: current_provider_key.clone(),
                    });
                }

                let rewrite_source_allowed = rewrite_provider_keys
                    .map(|providers| providers.contains(&current_provider_key))
                    .unwrap_or(true);
                if target_provider == "__status_only__"
                    || current_provider_key == target_provider
                    || !rewrite_source_allowed
                {
                    continue;
                }

                let metadata = fs::metadata(&path)?;
                let modified_time = FileTime::from_last_modification_time(&metadata);
                let modified_ms = metadata
                    .modified()
                    .ok()
                    .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                    .map(|value| value.as_millis())
                    .unwrap_or(0);

                let mut updated = parsed;
                if let Some(payload_map) = updated
                    .get_mut("payload")
                    .and_then(|value| value.as_object_mut())
                {
                    payload_map.insert(
                        "model_provider".to_string(),
                        JsonValue::String(target_provider.to_string()),
                    );
                }

                changes.push(SessionChange {
                    path,
                    original_first_line: first_line.first_line,
                    original_separator: first_line.separator,
                    original_offset: first_line.offset,
                    original_size: metadata.len(),
                    original_modified_ms: modified_ms,
                    original_modified_time: modified_time,
                    updated_first_line: serde_json::to_string(&updated).map_err(|err| {
                        CcrError::ConfigError(format!("序列化 session_meta 失败: {}", err))
                    })?,
                });
            }
        }

        Ok(SessionScan {
            changes,
            provider_counts,
            encrypted_counts,
            thread_candidates,
        })
    }

    fn assert_sqlite_writable(&self, busy_timeout: Duration) -> Result<()> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(());
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        conn.busy_timeout(busy_timeout).map_err(|err| {
            CcrError::DatabaseError(format!("设置 SQLite busy timeout 失败: {}", err))
        })?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(map_sqlite_busy_error)?;
        conn.execute_batch("ROLLBACK").map_err(|err| {
            CcrError::DatabaseError(format!("回滚 SQLite 可写性检测失败: {}", err))
        })?;
        Ok(())
    }

    fn read_sqlite_provider_counts(&self) -> Result<Option<CodexHistoryProviderBuckets>> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    CASE
                        WHEN model_provider IS NULL OR model_provider = '' THEN '(missing)'
                        ELSE model_provider
                    END AS model_provider,
                    archived,
                    COUNT(*) AS count
                FROM threads
                GROUP BY model_provider, archived
                ORDER BY archived, model_provider
                "#,
            )
            .map_err(|err| {
                CcrError::DatabaseError(format!("准备 SQLite provider 统计失败: {}", err))
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|err| {
                CcrError::DatabaseError(format!("查询 SQLite provider 统计失败: {}", err))
            })?;

        let mut counts = CodexHistoryProviderBuckets::default();
        for row in rows {
            let (provider, archived, count) = row.map_err(|err| {
                CcrError::DatabaseError(format!("读取 SQLite provider 统计失败: {}", err))
            })?;
            let bucket = if archived {
                &mut counts.archived_sessions
            } else {
                &mut counts.sessions
            };
            bucket.insert(provider, count as usize);
        }

        Ok(Some(counts))
    }

    fn read_sqlite_thread_rows(&self) -> Result<Option<Vec<ThreadDbRow>>> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        let columns = load_thread_columns(&conn)?;
        Ok(Some(load_thread_rows(&conn, &columns)?))
    }

    fn load_workspace_root_keys_for_diagnostics(&self) -> BTreeSet<String> {
        let file_path = self.codex_home.join(GLOBAL_STATE_FILE_NAME);
        let Ok(text) = fs::read_to_string(&file_path) else {
            return BTreeSet::new();
        };
        let Ok(data) = serde_json::from_str::<JsonValue>(&text) else {
            return BTreeSet::new();
        };
        collect_known_sidebar_projects(data.as_object(), &self.codex_home)
            .into_iter()
            .filter_map(|path| workspace_root_key(&path))
            .collect()
    }

    fn read_visibility_diagnostics(
        &self,
        current_visibility_provider: &str,
        thread_candidates: &[RolloutThreadCandidate],
        workspace_root_keys: BTreeSet<String>,
    ) -> Result<Option<CodexHistoryVisibilityDiagnostics>> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        let columns = load_thread_columns(&conn)?;
        let preview_column_present = columns.contains("preview");
        let has_user_event_column_present = columns.contains("has_user_event");
        let rows = load_thread_rows(&conn, &columns)?;
        let rows_by_id = rows
            .iter()
            .map(|row| (row.id.clone(), row))
            .collect::<BTreeMap<_, _>>();

        let mut sqlite_rows_for_rollouts = 0usize;
        let mut sqlite_missing_rows = 0usize;
        let mut provider_mismatch_rows = 0usize;
        let mut preview_empty_rows = 0usize;
        let mut has_user_event_missing_rows = 0usize;
        let mut cwd_mismatch_rows = 0usize;
        let mut exact_cwd_matches = 0usize;
        let mut normalized_cwd_matches = 0usize;
        let mut workspace_root_matches = 0usize;

        for candidate in thread_candidates {
            let Some(row) = rows_by_id.get(&candidate.id) else {
                sqlite_missing_rows += 1;
                continue;
            };
            sqlite_rows_for_rollouts += 1;

            if row.model_provider != current_visibility_provider {
                provider_mismatch_rows += 1;
            }
            if preview_column_present && row.preview.trim().is_empty() {
                preview_empty_rows += 1;
            }
            if has_user_event_column_present && !row.has_user_event {
                has_user_event_missing_rows += 1;
            }

            let candidate_cwd = candidate.cwd.trim();
            let row_cwd = row.cwd.trim();
            if !candidate_cwd.is_empty() || !row_cwd.is_empty() {
                if candidate_cwd == row_cwd {
                    exact_cwd_matches += 1;
                }
                let candidate_key = workspace_root_key(candidate_cwd);
                let row_key = workspace_root_key(row_cwd);
                if candidate_key.is_some() && candidate_key == row_key {
                    normalized_cwd_matches += 1;
                } else {
                    cwd_mismatch_rows += 1;
                }
            }

            if let Some(row_key) = workspace_root_key(row_cwd)
                && workspace_root_keys.contains(&row_key)
            {
                workspace_root_matches += 1;
            }
        }

        let verbatim_extended_cwd_rows = rows
            .iter()
            .filter(|row| row.cwd.trim().starts_with(r"\\?\"))
            .count();
        let candidate_ids = thread_candidates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect::<BTreeSet<_>>();
        let mut provider_rows = rows
            .iter()
            .filter(|row| !row.archived && row.model_provider == current_visibility_provider)
            .collect::<Vec<_>>();
        provider_rows.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.id.cmp(&right.id))
        });

        const DESKTOP_FIRST_PAGE_LIMIT: usize = 50;
        let desktop_first_page_size = provider_rows.len().min(DESKTOP_FIRST_PAGE_LIMIT);
        let mut desktop_first_page_matching_rows = 0usize;
        let mut rank_min = None::<usize>;
        let mut rank_max = None::<usize>;
        for (index, row) in provider_rows.iter().enumerate() {
            if !candidate_ids.contains(row.id.as_str()) {
                continue;
            }
            let rank = index + 1;
            if rank <= DESKTOP_FIRST_PAGE_LIMIT {
                desktop_first_page_matching_rows += 1;
            }
            rank_min = Some(rank_min.map_or(rank, |value| value.min(rank)));
            rank_max = Some(rank_max.map_or(rank, |value| value.max(rank)));
        }

        Ok(Some(CodexHistoryVisibilityDiagnostics {
            current_visibility_provider: current_visibility_provider.to_string(),
            rollout_threads: thread_candidates.len(),
            sqlite_rows_for_rollouts,
            sqlite_missing_rows,
            provider_mismatch_rows,
            preview_column_present,
            preview_empty_rows,
            has_user_event_column_present,
            has_user_event_missing_rows,
            cwd_mismatch_rows,
            exact_cwd_matches,
            normalized_cwd_matches,
            verbatim_extended_cwd_rows,
            workspace_root_candidates: thread_candidates.len(),
            workspace_root_matches,
            desktop_first_page_limit: DESKTOP_FIRST_PAGE_LIMIT,
            desktop_first_page_size,
            desktop_first_page_matching_rows,
            desktop_first_page_limited: rank_max
                .map(|rank| rank > DESKTOP_FIRST_PAGE_LIMIT)
                .unwrap_or(false),
            rank_min,
            rank_max,
        }))
    }

    fn read_sqlite_project_paths(
        &self,
        busy_timeout: Duration,
        thread_candidates: &[RolloutThreadCandidate],
    ) -> Result<Vec<String>> {
        let mut paths = BTreeSet::new();
        for candidate in thread_candidates {
            let record = build_rollout_thread_insert(candidate, DEFAULT_PROVIDER)?;
            if !record.cwd.trim().is_empty() {
                paths.insert(record.cwd);
            }
        }

        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(paths.into_iter().collect());
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        conn.busy_timeout(busy_timeout).map_err(|err| {
            CcrError::DatabaseError(format!("设置 SQLite busy timeout 失败: {}", err))
        })?;

        let candidate_ids: Vec<&str> = thread_candidates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect();
        if candidate_ids.is_empty() {
            return Ok(paths.into_iter().collect());
        }

        let placeholders = std::iter::repeat_n("?", candidate_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT DISTINCT cwd FROM threads WHERE TRIM(COALESCE(cwd, '')) <> '' AND id IN ({}) ORDER BY LOWER(cwd), cwd",
            placeholders
        );

        let mut stmt = match conn.prepare(&sql) {
            Ok(stmt) => stmt,
            Err(err)
                if err
                    .to_string()
                    .to_ascii_lowercase()
                    .contains("no such column: cwd") =>
            {
                return Ok(paths.into_iter().collect());
            }
            Err(err) => {
                return Err(CcrError::DatabaseError(format!(
                    "读取 SQLite cwd 列表失败: {}",
                    err
                )));
            }
        };

        let rows = stmt
            .query_map(params_from_iter(candidate_ids.iter().copied()), |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| CcrError::DatabaseError(format!("查询 SQLite cwd 列表失败: {}", err)))?;

        for row in rows {
            paths.insert(row.map_err(|err| {
                CcrError::DatabaseError(format!("读取 SQLite cwd 失败: {}", err))
            })?);
        }

        Ok(paths.into_iter().collect())
    }

    fn prepare_sidebar_sync(&self, workspace_roots: Vec<String>) -> Result<SidebarSyncPlan> {
        let file_path = self.codex_home.join(GLOBAL_STATE_FILE_NAME);
        let (exists, original_text, state_data) = if file_path.exists() {
            let text = fs::read_to_string(&file_path)?;
            let data = serde_json::from_str::<JsonValue>(&text).map_err(|err| {
                CcrError::ConfigError(format!("解析 {} 失败: {}", GLOBAL_STATE_FILE_NAME, err))
            })?;
            if !data.is_object() {
                return Err(CcrError::ConfigError(format!(
                    "{} 顶层必须是 JSON object",
                    GLOBAL_STATE_FILE_NAME
                )));
            }
            (true, Some(text), Some(data))
        } else {
            (false, None, None)
        };

        let known_sidebar_projects = collect_known_sidebar_projects(
            state_data.as_ref().and_then(JsonValue::as_object),
            &self.codex_home,
        );
        let sqlite_projects =
            collect_sidebar_project_candidates(&workspace_roots, &self.codex_home);
        let sqlite_project_keys: BTreeSet<String> = sqlite_projects
            .iter()
            .filter_map(|project| workspace_root_key(project))
            .collect();

        let normalized_candidates: Vec<String> = known_sidebar_projects
            .into_iter()
            .filter(|project| {
                workspace_root_key(project)
                    .map(|key| sqlite_project_keys.contains(&key))
                    .unwrap_or(false)
            })
            .collect();

        let existing_state = state_data
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        let mut workspace_roots_list = existing_state
            .get("electron-saved-workspace-roots")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();
        let mut project_order_list = existing_state
            .get("project-order")
            .and_then(JsonValue::as_array)
            .cloned()
            .unwrap_or_default();

        let mut workspace_root_keys: BTreeSet<String> = workspace_roots_list
            .iter()
            .filter_map(JsonValue::as_str)
            .filter_map(workspace_root_key)
            .collect();
        let mut project_order_keys: BTreeSet<String> = project_order_list
            .iter()
            .filter_map(JsonValue::as_str)
            .filter_map(workspace_root_key)
            .collect();

        let mut added_projects = Vec::new();
        for project in normalized_candidates {
            let Some(project_key) = workspace_root_key(&project) else {
                continue;
            };
            let missing_workspace_root = !workspace_root_keys.contains(&project_key);
            let missing_project_order = !project_order_keys.contains(&project_key);
            if !missing_workspace_root && !missing_project_order {
                continue;
            }

            added_projects.push(project.clone());
            if missing_workspace_root {
                workspace_roots_list.push(JsonValue::String(project.clone()));
                workspace_root_keys.insert(project_key.clone());
            }
            if missing_project_order {
                project_order_list.push(JsonValue::String(project.clone()));
                project_order_keys.insert(project_key);
            }
        }

        let next_text = if added_projects.is_empty() {
            None
        } else {
            let mut next_state = existing_state;
            next_state.insert(
                "electron-saved-workspace-roots".to_string(),
                JsonValue::Array(workspace_roots_list),
            );
            next_state.insert(
                "project-order".to_string(),
                JsonValue::Array(project_order_list),
            );
            Some(
                serde_json::to_string(&JsonValue::Object(next_state)).map_err(|err| {
                    CcrError::ConfigError(format!(
                        "序列化 {} 失败: {}",
                        GLOBAL_STATE_FILE_NAME, err
                    ))
                })?,
            )
        };

        Ok(SidebarSyncPlan {
            file_path,
            existed: exists,
            original_text,
            next_text,
            added_projects,
        })
    }

    fn apply_sidebar_sync(&self, plan: &SidebarSyncPlan) -> Result<Option<GlobalStateSnapshot>> {
        let Some(next_text) = &plan.next_text else {
            return Ok(None);
        };

        if let Some(parent) = plan.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&plan.file_path, next_text)?;

        Ok(Some(GlobalStateSnapshot {
            file_path: plan.file_path.clone(),
            existed: plan.existed,
            original_text: plan.original_text.clone(),
        }))
    }

    fn restore_global_state_snapshot(&self, snapshot: &GlobalStateSnapshot) -> Result<()> {
        if snapshot.existed {
            fs::write(
                &snapshot.file_path,
                snapshot.original_text.as_deref().unwrap_or_default(),
            )?;
        } else if snapshot.file_path.exists() {
            fs::remove_file(&snapshot.file_path)?;
        }
        Ok(())
    }

    fn prepare_session_index_repair(
        &self,
        thread_candidates: &[RolloutThreadCandidate],
        sqlite_rows: Option<&[ThreadDbRow]>,
        repair_policy: Option<&SqliteRepairPolicy>,
    ) -> Result<SessionIndexPlan> {
        let state = self.read_session_index_state()?;
        let mut seen_ids = state
            .entries_by_id
            .keys()
            .cloned()
            .collect::<BTreeSet<String>>();
        let mut entries_to_add = Vec::new();
        let candidates_by_id = thread_candidates
            .iter()
            .map(|candidate| (candidate.id.as_str(), candidate))
            .collect::<BTreeMap<_, _>>();

        if let Some(rows) = sqlite_rows {
            for row in rows {
                if !session_index_row_allowed(
                    row,
                    candidates_by_id.get(row.id.as_str()).copied(),
                    repair_policy,
                ) {
                    continue;
                }
                if !seen_ids.insert(row.id.clone()) {
                    continue;
                }
                entries_to_add.push(build_session_index_entry_from_sqlite_row(row));
            }
        }

        for candidate in thread_candidates {
            if !session_index_candidate_allowed(candidate, repair_policy) {
                continue;
            }
            if !seen_ids.insert(candidate.id.clone()) {
                continue;
            }
            entries_to_add.push(build_session_index_entry(candidate)?);
        }

        Ok(SessionIndexPlan {
            file_path: state.file_path,
            existed: state.existed,
            original_text: state.original_text,
            entries_to_add,
        })
    }

    fn read_session_index_state(&self) -> Result<SessionIndexState> {
        let file_path = self.codex_home.join(SESSION_INDEX_FILE);
        let original_text = if file_path.exists() {
            Some(fs::read_to_string(&file_path)?)
        } else {
            None
        };
        let entries_by_id = parse_session_index_entries(original_text.as_deref())?;

        Ok(SessionIndexState {
            file_path,
            existed: original_text.is_some(),
            original_text,
            entries_by_id,
        })
    }

    fn apply_session_index_repair(
        &self,
        plan: &SessionIndexPlan,
    ) -> Result<SessionIndexUpdateResult> {
        if plan.entries_to_add.is_empty() {
            return Ok(SessionIndexUpdateResult {
                snapshot: None,
                added_entries: 0,
            });
        }

        let mut lines = plan
            .original_text
            .as_deref()
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }

        for entry in &plan.entries_to_add {
            lines.push(serde_json::to_string(entry).map_err(|err| {
                CcrError::ConfigError(format!("序列化 session_index 条目失败: {}", err))
            })?);
        }

        let mut output = lines.join("\n");
        output.push('\n');
        write_text_atomic(&plan.file_path, &output)?;

        Ok(SessionIndexUpdateResult {
            snapshot: Some(SessionIndexSnapshot {
                file_path: plan.file_path.clone(),
                existed: plan.existed,
                original_text: plan.original_text.clone(),
            }),
            added_entries: plan.entries_to_add.len(),
        })
    }

    fn restore_session_index_snapshot(&self, snapshot: &SessionIndexSnapshot) -> Result<()> {
        restore_text_snapshot(
            &snapshot.file_path,
            snapshot.existed,
            snapshot.original_text.as_deref(),
        )
    }

    fn create_backup(
        &self,
        target_provider: &str,
        sidebar_plan: &SidebarSyncPlan,
        session_index_plan: &SessionIndexPlan,
    ) -> Result<PathBuf> {
        let backup_dir = self
            .backup_root()
            .join(Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string());
        fs::create_dir_all(&backup_dir)?;

        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        let global_state_path = self.codex_home.join(GLOBAL_STATE_FILE_NAME);
        let state_db_present = db_path.exists();
        let global_state_present = sidebar_plan.existed || global_state_path.exists();
        let session_index_present = session_index_plan.existed;
        let session_index_backed_up = !session_index_plan.entries_to_add.is_empty();

        if state_db_present {
            let target = backup_dir.join("db").join(SQLITE_FILE_NAME);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&db_path, &target)?;
        }

        if global_state_present && global_state_path.exists() {
            let target = backup_dir.join("global").join(GLOBAL_STATE_FILE_NAME);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&global_state_path, &target)?;
        }

        if session_index_backed_up && session_index_present && session_index_plan.file_path.exists()
        {
            let target = backup_dir.join("session-index").join(SESSION_INDEX_FILE);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&session_index_plan.file_path, &target)?;
        }

        let metadata = BackupMetadata {
            version: 2,
            namespace: BACKUP_NAMESPACE.to_string(),
            codex_home: self.codex_home.display().to_string(),
            target_provider: target_provider.to_string(),
            created_at: Utc::now(),
            state_db_present,
            global_state_present,
            session_index_present: session_index_backed_up.then_some(session_index_present),
            session_index_backed_up,
        };
        fs::write(
            backup_dir.join(METADATA_FILE_NAME),
            serde_json::to_vec_pretty(&metadata)
                .map_err(|err| CcrError::ConfigError(format!("序列化备份元数据失败: {}", err)))?,
        )?;
        self.write_session_manifest(&backup_dir, &[])?;
        Ok(backup_dir)
    }

    fn read_backup_metadata(&self, backup_dir: &Path) -> Result<BackupMetadata> {
        let metadata_path = backup_dir.join(METADATA_FILE_NAME);
        let text = fs::read_to_string(&metadata_path)?;
        let metadata: BackupMetadata = serde_json::from_str(&text).map_err(|err| {
            CcrError::ConfigError(format!(
                "解析历史同步备份元数据失败 ({}): {}",
                metadata_path.display(),
                err
            ))
        })?;
        if metadata.namespace != BACKUP_NAMESPACE {
            return Err(CcrError::ConfigError(format!(
                "备份目录 {} 不是 sync-history 管理的备份",
                backup_dir.display()
            )));
        }
        Ok(metadata)
    }

    fn write_session_manifest(
        &self,
        backup_dir: &Path,
        entries: &[AppliedSessionBackupEntry],
    ) -> Result<()> {
        let manifest = SessionBackupManifest {
            entries: entries.to_vec(),
        };
        fs::write(
            backup_dir.join(SESSION_MANIFEST_FILE_NAME),
            serde_json::to_vec_pretty(&manifest).map_err(|err| {
                CcrError::ConfigError(format!("序列化 rollout manifest 失败: {}", err))
            })?,
        )?;
        Ok(())
    }

    fn read_session_manifest(&self, backup_dir: &Path) -> Result<Vec<AppliedSessionBackupEntry>> {
        let manifest_path = backup_dir.join(SESSION_MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&manifest_path)?;
        let manifest: SessionBackupManifest = serde_json::from_str(&text).map_err(|err| {
            CcrError::ConfigError(format!(
                "解析 rollout manifest 失败 ({}): {}",
                manifest_path.display(),
                err
            ))
        })?;
        Ok(manifest.entries)
    }

    fn list_managed_backups(&self) -> Result<Vec<ManagedBackupDir>> {
        let backup_root = self.backup_root();
        if !backup_root.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(&backup_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path();
            if self.read_backup_metadata(&path).is_err() {
                continue;
            }
            backups.push(ManagedBackupDir {
                sort_key: path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    .to_string(),
                total_bytes: compute_dir_size(&path)?,
                path,
            });
        }

        Ok(backups)
    }

    fn apply_session_changes(&self, changes: &[SessionChange]) -> Result<SessionApplyResult> {
        let mut applied_changes = Vec::new();
        let mut skipped_locked_paths = Vec::new();

        for change in changes {
            match rewrite_rollout_first_line(change) {
                Ok(Some(entry)) => applied_changes.push(entry),
                Ok(None) => {}
                Err(err) if is_locked_error(&err) => {
                    skipped_locked_paths.push(change.path.clone());
                }
                Err(err) => return Err(err),
            }
        }

        Ok(SessionApplyResult {
            applied_changes,
            skipped_locked_paths,
        })
    }

    fn restore_session_entries(&self, entries: &[AppliedSessionBackupEntry]) -> Result<()> {
        for entry in entries {
            restore_rollout_first_line(entry)?;
        }
        Ok(())
    }

    fn preview_sqlite_provider_update(
        &self,
        target_provider: &str,
        busy_timeout: Duration,
        thread_candidates: &[RolloutThreadCandidate],
        repair_policy: &SqliteRepairPolicy,
    ) -> Result<SqliteUpdateResult> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(SqliteUpdateResult {
                updated_rows: 0,
                database_present: false,
            });
        }

        let conn = Connection::open(&db_path).map_err(|err| {
            CcrError::DatabaseError(format!("open state_5.sqlite failed: {}", err))
        })?;
        conn.busy_timeout(busy_timeout).map_err(|err| {
            CcrError::DatabaseError(format!("set SQLite busy timeout failed: {}", err))
        })?;

        let columns = load_thread_columns(&conn)?;
        let rows = load_thread_rows(&conn, &columns)?;
        let plan = plan_sqlite_repairs(
            target_provider,
            &columns,
            &rows,
            thread_candidates,
            repair_policy,
        )?;

        Ok(SqliteUpdateResult {
            updated_rows: plan.affected_ids.len(),
            database_present: true,
        })
    }

    fn update_sqlite_provider(
        &self,
        target_provider: &str,
        busy_timeout: Duration,
        thread_candidates: &[RolloutThreadCandidate],
        repair_policy: &SqliteRepairPolicy,
    ) -> Result<SqliteUpdateResult> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(SqliteUpdateResult {
                updated_rows: 0,
                database_present: false,
            });
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        conn.busy_timeout(busy_timeout).map_err(|err| {
            CcrError::DatabaseError(format!("设置 SQLite busy timeout 失败: {}", err))
        })?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(map_sqlite_busy_error)?;

        let columns = match load_thread_columns(&conn) {
            Ok(columns) => columns,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        };
        let rows = match load_thread_rows(&conn, &columns) {
            Ok(rows) => rows,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        };
        let plan = match plan_sqlite_repairs(
            target_provider,
            &columns,
            &rows,
            thread_candidates,
            repair_policy,
        ) {
            Ok(plan) => plan,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        };

        for id in &plan.provider_update_ids {
            if let Err(err) = conn.execute(
                "UPDATE threads SET model_provider = ?1 WHERE id = ?2 AND COALESCE(model_provider, '') <> ?1",
                [target_provider, id.as_str()],
            ) {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(map_sqlite_busy_error(err));
            }
        }

        for (id, preview) in &plan.preview_updates {
            if let Err(err) = conn.execute(
                "UPDATE threads SET preview = ?1 WHERE id = ?2 AND TRIM(COALESCE(preview, '')) = ''",
                [preview.as_str(), id.as_str()],
            ) {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(map_sqlite_busy_error(err));
            }
        }

        for id in &plan.has_user_event_update_ids {
            if let Err(err) = conn.execute(
                "UPDATE threads SET has_user_event = 1 WHERE id = ?1 AND COALESCE(has_user_event, 0) = 0",
                [id.as_str()],
            ) {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(map_sqlite_busy_error(err));
            }
        }

        for (id, cwd) in &plan.cwd_updates {
            if let Err(err) = conn.execute(
                "UPDATE threads SET cwd = ?1 WHERE id = ?2 AND COALESCE(cwd, '') <> ?1",
                [cwd.as_str(), id.as_str()],
            ) {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(map_sqlite_busy_error(err));
            }
        }

        for record in &plan.missing_inserts {
            if let Err(err) = insert_thread_row(&conn, record) {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        }

        conn.execute_batch("COMMIT").map_err(|err| {
            CcrError::DatabaseError(format!("提交 SQLite provider 更新失败: {}", err))
        })?;
        Ok(SqliteUpdateResult {
            updated_rows: plan.affected_ids.len(),
            database_present: true,
        })
    }

    fn restore_global_state_from_backup(
        &self,
        backup_dir: &Path,
        metadata: &BackupMetadata,
    ) -> Result<()> {
        let current = self.codex_home.join(GLOBAL_STATE_FILE_NAME);
        let backup_file = backup_dir.join("global").join(GLOBAL_STATE_FILE_NAME);

        if metadata.global_state_present {
            if backup_file.exists() {
                fs::copy(backup_file, current)?;
            }
        } else if current.exists() {
            fs::remove_file(current)?;
        }
        Ok(())
    }

    fn restore_sqlite_from_backup(
        &self,
        backup_dir: &Path,
        metadata: &BackupMetadata,
    ) -> Result<()> {
        let current = self.codex_home.join(SQLITE_FILE_NAME);
        let backup_file = backup_dir.join("db").join(SQLITE_FILE_NAME);

        if metadata.state_db_present {
            if backup_file.exists() {
                fs::copy(backup_file, current)?;
            }
        } else if current.exists() {
            fs::remove_file(current)?;
        }
        Ok(())
    }

    fn restore_session_index_from_backup(
        &self,
        backup_dir: &Path,
        metadata: &BackupMetadata,
    ) -> Result<bool> {
        if !metadata.session_index_backed_up {
            return Ok(false);
        }
        let Some(session_index_present) = metadata.session_index_present else {
            return Ok(false);
        };

        let current = self.codex_home.join(SESSION_INDEX_FILE);
        let backup_file = backup_dir.join("session-index").join(SESSION_INDEX_FILE);

        if session_index_present {
            if backup_file.exists() {
                fs::copy(backup_file, current)?;
            }
        } else if current.exists() {
            fs::remove_file(current)?;
        }
        Ok(true)
    }
}

fn collect_rollout_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            let file_name = path.file_name().and_then(OsStr::to_str)?;
            (file_name.starts_with("rollout-") && file_name.ends_with(".jsonl")).then_some(path)
        })
        .collect()
}

fn read_first_line_record(path: &Path) -> Result<FirstLineRecord> {
    let file = File::open(path).map_err(map_io_error)?;
    let mut reader = BufReader::new(file);
    let mut first_line_bytes = Vec::new();
    let bytes_read = reader
        .read_until(b'\n', &mut first_line_bytes)
        .map_err(map_io_error)?;
    if bytes_read == 0 {
        return Ok(FirstLineRecord {
            first_line: String::new(),
            separator: String::new(),
            offset: 0,
        });
    }

    let separator = if first_line_bytes.ends_with(b"\r\n") {
        first_line_bytes.truncate(first_line_bytes.len() - 2);
        "\r\n".to_string()
    } else if first_line_bytes.ends_with(b"\n") {
        first_line_bytes.truncate(first_line_bytes.len() - 1);
        "\n".to_string()
    } else {
        String::new()
    };

    let first_line = String::from_utf8(first_line_bytes)
        .map_err(|err| CcrError::FileIoError(format!("读取 rollout 首行失败: {}", err)))?;

    Ok(FirstLineRecord {
        first_line,
        separator,
        offset: bytes_read as u64,
    })
}

fn parse_session_meta_record(
    first_line: &str,
) -> Result<Option<(JsonValue, JsonMap<String, JsonValue>)>> {
    if first_line.trim().is_empty() {
        return Ok(None);
    }

    let parsed: JsonValue = serde_json::from_str(first_line)
        .map_err(|err| CcrError::ConfigError(format!("解析 rollout 首行 JSON 失败: {}", err)))?;
    let Some(root) = parsed.as_object() else {
        return Ok(None);
    };

    if root.get("type").and_then(JsonValue::as_str) != Some("session_meta") {
        return Ok(None);
    }

    let payload = root
        .get("payload")
        .and_then(JsonValue::as_object)
        .cloned()
        .ok_or_else(|| CcrError::ConfigError("session_meta.payload 不是 JSON object".into()))?;
    Ok(Some((parsed, payload)))
}

fn parse_session_index_entries(content: Option<&str>) -> Result<BTreeMap<String, JsonValue>> {
    let mut entries = BTreeMap::new();
    let Some(content) = content else {
        return Ok(entries);
    };

    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            CcrError::ConfigError(format!(
                "解析 {} 第 {} 行失败: {}",
                SESSION_INDEX_FILE,
                index + 1,
                err
            ))
        })?;
        let Some(id) = entry.get("id").and_then(JsonValue::as_str) else {
            continue;
        };
        if id.trim().is_empty() {
            continue;
        }
        entries.insert(id.to_string(), entry);
    }

    Ok(entries)
}

fn session_index_candidate_allowed(
    candidate: &RolloutThreadCandidate,
    repair_policy: Option<&SqliteRepairPolicy>,
) -> bool {
    repair_policy
        .filter(|policy| policy.broad_provider_repair)
        .map(|policy| {
            policy
                .allowed_provider_keys
                .contains(candidate.provider_key.as_str())
        })
        .unwrap_or(true)
}

fn session_index_row_allowed(
    row: &ThreadDbRow,
    candidate: Option<&RolloutThreadCandidate>,
    repair_policy: Option<&SqliteRepairPolicy>,
) -> bool {
    repair_policy
        .filter(|policy| policy.broad_provider_repair)
        .map(|policy| {
            policy
                .allowed_provider_keys
                .contains(row.model_provider.as_str())
                && candidate
                    .map(|candidate| {
                        policy
                            .allowed_provider_keys
                            .contains(candidate.provider_key.as_str())
                    })
                    .unwrap_or(true)
        })
        .unwrap_or(true)
}

fn build_session_index_entry_from_sqlite_row(row: &ThreadDbRow) -> JsonValue {
    let thread_name = [&row.title, &row.preview, &row.first_user_message]
        .into_iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .unwrap_or("Untitled");
    serde_json::json!({
        "id": row.id.clone(),
        "thread_name": thread_name,
        "updated_at": format_session_index_updated_at(row.updated_at),
    })
}

fn build_session_index_entry(candidate: &RolloutThreadCandidate) -> Result<JsonValue> {
    let record = build_rollout_thread_insert(candidate, DEFAULT_PROVIDER)?;
    let thread_name = if record.title.trim().is_empty() {
        "Untitled".to_string()
    } else {
        record.title
    };
    Ok(serde_json::json!({
        "id": candidate.id.clone(),
        "thread_name": thread_name,
        "updated_at": format_session_index_updated_at(record.updated_at),
    }))
}

fn format_session_index_updated_at(updated_at: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(updated_at, 0)
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

fn write_text_atomic(path: &Path, content: &str) -> Result<()> {
    let temp_file = if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        NamedTempFile::new_in(parent)
    } else {
        NamedTempFile::new()
    }
    .map_err(|err| CcrError::FileIoError(format!("创建临时文件失败: {}", err)))?;

    fs::write(temp_file.path(), content)
        .map_err(|err| CcrError::FileIoError(format!("写入临时文件失败: {}", err)))?;
    temp_file
        .persist(path)
        .map_err(|err| CcrError::FileIoError(format!("原子替换文件失败: {}", err)))?;
    Ok(())
}

fn restore_text_snapshot(path: &Path, existed: bool, original_text: Option<&str>) -> Result<()> {
    if existed {
        write_text_atomic(path, original_text.unwrap_or_default())?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn insert_thread_row(conn: &Connection, record: &RolloutThreadInsert) -> Result<usize> {
    let columns = load_thread_columns(conn)?;
    let mut selected_columns = Vec::new();
    let mut values = Vec::<SqlValue>::new();

    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "id",
        SqlValue::Text(record.id.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "rollout_path",
        SqlValue::Text(record.rollout_path.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "created_at",
        SqlValue::Integer(record.created_at),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "updated_at",
        SqlValue::Integer(record.updated_at),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "source",
        SqlValue::Text(record.source.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "model_provider",
        SqlValue::Text(record.model_provider.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "cwd",
        SqlValue::Text(record.cwd.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "title",
        SqlValue::Text(record.title.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "sandbox_policy",
        SqlValue::Text(record.sandbox_policy.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "approval_mode",
        SqlValue::Text(record.approval_mode.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "tokens_used",
        SqlValue::Integer(record.tokens_used),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "has_user_event",
        SqlValue::Integer(if record.has_user_event { 1 } else { 0 }),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "archived",
        SqlValue::Integer(if record.archived { 1 } else { 0 }),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "archived_at",
        record.archived_at.map(SqlValue::Integer),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "cli_version",
        SqlValue::Text(record.cli_version.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "first_user_message",
        SqlValue::Text(record.first_user_message.clone()),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "preview",
        SqlValue::Text(record.preview.clone()),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "agent_nickname",
        record.agent_nickname.clone().map(SqlValue::Text),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "agent_role",
        record.agent_role.clone().map(SqlValue::Text),
    );
    push_thread_value(
        &columns,
        &mut selected_columns,
        &mut values,
        "memory_mode",
        SqlValue::Text(record.memory_mode.clone()),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "model",
        record.model.clone().map(SqlValue::Text),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "reasoning_effort",
        record.reasoning_effort.clone().map(SqlValue::Text),
    );
    push_thread_value_opt(
        &columns,
        &mut selected_columns,
        &mut values,
        "agent_path",
        record.agent_path.clone().map(SqlValue::Text),
    );

    let placeholders = (0..selected_columns.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT OR IGNORE INTO threads ({}) VALUES ({})",
        selected_columns.join(", "),
        placeholders
    );
    conn.execute(&sql, params_from_iter(values))
        .map_err(|err| CcrError::DatabaseError(format!("插入缺失 thread 失败: {}", err)))
}

fn load_thread_columns(conn: &Connection) -> Result<BTreeSet<String>> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(threads)")
        .map_err(|err| CcrError::DatabaseError(format!("读取 threads schema 失败: {}", err)))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| CcrError::DatabaseError(format!("读取 threads schema 行失败: {}", err)))?;

    let mut columns = BTreeSet::new();
    for row in rows {
        columns.insert(row.map_err(|err| {
            CcrError::DatabaseError(format!("解析 threads schema 失败: {}", err))
        })?);
    }
    Ok(columns)
}

fn load_thread_rows(conn: &Connection, columns: &BTreeSet<String>) -> Result<Vec<ThreadDbRow>> {
    let model_provider_expr = if columns.contains("model_provider") {
        "COALESCE(model_provider, '')"
    } else {
        "''"
    };
    let archived_expr = if columns.contains("archived") {
        "COALESCE(archived, 0)"
    } else {
        "0"
    };
    let cwd_expr = if columns.contains("cwd") {
        "COALESCE(cwd, '')"
    } else {
        "''"
    };
    let title_expr = if columns.contains("title") {
        "COALESCE(title, '')"
    } else {
        "''"
    };
    let preview_expr = if columns.contains("preview") {
        "COALESCE(preview, '')"
    } else {
        "''"
    };
    let first_user_message_expr = if columns.contains("first_user_message") {
        "COALESCE(first_user_message, '')"
    } else {
        "''"
    };
    let has_user_event_expr = if columns.contains("has_user_event") {
        "COALESCE(has_user_event, 0)"
    } else {
        "0"
    };
    let updated_at_expr = if columns.contains("updated_at") {
        "COALESCE(updated_at, 0)"
    } else {
        "0"
    };
    let sql = format!(
        "SELECT id, {model_provider_expr}, {archived_expr}, {cwd_expr}, {title_expr}, {preview_expr}, {first_user_message_expr}, {has_user_event_expr}, {updated_at_expr} FROM threads"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| CcrError::DatabaseError(format!("准备读取 threads 行失败: {}", err)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ThreadDbRow {
                id: row.get(0)?,
                model_provider: row.get::<_, String>(1)?.trim().to_string(),
                archived: row.get::<_, i64>(2)? != 0,
                cwd: row.get(3)?,
                title: row.get(4)?,
                preview: row.get(5)?,
                first_user_message: row.get(6)?,
                has_user_event: row.get::<_, i64>(7)? != 0,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|err| CcrError::DatabaseError(format!("查询 threads 行失败: {}", err)))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(
            row.map_err(|err| CcrError::DatabaseError(format!("读取 threads 行失败: {}", err)))?,
        );
    }
    Ok(result)
}

fn plan_sqlite_repairs(
    target_provider: &str,
    columns: &BTreeSet<String>,
    rows: &[ThreadDbRow],
    thread_candidates: &[RolloutThreadCandidate],
    repair_policy: &SqliteRepairPolicy,
) -> Result<SqliteRepairPlan> {
    let rows_by_id = rows
        .iter()
        .map(|row| (row.id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut candidate_records = BTreeMap::<String, (RolloutThreadInsert, String)>::new();
    for candidate in thread_candidates {
        candidate_records.insert(
            candidate.id.clone(),
            (
                build_rollout_thread_insert(candidate, target_provider)?,
                candidate.provider_key.clone(),
            ),
        );
    }

    let mut provider_update_ids = BTreeSet::new();
    let mut preview_updates = BTreeMap::new();
    let mut has_user_event_update_ids = BTreeSet::new();
    let mut cwd_updates = BTreeMap::new();
    let mut missing_inserts = Vec::new();
    let mut affected_ids = BTreeSet::new();

    for row in rows {
        let is_candidate = candidate_records.contains_key(&row.id);
        let provider_key = row.model_provider.trim();
        let row_allowed_by_broad_policy = repair_policy.broad_provider_repair
            && repair_policy.allowed_provider_keys.contains(provider_key);
        let candidate_allowed_by_broad_policy = candidate_records
            .get(&row.id)
            .map(|(_, candidate_provider_key)| {
                repair_policy
                    .allowed_provider_keys
                    .contains(candidate_provider_key.as_str())
            })
            .unwrap_or(true);
        let allowed_provider_update = if repair_policy.broad_provider_repair {
            row_allowed_by_broad_policy && candidate_allowed_by_broad_policy
        } else {
            is_candidate
        };
        if allowed_provider_update && provider_key != target_provider {
            provider_update_ids.insert(row.id.clone());
            affected_ids.insert(row.id.clone());
        }

        let Some((record, _)) = candidate_records.get(&row.id) else {
            continue;
        };
        if repair_policy.broad_provider_repair
            && (!row_allowed_by_broad_policy || !candidate_allowed_by_broad_policy)
        {
            continue;
        }
        if columns.contains("preview")
            && row.preview.trim().is_empty()
            && !record.preview.trim().is_empty()
        {
            preview_updates.insert(row.id.clone(), record.preview.clone());
            affected_ids.insert(row.id.clone());
        }
        if columns.contains("has_user_event") && !row.has_user_event && record.has_user_event {
            has_user_event_update_ids.insert(row.id.clone());
            affected_ids.insert(row.id.clone());
        }
        if columns.contains("cwd")
            && !record.cwd.trim().is_empty()
            && row.cwd.trim() != record.cwd.trim()
        {
            cwd_updates.insert(row.id.clone(), record.cwd.clone());
            affected_ids.insert(row.id.clone());
        }
    }

    for (id, (record, candidate_provider_key)) in candidate_records {
        if rows_by_id.contains_key(&id) {
            continue;
        }
        if repair_policy.broad_provider_repair
            && !repair_policy
                .allowed_provider_keys
                .contains(candidate_provider_key.as_str())
        {
            continue;
        }
        affected_ids.insert(id);
        missing_inserts.push(record);
    }

    Ok(SqliteRepairPlan {
        provider_update_ids,
        preview_updates,
        has_user_event_update_ids,
        cwd_updates,
        missing_inserts,
        affected_ids,
    })
}

fn push_thread_value(
    available_columns: &BTreeSet<String>,
    selected_columns: &mut Vec<String>,
    values: &mut Vec<SqlValue>,
    column: &str,
    value: SqlValue,
) {
    if available_columns.contains(column) {
        selected_columns.push(column.to_string());
        values.push(value);
    }
}

fn push_thread_value_opt(
    available_columns: &BTreeSet<String>,
    selected_columns: &mut Vec<String>,
    values: &mut Vec<SqlValue>,
    column: &str,
    value: Option<SqlValue>,
) {
    if available_columns.contains(column) {
        selected_columns.push(column.to_string());
        values.push(value.unwrap_or(SqlValue::Null));
    }
}

fn build_rollout_thread_insert(
    candidate: &RolloutThreadCandidate,
    target_provider: &str,
) -> Result<RolloutThreadInsert> {
    let file = File::open(&candidate.path).map_err(map_io_error)?;
    let reader = BufReader::new(file);

    let mut source = String::new();
    let mut cwd = String::new();
    let mut title = String::new();
    let mut sandbox_policy = String::new();
    let mut approval_mode = String::new();
    let mut cli_version = String::new();
    let mut first_user_message = String::new();
    let mut goal = String::new();
    let mut memory_mode = "enabled".to_string();
    let mut model = None;
    let mut reasoning_effort = None;
    let mut agent_nickname = None;
    let mut agent_role = None;
    let mut agent_path = None;

    for line_result in reader.lines() {
        let line = line_result.map_err(map_io_error)?;
        if line.trim().is_empty() {
            continue;
        }

        let value: JsonValue = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if value.get("type").and_then(JsonValue::as_str) == Some("session_meta") {
            if let Some(payload) = value.get("payload").and_then(JsonValue::as_object) {
                source = encode_source(payload.get("source"));
                cwd = payload
                    .get("cwd")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                title = payload
                    .get("title")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                sandbox_policy = payload
                    .get("sandbox_policy")
                    .or_else(|| payload.get("sandbox_mode"))
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                approval_mode = payload
                    .get("approval_mode")
                    .or_else(|| payload.get("approval_policy"))
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                cli_version = payload
                    .get("cli_version")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                goal = payload
                    .get("goal")
                    .and_then(JsonValue::as_str)
                    .unwrap_or_default()
                    .to_string();
                memory_mode = payload
                    .get("memory_mode")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("enabled")
                    .to_string();
                model = payload
                    .get("model")
                    .and_then(JsonValue::as_str)
                    .map(str::to_string);
                reasoning_effort = payload
                    .get("reasoning_effort")
                    .or_else(|| payload.get("model_reasoning_effort"))
                    .and_then(JsonValue::as_str)
                    .map(str::to_string);

                if let Some(thread_spawn) = payload
                    .get("source")
                    .and_then(|value| value.get("subagent"))
                    .and_then(|value| value.get("thread_spawn"))
                    .and_then(JsonValue::as_object)
                {
                    agent_nickname = thread_spawn
                        .get("agent_nickname")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string);
                    agent_role = thread_spawn
                        .get("agent_role")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string);
                    agent_path = thread_spawn
                        .get("agent_path")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string);
                }
            }
            continue;
        }

        if first_user_message.is_empty() {
            first_user_message = extract_first_user_message(&value).unwrap_or_default();
        }
    }

    if title.is_empty() {
        title = first_user_message.clone();
    }
    let preview = choose_thread_preview(&first_user_message, &title, &goal);

    let (created_at, updated_at) = extract_rollout_time_bounds(&candidate.path)?;

    Ok(RolloutThreadInsert {
        id: candidate.id.clone(),
        rollout_path: candidate.path.display().to_string(),
        created_at,
        updated_at,
        source,
        model_provider: target_provider.to_string(),
        cwd,
        title,
        sandbox_policy,
        approval_mode,
        tokens_used: 0,
        has_user_event: !first_user_message.is_empty(),
        archived: candidate.archived,
        archived_at: candidate.archived.then_some(candidate.updated_at),
        cli_version,
        first_user_message,
        preview,
        agent_nickname,
        agent_role,
        memory_mode,
        model,
        reasoning_effort,
        agent_path,
    })
}

fn extract_rollout_time_bounds(path: &Path) -> Result<(i64, i64)> {
    let file = File::open(path).map_err(map_io_error)?;
    let reader = BufReader::new(file);
    let mut created_at = None;
    let mut updated_at = None;

    for line_result in reader.lines() {
        let line = line_result.map_err(map_io_error)?;
        if line.trim().is_empty() {
            continue;
        }

        let value: JsonValue = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if created_at.is_none() {
            created_at = extract_timestamp_secs(&value);
        }
        updated_at = updated_at.max(extract_timestamp_secs(&value));
    }

    let fallback_ts = fs::metadata(path)?
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0);
    let created_at = created_at.unwrap_or(fallback_ts);
    let updated_at = updated_at.unwrap_or(created_at);
    Ok((created_at, updated_at))
}

fn encode_source(value: Option<&JsonValue>) -> String {
    match value {
        Some(JsonValue::String(text)) => text.clone(),
        Some(other) => serde_json::to_string(other).unwrap_or_default(),
        None => String::new(),
    }
}

fn extract_timestamp_secs(value: &JsonValue) -> Option<i64> {
    value
        .get("timestamp")
        .and_then(JsonValue::as_str)
        .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok())
        .map(|ts| ts.timestamp())
}

fn extract_first_user_message(value: &JsonValue) -> Option<String> {
    let event_type = value.get("type").and_then(JsonValue::as_str);
    let payload = value.get("payload");

    if event_type == Some("event_msg")
        && payload
            .and_then(|p| p.get("type"))
            .and_then(JsonValue::as_str)
            == Some("user_message")
    {
        return payload
            .and_then(|p| p.get("message"))
            .and_then(JsonValue::as_str)
            .map(str::to_string);
    }

    if event_type == Some("response_item")
        && payload
            .and_then(|p| p.get("type"))
            .and_then(JsonValue::as_str)
            == Some("message")
        && payload
            .and_then(|p| p.get("role"))
            .and_then(JsonValue::as_str)
            == Some("user")
        && let Some(content) = payload
            .and_then(|p| p.get("content"))
            .and_then(JsonValue::as_array)
    {
        let mut parts = Vec::new();
        for item in content {
            if let Some(text) = item
                .get("text")
                .and_then(JsonValue::as_str)
                .or_else(|| item.get("input_text").and_then(JsonValue::as_str))
            {
                parts.push(text.to_string());
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }

    None
}

fn choose_thread_preview(first_user_message: &str, title: &str, goal: &str) -> String {
    const PREVIEW_MAX_CHARS: usize = 240;
    for value in [first_user_message, title, goal] {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return truncate_chars(trimmed, PREVIEW_MAX_CHARS);
        }
    }
    String::new()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn increment_provider_count(
    buckets: &mut CodexHistoryProviderBuckets,
    scope: SessionScope,
    provider: &str,
) {
    let bucket = match scope {
        SessionScope::Sessions => &mut buckets.sessions,
        SessionScope::ArchivedSessions => &mut buckets.archived_sessions,
    };
    *bucket.entry(provider.to_string()).or_insert(0) += 1;
}

fn rewrite_rollout_first_line(change: &SessionChange) -> Result<Option<AppliedSessionBackupEntry>> {
    #[cfg(windows)]
    {
        rewrite_rollout_first_line_windows(change)
    }
    #[cfg(not(windows))]
    {
        rewrite_rollout_first_line_portable(change)
    }
}

#[cfg(windows)]
fn rewrite_rollout_first_line_windows(
    change: &SessionChange,
) -> Result<Option<AppliedSessionBackupEntry>> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&change.path)
        .map_err(map_io_error)?;

    let snapshot = file.metadata().map_err(map_io_error)?;
    let modified_ms = snapshot
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis())
        .unwrap_or(0);
    if snapshot.len() != change.original_size || modified_ms != change.original_modified_ms {
        return Ok(None);
    }

    let record = read_first_line_from_handle(&mut file)?;
    if record.first_line != change.original_first_line || record.offset != change.original_offset {
        return Ok(None);
    }

    let mut rest = Vec::new();
    file.seek(SeekFrom::Start(change.original_offset))
        .map_err(map_io_error)?;
    file.read_to_end(&mut rest).map_err(map_io_error)?;

    file.set_len(0).map_err(map_io_error)?;
    file.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
    file.write_all(change.updated_first_line.as_bytes())
        .map_err(map_io_error)?;
    if !change.original_separator.is_empty() {
        file.write_all(change.original_separator.as_bytes())
            .map_err(map_io_error)?;
    }
    file.write_all(&rest).map_err(map_io_error)?;
    file.flush().map_err(map_io_error)?;
    drop(file);
    restore_file_mtime(&change.path, change.original_modified_time)?;

    Ok(Some(AppliedSessionBackupEntry {
        path: change.path.clone(),
        original_first_line: change.original_first_line.clone(),
        original_separator: change.original_separator.clone(),
    }))
}

fn restore_rollout_first_line(entry: &AppliedSessionBackupEntry) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&entry.path)
            .map_err(map_io_error)?;
        let record = read_first_line_from_handle(&mut file)?;
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(record.offset))
            .map_err(map_io_error)?;
        file.read_to_end(&mut rest).map_err(map_io_error)?;
        file.set_len(0).map_err(map_io_error)?;
        file.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
        file.write_all(entry.original_first_line.as_bytes())
            .map_err(map_io_error)?;
        if !entry.original_separator.is_empty() {
            file.write_all(entry.original_separator.as_bytes())
                .map_err(map_io_error)?;
        }
        file.write_all(&rest).map_err(map_io_error)?;
        file.flush().map_err(map_io_error)?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let mut file = File::open(&entry.path).map_err(map_io_error)?;
        let record = read_first_line_from_handle(&mut file)?;
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(record.offset))
            .map_err(map_io_error)?;
        file.read_to_end(&mut rest).map_err(map_io_error)?;

        let parent = entry
            .path
            .parent()
            .ok_or_else(|| CcrError::FileIoError("无法解析 rollout 目录".into()))?;
        let mut temp = NamedTempFile::new_in(parent)
            .map_err(|err| CcrError::FileIoError(format!("创建 rollout 临时文件失败: {}", err)))?;
        temp.write_all(entry.original_first_line.as_bytes())
            .map_err(map_io_error)?;
        if !entry.original_separator.is_empty() {
            temp.write_all(entry.original_separator.as_bytes())
                .map_err(map_io_error)?;
        }
        temp.write_all(&rest).map_err(map_io_error)?;
        temp.flush().map_err(map_io_error)?;
        temp.persist(&entry.path)
            .map_err(|err| CcrError::FileIoError(format!("恢复 rollout 文件失败: {}", err)))?;
        Ok(())
    }
}

fn read_first_line_from_handle(file: &mut File) -> Result<FirstLineRecord> {
    file.seek(SeekFrom::Start(0)).map_err(map_io_error)?;
    let mut collected = Vec::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(map_io_error)?;
        if bytes_read == 0 {
            break;
        }
        collected.extend_from_slice(&buffer[..bytes_read]);
        if let Some(index) = collected.iter().position(|byte| *byte == b'\n') {
            let offset = (index + 1) as u64;
            let separator = if index > 0 && collected[index - 1] == b'\r' {
                collected.truncate(index - 1);
                "\r\n".to_string()
            } else {
                collected.truncate(index);
                "\n".to_string()
            };
            let first_line = String::from_utf8(collected)
                .map_err(|err| CcrError::FileIoError(format!("读取 rollout 首行失败: {}", err)))?;
            return Ok(FirstLineRecord {
                first_line,
                separator,
                offset,
            });
        }
    }

    let offset = collected.len() as u64;
    let first_line = String::from_utf8(collected)
        .map_err(|err| CcrError::FileIoError(format!("读取 rollout 首行失败: {}", err)))?;
    Ok(FirstLineRecord {
        first_line,
        separator: String::new(),
        offset,
    })
}

fn map_io_error(err: std::io::Error) -> CcrError {
    if is_locked_file_io_error(&err) {
        return CcrError::FileLockError(format!("rollout 文件正在被占用: {}", err));
    }
    CcrError::IoError(err)
}

fn restore_file_mtime(path: &Path, modified_time: FileTime) -> Result<()> {
    filetime::set_file_mtime(path, modified_time)
        .map_err(|err| CcrError::FileIoError(format!("restore rollout mtime failed: {}", err)))
}

fn is_locked_file_io_error(err: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        matches!(err.raw_os_error(), Some(5 | 32 | 33))
    }

    #[cfg(not(windows))]
    {
        matches!(err.kind(), std::io::ErrorKind::WouldBlock)
    }
}

fn is_locked_error(err: &CcrError) -> bool {
    matches!(err, CcrError::FileLockError(_))
}

fn map_sqlite_busy_error(err: rusqlite::Error) -> CcrError {
    match err {
        rusqlite::Error::SqliteFailure(inner, _) => {
            use rusqlite::ErrorCode;
            if matches!(
                inner.code,
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
            ) {
                return CcrError::DatabaseError(
                    "state_5.sqlite 当前被占用，请关闭 Codex / Codex App 后重试".to_string(),
                );
            }
            CcrError::DatabaseError(inner.to_string())
        }
        other => CcrError::DatabaseError(other.to_string()),
    }
}

fn normalize_bridge_option(bridge: Option<&str>) -> Result<Option<String>> {
    let Some(bridge) = bridge.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if bridge != BRIDGE_OFFICIAL_CUSTOM {
        return Err(CcrError::ValidationError(format!(
            "unsupported --bridge value '{bridge}'; supported value: {BRIDGE_OFFICIAL_CUSTOM}"
        )));
    }
    Ok(Some(bridge.to_string()))
}

fn canonical_runtime_provider(current: &CurrentProvider) -> String {
    if current.provider.trim() == DEFAULT_PROVIDER {
        DEFAULT_PROVIDER.to_string()
    } else {
        THIRD_PARTY_PROVIDER.to_string()
    }
}

fn normalize_include_providers(providers: &[String]) -> BTreeSet<String> {
    providers
        .iter()
        .filter_map(|provider| {
            let trimmed = provider.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "(missing)" {
                Some(String::new())
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

fn bridge_provider_keys(extra_provider_keys: &BTreeSet<String>) -> BTreeSet<String> {
    let mut providers = BTreeSet::from([
        DEFAULT_PROVIDER.to_string(),
        THIRD_PARTY_PROVIDER.to_string(),
        String::new(),
    ]);
    providers.extend(extra_provider_keys.iter().cloned());
    providers
}

fn provider_display_name(provider_key: &str) -> String {
    if provider_key.trim().is_empty() {
        "(missing)".to_string()
    } else {
        provider_key.to_string()
    }
}

fn rollout_has_encrypted_content(path: &Path) -> Result<bool> {
    let file = File::open(path).map_err(map_io_error)?;
    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        let line = line_result.map_err(map_io_error)?;
        if !line.contains("encrypted_content") {
            continue;
        }
        match serde_json::from_str::<JsonValue>(&line) {
            Ok(value) if json_contains_key(&value, "encrypted_content") => return Ok(true),
            Ok(_) => {}
            Err(_) => return Ok(true),
        }
    }
    Ok(false)
}

fn json_contains_key(value: &JsonValue, needle: &str) -> bool {
    match value {
        JsonValue::Object(map) => {
            map.contains_key(needle) || map.values().any(|value| json_contains_key(value, needle))
        }
        JsonValue::Array(values) => values.iter().any(|value| json_contains_key(value, needle)),
        _ => false,
    }
}

fn compute_dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in WalkDir::new(path) {
        let entry = entry.map_err(|err| CcrError::IoError(err.into()))?;
        if entry.file_type().is_file() {
            total += entry
                .metadata()
                .map_err(|err| CcrError::IoError(err.into()))?
                .len();
        }
    }
    Ok(total)
}

fn normalize_workspace_root_path(workspace_root: &str) -> Option<String> {
    let mut normalized = workspace_root.trim().replace('/', "\\");
    if normalized.is_empty() {
        return None;
    }

    if let Some(stripped) = normalized.strip_prefix(r"\\?\") {
        normalized = stripped.to_string();
    }

    let mut collapsed = String::with_capacity(normalized.len());
    let mut previous_was_slash = false;
    for ch in normalized.chars() {
        let is_slash = ch == '\\';
        if is_slash && previous_was_slash {
            continue;
        }
        previous_was_slash = is_slash;
        collapsed.push(ch);
    }
    normalized = collapsed;

    if normalized.len() > 3 {
        while normalized.ends_with('\\') {
            normalized.pop();
        }
    }

    if normalized.len() >= 2 {
        let bytes = normalized.as_bytes();
        if bytes[1] == b':' && bytes[0].is_ascii_lowercase() {
            normalized.replace_range(0..1, &normalized[0..1].to_ascii_uppercase());
        }
    }

    Some(normalized)
}

fn workspace_root_key(workspace_root: &str) -> Option<String> {
    normalize_workspace_root_path(workspace_root).map(|value| value.to_ascii_lowercase())
}

fn compare_workspace_roots(left: &str, right: &str) -> std::cmp::Ordering {
    let left_key = workspace_root_key(left).unwrap_or_default();
    let right_key = workspace_root_key(right).unwrap_or_default();
    left_key.cmp(&right_key).then_with(|| left.cmp(right))
}

fn is_codex_worktree_path(workspace_root_key_value: &str, codex_home_key: Option<&str>) -> bool {
    if let Some(codex_home_key) = codex_home_key {
        let prefix = format!(r"{}\worktrees\", codex_home_key);
        if workspace_root_key_value.starts_with(&prefix) {
            return true;
        }
    }
    workspace_root_key_value.contains(r"\.codex\worktrees\")
}

fn collect_sidebar_project_candidates(
    workspace_roots: &[String],
    codex_home: &Path,
) -> Vec<String> {
    let codex_home_key = normalize_workspace_root_path(&codex_home.display().to_string())
        .and_then(|value| workspace_root_key(&value));
    let mut unique_paths = BTreeMap::<String, String>::new();

    for workspace_root in workspace_roots {
        let Some(normalized) = normalize_workspace_root_path(workspace_root) else {
            continue;
        };
        let Some(key) = workspace_root_key(&normalized) else {
            continue;
        };
        if codex_home_key.as_deref() == Some(key.as_str()) {
            continue;
        }
        if is_codex_worktree_path(&key, codex_home_key.as_deref()) {
            continue;
        }
        unique_paths.entry(key).or_insert(normalized);
    }

    let mut values = unique_paths.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| compare_workspace_roots(left, right));
    values
}

fn collect_known_sidebar_projects(
    global_state: Option<&JsonMap<String, JsonValue>>,
    codex_home: &Path,
) -> Vec<String> {
    let Some(global_state) = global_state else {
        return Vec::new();
    };

    let mut workspace_roots = Vec::new();
    for key in [
        "electron-saved-workspace-roots",
        "project-order",
        "active-workspace-roots",
    ] {
        if let Some(values) = global_state.get(key).and_then(JsonValue::as_array) {
            workspace_roots.extend(
                values
                    .iter()
                    .filter_map(JsonValue::as_str)
                    .map(str::to_string),
            );
        }
    }

    if let Some(hints) = global_state
        .get("thread-workspace-root-hints")
        .and_then(JsonValue::as_object)
    {
        workspace_roots.extend(
            hints
                .values()
                .filter_map(JsonValue::as_str)
                .map(str::to_string),
        );
    }

    collect_sidebar_project_candidates(&workspace_roots, codex_home)
}

#[cfg(not(windows))]
fn rewrite_rollout_first_line_portable(
    change: &SessionChange,
) -> Result<Option<AppliedSessionBackupEntry>> {
    let metadata = fs::metadata(&change.path)?;
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis())
        .unwrap_or(0);
    if metadata.len() != change.original_size || modified_ms != change.original_modified_ms {
        return Ok(None);
    }

    let mut file = File::open(&change.path).map_err(map_io_error)?;
    let record = read_first_line_from_handle(&mut file)?;
    if record.first_line != change.original_first_line || record.offset != change.original_offset {
        return Ok(None);
    }

    let mut rest = Vec::new();
    file.seek(SeekFrom::Start(change.original_offset))
        .map_err(map_io_error)?;
    file.read_to_end(&mut rest).map_err(map_io_error)?;

    let parent = change
        .path
        .parent()
        .ok_or_else(|| CcrError::FileIoError("无法解析 rollout 目录".into()))?;
    let mut temp = NamedTempFile::new_in(parent)
        .map_err(|err| CcrError::FileIoError(format!("创建 rollout 临时文件失败: {}", err)))?;
    temp.write_all(change.updated_first_line.as_bytes())
        .map_err(map_io_error)?;
    if !change.original_separator.is_empty() {
        temp.write_all(change.original_separator.as_bytes())
            .map_err(map_io_error)?;
    }
    temp.write_all(&rest).map_err(map_io_error)?;
    temp.flush().map_err(map_io_error)?;
    restore_file_mtime(temp.path(), change.original_modified_time)?;
    temp.persist(&change.path)
        .map_err(|err| CcrError::FileIoError(format!("替换 rollout 文件失败: {}", err)))?;

    Ok(Some(AppliedSessionBackupEntry {
        path: change.path.clone(),
        original_first_line: change.original_first_line.clone(),
        original_separator: change.original_separator.clone(),
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use filetime::FileTime;
    use rusqlite::params;
    use tempfile::tempdir;

    fn create_service(temp: &Path) -> CodexHistorySyncService {
        let codex_home = temp.join(".codex");
        fs::create_dir_all(codex_home.join("sessions/2026/04/09")).unwrap();
        fs::create_dir_all(codex_home.join("archived_sessions/2026/04/08")).unwrap();
        CodexHistorySyncService::with_codex_home(Some(codex_home)).unwrap()
    }

    fn write_config(codex_home: &Path, provider: Option<&str>) {
        let content = provider
            .map(|provider| format!("model_provider = \"{}\"\n", provider))
            .unwrap_or_default();
        fs::write(codex_home.join("config.toml"), content).unwrap();
    }

    fn write_rollout_with_timestamps(
        path: &Path,
        id: &str,
        provider: &str,
        session_ts: &str,
        user_ts: &str,
        cwd: &str,
    ) {
        let content = format!(
            "{{\"timestamp\":\"{}\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{}\",\"cwd\":\"{}\",\"model_provider\":\"{}\"}}}}\n{{\"timestamp\":\"{}\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello\"}}}}\n",
            session_ts,
            id,
            cwd.replace('\\', "\\\\"),
            provider,
            user_ts
        );
        fs::write(path, content).unwrap();
    }

    fn format_test_timestamp(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%Y-%m-%dT%H:%M:%S.000Z").to_string()
    }

    fn write_rollout(path: &Path, id: &str, provider: &str) {
        let session_at = Utc::now() - chrono::Duration::days(2);
        write_rollout_with_timestamps(
            path,
            id,
            provider,
            &format_test_timestamp(session_at),
            &format_test_timestamp(session_at + chrono::Duration::seconds(1)),
            r"E:\Repo",
        );
    }

    fn write_rollout_without_timestamps(path: &Path, id: &str, provider: &str, cwd: &str) {
        let content = format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{}\",\"cwd\":\"{}\",\"model_provider\":\"{}\"}}}}\n{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello\"}}}}\n",
            id,
            cwd.replace('\\', "\\\\"),
            provider,
        );
        fs::write(path, content).unwrap();
    }

    fn write_global_state(codex_home: &Path, content: &str) {
        fs::write(codex_home.join(GLOBAL_STATE_FILE_NAME), content).unwrap();
    }

    fn write_session_index(codex_home: &Path, content: &str) {
        fs::write(codex_home.join(SESSION_INDEX_FILE), content).unwrap();
    }

    fn write_state_db(codex_home: &Path, rows: &[(&str, &str, bool, &str)]) {
        let conn = Connection::open(codex_home.join(SQLITE_FILE_NAME)).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                model_provider TEXT,
                archived INTEGER NOT NULL DEFAULT 0,
                cwd TEXT,
                first_user_message TEXT NOT NULL DEFAULT ''
            );
            "#,
        )
        .unwrap();
        for (id, provider, archived, cwd) in rows {
            conn.execute(
                "INSERT INTO threads (id, model_provider, archived, cwd, first_user_message) VALUES (?1, ?2, ?3, ?4, '')",
                params![id, provider, if *archived { 1 } else { 0 }, cwd],
            )
            .unwrap();
        }
    }

    fn write_state_db_with_visibility_columns(
        codex_home: &Path,
        rows: &[(&str, &str, bool, &str, &str, bool, i64)],
    ) {
        let conn = Connection::open(codex_home.join(SQLITE_FILE_NAME)).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                rollout_path TEXT,
                model_provider TEXT,
                archived INTEGER NOT NULL DEFAULT 0,
                cwd TEXT,
                preview TEXT NOT NULL DEFAULT '',
                first_user_message TEXT NOT NULL DEFAULT '',
                has_user_event INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .unwrap();
        for (id, provider, archived, cwd, preview, has_user_event, updated_at) in rows {
            conn.execute(
                "INSERT INTO threads (id, model_provider, archived, cwd, preview, has_user_event, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    provider,
                    if *archived { 1 } else { 0 },
                    cwd,
                    preview,
                    if *has_user_event { 1 } else { 0 },
                    updated_at
                ],
            )
            .unwrap();
        }
    }

    fn write_rollout_with_encrypted_content(path: &Path, id: &str, provider: &str) {
        let session_at = Utc::now() - chrono::Duration::days(1);
        let content = format!(
            "{{\"timestamp\":\"{}\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{}\",\"cwd\":\"E:\\\\Repo\",\"model_provider\":\"{}\"}}}}\n{{\"timestamp\":\"{}\",\"type\":\"response_item\",\"payload\":{{\"encrypted_content\":\"ciphertext\"}}}}\n{{\"timestamp\":\"{}\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello\"}}}}\n",
            format_test_timestamp(session_at),
            id,
            provider,
            format_test_timestamp(session_at + chrono::Duration::seconds(1)),
            format_test_timestamp(session_at + chrono::Duration::seconds(2)),
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn status_uses_implicit_openai_when_root_provider_missing() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, None);
        write_rollout(
            &service
                .codex_home
                .join("sessions/2026/04/09/rollout-a.jsonl"),
            "thread-a",
            "custom",
        );
        write_state_db(
            &service.codex_home,
            &[("thread-a", "custom", false, r"E:\Repo")],
        );

        let status = service.status().unwrap();
        assert_eq!(status.current_provider, "openai");
        assert!(status.current_provider_implicit);
        assert_eq!(status.rollout_counts.sessions.get("custom"), Some(&1));
        assert_eq!(
            status
                .sqlite_counts
                .as_ref()
                .unwrap()
                .sessions
                .get("custom"),
            Some(&1)
        );
    }

    #[test]
    fn sync_defaults_to_bridge_when_provider_implicit_without_args() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, None);
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-implicit-default.jsonl");
        write_rollout(&session_path, "thread-implicit-default", "custom");
        write_state_db(
            &service.codex_home,
            &[("thread-implicit-default", "custom", false, r"E:\Repo")],
        );

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();

        assert_eq!(result.bridge.as_deref(), Some(BRIDGE_OFFICIAL_CUSTOM));
        assert_eq!(result.target_provider, "openai");
        assert_eq!(result.changed_rollout_files, 1);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"openai""#));
    }

    #[test]
    fn sync_accepts_explicit_custom_when_current_provider_is_implicit() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, None);
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-a.jsonl");
        write_rollout(&session_path, "thread-a", "openai");
        write_state_db(
            &service.codex_home,
            &[("thread-a", "openai", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                provider: Some("custom".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.target_provider, "custom");
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);

        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let provider: String = conn
            .query_row(
                "SELECT model_provider FROM threads WHERE id = 'thread-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(provider, "custom");
    }

    #[test]
    fn bridge_official_custom_targets_implicit_openai_without_provider() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, None);
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-bridge-openai.jsonl");
        write_rollout(&session_path, "thread-bridge-openai", "custom");
        write_state_db(
            &service.codex_home,
            &[("thread-bridge-openai", "custom", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                bridge: Some(BRIDGE_OFFICIAL_CUSTOM.to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(result.target_provider, "openai");
        assert_eq!(result.bridge.as_deref(), Some(BRIDGE_OFFICIAL_CUSTOM));
        assert_eq!(result.changed_rollout_files, 1);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"openai""#));
    }

    #[test]
    fn bridge_official_custom_targets_custom_for_third_party_runtime() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("custom"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-bridge-custom.jsonl");
        write_rollout(&session_path, "thread-bridge-custom", "openai");
        write_state_db(
            &service.codex_home,
            &[("thread-bridge-custom", "openai", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                bridge: Some(BRIDGE_OFFICIAL_CUSTOM.to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(result.target_provider, "custom");
        assert_eq!(result.changed_rollout_files, 1);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));
    }

    #[test]
    fn status_reports_missing_session_index_entries() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-missing.jsonl");
        write_rollout(&session_path, "thread-index-missing", "openai");

        let status = service.status().unwrap();

        assert!(!status.session_index_present);
        assert_eq!(status.missing_session_index_entries, 1);
    }

    #[test]
    fn status_skips_existing_session_index_entry() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-existing.jsonl");
        write_rollout(&session_path, "thread-index-existing", "openai");
        write_session_index(
            &service.codex_home,
            "{\"id\":\"thread-index-existing\",\"thread_name\":\"Existing\",\"updated_at\":\"2026-04-09T00:00:00.000000Z\"}\n",
        );

        let status = service.status().unwrap();

        assert!(status.session_index_present);
        assert_eq!(status.missing_session_index_entries, 0);
    }

    #[test]
    fn sync_dry_run_reports_session_index_without_writing() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-dry-run.jsonl");
        write_rollout(&session_path, "thread-index-dry-run", "openai");

        let result = service
            .sync(CodexHistorySyncOptions {
                dry_run: true,
                ..Default::default()
            })
            .unwrap();

        assert!(!result.session_index_present);
        assert_eq!(result.missing_session_index_entries, 1);
        assert_eq!(result.added_session_index_entries, 0);
        assert!(!service.codex_home.join(SESSION_INDEX_FILE).exists());
        assert!(result.backup_dir.is_none());
    }

    #[test]
    fn sync_appends_missing_session_index_entries() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-write.jsonl");
        write_rollout(&session_path, "thread-index-write", "openai");

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();

        assert_eq!(result.missing_session_index_entries, 1);
        assert_eq!(result.added_session_index_entries, 1);
        let index = fs::read_to_string(service.codex_home.join(SESSION_INDEX_FILE)).unwrap();
        assert!(index.contains(r#""id":"thread-index-write""#));
        assert!(index.contains(r#""thread_name":"hello""#));
        assert!(index.contains(r#""updated_at":"#));
    }

    #[test]
    fn sync_appends_missing_session_index_entries_from_sqlite_threads() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_state_db_with_visibility_columns(
            &service.codex_home,
            &[(
                "thread-index-sqlite",
                "openai",
                false,
                r"E:\Repo",
                "SQLite preview",
                true,
                1_775_000_000,
            )],
        );

        let status = service.status().unwrap();
        assert_eq!(status.missing_session_index_entries, 1);

        let dry_run = service
            .sync(CodexHistorySyncOptions {
                dry_run: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(dry_run.missing_session_index_entries, 1);
        assert!(!service.codex_home.join(SESSION_INDEX_FILE).exists());

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();

        assert_eq!(result.added_session_index_entries, 1);
        let index = fs::read_to_string(service.codex_home.join(SESSION_INDEX_FILE)).unwrap();
        assert!(index.contains(r#""id":"thread-index-sqlite""#));
        assert!(index.contains(r#""thread_name":"SQLite preview""#));
    }

    #[test]
    fn restore_removes_session_index_created_by_sync() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-restore-created.jsonl");
        write_rollout(&session_path, "thread-index-restore-created", "openai");

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert!(service.codex_home.join(SESSION_INDEX_FILE).exists());

        let restore = service
            .restore(result.backup_dir.as_ref().unwrap())
            .unwrap();

        assert!(restore.restored_session_index);
        assert!(!service.codex_home.join(SESSION_INDEX_FILE).exists());
    }

    #[test]
    fn restore_reverts_existing_session_index_content() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let original_index = "{\"id\":\"thread-index-kept\",\"thread_name\":\"Kept\",\"updated_at\":\"2026-04-09T00:00:00.000000Z\"}\n";
        write_session_index(&service.codex_home, original_index);
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-restore-existing.jsonl");
        write_rollout(&session_path, "thread-index-restore-existing", "openai");

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.added_session_index_entries, 1);

        let restore = service
            .restore(result.backup_dir.as_ref().unwrap())
            .unwrap();

        assert!(restore.restored_session_index);
        assert_eq!(
            fs::read_to_string(service.codex_home.join(SESSION_INDEX_FILE)).unwrap(),
            original_index
        );
    }

    #[test]
    fn bridge_skips_unknown_rollout_provider_without_include_provider() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-unknown.jsonl");
        write_rollout(&session_path, "thread-unknown", "duckcoding");
        write_state_db(
            &service.codex_home,
            &[("thread-unknown", "duckcoding", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                bridge: Some(BRIDGE_OFFICIAL_CUSTOM.to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(result.changed_rollout_files, 0);
        assert_eq!(result.sqlite_rows_updated, 0);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"duckcoding""#));
    }

    #[test]
    fn bridge_skips_unknown_missing_sqlite_row_without_include_provider() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-unknown-missing-db.jsonl");
        write_rollout(&session_path, "thread-unknown-missing-db", "duckcoding");
        write_state_db(&service.codex_home, &[]);

        let result = service
            .sync(CodexHistorySyncOptions {
                bridge: Some(BRIDGE_OFFICIAL_CUSTOM.to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(result.changed_rollout_files, 0);
        assert_eq!(result.sqlite_rows_updated, 0);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"duckcoding""#));
        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM threads", [], |row| row.get(0))
            .unwrap();
        assert_eq!(row_count, 0);
    }

    #[test]
    fn bridge_include_provider_allows_unknown_rollout_and_sqlite_provider() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-include-provider.jsonl");
        write_rollout(&session_path, "thread-include-provider", "duckcoding");
        write_state_db(
            &service.codex_home,
            &[("thread-include-provider", "duckcoding", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                bridge: Some(BRIDGE_OFFICIAL_CUSTOM.to_string()),
                include_providers: vec!["duckcoding".to_string()],
                ..Default::default()
            })
            .unwrap();

        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"openai""#));
    }

    #[test]
    fn sync_dry_run_previews_without_writing_state() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        let original_state = r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{"thread-a":"E:\\Repo"}}"#;
        write_global_state(&service.codex_home, original_state);
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-a.jsonl");
        write_rollout(&session_path, "thread-a", "custom");
        write_state_db(
            &service.codex_home,
            &[("thread-a", "custom", false, r"E:\Repo")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                dry_run: true,
                ..Default::default()
            })
            .unwrap();
        assert!(result.dry_run);
        assert!(result.backup_dir.is_none());
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);
        assert_eq!(result.added_sidebar_projects, 1);
        assert_eq!(result.rollout_counts.sessions.get("custom"), Some(&1));
        assert_eq!(
            result
                .sqlite_counts
                .as_ref()
                .unwrap()
                .sessions
                .get("custom"),
            Some(&1)
        );
        assert!(!service.backup_root().exists());

        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));
        let global_state =
            fs::read_to_string(service.codex_home.join(GLOBAL_STATE_FILE_NAME)).unwrap();
        assert_eq!(global_state, original_state);

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let provider: String = conn
            .query_row(
                "SELECT model_provider FROM threads WHERE id = 'thread-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(provider, "custom");
    }

    #[test]
    fn sync_rewrites_rollout_sqlite_and_sidebar_then_restore_reverts() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":["E:\\Existing"],"project-order":["E:\\Existing"],"thread-workspace-root-hints":{"alpha":"E:\\Repo"}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-a.jsonl");
        write_rollout(&session_path, "thread-a", "custom");
        write_state_db(
            &service.codex_home,
            &[("thread-a", "custom", false, r"E:\Repo")],
        );

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.target_provider, "openai");
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.added_sidebar_projects, 1);
        assert_eq!(result.sqlite_rows_updated, 1);

        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"openai""#));

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let provider: String = conn
            .query_row(
                "SELECT model_provider FROM threads WHERE id = 'thread-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(provider, "openai");

        let global_state =
            fs::read_to_string(service.codex_home.join(GLOBAL_STATE_FILE_NAME)).unwrap();
        assert!(global_state.contains(r#"E:\\Repo"#));

        service
            .restore(result.backup_dir.as_ref().unwrap())
            .unwrap();
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));

        let provider: String = conn
            .query_row(
                "SELECT model_provider FROM threads WHERE id = 'thread-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn sync_rolls_back_when_global_state_is_invalid() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(&service.codex_home, "{invalid json");
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-a.jsonl");
        write_rollout(&session_path, "thread-a", "custom");
        write_state_db(
            &service.codex_home,
            &[("thread-a", "custom", false, r"E:\Broken")],
        );

        let err = service
            .sync(CodexHistorySyncOptions::default())
            .unwrap_err();
        assert!(err.to_string().contains(GLOBAL_STATE_FILE_NAME));

        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));
    }

    #[test]
    fn sync_backfills_missing_sqlite_threads_from_rollouts() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":["E:\\Repo"],"project-order":["E:\\Repo"],"thread-workspace-root-hints":{"alpha":"E:\\Repo"}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-a.jsonl");
        write_rollout(&session_path, "thread-a", "custom");
        write_state_db(&service.codex_home, &[]);

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.sqlite_rows_updated, 1);

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let row: (String, String) = conn
            .query_row(
                "SELECT model_provider, cwd FROM threads WHERE id = 'thread-a'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(row.0, "openai");
        assert_eq!(row.1, r"E:\Repo");
    }

    #[test]
    fn sync_backfills_missing_sqlite_threads_with_preview_when_column_exists() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":["E:\\Repo"],"project-order":["E:\\Repo"],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-preview-insert.jsonl");
        write_rollout(&session_path, "thread-preview-insert", "custom");
        write_state_db_with_visibility_columns(&service.codex_home, &[]);

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.sqlite_rows_updated, 1);

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let row: (String, i64, String) = conn
            .query_row(
                "SELECT preview, has_user_event, first_user_message FROM threads WHERE id = 'thread-preview-insert'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(row.0, "hello");
        assert_eq!(row.1, 1);
        assert_eq!(row.2, "hello");
    }

    #[test]
    fn sync_repairs_existing_sqlite_preview_has_user_event_and_cwd() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":["E:\\Repo"],"project-order":["E:\\Repo"],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-preview-repair.jsonl");
        write_rollout(&session_path, "thread-preview-repair", "custom");
        write_state_db_with_visibility_columns(
            &service.codex_home,
            &[(
                "thread-preview-repair",
                "custom",
                false,
                r"\\?\E:\Repo",
                "",
                false,
                1,
            )],
        );

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.sqlite_rows_updated, 1);

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let row: (String, i64, String, String) = conn
            .query_row(
                "SELECT model_provider, has_user_event, preview, cwd FROM threads WHERE id = 'thread-preview-repair'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(row.0, "openai");
        assert_eq!(row.1, 1);
        assert_eq!(row.2, "hello");
        assert_eq!(row.3, r"E:\Repo");
    }

    #[test]
    fn status_reports_visibility_and_encrypted_diagnostics() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":["E:\\Repo"],"project-order":["E:\\Repo"],"thread-workspace-root-hints":{}}"#,
        );
        let session_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-encrypted.jsonl");
        write_rollout_with_encrypted_content(&session_path, "thread-encrypted", "custom");
        write_state_db_with_visibility_columns(
            &service.codex_home,
            &[(
                "thread-encrypted",
                "custom",
                false,
                r"\\?\E:\Repo",
                "",
                false,
                10,
            )],
        );

        let status = service.status().unwrap();
        assert_eq!(status.encrypted_counts.sessions.get("custom"), Some(&1));
        let visibility = status.visibility.unwrap();
        assert_eq!(visibility.provider_mismatch_rows, 1);
        assert_eq!(visibility.preview_empty_rows, 1);
        assert_eq!(visibility.has_user_event_missing_rows, 1);
        assert_eq!(visibility.normalized_cwd_matches, 1);
        assert_eq!(visibility.verbatim_extended_cwd_rows, 1);
    }

    #[test]
    fn sync_defaults_to_recent_seven_days_only() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );

        let recent_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-recent.jsonl");
        let old_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-old.jsonl");
        let recent_at = Utc::now() - chrono::Duration::days(2);
        let old_at = Utc::now() - chrono::Duration::days(20);
        write_rollout_with_timestamps(
            &recent_path,
            "thread-recent",
            "custom",
            &format_test_timestamp(recent_at),
            &format_test_timestamp(recent_at + chrono::Duration::seconds(1)),
            r"E:\Recent",
        );
        write_rollout_with_timestamps(
            &old_path,
            "thread-old",
            "custom",
            &format_test_timestamp(old_at),
            &format_test_timestamp(old_at + chrono::Duration::seconds(1)),
            r"E:\Old",
        );
        write_state_db(
            &service.codex_home,
            &[
                ("thread-recent", "custom", false, r"E:\Recent"),
                ("thread-old", "custom", false, r"E:\Old"),
            ],
        );

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.added_sidebar_projects, 0);
        assert_eq!(result.sqlite_rows_updated, 1);

        let recent_rollout = fs::read_to_string(&recent_path).unwrap();
        let old_rollout = fs::read_to_string(&old_path).unwrap();
        assert!(recent_rollout.contains(r#""model_provider":"openai""#));
        assert!(old_rollout.contains(r#""model_provider":"custom""#));

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let rows: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, model_provider FROM threads ORDER BY id")
                .unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .map(|row| row.unwrap())
                .collect()
        };
        assert_eq!(
            rows,
            vec![
                ("thread-old".to_string(), "custom".to_string()),
                ("thread-recent".to_string(), "openai".to_string())
            ]
        );

        let global_state =
            fs::read_to_string(service.codex_home.join(GLOBAL_STATE_FILE_NAME)).unwrap();
        assert_eq!(
            global_state,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#
        );
    }

    #[test]
    fn sync_respects_custom_max_age_days() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );

        let old_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-old.jsonl");
        let old_at = Utc::now() - chrono::Duration::days(20);
        write_rollout_with_timestamps(
            &old_path,
            "thread-old",
            "custom",
            &format_test_timestamp(old_at),
            &format_test_timestamp(old_at + chrono::Duration::seconds(1)),
            r"E:\Old",
        );
        write_state_db(
            &service.codex_home,
            &[("thread-old", "custom", false, r"E:\Old")],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                max_age_days: 30,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);

        let old_rollout = fs::read_to_string(&old_path).unwrap();
        assert!(old_rollout.contains(r#""model_provider":"openai""#));
    }

    #[test]
    fn sync_all_history_covers_old_rollouts_without_touching_unknown_sqlite_rows() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );

        let old_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-old-all-history.jsonl");
        let old_at = Utc::now() - chrono::Duration::days(20);
        write_rollout_with_timestamps(
            &old_path,
            "thread-old-all-history",
            "custom",
            &format_test_timestamp(old_at),
            &format_test_timestamp(old_at + chrono::Duration::seconds(1)),
            r"E:\Old",
        );
        let unknown_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-old-unknown-all-history.jsonl");
        write_rollout_with_timestamps(
            &unknown_path,
            "thread-old-unknown-all-history",
            "duckcoding",
            &format_test_timestamp(old_at),
            &format_test_timestamp(old_at + chrono::Duration::seconds(1)),
            r"E:\Unknown",
        );
        write_state_db(
            &service.codex_home,
            &[
                ("thread-old-all-history", "custom", false, r"E:\Old"),
                ("thread-unknown-db", "duckcoding", false, r"E:\Unknown"),
            ],
        );

        let result = service
            .sync(CodexHistorySyncOptions {
                all_history: true,
                ..Default::default()
            })
            .unwrap();
        assert!(result.all_history);
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);

        let conn = Connection::open(service.codex_home.join(SQLITE_FILE_NAME)).unwrap();
        let unknown_rollout = fs::read_to_string(&unknown_path).unwrap();
        assert!(unknown_rollout.contains(r#""model_provider":"duckcoding""#));
        let rows: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, model_provider FROM threads ORDER BY id")
                .unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .map(|row| row.unwrap())
                .collect()
        };
        assert_eq!(
            rows,
            vec![
                ("thread-old-all-history".to_string(), "openai".to_string()),
                ("thread-unknown-db".to_string(), "duckcoding".to_string()),
            ]
        );
    }

    #[test]
    fn sync_rejects_zero_max_age_days() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));

        let err = service
            .sync(CodexHistorySyncOptions {
                max_age_days: 0,
                ..Default::default()
            })
            .unwrap_err();
        assert!(err.to_string().contains("--max-age-days"));
    }

    #[test]
    fn sync_uses_mtime_when_rollout_has_no_timestamps() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        write_config(&service.codex_home, Some("openai"));
        write_global_state(
            &service.codex_home,
            r#"{"electron-saved-workspace-roots":[],"project-order":[],"thread-workspace-root-hints":{}}"#,
        );

        let recent_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-recent-mtime.jsonl");
        let old_path = service
            .codex_home
            .join("sessions/2026/04/09/rollout-old-mtime.jsonl");
        write_rollout_without_timestamps(
            &recent_path,
            "thread-recent-mtime",
            "custom",
            r"E:\RecentMtime",
        );
        write_rollout_without_timestamps(&old_path, "thread-old-mtime", "custom", r"E:\OldMtime");
        let recent_mtime = FileTime::from_unix_time(Utc::now().timestamp() - 2 * 24 * 3600, 0);
        filetime::set_file_mtime(&recent_path, recent_mtime).unwrap();
        filetime::set_file_mtime(
            &old_path,
            FileTime::from_unix_time(Utc::now().timestamp() - 20 * 24 * 3600, 0),
        )
        .unwrap();
        write_state_db(
            &service.codex_home,
            &[
                ("thread-recent-mtime", "custom", false, r"E:\RecentMtime"),
                ("thread-old-mtime", "custom", false, r"E:\OldMtime"),
            ],
        );

        let result = service.sync(CodexHistorySyncOptions::default()).unwrap();
        assert_eq!(result.changed_rollout_files, 1);
        assert_eq!(result.sqlite_rows_updated, 1);

        let recent_rollout = fs::read_to_string(&recent_path).unwrap();
        let old_rollout = fs::read_to_string(&old_path).unwrap();
        assert!(recent_rollout.contains(r#""model_provider":"openai""#));
        assert!(old_rollout.contains(r#""model_provider":"custom""#));
        let actual_mtime =
            FileTime::from_last_modification_time(&fs::metadata(&recent_path).unwrap());
        assert_eq!(actual_mtime, recent_mtime);
    }

    #[test]
    fn prune_backups_keeps_latest_managed_entries() {
        let dir = tempdir().unwrap();
        let service = create_service(dir.path());
        let backup_root = service.backup_root();
        fs::create_dir_all(&backup_root).unwrap();

        for name in [
            "20260409T000001000Z",
            "20260409T000002000Z",
            "20260409T000003000Z",
        ] {
            let path = backup_root.join(name);
            fs::create_dir_all(&path).unwrap();
            let metadata = BackupMetadata {
                version: 1,
                namespace: BACKUP_NAMESPACE.to_string(),
                codex_home: service.codex_home.display().to_string(),
                target_provider: "openai".to_string(),
                created_at: Utc::now(),
                state_db_present: false,
                global_state_present: false,
                session_index_present: None,
                session_index_backed_up: false,
            };
            fs::write(
                path.join(METADATA_FILE_NAME),
                serde_json::to_vec(&metadata).unwrap(),
            )
            .unwrap();
        }

        let prune = service.prune_backups(2).unwrap();
        assert_eq!(prune.deleted_count, 1);
        assert_eq!(prune.remaining_count, 2);
    }
}
