//! Install execution: child process spawn, event streaming, and cancellation.
//!
//! The executor spawns the planned command as a child process, streams stdout/stderr
//! as `Log` events, detects progress stages, and handles graceful-then-forceful
//! cancellation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::services::install_ring_buffer::{RingBufferHandle, redact};
use crate::services::install_types::{
    AttemptId, FailureKind, InstallEvent, InstallPlan, LogStream, ProgressStage,
};

/// Timeout for graceful termination before forceful kill.
const GRACEFUL_TIMEOUT_MS: u64 = 2000;

/// Spawn an install attempt and return a receiver for streamed events.
///
/// The caller owns the `CancellationToken` and can signal cancellation at any time.
pub fn run_attempt(
    plan: InstallPlan,
    attempt_id: AttemptId,
    cancel_token: CancellationToken,
    ring: RingBufferHandle,
) -> mpsc::Receiver<InstallEvent> {
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let span = tracing::info_span!("llmusage.install", %attempt_id);
        let _enter = span.enter();
        drop(_enter); // Don't hold span across awaits; use structured events instead.

        tracing::info!(
            %attempt_id,
            command = %plan.command,
            args = ?plan.args,
            platform = ?plan.platform,
            pm = ?plan.package_manager,
            "starting install attempt"
        );

        let start = Instant::now();

        // Spawn the child process.
        let mut child = match Command::new(&plan.command)
            .args(&plan.args)
            .envs(&plan.envs)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => {
                let ev = InstallEvent::Started {
                    attempt_id,
                    plan: plan.clone(),
                };
                emit(&tx, &ring, &ev).await;
                c
            }
            Err(e) => {
                let ev = InstallEvent::Failed {
                    attempt_id,
                    failure_kind: FailureKind::SpawnFailed,
                    exit_code: None,
                    stderr_excerpt: None,
                    error_message: format!("failed to spawn `{}`: {e}", plan.command),
                };
                emit(&tx, &ring, &ev).await;
                tracing::error!(%attempt_id, error = %e, "spawn failed");
                return;
            }
        };

        // Set up line readers for stdout and stderr.
        let seq = AtomicU64::new(0);

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let tx_stdout = tx.clone();
        let ring_stdout = ring.clone();
        let stdout_task = tokio::spawn(async move {
            if let Some(stdout) = stdout {
                pipe_lines(
                    stdout,
                    LogStream::Stdout,
                    attempt_id,
                    &seq,
                    &tx_stdout,
                    &ring_stdout,
                )
                .await;
            }
        });

        let tx_stderr = tx.clone();
        let ring_stderr = ring.clone();
        let seq_stderr = AtomicU64::new(10000); // offset to avoid collision
        let stderr_task = tokio::spawn(async move {
            if let Some(stderr) = stderr {
                pipe_lines(
                    stderr,
                    LogStream::Stderr,
                    attempt_id,
                    &seq_stderr,
                    &tx_stderr,
                    &ring_stderr,
                )
                .await;
            }
        });

        // Supervisor: race child.wait() against cancel_token.cancelled()
        let outcome = tokio::select! {
            status = child.wait() => {
                ChildOutcome::Exited(status.map(|s| s.code()))
            }
            _ = cancel_token.cancelled() => {
                let requested_at_ms = epoch_ms();
                graceful_then_forceful(&mut child).await;
                ChildOutcome::Cancelled { requested_at_ms }
            }
        };

        // Drain stdout/stderr readers before emitting terminal event.
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Emit terminal event.
        let terminal = match outcome {
            ChildOutcome::Exited(Ok(Some(0))) => InstallEvent::Succeeded {
                attempt_id,
                duration_ms,
                installed_version: None, // Could be probed post-install
            },
            ChildOutcome::Exited(Ok(code)) => InstallEvent::Failed {
                attempt_id,
                failure_kind: FailureKind::NonZeroExit,
                exit_code: code,
                stderr_excerpt: None,
                error_message: format!(
                    "`{} {}` exited with code {}",
                    plan.command,
                    plan.args.join(" "),
                    code.map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                ),
            },
            ChildOutcome::Exited(Err(e)) => InstallEvent::Failed {
                attempt_id,
                failure_kind: FailureKind::InternalError,
                exit_code: None,
                stderr_excerpt: None,
                error_message: format!("error waiting for child: {e}"),
            },
            ChildOutcome::Cancelled { requested_at_ms } => InstallEvent::Cancelled {
                attempt_id,
                requested_at_ms,
            },
        };

        tracing::info!(%attempt_id, kind = ?terminal_kind(&terminal), "install attempt finished");
        emit(&tx, &ring, &terminal).await;
    });

    rx
}

