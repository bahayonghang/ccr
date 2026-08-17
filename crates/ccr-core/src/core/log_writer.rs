use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub(crate) const LOG_FILE_PREFIX: &str = "ccr.log";

pub(crate) fn daily_log_file_name(date: &str) -> String {
    format!("{LOG_FILE_PREFIX}.{date}")
}

pub(crate) fn utc_log_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

pub(crate) fn is_managed_log_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("ccr.log."))
}

pub(crate) fn set_owner_only_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn set_owner_only_file(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn tighten_existing_log_files(log_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_managed_log_file(&path) {
            let _ = set_owner_only_file(&path);
        }
    }
}

pub(crate) struct SecureDailyWriter {
    inner: RollingFileAppender,
    log_dir: PathBuf,
    last_date: Mutex<String>,
    disabled: AtomicBool,
}

impl SecureDailyWriter {
    pub(crate) fn new(log_dir: PathBuf) -> Self {
        let today = utc_log_date();
        let inner = RollingFileAppender::new(Rotation::DAILY, &log_dir, LOG_FILE_PREFIX);
        let today_path = log_dir.join(daily_log_file_name(&today));
        if today_path.exists() && set_owner_only_file(&today_path).is_err() {
            return Self {
                inner,
                log_dir,
                last_date: Mutex::new(today),
                disabled: AtomicBool::new(true),
            };
        }

        Self {
            inner,
            log_dir,
            last_date: Mutex::new(today),
            disabled: AtomicBool::new(false),
        }
    }

    fn chmod_today_or_disable(&self, today: &str) -> bool {
        let path = self.log_dir.join(daily_log_file_name(today));
        if !path.exists() {
            return true;
        }
        if set_owner_only_file(&path).is_err() {
            self.disabled.store(true, Ordering::SeqCst);
            return false;
        }
        true
    }
}

impl Write for SecureDailyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.disabled.load(Ordering::SeqCst) {
            return Ok(buf.len());
        }

        let written = self.inner.write(buf)?;
        let today = utc_log_date();
        let mut last_date = self
            .last_date
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if last_date.as_str() != today {
            if self.chmod_today_or_disable(&today) {
                *last_date = today;
            }
        } else if !self.chmod_today_or_disable(&today) {
            return Ok(written);
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.disabled.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.inner.flush()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn managed_name_matches_daily_files_only() {
        assert!(is_managed_log_file(Path::new("ccr.log.2026-07-12")));
        assert!(!is_managed_log_file(Path::new("ccr.log")));
        assert!(!is_managed_log_file(Path::new("other.log.2026-01-01")));
    }

    #[cfg(unix)]
    #[test]
    fn daily_writer_sets_owner_only_on_created_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let mut writer = SecureDailyWriter::new(temp.path().to_path_buf());
        writer.write_all(b"hello\n").unwrap();
        writer.flush().unwrap();

        let path = temp.path().join(daily_log_file_name(&utc_log_date()));
        assert!(path.exists());
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn chmod_helper_sets_existing_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("ccr.log.2026-08-18");
        fs::write(&path, b"x").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        set_owner_only_file(&path).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
