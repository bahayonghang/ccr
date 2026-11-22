//! Consistent remote snapshot recipe and read-only validation. No migrations.
use crate::{AppPaths, UsageError, open_dashboard};
use rusqlite::{Connection, OpenFlags};

/// Successful helper response when the live database path is absent.
pub const DB_MISSING_RESPONSE: &str = "ccr_usage_snapshot_db_missing_v1";

/// Python's standard SQLite backup API includes committed WAL pages. The source
/// is opened read-only; only a fresh owner-private directory receives writes.
pub const BACKUP_SCRIPT: &str = r#"import contextlib, os, pathlib, sqlite3, sys, tempfile, time
root = pathlib.Path(sys.argv[1])
source = root / 'llmusage.db'
def unlinked(path):
    for part in (path, *path.parents):
        if part.is_symlink() or (hasattr(part, 'is_junction') and part.is_junction()):
            raise RuntimeError('snapshot_symlink')
unlinked(root)
unlinked(source)
try:
    source.lstat()
except FileNotFoundError:
    print('ccr_usage_snapshot_db_missing_v1', flush=True)
    sys.exit(0)
deadline = time.monotonic() + 8
def progress(status, remaining, total):
    if time.monotonic() > deadline or total * page_size > 64 * 1024 * 1024:
        raise RuntimeError('snapshot_budget')