enum ChildOutcome {
    Exited(Result<Option<i32>, std::io::Error>),
    Cancelled { requested_at_ms: u64 },
}

/// Send graceful signal, wait up to GRACEFUL_TIMEOUT_MS, then forceful kill.
async fn graceful_then_forceful(child: &mut Child) {
    // First attempt: try to kill gracefully.
    // On Unix, we send SIGTERM via the kill syscall.
    // On Windows, start_kill sends TerminateProcess.
    #[cfg(unix)]
    {
        if let Some(pid) = child.id() {
            // SAFETY: sending SIGTERM to a known child process.
            // SIGTERM = 15 on all Unix platforms.
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }
        }
    }

    #[cfg(not(unix))]
    {
        // On Windows, start_kill is the only option (TerminateProcess).
        let _ = child.start_kill();
    }

    // Wait for graceful exit.
    let graceful = tokio::time::timeout(
        std::time::Duration::from_millis(GRACEFUL_TIMEOUT_MS),
        child.wait(),
    )
    .await;

    if graceful.is_err() {
        // Graceful timeout expired — forceful kill.
        let _ = child.kill().await;
    }
}

/// Pipe lines from an async reader to the event channel.
async fn pipe_lines<R: tokio::io::AsyncRead + Unpin>(
    reader: R,
    stream: LogStream,
    attempt_id: AttemptId,
    seq: &AtomicU64,
    tx: &mpsc::Sender<InstallEvent>,
    ring: &RingBufferHandle,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let redacted = redact(&line);
        let current_seq = seq.fetch_add(1, Ordering::Relaxed);

        // Detect progress stage from line content.
        if let Some(stage) = detect_progress_stage(&redacted) {
            let progress_ev = InstallEvent::Progress {
                attempt_id,
                stage,
                detail: Some(redacted.clone()),
            };
            emit(tx, ring, &progress_ev).await;
        }

        let ev = InstallEvent::Log {
            attempt_id,
            stream,
            line: redacted,
            seq: current_seq,
        };
        emit(tx, ring, &ev).await;
    }
}

/// Detect progress stage from a log line.
fn detect_progress_stage(line: &str) -> Option<ProgressStage> {
    let lower = line.to_lowercase();
    if lower.contains("downloading") || lower.contains("fetching") {
        Some(ProgressStage::Downloading)
    } else if lower.contains("compiling") || lower.contains("building") {
        Some(ProgressStage::Compiling)
    } else if lower.contains("installing") {
        Some(ProgressStage::Installing)
    } else if lower.contains("resolving") || lower.contains("updating") {
        Some(ProgressStage::Resolving)
    } else if lower.contains("finished") || lower.contains("installed") {
        Some(ProgressStage::Finalizing)
    } else {
        None
    }
}

/// Emit an event to both the channel and the ring buffer.
async fn emit(tx: &mpsc::Sender<InstallEvent>, ring: &RingBufferHandle, event: &InstallEvent) {
    ring.record(event);
    tracing::debug!(kind = ?terminal_kind(event), "emit install event");
    // Best-effort send; if the receiver is dropped, we just log.
    let _ = tx.send(event.clone()).await;
}

fn terminal_kind(event: &InstallEvent) -> &'static str {
    match event {
        InstallEvent::Started { .. } => "started",
        InstallEvent::Log { .. } => "log",
        InstallEvent::Progress { .. } => "progress",
        InstallEvent::Succeeded { .. } => "succeeded",
        InstallEvent::Failed { .. } => "failed",
        InstallEvent::Cancelled { .. } => "cancelled",
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
