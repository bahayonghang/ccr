//! Codex session trash and restore service.
//!
//! This service owns destructive session moves. It keeps a CCR-owned trash
//! manifest so deleted conversations can be restored without overwriting active
//! rollout files or `session_index.jsonl` entries.

use crate::managers::CodexConfigManager;
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, Utc};
use filetime::FileTime;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

const SESSION_INDEX_FILE: &str = "session_index.jsonl";
const SESSION_TRASH_ROOT_DIR: &str = "session-trash";
const MANIFEST_FILE_NAME: &str = "manifest.json";
const ROLLOUT_FILE_NAME: &str = "rollout.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexTrashedSessionRecord {
    pub session_id: String,
    pub title: String,
    pub cwd: Option<String>,
    pub deleted_at: DateTime<Utc>,
    pub original_relative_path: String,
    pub original_codex_home: PathBuf,
    pub trash_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSessionTrashSummary {
    pub requested_session_count: usize,
    pub trashed_session_count: usize,
    pub trash_root: PathBuf,
    pub trashed_sessions: Vec<CodexTrashedSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSessionRestoreSummary {
    pub requested_session_count: usize,
    pub restored_session_count: usize,
    pub restored_sessions: Vec<CodexTrashedSessionRecord>,
}

pub struct CodexSessionTrashService {
    codex_home: PathBuf,
}

#[derive(Debug, Clone)]
struct ActiveSessionSnapshot {
    session_id: String,
    title: String,
    cwd: Option<String>,
    rollout_path: PathBuf,
    relative_path: PathBuf,
    session_index_entry: Option<JsonValue>,
    modified_unix_secs: i64,
    modified_nanos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrashedSessionManifest {
    version: u32,
    session_id: String,
    title: String,
    cwd: Option<String>,
    deleted_at: DateTime<Utc>,
    original_rollout_path: PathBuf,
    original_relative_path: String,
    original_codex_home: PathBuf,
    session_index_entry: Option<JsonValue>,
    rollout_modified_unix_secs: i64,
    rollout_modified_nanos: u32,
}

#[derive(Debug, Clone)]
struct TrashEntry {
    dir: PathBuf,
    rollout_path: PathBuf,
    manifest: TrashedSessionManifest,
}

#[derive(Debug, Clone)]
struct SessionIndexSnapshot {
    path: PathBuf,
    existed: bool,
    content: Option<String>,
}

impl CodexSessionTrashService {
    pub fn new() -> Result<Self> {
        let config_manager = CodexConfigManager::with_default()?;
        let codex_home = config_manager
            .config_path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CcrError::ConfigError("无法解析 Codex 目录".into()))?;
        Ok(Self { codex_home })
    }

    pub fn with_codex_home<P>(codex_home: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            codex_home: codex_home.into(),
        }
    }

    pub fn trash_root(&self) -> PathBuf {
        self.codex_home
            .join("backups_state")
            .join(SESSION_TRASH_ROOT_DIR)
    }

    pub fn list_trashed_sessions(&self) -> Result<Vec<CodexTrashedSessionRecord>> {
        let mut records = self
            .load_trash_entries()?
            .into_iter()
            .map(|entry| trash_entry_record(&entry))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .deleted_at
                .cmp(&left.deleted_at)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        Ok(records)
    }

    pub fn trash_sessions<I, S>(&self, session_ids: I) -> Result<CodexSessionTrashSummary>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested_ids = normalize_session_ids(session_ids)?;
        let snapshots_by_id = self.load_active_session_snapshots()?;
        let trash_root = self.trash_root();
        fs::create_dir_all(&trash_root)?;

        let mut trashed_sessions = Vec::new();
        for session_id in &requested_ids {
            let snapshot = snapshots_by_id.get(session_id).ok_or_else(|| {
                CcrError::ResourceNotFound(format!("Codex session not found: {session_id}"))
            })?;
            let entry = self.write_snapshot_to_trash(snapshot)?;
            let index_snapshot = self.remove_session_index_entry(&snapshot.session_id)?;
            let remove_result = fs::remove_file(&snapshot.rollout_path).map_err(CcrError::IoError);
            if let Err(err) = remove_result {
                self.restore_session_index_snapshot(&index_snapshot)?;
                let _ = fs::remove_dir_all(&entry.dir);
                return Err(err);
            }
            trashed_sessions.push(trash_entry_record(&entry));
        }

        Ok(CodexSessionTrashSummary {
            requested_session_count: requested_ids.len(),
            trashed_session_count: trashed_sessions.len(),
            trash_root,
            trashed_sessions,
        })
    }

    pub fn restore_sessions<I, S>(&self, session_ids: I) -> Result<CodexSessionRestoreSummary>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested_ids = normalize_session_ids(session_ids)?;
        let entries = self.load_trash_entries()?;
        let mut entries_by_id = BTreeMap::new();
        for entry in entries {
            entries_by_id
                .entry(entry.manifest.session_id.clone())
                .or_insert(entry);
        }

        let mut restored_sessions = Vec::new();
        for session_id in &requested_ids {
            let entry = entries_by_id.get(session_id).ok_or_else(|| {
                CcrError::ResourceNotFound(format!("Trashed Codex session not found: {session_id}"))
            })?;
            self.restore_trash_entry(entry)?;
            restored_sessions.push(trash_entry_record(entry));
        }

        Ok(CodexSessionRestoreSummary {
            requested_session_count: requested_ids.len(),
            restored_session_count: restored_sessions.len(),
            restored_sessions,
        })
    }

    fn load_active_session_snapshots(&self) -> Result<BTreeMap<String, ActiveSessionSnapshot>> {
        let session_index = self.read_session_index_map()?;
        let mut snapshots = BTreeMap::new();

        for root_name in ["sessions", "archived_sessions"] {
            let root = self.codex_home.join(root_name);
            if !root.exists() {
                continue;
            }
            for rollout_path in collect_rollout_files(&root) {
                let Some(meta) = read_rollout_session_meta(&rollout_path)? else {
                    continue;
                };
                let Some(session_id) = session_meta_id(&meta) else {
                    continue;
                };
                let relative_path = rollout_path
                    .strip_prefix(&self.codex_home)
                    .unwrap_or(rollout_path.as_path())
                    .to_path_buf();
                let metadata = fs::metadata(&rollout_path)?;
                let modified = FileTime::from_last_modification_time(&metadata);
                let title = session_index
                    .get(&session_id)
                    .and_then(session_index_title)
                    .or_else(|| read_first_user_message(&rollout_path).ok().flatten())
                    .unwrap_or_else(|| session_id.clone());
                let cwd = session_meta_cwd(&meta);

                snapshots
                    .entry(session_id.clone())
                    .or_insert(ActiveSessionSnapshot {
                        session_id: session_id.clone(),
                        title,
                        cwd,
                        rollout_path,
                        relative_path,
                        session_index_entry: session_index.get(&session_id).cloned(),
                        modified_unix_secs: modified.unix_seconds(),
                        modified_nanos: modified.nanoseconds(),
                    });
            }
        }

        Ok(snapshots)
    }

    fn write_snapshot_to_trash(&self, snapshot: &ActiveSessionSnapshot) -> Result<TrashEntry> {
        let entry_dir = self.trash_root().join(format!(
            "{}-{}",
            Utc::now().format("%Y%m%dT%H%M%S%3fZ"),
            sanitize_path_part(&snapshot.session_id)
        ));
        fs::create_dir_all(&entry_dir)?;
        let rollout_target = entry_dir.join(ROLLOUT_FILE_NAME);
        fs::copy(&snapshot.rollout_path, &rollout_target)?;

        let manifest = TrashedSessionManifest {
            version: 1,
            session_id: snapshot.session_id.clone(),
            title: snapshot.title.clone(),
            cwd: snapshot.cwd.clone(),
            deleted_at: Utc::now(),
            original_rollout_path: snapshot.rollout_path.clone(),
            original_relative_path: snapshot.relative_path.to_string_lossy().replace('\\', "/"),
            original_codex_home: self.codex_home.clone(),
            session_index_entry: Some(
                snapshot
                    .session_index_entry
                    .clone()
                    .unwrap_or_else(|| build_session_index_entry(snapshot)),
            ),
            rollout_modified_unix_secs: snapshot.modified_unix_secs,
            rollout_modified_nanos: snapshot.modified_nanos,
        };

        let manifest_text = serde_json::to_string_pretty(&manifest)
            .map_err(|err| CcrError::ConfigError(format!("序列化会话废纸篓清单失败: {err}")))?;
        write_text_atomic(
            &entry_dir.join(MANIFEST_FILE_NAME),
            &format!("{manifest_text}\n"),
        )?;

        Ok(TrashEntry {
            dir: entry_dir,
            rollout_path: rollout_target,
            manifest,
        })
    }

    fn remove_session_index_entry(&self, session_id: &str) -> Result<SessionIndexSnapshot> {
        let snapshot = self.read_session_index_snapshot()?;
        let Some(content) = snapshot.content.as_deref() else {
            return Ok(snapshot);
        };

        let retained = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return false;
                }
                serde_json::from_str::<JsonValue>(trimmed)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("id")
                            .and_then(JsonValue::as_str)
                            .map(str::to_string)
                    })
                    .map(|id| id != session_id)
                    .unwrap_or(true)
            })
            .map(str::to_string)
            .collect::<Vec<_>>();
        let next = if retained.is_empty() {
            String::new()
        } else {
            format!("{}\n", retained.join("\n"))
        };
        write_text_atomic(&snapshot.path, &next)?;
        Ok(snapshot)
    }

    fn restore_trash_entry(&self, entry: &TrashEntry) -> Result<()> {
        if !entry.rollout_path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Trashed rollout file not found: {}",
                entry.rollout_path.display()
            )));
        }

        let target_path = self.resolve_restore_target(&entry.manifest)?;
        if target_path.exists() {
            return Err(CcrError::ValidationError(format!(
                "目标 rollout 已存在，无法恢复: {}",
                target_path.display()
            )));
        }

        let index_snapshot = self.read_session_index_snapshot()?;
        if session_index_contains_id(
            index_snapshot.content.as_deref(),
            &entry.manifest.session_id,
        )? {
            return Err(CcrError::ValidationError(format!(
                "session_index.jsonl 已存在会话，无法恢复: {}",
                entry.manifest.session_id
            )));
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&entry.rollout_path, &target_path)?;
        let mtime = FileTime::from_unix_time(
            entry.manifest.rollout_modified_unix_secs,
            entry.manifest.rollout_modified_nanos,
        );
        filetime::set_file_mtime(&target_path, mtime)
            .map_err(|err| CcrError::FileIoError(format!("恢复 rollout 修改时间失败: {err}")))?;

        let restore_result = self.append_session_index_entry(
            index_snapshot.content.as_deref(),
            entry.manifest.session_index_entry.as_ref(),
        );
        if let Err(err) = restore_result {
            let _ = fs::remove_file(&target_path);
            self.restore_session_index_snapshot(&index_snapshot)?;
            return Err(err);
        }

        if let Err(err) = fs::remove_dir_all(&entry.dir) {
            return Err(CcrError::FileIoError(format!(
                "会话已恢复，但清理废纸篓条目失败 ({}): {}",
                entry.dir.display(),
                err
            )));
        }
        Ok(())
    }

    fn resolve_restore_target(&self, manifest: &TrashedSessionManifest) -> Result<PathBuf> {
        let relative = normalize_manifest_relative_path(&manifest.original_relative_path)?;
        Ok(self.codex_home.join(relative))
    }

    fn append_session_index_entry(
        &self,
        original_content: Option<&str>,
        entry: Option<&JsonValue>,
    ) -> Result<()> {
        let entry = entry
            .ok_or_else(|| CcrError::ConfigError("废纸篓清单缺少 session_index 条目".into()))?;
        let serialized = serde_json::to_string(entry).map_err(|err| {
            CcrError::ConfigError(format!("序列化 session_index 条目失败: {err}"))
        })?;
        let mut lines = original_content
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        lines.push(serialized);
        write_text_atomic(
            &self.codex_home.join(SESSION_INDEX_FILE),
            &format!("{}\n", lines.join("\n")),
        )
    }

    fn read_session_index_snapshot(&self) -> Result<SessionIndexSnapshot> {
        let path = self.codex_home.join(SESSION_INDEX_FILE);
        let content = if path.exists() {
            Some(fs::read_to_string(&path)?)
        } else {
            None
        };
        Ok(SessionIndexSnapshot {
            path,
            existed: content.is_some(),
            content,
        })
    }

    fn restore_session_index_snapshot(&self, snapshot: &SessionIndexSnapshot) -> Result<()> {
        if snapshot.existed {
            write_text_atomic(
                &snapshot.path,
                snapshot.content.as_deref().unwrap_or_default(),
            )?;
        } else if snapshot.path.exists() {
            fs::remove_file(&snapshot.path)?;
        }
        Ok(())
    }

    fn read_session_index_map(&self) -> Result<BTreeMap<String, JsonValue>> {
        let snapshot = self.read_session_index_snapshot()?;
        let mut entries = BTreeMap::new();
        let Some(content) = snapshot.content else {
            return Ok(entries);
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) else {
                continue;
            };
            let Some(id) = value.get("id").and_then(JsonValue::as_str) else {
                continue;
            };
            entries.insert(id.to_string(), value);
        }
        Ok(entries)
    }

    fn load_trash_entries(&self) -> Result<Vec<TrashEntry>> {
        let root = self.trash_root();
        if !root.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let dir = entry.path();
            let manifest_path = dir.join(MANIFEST_FILE_NAME);
            if !manifest_path.exists() {
                continue;
            }
            let content = fs::read_to_string(&manifest_path)?;
            let manifest: TrashedSessionManifest =
                serde_json::from_str(&content).map_err(|err| {
                    CcrError::ConfigError(format!(
                        "解析会话废纸篓清单失败 ({}): {}",
                        manifest_path.display(),
                        err
                    ))
                })?;
            entries.push(TrashEntry {
                dir,
                rollout_path: entry.path().join(ROLLOUT_FILE_NAME),
                manifest,
            });
        }
        Ok(entries)
    }
}

