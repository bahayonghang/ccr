//! Captured SSH CLI execution and private, consistent read-only usage snapshots.
use super::cli::{STDERR_MAX_LINE_BYTES, STDERR_TAIL_LINES, STDOUT_MAX_LINE_BYTES};
use super::{
    JobEvent, SyncCommandOptions, SyncSummaryEvent, error::LlmusageAdapterError,
    events::parse_ndjson_event,
};
use crate::{
    environments::{
        io::{EnvironmentIo, EnvironmentPath, Error},
        ssh_security::RemotePosixPath,
    },
    jobs::process::ProcessDescriptor,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

fn failure(error: Error) -> LlmusageAdapterError {
    match error {
        Error::Cancelled | Error::Ssh("ssh_cancelled") => LlmusageAdapterError::Cancelled,
        Error::Ssh("ssh_cleanup_failed" | "ssh_network_error") => {
            LlmusageAdapterError::CleanupFailed("remote_exit_or_cleanup_unconfirmed".into())
        }
        _ => LlmusageAdapterError::Cli("remote_usage_unavailable".into()),
    }
}
fn execution_failure(error: Error) -> LlmusageAdapterError {
    if matches!(error, Error::Cancelled | Error::Ssh("ssh_cancelled")) {
        LlmusageAdapterError::CleanupFailed("remote_exit_unconfirmed".into())
    } else {
        failure(error)
    }
}
fn home(io: &EnvironmentIo) -> Result<String, LlmusageAdapterError> {
    match io.roots().home() {
        EnvironmentPath::Ssh(home) => Ok(home.as_str().trim_end_matches('/').into()),
        _ => Err(LlmusageAdapterError::Cli(
            "remote_environment_required".into(),
        )),
    }
}
/// A fresh cache directory belongs to one host and one request, and is removed on drop.
pub struct Snapshot {
    directory: tempfile::TempDir,
}
impl Snapshot {
    pub fn paths(&self) -> ccr_usage::AppPaths {
        ccr_usage::AppPaths::from_root(self.directory.path())
    }
    pub async fn capture(
        io: &EnvironmentIo,
        cache: &Path,
        cancel: CancellationToken,
    ) -> Result<Self, LlmusageAdapterError> {
        if cancel.is_cancelled() {
            return Err(LlmusageAdapterError::Cancelled);
        }
        io.check_authority().map_err(failure)?;
        let root = format!("{}/.llmusage", home(io)?);
        let descriptor = ProcessDescriptor::usage_snapshot();
        let output = io
            .execute(
                &descriptor,
                "python3",
                &[
                    "-c".into(),
                    ccr_usage::snapshot::BACKUP_SCRIPT.into(),
                    root.clone(),
                ],
                // 已发私有 backup 必须在原 descriptor 期限内终结并给出精确清理路径。
                // 视图取消只取消后续读取，不能用 mutating exec 的取消撤销共享连接。
                CancellationToken::new(),
            )
            .await
            .map_err(execution_failure)?;
        if !output.success {
            return Err(LlmusageAdapterError::Cli(
                "remote_snapshot_requires_python3_sqlite_and_readable_database".into(),
            ));
        }
        let stdout = std::str::from_utf8(&output.stdout)
            .map_err(|_| LlmusageAdapterError::Cli("remote_snapshot_path".into()))?;
        if output.stderr.is_empty()
            && stdout
                .lines()
                .eq([ccr_usage::snapshot::DB_MISSING_RESPONSE])
        {
            io.check_authority().map_err(failure)?;
            if cancel.is_cancelled() {
                return Err(LlmusageAdapterError::Cancelled);
            }
            return Err(LlmusageAdapterError::DbMissing(
                Path::new(&root).join("llmusage.db"),
            ));
        }
        let target = stdout.trim();
        let prefix = format!("{root}/.ccr-snapshot-");
        let suffix = target
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix("/llmusage.db"));
        if suffix.is_none_or(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        }) {
            return Err(LlmusageAdapterError::CleanupFailed(
                "remote_snapshot_path_unconfirmed".into(),
            ));
        }
        let result = async {
            let remote = io
                .at_remote_root(
                    RemotePosixPath::root(Some(&root))
                        .map_err(|_| LlmusageAdapterError::Cli("remote_snapshot_path".into()))?,
                )
                .map_err(failure)?;
            let relative = target
                .strip_prefix(&format!("{root}/"))
                .ok_or_else(|| LlmusageAdapterError::Cli("remote_snapshot_path".into()))?;
            let content = remote
                .read_usage_bytes(relative, cancel.clone())
                .await
                .map_err(failure)?;
            let cache = io.cache_root(cache).map_err(failure)?;
            std::fs::create_dir_all(&cache)
                .map_err(|_| LlmusageAdapterError::Cli("snapshot_cache".into()))?;
            ccr_core::core::guarded_write::restrict_owner_only_directory(&cache)
                .map_err(|_| LlmusageAdapterError::Cli("snapshot_permissions".into()))?;
            let directory = tempfile::Builder::new()
                .prefix("usage-")
                .tempdir_in(cache)
                .map_err(|_| LlmusageAdapterError::Cli("snapshot_cache".into()))?;
            ccr_core::core::guarded_write::restrict_owner_only_directory(directory.path())
                .map_err(|_| LlmusageAdapterError::Cli("snapshot_permissions".into()))?;
            let snapshot = Self { directory };
            ccr_core::core::LockManager::with_scoped_directory(
                &snapshot.directory.path().join(".locks"),
                || {
                    ccr_core::core::write_guarded(
                        &snapshot.paths().db_path,
                        &content,
                        &ccr_core::core::WriteOptions {
                            secret: true,
                            ..Default::default()
                        },
                    )
                },
            )
            .map_err(|_| LlmusageAdapterError::Cli("snapshot_cache".into()))?;
            ccr_usage::snapshot::validate(&snapshot.paths()).map_err(|error| match error {
                ccr_usage::UsageError::SchemaUnsupported { expected, actual } => {
                    LlmusageAdapterError::SchemaUnsupported { expected, actual }
                }
                ccr_usage::UsageError::FeatureUnavailable { feature, reason } => {
                    LlmusageAdapterError::FeatureUnavailable { feature, reason }
                }
                _ => LlmusageAdapterError::DbUnreadable("remote_snapshot_integrity".into()),
            })?;
            Ok(snapshot)
        }
        .await;
        // 取消读取后仍等待精确清理；旧 authority 绝不向同 host 新连接借权限。
        let cleanup = io
            .execute(
                &descriptor,
                "python3",
                &[
                    "-c".into(),
                    ccr_usage::snapshot::CLEANUP_SCRIPT.into(),
                    target.into(),
                ],
                CancellationToken::new(),
            )
            .await;
        if !cleanup.is_ok_and(|output| output.success) {
            return Err(LlmusageAdapterError::CleanupFailed(
                "remote_snapshot_cleanup_unconfirmed".into(),
            ));
        }
        io.check_authority().map_err(failure)?;
        if cancel.is_cancelled() {
            return Err(LlmusageAdapterError::Cancelled);
        }
        result
    }
}

