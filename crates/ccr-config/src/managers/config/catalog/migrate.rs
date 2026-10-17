//! Persistent migration from a legacy `profiles.toml` into `profiles.db`.

use super::{
    DbInspect, acquire_catalog_lock, attempt_file, create_database, create_restricted_empty,
    inspect_db, mark_source_retired, open_rw, remove_sqlite_files, replace_with_import,
    restrict_owner_only,
};
use crate::models::{Platform, PlatformPaths};
use ccr_core::{CcrError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MigrationFailpoint {
    BeforeAuthoritativeCommit,
    AfterAuthoritativeCommitBeforeMove,
    DuringMove,
    AfterMoveBeforeSourceRetired,
    MoveFails,
}

enum Step {
    Done,
    Retry,
}

pub fn migrate_catalogs() -> Result<()> {
    for platform in Platform::implemented() {
        let paths = PlatformPaths::new(platform)?;
        migrate_paths(&paths, None)?;
    }
    Ok(())
}

pub(crate) fn migrate_paths(
    paths: &PlatformPaths,
    failpoint: Option<MigrationFailpoint>,
) -> Result<()> {
    let database = paths.profiles_db();
    let legacy = &paths.profiles_file;
    let attempt = attempt_file(&database);
    // 两者都不存在时不建库，也不创建锁文件。
    if !legacy.exists() && !database.exists() && !attempt.exists() {
        return Ok(());
    }
    let _lock = acquire_catalog_lock(&database)?;
    for _ in 0..4 {
        match migration_step(paths, &database, failpoint)? {
            Step::Done => return Ok(()),
            Step::Retry => continue,
        }
    }
    Err(CcrError::ConfigError("配置迁移未能收敛".into()))
}

fn migration_step(
    paths: &PlatformPaths,
    database: &Path,
    failpoint: Option<MigrationFailpoint>,
) -> Result<Step> {
    let legacy = &paths.profiles_file;
    if legacy.is_dir() || database.is_dir() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            legacy.display()
        )));
    }
    let legacy_file = legacy.is_file();
    let attempt = attempt_file(database);
    if attempt.is_dir() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            attempt.display()
        )));
    }
    let attempt_present = attempt.is_file();

    if !database.exists() {
        if attempt_present {
            fs::remove_file(&attempt)
                .map_err(|error| CcrError::FileIoError(format!("删除迁移标记失败: {error}")))?;
            return Ok(Step::Retry);
        }
        if !legacy_file {
            return Ok(Step::Done);
        }
        import_new(paths, database, failpoint)?;
        return Ok(Step::Done);
    }

    if !database.is_file() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            database.display()
        )));
    }

    match inspect_db(database)? {
        DbInspect::Unreadable(_) if attempt_present && legacy_file => {
            remove_incomplete_attempt(database, &attempt);
            Ok(Step::Retry)
        }
        DbInspect::Unreadable(error) => Err(error),
        DbInspect::NoPhase if attempt_present && legacy_file => {
            remove_incomplete_attempt(database, &attempt);
            Ok(Step::Retry)
        }
        DbInspect::NoPhase => Err(CcrError::ConfigError(
            "配置目录已存在且无法确认迁移阶段".into(),
        )),
        DbInspect::Ready(super::Phase::DataAuthoritative) if legacy_file => {
            move_legacy(paths, failpoint)?;
            if failpoint == Some(MigrationFailpoint::AfterMoveBeforeSourceRetired) {
                return Err(CcrError::ConfigError("迁移在停用遗留配置前中断".into()));
            }
            retire(database)?;
            Ok(Step::Done)
        }
        DbInspect::Ready(super::Phase::DataAuthoritative) => {
            retire(database)?;
            Ok(Step::Done)
        }
        DbInspect::Ready(super::Phase::SourceRetired) if legacy_file => {
            move_legacy(paths, failpoint)?;
            Ok(Step::Done)
        }
        DbInspect::Ready(super::Phase::SourceRetired) => Ok(Step::Done),
    }
}

fn import_new(
    paths: &PlatformPaths,
    database: &Path,
    failpoint: Option<MigrationFailpoint>,
) -> Result<()> {
    let attempt = attempt_file(database);
    let id = new_attempt_id();
    write_attempt(&attempt, &id)?;
    let outcome = import_new_inner(paths, database, failpoint);
    if outcome.is_err() {
        cleanup_failed_attempt(database, &attempt, &id);
    }
    outcome
}

