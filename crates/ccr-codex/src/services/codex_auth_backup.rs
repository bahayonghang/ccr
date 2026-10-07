//! Auth backup pools keep every existing file. Pool locks precede writer leaf locks.

use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{WriteOptions, content_version_token, write_guarded};
use ccr_core::core::lock::LockManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) enum AuthBackupPool<'a> {
    Registry,
    Account(&'a str),
}

impl AuthBackupPool<'_> {
    fn prefix(&self) -> String {
        match self {
            Self::Registry => "auth_registry_".into(),
            Self::Account(name) => format!("auth_account_{name}_"),
        }
    }

    fn extension(&self) -> &str {
        match self {
            Self::Registry => "toml",
            Self::Account(_) => "json",
        }
    }

    fn error(&self, error: impl std::fmt::Display) -> CcrError {
        let prefix = match self {
            Self::Registry => "备份注册表失败",
            Self::Account(_) => "备份 auth 文件失败",
        };
        CcrError::ConfigError(format!("{prefix}: {error}"))
    }

    fn contains(&self, name: &str) -> bool {
        let mut prefix = self.prefix();
        let folded_name;
        let name = if cfg!(windows) {
            prefix.make_ascii_lowercase();
            folded_name = name.to_ascii_lowercase();
            folded_name.as_str()
        } else {
            name
        };
        let suffix = format!(".{}", self.extension());
        let Some(stamp) = name
            .strip_prefix(&prefix)
            .and_then(|name| name.strip_suffix(&suffix))
        else {
            return false;
        };
        // Match the complete alias, including underscores, before the timestamp.
        let bytes = stamp.as_bytes();
        bytes.len() >= 15
            && bytes[..8].iter().all(u8::is_ascii_digit)
            && bytes[8] == b'_'
            && bytes[9..15].iter().all(u8::is_ascii_digit)
            && (bytes.len() == 15
                || (bytes[15] == b'_'
                    && bytes.len() > 16
                    && bytes[16..].iter().all(u8::is_ascii_digit)))
    }
}

pub(crate) fn backup_auth_file(
    source: &Path,
    directory: &Path,
    pool: AuthBackupPool<'_>,
) -> Result<Option<PathBuf>> {
    backup_auth_file_at(
        source,
        directory,
        pool,
        &chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
    )
}