#[derive(Default)]
struct Decoder {
    line: Vec<u8>,
    summary: Option<SyncSummaryEvent>,
    error: Option<LlmusageAdapterError>,
    stderr_line: Vec<u8>,
    stderr_tail: std::collections::VecDeque<Vec<u8>>,
}
impl Decoder {
    fn stderr(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if *byte == b'\n' {
                if self.stderr_tail.len() >= STDERR_TAIL_LINES {
                    self.stderr_tail.pop_front();
                }
                self.stderr_tail
                    .push_back(std::mem::take(&mut self.stderr_line));
            } else if self.stderr_line.len() < STDERR_MAX_LINE_BYTES {
                self.stderr_line.push(*byte);
            }
        }
    }
    fn line(&mut self, on_event: &dyn Fn(JobEvent)) {
        let result = std::str::from_utf8(&self.line)
            .map_err(|_| "remote_ndjson_utf8".into())
            .and_then(parse_ndjson_event);
        self.line.clear();
        match result {
            Ok(Some(JobEvent::Finished { summary })) => {
                if self.summary.is_none() {
                    self.summary = Some(summary);
                }
            }
            Ok(Some(JobEvent::Cancelled)) => self.error = Some(LlmusageAdapterError::Cancelled),
            Ok(Some(JobEvent::Failed { .. })) | Err(_) => {
                self.error = Some(LlmusageAdapterError::Cli("remote_ndjson_protocol".into()))
            }
            Ok(Some(event)) if self.summary.is_none() => on_event(event),
            _ => {}
        }
    }
    fn chunk(&mut self, bytes: &[u8], on_event: &dyn Fn(JobEvent)) {
        for byte in bytes {
            if self.error.is_some() {
                return;
            }
            if *byte == b'\n' {
                self.line(on_event);
            } else if self.line.len() < STDOUT_MAX_LINE_BYTES {
                self.line.push(*byte);
            } else {
                self.error = Some(LlmusageAdapterError::Cli(
                    "llmusage_stdout_line_too_long".into(),
                ));
            }
        }
    }
}