directory = tempfile.mkdtemp(prefix='.ccr-snapshot-', dir=root)
os.chmod(directory, 0o700)
target = pathlib.Path(directory) / 'llmusage.db'
try:
    with contextlib.closing(sqlite3.connect(source.as_uri() + '?mode=ro', uri=True, timeout=1)) as src:
        page_size = src.execute('PRAGMA page_size').fetchone()[0]
        pages = src.execute('PRAGMA page_count').fetchone()[0]
        if pages * page_size > 64 * 1024 * 1024:
            raise RuntimeError('snapshot_budget')
        fd = os.open(target, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        os.close(fd)
        with contextlib.closing(sqlite3.connect(target)) as dst:
            src.backup(dst, pages=64, progress=progress, sleep=0.01)
            dst.execute('PRAGMA journal_mode=DELETE')
    with open(target, 'r+b') as file:
        os.fsync(file.fileno())
    print(str(target), flush=True)
except BaseException:
    for suffix in ('', '-wal', '-shm', '-journal'):
        try: os.unlink(str(target) + suffix)
        except FileNotFoundError: pass
    os.rmdir(directory)
    raise
"#;

/// Cleanup accepts only the generated snapshot file, never the live database.
pub const CLEANUP_SCRIPT: &str = r#"import os, pathlib, sys
target = pathlib.Path(sys.argv[1])
if target.name != 'llmusage.db' or not target.parent.name.startswith('.ccr-snapshot-'):
    raise RuntimeError('invalid_snapshot')
for part in (target, *target.parents):
    if part.is_symlink() or (hasattr(part, 'is_junction') and part.is_junction()):
        raise RuntimeError('snapshot_symlink')
for suffix in ('', '-wal', '-shm', '-journal'):
    try: os.unlink(str(target) + suffix)
    except FileNotFoundError: pass
os.rmdir(target.parent)
"#;

/// Validate a downloaded standalone snapshot without creating or migrating it.
pub fn validate(paths: &AppPaths) -> Result<(), UsageError> {
    let conn = Connection::open_with_flags(&paths.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| UsageError::DbUnreadable(error.to_string()))?;
    let integrity: String = conn
        .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
        .map_err(|error| UsageError::DbUnreadable(error.to_string()))?;
    if integrity != "ok" {
        return Err(UsageError::DbUnreadable("snapshot_integrity".into()));
    }
    drop(conn);
    open_dashboard(paths.clone()).map(|_| ())
}

#[cfg(all(test, feature = "test-fixtures"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::{
        QueryFilter, ReportTimezone,
        fixtures::{SeedBucket, create_projection_db, seed_bucket, set_schema_version},
    };
    use std::{path::Path, process::Command};
    fn python(script: &str, root: &Path) -> std::process::Output {
        Command::new(if cfg!(windows) { "python" } else { "python3" })
            .args(["-c", script])
            .arg(root)
            .output()
            .unwrap()
    }
    #[test]
    fn backup_reports_an_absent_database_before_creating_staging() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join(".llmusage");
        let assert_missing = |output: std::process::Output| {
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            assert_eq!(
                String::from_utf8(output.stdout).unwrap().trim_end(),
                DB_MISSING_RESPONSE
            );
        };

        assert_missing(python(BACKUP_SCRIPT, &root));
        assert!(!root.exists());
        std::fs::create_dir(&root).unwrap();
        assert_missing(python(BACKUP_SCRIPT, &root));
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);

        let source = root.join("llmusage.db");
        std::fs::write(&source, b"unreadable SQLite bytes").unwrap();
        let output = python(BACKUP_SCRIPT, &root);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(std::fs::read(&source).unwrap(), b"unreadable SQLite bytes");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    }

    #[test]
    fn backup_includes_committed_wal_without_modifying_source_and_schema_is_read_only() {
        let root = tempfile::tempdir().unwrap();
        let paths = create_projection_db(root.path());
        let conn = Connection::open(&paths.db_path).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;")
            .unwrap();
        seed_bucket(
            &conn,
            &SeedBucket {
                event_count: 37,
                total_tokens: 91,
                ..Default::default()
            },
        );
        let bytes = std::fs::read(&paths.db_path).unwrap();
        let wal = std::fs::read(paths.db_path.with_file_name("llmusage.db-wal")).unwrap();
        assert!(!wal.is_empty());
        let output = python(BACKUP_SCRIPT, root.path());
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let file = String::from_utf8(output.stdout).unwrap();
        let snapshot = AppPaths::from_root(Path::new(file.trim()).parent().unwrap());
        validate(&snapshot).unwrap();
        let projected = open_dashboard(snapshot.clone())
            .unwrap()
            .overview(&QueryFilter {
                timezone: ReportTimezone::Utc,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(projected.total_events, 37);
        assert_eq!(std::fs::read(&paths.db_path).unwrap(), bytes);
        assert_eq!(
            std::fs::read(paths.db_path.with_file_name("llmusage.db-wal")).unwrap(),
            wal
        );
        assert!(!snapshot.db_path.with_file_name("llmusage.db-wal").exists());
        assert!(
            python(CLEANUP_SCRIPT, Path::new(file.trim()))
                .status
                .success()
        );
        drop(conn);
        for version in [9, 10, 14, 99] {
            set_schema_version(&paths, version);
            let before = std::fs::read(&paths.db_path).unwrap();
            let result = validate(&paths);
            assert_eq!(result.is_ok(), version >= 10);
            assert_eq!(std::fs::read(&paths.db_path).unwrap(), before);
            if version >= 10 {
                let dashboard = open_dashboard(paths.clone()).unwrap();
                assert_eq!(
                    dashboard
                        .provider_breakdown(&QueryFilter {
                            timezone: ReportTimezone::Utc,
                            ..Default::default()
                        })
                        .is_ok(),
                    version >= 14
                );
            }
        }
    }
    #[test]
    fn helper_rejects_linked_root_source_and_cleanup_before_touching_destination() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        std::fs::create_dir(&source).unwrap();
        let linked_root = root.path().join("linked");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&source, &linked_root).unwrap();
        #[cfg(windows)]
        assert!(
            Command::new("cmd")
                .args(["/c", "mklink", "/J"])
                .arg(&linked_root)
                .arg(&source)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert!(!python(BACKUP_SCRIPT, &linked_root).status.success());
        assert_eq!(std::fs::read_dir(&source).unwrap().count(), 0);
        let external = root.path().join("external");
        std::fs::write(&external, b"keep").unwrap();
        std::fs::hard_link(&external, source.join("llmusage.db")).unwrap();
        // 对 source 的实际链接检测使用文件符号链接；Windows 无开发者模式时改用目录 junction。
        std::fs::remove_file(source.join("llmusage.db")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&external, source.join("llmusage.db")).unwrap();
        #[cfg(windows)]
        assert!(
            Command::new("cmd")
                .args(["/c", "mklink", "/J"])
                .arg(source.join("llmusage.db"))
                .arg(root.path())
                .output()
                .unwrap()
                .status
                .success()
        );
        assert!(!python(BACKUP_SCRIPT, &source).status.success());
        assert_eq!(std::fs::read_dir(&source).unwrap().count(), 1);
        assert!(
            !python(
                CLEANUP_SCRIPT,
                &linked_root.join(".ccr-snapshot-fixture/llmusage.db")
            )
            .status
            .success()
        );
        assert_eq!(std::fs::read(&external).unwrap(), b"keep");
    }
}
