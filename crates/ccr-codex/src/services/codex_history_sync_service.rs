//! Codex 历史同步服务
//!
//! 修复 Codex 在不同 provider namespace 下的历史会话可见性问题。

use crate::managers::CodexConfigManager;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use chrono::{DateTime, Utc};
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, params, params_from_iter};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
#[cfg(not(windows))]
use tempfile::NamedTempFile;
#[cfg(windows)]
use std::fs::OpenOptions;
use walkdir::WalkDir;

const DEFAULT_PROVIDER: &str = "openai";
const BACKUP_NAMESPACE: &str = "codex_sync_history";
const BACKUP_DIR_NAME: &str = "sync-history";
const GLOBAL_STATE_FILE_NAME: &str = ".codex-global-state.json";
const SQLITE_FILE_NAME: &str = "state_5.sqlite";
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
    pub rollout_counts: CodexHistoryProviderBuckets,
    pub sqlite_counts: Option<CodexHistoryProviderBuckets>,
    pub backup_root: PathBuf,
    pub backup_summary: CodexHistoryBackupSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexHistorySyncResult {
    pub codex_home: PathBuf,
    pub target_provider: String,
    pub previous_provider: String,
    pub backup_dir: PathBuf,
    pub changed_rollout_files: usize,
    pub added_sidebar_projects: usize,
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
}

#[derive(Debug, Clone)]
pub struct CodexHistorySyncOptions {
    pub provider: Option<String>,
    pub keep_count: usize,
    pub sqlite_busy_timeout: Duration,
}