fn normalize_session_ids<I, S>(session_ids: I) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for session_id in session_ids {
        let trimmed = session_id.as_ref().trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() {
        return Err(CcrError::ValidationError(
            "请至少提供一个 Codex session id".into(),
        ));
    }
    Ok(result)
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

fn read_rollout_session_meta(path: &Path) -> Result<Option<JsonValue>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed)
            .map_err(|err| CcrError::ConfigError(format!("解析 rollout 首行失败: {err}")))?;
        if value.get("type").and_then(JsonValue::as_str) == Some("session_meta") {
            return Ok(Some(value));
        }
        return Ok(None);
    }
    Ok(None)
}

fn read_first_user_message(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let Ok(value) = serde_json::from_str::<JsonValue>(line.trim()) else {
            continue;
        };
        if let Some(message) = value
            .get("payload")
            .filter(|_| value.get("type").and_then(JsonValue::as_str) == Some("event_msg"))
            .filter(|payload| {
                payload.get("type").and_then(JsonValue::as_str) == Some("user_message")
            })
            .and_then(|payload| payload.get("message"))
            .and_then(JsonValue::as_str)
        {
            return Ok(Some(message.to_string()));
        }
    }
    Ok(None)
}

fn session_meta_id(meta: &JsonValue) -> Option<String> {
    meta.get("payload")
        .and_then(|payload| payload.get("id").or_else(|| payload.get("session_id")))
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            meta.get("id")
                .or_else(|| meta.get("session_id"))
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn session_meta_cwd(meta: &JsonValue) -> Option<String> {
    meta.get("payload")
        .and_then(|payload| payload.get("cwd"))
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn session_index_title(entry: &JsonValue) -> Option<String> {
    ["thread_name", "threadName", "title", "name"]
        .iter()
        .filter_map(|key| entry.get(*key))
        .find_map(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn session_index_contains_id(content: Option<&str>, session_id: &str) -> Result<bool> {
    let Some(content) = content else {
        return Ok(false);
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(trimmed).map_err(|err| {
            CcrError::ConfigError(format!("解析 session_index.jsonl 条目失败: {err}"))
        })?;
        if value.get("id").and_then(JsonValue::as_str) == Some(session_id) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn build_session_index_entry(snapshot: &ActiveSessionSnapshot) -> JsonValue {
    json!({
        "id": snapshot.session_id,
        "thread_name": if snapshot.title.trim().is_empty() {
            "Untitled"
        } else {
            snapshot.title.as_str()
        },
        "updated_at": DateTime::<Utc>::from_timestamp(snapshot.modified_unix_secs, 0)
            .unwrap_or_else(Utc::now)
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    })
}

fn trash_entry_record(entry: &TrashEntry) -> CodexTrashedSessionRecord {
    CodexTrashedSessionRecord {
        session_id: entry.manifest.session_id.clone(),
        title: entry.manifest.title.clone(),
        cwd: entry.manifest.cwd.clone(),
        deleted_at: entry.manifest.deleted_at,
        original_relative_path: entry.manifest.original_relative_path.clone(),
        original_codex_home: entry.manifest.original_codex_home.clone(),
        trash_dir: entry.dir.clone(),
    }
}

fn normalize_manifest_relative_path(relative_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(CcrError::ValidationError(
            "废纸篓清单中的相对路径不安全".into(),
        ));
    }
    let first = path
        .components()
        .next()
        .ok_or_else(|| CcrError::ValidationError("废纸篓清单缺少原始 rollout 相对路径".into()))?;
    let first_text = first.as_os_str().to_string_lossy();
    if first_text != "sessions" && first_text != "archived_sessions" {
        return Err(CcrError::ValidationError(
            "废纸篓清单的原始路径必须位于 sessions 或 archived_sessions".into(),
        ));
    }
    Ok(path)
}

fn sanitize_path_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn write_text_atomic(path: &Path, content: &str) -> Result<()> {
    let temp_file = if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        NamedTempFile::new_in(parent)
    } else {
        NamedTempFile::new()
    }
    .map_err(|err| CcrError::FileIoError(format!("创建临时文件失败: {err}")))?;
    {
        let mut file = File::create(temp_file.path())?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
    }
    temp_file
        .persist(path)
        .map_err(|err| CcrError::FileIoError(format!("原子替换文件失败: {err}")))?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_service() -> (CodexSessionTrashService, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let codex_home = dir.path().join(".codex");
        fs::create_dir_all(codex_home.join("sessions/2026/04/09")).unwrap();
        fs::create_dir_all(codex_home.join("archived_sessions/2026/04/08")).unwrap();
        (CodexSessionTrashService::with_codex_home(codex_home), dir)
    }

    fn write_rollout(path: &Path, id: &str, cwd: &str) {
        let content = format!(
            "{{\"timestamp\":\"2026-04-09T00:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{{\"id\":\"{}\",\"cwd\":\"{}\",\"model_provider\":\"openai\"}}}}\n{{\"timestamp\":\"2026-04-09T00:00:01.000Z\",\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"hello {}\"}}}}\n",
            id,
            cwd.replace('\\', "\\\\"),
            id
        );
        fs::write(path, content).unwrap();
    }

    fn write_session_index(codex_home: &Path, content: &str) {
        fs::write(codex_home.join(SESSION_INDEX_FILE), content).unwrap();
    }

    #[test]
    fn list_trashed_sessions_is_empty_when_trash_root_missing() {
        let (service, _dir) = create_service();

        let records = service.list_trashed_sessions().unwrap();

        assert!(records.is_empty());
    }

    #[test]
    fn trash_moves_rollout_and_removes_session_index_entry() {
        let (service, _dir) = create_service();
        let rollout = service
            .codex_home
            .join("sessions/2026/04/09/rollout-trash-one.jsonl");
        write_rollout(&rollout, "thread-trash-one", r"E:\Repo");
        write_session_index(
            &service.codex_home,
            "{\"id\":\"thread-trash-one\",\"thread_name\":\"Trash One\",\"updated_at\":\"2026-04-09T00:00:00.000000Z\"}\n",
        );

        let summary = service.trash_sessions(["thread-trash-one"]).unwrap();

        assert_eq!(summary.trashed_session_count, 1);
        assert!(!rollout.exists());
        let index = fs::read_to_string(service.codex_home.join(SESSION_INDEX_FILE)).unwrap();
        assert!(!index.contains("thread-trash-one"));
        let listed = service.list_trashed_sessions().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].session_id, "thread-trash-one");
    }

    #[test]
    fn trash_handles_multiple_sessions() {
        let (service, _dir) = create_service();
        let first = service
            .codex_home
            .join("sessions/2026/04/09/rollout-trash-a.jsonl");
        let second = service
            .codex_home
            .join("archived_sessions/2026/04/08/rollout-trash-b.jsonl");
        write_rollout(&first, "thread-trash-a", r"E:\RepoA");
        write_rollout(&second, "thread-trash-b", r"E:\RepoB");

        let summary = service
            .trash_sessions(["thread-trash-a", "thread-trash-b"])
            .unwrap();

        assert_eq!(summary.trashed_session_count, 2);
        assert!(!first.exists());
        assert!(!second.exists());
        assert_eq!(service.list_trashed_sessions().unwrap().len(), 2);
    }

    #[test]
    fn restore_recreates_rollout_appends_index_and_removes_trash_entry() {
        let (service, _dir) = create_service();
        let rollout = service
            .codex_home
            .join("sessions/2026/04/09/rollout-restore-one.jsonl");
        write_rollout(&rollout, "thread-restore-one", r"E:\Repo");
        let mtime = FileTime::from_unix_time(1_775_000_000, 100);
        filetime::set_file_mtime(&rollout, mtime).unwrap();

        service.trash_sessions(["thread-restore-one"]).unwrap();
        assert!(!rollout.exists());

        let summary = service.restore_sessions(["thread-restore-one"]).unwrap();

        assert_eq!(summary.restored_session_count, 1);
        assert!(rollout.exists());
        let index = fs::read_to_string(service.codex_home.join(SESSION_INDEX_FILE)).unwrap();
        assert!(index.contains(r#""id":"thread-restore-one""#));
        let actual_mtime = FileTime::from_last_modification_time(&fs::metadata(&rollout).unwrap());
        assert_eq!(actual_mtime, mtime);
        assert!(service.list_trashed_sessions().unwrap().is_empty());
    }

    #[test]
    fn restore_refuses_existing_rollout_and_keeps_trash_entry() {
        let (service, _dir) = create_service();
        let rollout = service
            .codex_home
            .join("sessions/2026/04/09/rollout-conflict.jsonl");
        write_rollout(&rollout, "thread-conflict", r"E:\Repo");

        service.trash_sessions(["thread-conflict"]).unwrap();
        write_rollout(&rollout, "thread-other", r"E:\Repo");

        let err = service.restore_sessions(["thread-conflict"]).unwrap_err();

        assert!(err.to_string().contains("rollout"));
        assert_eq!(service.list_trashed_sessions().unwrap().len(), 1);
    }

    #[test]
    fn restore_refuses_existing_session_index_id_and_keeps_trash_entry() {
        let (service, _dir) = create_service();
        let rollout = service
            .codex_home
            .join("sessions/2026/04/09/rollout-index-conflict.jsonl");
        write_rollout(&rollout, "thread-index-conflict", r"E:\Repo");

        service.trash_sessions(["thread-index-conflict"]).unwrap();
        write_session_index(
            &service.codex_home,
            "{\"id\":\"thread-index-conflict\",\"thread_name\":\"Conflict\",\"updated_at\":\"2026-04-09T00:00:00.000000Z\"}\n",
        );

        let err = service
            .restore_sessions(["thread-index-conflict"])
            .unwrap_err();

        assert!(err.to_string().contains("session_index"));
        assert_eq!(service.list_trashed_sessions().unwrap().len(), 1);
    }
}