fn import_new_inner(
    paths: &PlatformPaths,
    database: &Path,
    failpoint: Option<MigrationFailpoint>,
) -> Result<()> {
    let connection = create_database(database)?;
    let content = fs::read_to_string(&paths.profiles_file).map_err(|error| {
        CcrError::FileIoError(format!(
            "读取遗留配置失败 {}: {error}",
            paths.profiles_file.display()
        ))
    })?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(super::sql_error)?;
    if let Err(error) = replace_with_import(&connection, &content) {
        let _ = connection.execute_batch("ROLLBACK");
        return Err(error);
    }
    if failpoint == Some(MigrationFailpoint::BeforeAuthoritativeCommit) {
        let _ = connection.execute_batch("ROLLBACK");
        return Err(CcrError::ConfigError("迁移在权威提交前中断".into()));
    }
    connection
        .execute_batch("COMMIT")
        .map_err(super::sql_error)?;
    super::checkpoint(&connection)?;
    let _ = fs::remove_file(attempt_file(database));
    tracing::debug!(path = %database.display(), "profile catalog is data_authoritative");

    if failpoint == Some(MigrationFailpoint::AfterAuthoritativeCommitBeforeMove) {
        return Err(CcrError::ConfigError("迁移在移动遗留配置前中断".into()));
    }
    move_legacy(paths, failpoint)?;
    if failpoint == Some(MigrationFailpoint::AfterMoveBeforeSourceRetired) {
        return Err(CcrError::ConfigError("迁移在停用遗留配置前中断".into()));
    }
    mark_source_retired(&connection)?;
    tracing::debug!(path = %database.display(), "profile catalog source is retired");
    Ok(())
}

fn move_legacy(paths: &PlatformPaths, failpoint: Option<MigrationFailpoint>) -> Result<()> {
    if failpoint == Some(MigrationFailpoint::MoveFails) {
        return Err(CcrError::FileIoError("移动遗留配置失败".into()));
    }
    if failpoint == Some(MigrationFailpoint::DuringMove) {
        fs::create_dir_all(&paths.backups_dir)
            .map_err(|error| CcrError::FileIoError(format!("创建备份目录失败: {error}")))?;
        return Err(CcrError::ConfigError("迁移在移动遗留配置时中断".into()));
    }
    relocate_legacy(&paths.profiles_file, &paths.backups_dir)
}

fn retire(database: &Path) -> Result<()> {
    let connection = open_rw(database)?;
    mark_source_retired(&connection)
}

fn relocate_legacy(source: &Path, backups: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(backups)
        .map_err(|error| CcrError::FileIoError(format!("创建备份目录失败: {error}")))?;
    let destination = unique_backup(backups);
    fs::rename(source, &destination)
        .map_err(|error| CcrError::FileIoError(format!("移动遗留配置失败: {error}")))?;
    Ok(())
}

fn unique_backup(backups: &Path) -> PathBuf {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%6f");
    let mut destination = backups.join(format!("profiles.{stamp}.toml"));
    let mut suffix = 0_u32;
    while destination.exists() {
        suffix += 1;
        destination = backups.join(format!("profiles.{stamp}.{suffix}.toml"));
    }
    destination
}

fn write_attempt(path: &Path, id: &str) -> Result<()> {
    create_restricted_empty(path)?;
    fs::write(path, id)
        .map_err(|error| CcrError::FileIoError(format!("写入迁移标记失败: {error}")))?;
    restrict_owner_only(path)?;
    Ok(())
}

fn cleanup_failed_attempt(database: &Path, attempt: &Path, id: &str) {
    let marker = fs::read_to_string(attempt).ok();
    if marker.as_deref().map(str::trim) != Some(id) {
        return;
    }
    if matches!(inspect_db(database), Ok(DbInspect::Ready(_))) {
        return;
    }
    remove_sqlite_files(database);
    let _ = fs::remove_file(attempt);
}

fn remove_incomplete_attempt(database: &Path, attempt: &Path) {
    remove_sqlite_files(database);
    let _ = fs::remove_file(attempt);
}

fn new_attempt_id() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}-{sequence:x}-{}", std::process::id())
}