fn backup_auth_file_at(
    source: &Path,
    directory: &Path,
    pool: AuthBackupPool<'_>,
    timestamp: &str,
) -> Result<Option<PathBuf>> {
    let absolute = ccr_core::core::guarded_write::normalized_resource_path(directory)
        .map_err(|error| pool.error(error))?;
    let mut identity = format!(
        "{}:{}:{}",
        absolute.display(),
        pool.prefix(),
        pool.extension()
    );
    if cfg!(windows) {
        identity.make_ascii_lowercase();
    }
    let resource = format!(
        "codex_auth_backup_{}",
        content_version_token(identity.as_bytes())
    );
    let _lock =
        LockManager::with_default_path()?.lock_resource(&resource, Duration::from_secs(10))?;
    let content = match fs::read(source) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(pool.error(error)),
    };
    fs::create_dir_all(directory)
        .map_err(|error| CcrError::ConfigError(format!("创建备份目录失败: {error}")))?;
    let mut backups = Vec::new();
    for entry in fs::read_dir(directory).map_err(|error| pool.error(error))? {
        let entry = entry.map_err(|error| pool.error(error))?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| pool.contains(name))
        {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .map_err(|error| pool.error(error))?;
            backups.push((modified, entry.path()));
        }
    }
    backups.sort_by(|a, b| b.cmp(a));
    if let Some((_, existing)) = backups.first()
        && fs::read(existing).is_ok_and(|bytes| bytes == content)
        && filetime::set_file_mtime(existing, filetime::FileTime::now()).is_ok()
    {
        return Ok(Some(existing.clone()));
    }
    let prefix = pool.prefix();
    let extension = pool.extension();
    let mut path = directory.join(format!("{prefix}{timestamp}.{extension}"));
    let mut sequence = 0u64;
    while path.try_exists().map_err(|error| pool.error(error))? {
        sequence = sequence
            .checked_add(1)
            .ok_or_else(|| pool.error("备份序号已用尽"))?;
        path = directory.join(format!("{prefix}{timestamp}_{sequence}.{extension}"));
    }
    write_guarded(
        &path,
        &content,
        &WriteOptions {
            secret: true,
            ..Default::default()
        },
    )
    .map_err(|error| pool.error(error))?;
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::test_support::TestCodexEnv;

    #[test]
    fn p3_same_second_versions_and_exact_pools_preserve_all_old_files() {
        let env = TestCodexEnv::new();
        let directory = env.ccr_codex_dir().join("auth/backups");
        fs::create_dir_all(&directory).unwrap();
        for index in 0..16 {
            fs::write(
                directory.join(format!("auth_account_foo_20251001_000000_{index}.json")),
                format!("retained-{index}"),
            )
            .unwrap();
        }
        let other = directory.join("auth_account_foo_bar_20261006_000000.json");
        fs::write(&other, b"first").unwrap();
        let registry = directory.join("auth_registry_20261006_000000.toml");
        fs::write(&registry, b"first").unwrap();
        let original: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            })
            .collect();
        let source = env.codex_dir().join("synthetic.json");
        fs::write(&source, b"first").unwrap();
        let first = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Account("foo"),
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            first.file_name().unwrap(),
            "auth_account_foo_20261006_000000.json"
        );
        fs::write(&source, b"second").unwrap();
        let second = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Account("foo"),
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            second.file_name().unwrap(),
            "auth_account_foo_20261006_000000_1.json"
        );
        assert_eq!(fs::read(&first).unwrap(), b"first");
        assert_eq!(fs::read(&second).unwrap(), b"second");
        for (path, bytes) in original {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 20);
        assert!(AuthBackupPool::Account("foo").contains("auth_account_foo_20261006_000000_1.json"));
        assert!(
            !AuthBackupPool::Account("foo").contains("auth_account_foo_bar_20261006_000000.json")
        );
        assert!(
            !AuthBackupPool::Account("foo")
                .contains("auth_account_foo_20261006_000000_1_20261006_000000.json")
        );
    }

    #[test]
    fn p3_backup_deduplicates_only_latest_pool_content_and_refreshes_mtime() {
        let env = TestCodexEnv::new();
        let source = env.codex_dir().join("synthetic.json");
        let directory = env.ccr_codex_dir().join("auth/backups");
        fs::write(&source, b"first").unwrap();
        let first = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Registry,
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        filetime::set_file_mtime(&first, filetime::FileTime::from_unix_time(1, 0)).unwrap();
        let identical = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Registry,
            "20261006_000001",
        )
        .unwrap()
        .unwrap();
        assert_eq!(identical, first);
        assert!(
            fs::metadata(&first).unwrap().modified().unwrap()
                > std::time::UNIX_EPOCH + Duration::from_secs(1)
        );
        fs::write(&source, b"second").unwrap();
        let second = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Registry,
            "20261006_000001",
        )
        .unwrap()
        .unwrap();
        filetime::set_file_mtime(&first, filetime::FileTime::from_unix_time(1, 0)).unwrap();
        fs::write(&source, b"first").unwrap();
        let third = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Registry,
            "20261006_000002",
        )
        .unwrap()
        .unwrap();
        assert_ne!(third, first);
        assert_ne!(third, second);
        assert_eq!(fs::read_dir(directory).unwrap().count(), 3);
    }

    #[test]
    fn p3_concurrent_same_second_backups_do_not_overwrite() {
        let env = TestCodexEnv::new();
        let directory = env.ccr_codex_dir().join("auth/backups");
        let mut workers = Vec::new();
        for index in 0..8 {
            let source = env.codex_dir().join(format!("source-{index}.json"));
            fs::write(&source, format!("version-{index}")).unwrap();
            let directory = directory.clone();
            workers.push(std::thread::spawn(move || {
                let path = backup_auth_file_at(
                    &source,
                    &directory,
                    AuthBackupPool::Account("concurrent"),
                    "20261006_000000",
                )
                .unwrap()
                .unwrap();
                assert_eq!(
                    fs::read(&path).unwrap(),
                    format!("version-{index}").as_bytes()
                );
                path
            }));
        }
        let paths: std::collections::BTreeSet<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(paths.len(), 8);
        assert_eq!(fs::read_dir(directory).unwrap().count(), 8);
    }

    #[test]
    fn p3_backup_process_child() {
        let Some(source) = std::env::var_os("CCR_P3_BACKUP_SOURCE") else {
            return;
        };
        let directory = std::env::var_os("CCR_P3_BACKUP_DIRECTORY").unwrap();
        let source = PathBuf::from(source);
        let content = fs::read(&source).unwrap();
        let path = backup_auth_file_at(
            &source,
            &PathBuf::from(directory),
            AuthBackupPool::Account("process"),
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        assert_eq!(fs::read(path).unwrap(), content);
    }

    #[test]
    fn p3_independent_processes_keep_every_backup_version() {
        use std::process::{Command, Stdio};
        let env = TestCodexEnv::new();
        let directory = env.ccr_codex_dir().join("auth/backups");
        let mut children = Vec::new();
        for index in 0..3 {
            let source = env.codex_dir().join(format!("child-{index}.json"));
            fs::write(&source, format!("child-version-{index}")).unwrap();
            children.push(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "services::codex_auth_backup::tests::p3_backup_process_child",
                    ])
                    .env("CCR_LOCK_DIR", env.lock_dir())
                    .env("CCR_P3_BACKUP_SOURCE", source)
                    .env("CCR_P3_BACKUP_DIRECTORY", &directory)
                    .stdout(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }
        let contents: std::collections::BTreeSet<_> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| fs::read(entry.unwrap().path()).unwrap())
            .collect();
        assert_eq!(
            contents,
            (0..3)
                .map(|index| format!("child-version-{index}").into_bytes())
                .collect()
        );
    }

    #[cfg(windows)]
    #[test]
    fn p3_windows_case_aliases_share_backup_pool_and_private_dacl() {
        use std::process::Command;
        let env = TestCodexEnv::new();
        let directory = env.ccr_codex_dir().join("auth/backups");
        let source = env.codex_dir().join("synthetic.json");
        fs::write(&source, b"first").unwrap();
        let first = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Account("Foo"),
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        let identical = backup_auth_file_at(
            &source,
            &directory,
            AuthBackupPool::Account("foo"),
            "20261006_000000",
        )
        .unwrap()
        .unwrap();
        assert_eq!(first, identical);
        let acl_check = r#"
$p3Acl = [System.IO.File]::GetAccessControl($env:CCR_P3_TEST_BACKUP_PATH)
$p3Sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
if (-not $p3Acl.AreAccessRulesProtected) { exit 1 }
$p3Rules = @($p3Acl.Access)
if ($p3Rules.Count -ne 1) { exit 2 }
if ($p3Rules[0].IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value -ne $p3Sid) { exit 3 }
if ($p3Rules[0].AccessControlType -ne 'Allow') { exit 4 }
if (($p3Rules[0].FileSystemRights -band [System.Security.AccessControl.FileSystemRights]::FullControl) -ne [System.Security.AccessControl.FileSystemRights]::FullControl) { exit 5 }
"#;
        assert!(
            Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", acl_check])
                .env("CCR_P3_TEST_BACKUP_PATH", &first)
                .status()
                .unwrap()
                .success()
        );
        let pool_identity = format!(
            "{}:auth_account_foo_:json",
            ccr_core::core::guarded_write::normalized_resource_path(&directory)
                .unwrap()
                .display()
        )
        .to_ascii_lowercase();
        let resource = format!(
            "codex_auth_backup_{}",
            content_version_token(pool_identity.as_bytes())
        );
        let held = LockManager::with_default_path()
            .unwrap()
            .lock_resource(&resource, Duration::ZERO)
            .unwrap();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        fs::write(&source, b"second").unwrap();
        let worker = std::thread::spawn(move || {
            let path = backup_auth_file_at(
                &source,
                &directory,
                AuthBackupPool::Account("FOO"),
                "20261006_000000",
            )
            .unwrap()
            .unwrap();
            done_tx.send(path).unwrap();
        });
        assert!(done_rx.recv_timeout(Duration::from_millis(150)).is_err());
        drop(held);
        let second = done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        worker.join().unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(first).unwrap(), b"first");
        assert_eq!(fs::read(second).unwrap(), b"second");
    }

    #[cfg(unix)]
    #[test]
    fn p3_new_backup_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let env = TestCodexEnv::new();
        let source = env.codex_dir().join("synthetic.json");
        fs::write(&source, b"synthetic credential fixture").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o644)).unwrap();
        let path = backup_auth_file(
            &source,
            &env.ccr_codex_dir().join("auth/backups"),
            AuthBackupPool::Registry,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