impl Default for CodexHistorySyncOptions {
    fn default() -> Self {
        Self {
            provider: None,
            keep_count: DEFAULT_KEEP_COUNT,
            sqlite_busy_timeout: SQLITE_BUSY_TIMEOUT,
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
    updated_first_line: String,
}

#[derive(Debug, Clone)]
struct SessionScan {
    changes: Vec<SessionChange>,
    provider_counts: CodexHistoryProviderBuckets,
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
    agent_nickname: Option<String>,
    agent_role: Option<String>,
    memory_mode: String,
    model: Option<String>,
    reasoning_effort: Option<String>,
    agent_path: Option<String>,
}

#[derive(Debug, Clone)]
struct SidebarSyncPlan {
    file_path: PathBuf,
    existed: bool,
    original_text: Option<String>,
    next_text: Option<String>,
    added_projects: Vec<String>,
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
        let rollout_counts = self
            .scan_session_changes("__status_only__")?
            .provider_counts;
        let sqlite_counts = self.read_sqlite_provider_counts()?;
        let backup_summary = self.backup_summary()?;

        Ok(CodexHistorySyncStatus {
            codex_home: self.codex_home.clone(),
            current_provider: current.provider,
            current_provider_implicit: current.implicit,
            rollout_counts,
            sqlite_counts,
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

        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        let current = self.current_provider()?;
        let target_provider = options
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| current.provider.clone());

        let scan = self.scan_session_changes(&target_provider)?;
        let sidebar_plan = self
            .prepare_sidebar_sync(self.read_sqlite_project_paths(options.sqlite_busy_timeout)?)?;
        self.assert_sqlite_writable(options.sqlite_busy_timeout)?;

        let backup_dir = self.create_backup(&target_provider, &sidebar_plan)?;
        let mut applied_rollout_entries = Vec::new();
        let mut global_state_snapshot = None;

        let sync_result = (|| -> Result<(SessionApplyResult, SqliteUpdateResult)> {
            let apply_result = self.apply_session_changes(&scan.changes)?;
            applied_rollout_entries = apply_result.applied_changes.clone();
            self.write_session_manifest(&backup_dir, &applied_rollout_entries)?;

            if let Some(snapshot) = self.apply_sidebar_sync(&sidebar_plan)? {
                global_state_snapshot = Some(snapshot);
            }

            let sqlite_result = self.update_sqlite_provider(
                &target_provider,
                options.sqlite_busy_timeout,
                &scan.thread_candidates,
            )?;
            Ok((apply_result, sqlite_result))
        })();

        match sync_result {
            Ok((apply_result, sqlite_result)) => {
                let backup_cleanup = Some(self.prune_backups_inner(options.keep_count)?);
                Ok(CodexHistorySyncResult {
                    codex_home: self.codex_home.clone(),
                    target_provider,
                    previous_provider: current.provider,
                    backup_dir,
                    changed_rollout_files: apply_result.applied_changes.len(),
                    added_sidebar_projects: sidebar_plan.added_projects.len(),
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
                Err(err)
            }
        }
    }

    pub fn restore<P: AsRef<Path>>(&self, backup_dir: P) -> Result<CodexHistoryRestoreResult> {
        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        let backup_dir = backup_dir.as_ref().to_path_buf();
        let metadata = self.read_backup_metadata(&backup_dir)?;
        let session_entries = self.read_session_manifest(&backup_dir)?;
        self.restore_session_entries(&session_entries)?;
        self.restore_global_state_from_backup(&backup_dir, &metadata)?;
        self.restore_sqlite_from_backup(&backup_dir, &metadata)?;

        Ok(CodexHistoryRestoreResult {
            codex_home: self.codex_home.clone(),
            backup_dir,
            target_provider: metadata.target_provider,
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

    fn scan_session_changes(&self, target_provider: &str) -> Result<SessionScan> {
        let mut changes = Vec::new();
        let mut provider_counts = CodexHistoryProviderBuckets::default();
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

                let current_provider = payload
                    .get("model_provider")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("(missing)")
                    .to_string();
                increment_provider_count(&mut provider_counts, scope, &current_provider);

                if let Some(id) = payload
                    .get("id")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.trim().is_empty())
                {
                    thread_candidates.push(RolloutThreadCandidate {
                        id: id.to_string(),
                        path: path.clone(),
                        archived: matches!(scope, SessionScope::ArchivedSessions),
                    });
                }

                if target_provider == "__status_only__" || current_provider == target_provider {
                    continue;
                }

                let metadata = fs::metadata(&path)?;
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
                    updated_first_line: serde_json::to_string(&updated).map_err(|err| {
                        CcrError::ConfigError(format!("序列化 session_meta 失败: {}", err))
                    })?,
                });
            }
        }

        Ok(SessionScan {
            changes,
            provider_counts,
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

    fn read_sqlite_project_paths(&self, busy_timeout: Duration) -> Result<Vec<String>> {
        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        if !db_path.exists() {
            return Ok(Vec::new());
        }

        let conn = Connection::open(&db_path)
            .map_err(|err| CcrError::DatabaseError(format!("打开 state_5.sqlite 失败: {}", err)))?;
        conn.busy_timeout(busy_timeout).map_err(|err| {
            CcrError::DatabaseError(format!("设置 SQLite busy timeout 失败: {}", err))
        })?;

        let mut stmt = match conn.prepare(
            r#"
            SELECT DISTINCT cwd
            FROM threads
            WHERE TRIM(COALESCE(cwd, '')) <> ''
            ORDER BY LOWER(cwd), cwd
            "#,
        ) {
            Ok(stmt) => stmt,
            Err(err)
                if err
                    .to_string()
                    .to_ascii_lowercase()
                    .contains("no such column: cwd") =>
            {
                return Ok(Vec::new());
            }
            Err(err) => {
                return Err(CcrError::DatabaseError(format!(
                    "读取 SQLite cwd 列表失败: {}",
                    err
                )));
            }
        };

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|err| CcrError::DatabaseError(format!("查询 SQLite cwd 列表失败: {}", err)))?;

        let mut paths = Vec::new();
        for row in rows {
            paths.push(row.map_err(|err| {
                CcrError::DatabaseError(format!("读取 SQLite cwd 失败: {}", err))
            })?);
        }

        Ok(paths)
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

    fn create_backup(
        &self,
        target_provider: &str,
        sidebar_plan: &SidebarSyncPlan,
    ) -> Result<PathBuf> {
        let backup_dir = self
            .backup_root()
            .join(Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string());
        fs::create_dir_all(&backup_dir)?;

        let db_path = self.codex_home.join(SQLITE_FILE_NAME);
        let global_state_path = self.codex_home.join(GLOBAL_STATE_FILE_NAME);
        let state_db_present = db_path.exists();
        let global_state_present = sidebar_plan.existed || global_state_path.exists();

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

        let metadata = BackupMetadata {
            version: 1,
            namespace: BACKUP_NAMESPACE.to_string(),
            codex_home: self.codex_home.display().to_string(),
            target_provider: target_provider.to_string(),
            created_at: Utc::now(),
            state_db_present,
            global_state_present,
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

    fn update_sqlite_provider(
        &self,
        target_provider: &str,
        busy_timeout: Duration,
        thread_candidates: &[RolloutThreadCandidate],
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
        let updated_rows = conn.execute(
            "UPDATE threads SET model_provider = ?1 WHERE COALESCE(model_provider, '') <> ?1",
            params![target_provider],
        );

        let mut affected_rows = match updated_rows {
            Ok(updated_rows) => updated_rows,
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(map_sqlite_busy_error(err));
            }
        };

        let existing_ids = load_existing_thread_ids(&conn)?;
        for candidate in thread_candidates {
            if existing_ids.contains(&candidate.id) {
                continue;
            }

            let record = build_rollout_thread_insert(candidate, target_provider)?;
            affected_rows += insert_thread_row(&conn, &record)?;
        }

        conn.execute_batch("COMMIT").map_err(|err| {
            CcrError::DatabaseError(format!("提交 SQLite provider 更新失败: {}", err))
        })?;
        Ok(SqliteUpdateResult {
            updated_rows: affected_rows,
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

fn load_existing_thread_ids(conn: &Connection) -> Result<BTreeSet<String>> {
    let mut stmt = conn
        .prepare("SELECT id FROM threads")
        .map_err(|err| CcrError::DatabaseError(format!("准备读取 thread ids 失败: {}", err)))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|err| CcrError::DatabaseError(format!("读取 thread ids 失败: {}", err)))?;

    let mut ids = BTreeSet::new();
    for row in rows {
        ids.insert(
            row.map_err(|err| CcrError::DatabaseError(format!("读取 thread id 失败: {}", err)))?,
        );
    }
    Ok(ids)
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

    let mut created_at = None;
    let mut updated_at = None;
    let mut source = String::new();
    let mut cwd = String::new();
    let mut title = String::new();
    let mut sandbox_policy = String::new();
    let mut approval_mode = String::new();
    let mut cli_version = String::new();
    let mut first_user_message = String::new();
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

        if created_at.is_none() {
            created_at = extract_timestamp_secs(&value);
        }
        updated_at = updated_at.max(extract_timestamp_secs(&value));

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

    let fallback_ts = fs::metadata(&candidate.path)?
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0);
    let created_at = created_at.unwrap_or(fallback_ts);
    let updated_at = updated_at.unwrap_or(created_at);

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
        archived_at: candidate.archived.then_some(updated_at),
        cli_version,
        first_user_message,
        agent_nickname,
        agent_role,
        memory_mode,
        model,
        reasoning_effort,
        agent_path,
    })
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
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

    fn write_rollout(path: &Path, id: &str, provider: &str) {
        let content = format!(
            "{{\"timestamp\":\"2026-04-09T00:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{}\",\"cwd\":\"E:\\\\Repo\",\"model_provider\":\"{}\"}}}}\n{{\"timestamp\":\"2026-04-09T00:00:01.000Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello\"}}}}\n",
            id, provider
        );
        fs::write(path, content).unwrap();
    }

    fn write_global_state(codex_home: &Path, content: &str) {
        fs::write(codex_home.join(GLOBAL_STATE_FILE_NAME), content).unwrap();
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

        service.restore(&result.backup_dir).unwrap();
        let rollout = fs::read_to_string(&session_path).unwrap();
        assert!(rollout.contains(r#""model_provider":"custom""#));
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
    temp.persist(&change.path)
        .map_err(|err| CcrError::FileIoError(format!("替换 rollout 文件失败: {}", err)))?;

    Ok(Some(AppliedSessionBackupEntry {
        path: change.path.clone(),
        original_first_line: change.original_first_line.clone(),
        original_separator: change.original_separator.clone(),
    }))
}