/// The existing NDJSON parser receives bounded live lines; Finished is deferred
/// until the exec channel, stdout and stderr have all reached terminal state.
pub async fn run_sync(
    io: &EnvironmentIo,
    options: SyncCommandOptions,
    cancel: CancellationToken,
    on_event: Arc<dyn Fn(JobEvent) + Send + Sync>,
) -> Result<SyncSummaryEvent, LlmusageAdapterError> {
    if cancel.is_cancelled() {
        return Err(LlmusageAdapterError::Cancelled);
    }
    let cli = io
        .detect_execution_cli(cancel.clone())
        .await
        .map_err(failure)?
        .into_iter()
        .find(|cli| cli.name == "llmusage")
        .and_then(|cli| cli.path)
        .ok_or(LlmusageAdapterError::CliMissing)?;
    let mut args = vec![
        "RUST_LOG=off".into(),
        "NO_COLOR=1".into(),
        cli.clone(),
        "--home".into(),
        format!("{}/.llmusage", home(io)?),
        "sync".into(),
        "--json-events".into(),
    ];
    args.extend(
        super::cli::sync_option_args(&options)
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned()),
    );
    let decoder = Arc::new(Mutex::new(Decoder::default()));
    let observer_decoder = decoder.clone();
    let observer_cancel = cancel.child_token();
    let execution_cancel = observer_cancel.clone();
    let progress = on_event.clone();
    let observer = Arc::new(move |stderr: bool, bytes: &[u8]| {
        let mut decoder = observer_decoder.lock().unwrap_or_else(|p| p.into_inner());
        if stderr {
            decoder.stderr(bytes);
        } else {
            decoder.chunk(bytes, progress.as_ref());
            if decoder.error.is_some() {
                observer_cancel.cancel();
            }
        }
    });
    let descriptor = ProcessDescriptor::llmusage(&cli).map_err(LlmusageAdapterError::Cli)?;
    let output = io
        .execute_usage_stream(&descriptor, "env", &args, execution_cancel, observer)
        .await
        .map_err(execution_failure)?;
    let mut decoder = decoder
        .lock()
        .map_err(|_| LlmusageAdapterError::Cli("remote_decoder".into()))?;
    if !decoder.line.is_empty() {
        decoder.line(on_event.as_ref());
    }
    if let Some(error) = decoder.error.take() {
        return Err(error);
    }
    if !output.success {
        return Err(LlmusageAdapterError::Cli(
            "remote_llmusage_exit_failed".into(),
        ));
    }
    if cancel.is_cancelled() {
        return Err(LlmusageAdapterError::Cancelled);
    }
    decoder
        .summary
        .take()
        .ok_or_else(|| LlmusageAdapterError::Cli("llmusage_finished_event_missing".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_decoder_keeps_original_line_and_stderr_tail_budgets() {
        let mut decoder = Decoder::default();
        let mut line = vec![b'x'; STDERR_MAX_LINE_BYTES + 1];
        line.push(b'\n');
        for _ in 0..100 {
            decoder.stderr(&line);
        }
        assert_eq!(decoder.stderr_tail.len(), STDERR_TAIL_LINES);
        assert!(
            decoder
                .stderr_tail
                .iter()
                .all(|line| line.len() == STDERR_MAX_LINE_BYTES)
        );
        assert!(decoder.stderr_line.is_empty());
        decoder.chunk(&vec![b'x'; STDOUT_MAX_LINE_BYTES + 1], &|_| {});
        assert!(decoder.error.is_some());
        assert_eq!(decoder.line.len(), STDOUT_MAX_LINE_BYTES);
    }
}
