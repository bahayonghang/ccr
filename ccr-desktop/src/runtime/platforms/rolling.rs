//! Local session usage, separate from llmusage and remote account quota.
use super::*;
use ccr_codex::services::{CodexRollingUsage, CodexScanDiagnostics, CodexUsageService};
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Report {
    pub as_of: DateTime<Utc>,
    pub usage: CodexRollingUsage,
    pub diagnostics: CodexScanDiagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Missing,
    Unavailable,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => "Local Codex sessions only. / 仅支持本地 Codex 会话",
            Self::Missing => {
                "Codex sessions directory is missing; usage is unknown. / Codex 会话目录不存在，用量未知"
            }
            Self::Unavailable => {
                "Cannot safely read local Codex usage. Retry after checking access. / 无法安全读取本地 Codex 用量，请检查权限后重试"
            }
        }
    }
}

fn check_path(path: &Path) -> Result<(), Error> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(Error::Unavailable);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if metadata.file_attributes() & 0x400 != 0 {
                        return Err(Error::Unavailable);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Unavailable),
        }
    }
    Ok(())
}

fn read(source: &ConfigSource, as_of: DateTime<Utc>) -> Result<Report, Error> {
    let roots = roots(source).map_err(|_| Error::Unavailable)?;
    let sessions = roots.codex_runtime.join("sessions");
    for path in [
        &sessions,
        &roots.codex_runtime.join(".ccr-codex-usage-cache.json"),
        &roots.locks,
    ] {
        check_path(path)?;
    }
    match std::fs::read_dir(&sessions) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(Error::Missing),
        Err(_) => return Err(Error::Unavailable),
    }
    // Windows directory junctions are reparse points, not always symlinks in
    // DirEntry::file_type. Reject them before the domain's recursive scanner.
    let mut directories = vec![sessions];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).map_err(|_| Error::Unavailable)? {
            let entry = entry.map_err(|_| Error::Unavailable)?;
            check_path(&entry.path())?;
            if entry.file_type().map_err(|_| Error::Unavailable)?.is_dir() {
                directories.push(entry.path());
            }
        }
    }
    // The domain scanner writes a derived cache. Keep nested guarded writes on
    // this blocking thread and inside the captured environment's lock directory.
    let scan = LockManager::with_scoped_directory(&roots.locks, || {
        CodexUsageService::new(roots.codex_runtime.clone()).scan(as_of)
    })
    .map_err(|_| Error::Unavailable)?;
    Ok(Report {
        as_of,
        usage: CodexUsageService::compute_rolling_usage_at(&scan.records, as_of),
        diagnostics: scan.diagnostics,
    })
}

impl Services {
    pub fn codex_rolling_usage(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<Report, Error>, AdmissionError> {
        let supported =
            matches!(key.environment, EnvironmentId::Local(_)) && key.platform == Platform::Codex;
        let source = self.source.clone();
        self.executor.read(key, move |_, _| {
            if !supported {
                return Err(Error::Unsupported);
            }
            read(&source, Utc::now())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rolling_rejects_remote_and_other_platform_before_resolving_source() {
        use ccr_app::runtime::{RuntimeOwner, TaskEvent};
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        // Path resolution would fail: unsupported requests must never get there.
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: "relative-profiles".into(),
                registry: "relative-registry".into(),
                delay: std::time::Duration::ZERO,
            },
        );
        for (environment, platform) in [
            (EnvironmentId::Wsl("synthetic".into()), Platform::Codex),
            (EnvironmentId::Ssh("synthetic".into()), Platform::Codex),
            (EnvironmentId::Local("synthetic".into()), Platform::Claude),
        ] {
            let mut subscription = services
                .codex_rolling_usage(RequestKey {
                    environment,
                    platform,
                    generation: 0,
                    request_id: 1,
                })
                .unwrap_or_else(|e| panic!("admission: {e:?}"));
            loop {
                match subscription.next().await {
                    Some(TaskEvent::Terminal(result)) => {
                        assert!(matches!(
                            result.as_ref(),
                            Completion::Failed(Error::Unsupported)
                        ));
                        break;
                    }
                    Some(_) => {}
                    None => panic!("missing terminal"),
                }
            }
        }
        tokio::task::spawn_blocking(move || owner.shutdown())
            .await
            .unwrap_or_else(|e| panic!("shutdown: {e}"));
    }

    #[test]
    fn captured_rolling_windows_cache_and_missing_source_are_distinct() {
        let temp = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let source = ConfigSource::Fixture {
            profiles: temp.path().join("profiles.toml"),
            registry: temp.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let as_of = DateTime::parse_from_rfc3339("2026-11-06T12:00:00Z")
            .unwrap_or_else(|e| panic!("clock: {e}"))
            .with_timezone(&Utc);
        assert_eq!(read(&source, as_of).err(), Some(Error::Missing));
        let roots = roots(&source).unwrap_or_else(|e| panic!("roots: {e:?}"));
        let sessions = roots.codex_runtime.join("sessions");
        std::fs::create_dir_all(&sessions).unwrap_or_else(|e| panic!("mkdir: {e}"));
        let mut lines = vec![
            serde_json::json!({"type":"session_meta","payload":{"id":"synthetic-session"}})
                .to_string(),
        ];
        lines.extend(
            [(1, 10), (6, 20), (192, 30)]
                .into_iter()
                .map(|(hours, tokens)| {
                    serde_json::json!({
                "type":"response.completed", "timestamp":as_of-chrono::Duration::hours(hours),
                "response":{"id":format!("synthetic-{hours}"),"model":"synthetic-model",
                    "usage":{"input_tokens":tokens,"output_tokens":1,"total_tokens":tokens+1}}
            }).to_string()
                }),
        );
        std::fs::write(sessions.join("synthetic.jsonl"), lines.join("\n"))
            .unwrap_or_else(|e| panic!("write: {e}"));
        let report = read(&source, as_of).unwrap_or_else(|e| panic!("scan: {e:?}"));
        assert_eq!(report.as_of, as_of);
        assert_eq!(report.usage.five_hour.total_input_tokens, 10);
        assert_eq!(report.usage.seven_day.total_input_tokens, 30);
        assert_eq!(report.usage.all_time.total_input_tokens, 60);
        assert_eq!(report.usage.all_time.total_requests, 3);
        assert_eq!(report.diagnostics.files, 1);
        assert!(report.diagnostics.is_complete());
        assert!(
            roots
                .codex_runtime
                .join(".ccr-codex-usage-cache.json")
                .is_file()
        );
        std::fs::write(sessions.join("invalid.jsonl"), "invalid json")
            .unwrap_or_else(|e| panic!("write: {e}"));
        assert!(
            !read(&source, as_of)
                .unwrap_or_else(|e| panic!("scan: {e:?}"))
                .diagnostics
                .is_complete()
        );
    }
}
