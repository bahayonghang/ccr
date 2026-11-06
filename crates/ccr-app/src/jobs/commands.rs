//! App-owned command admission, bounded output and managed-process completion.
//!
//! Extracted from the existing desktop command owner. Adapters retain their wire
//! DTOs/event emitters; only this owner publishes terminal snapshots after cleanup.
use std::{
    collections::{HashMap, VecDeque},
    ffi::OsString,
    process::Stdio,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, BufReader},
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::{
    policy::{CommandExecutionRequest, validate_command_request},
    process::{ProcessDescriptor, ProcessGateway, SidecarContext, read_bounded_line},
};
use crate::runtime::{Progress, ProgressReporter, RequestKey};

pub const MAX_JOBS: usize = 64;
pub const TTL: Duration = Duration::from_secs(30 * 60);
pub const MAX_LINES: usize = 500;
pub const MAX_BYTES: usize = 512 * 1024;
const OUTPUT_CAPACITY: usize = 256;
const BATCH_LINES: usize = 50;
const BATCH_INTERVAL: Duration = Duration::from_millis(100);
const CLEANUP: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
    TimedOut,
    CleanupFailed,
    Unavailable,
}
impl Status {
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Queued | Self::Running)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stdout,
    Stderr,
    System,
}

#[derive(Clone, Debug)]
pub struct Delta {
    pub job_id: String,
    pub seq: u64,
    pub channel: Channel,
    pub lines: Vec<String>,
    pub dropped_count: usize,
    pub status: Option<Status>,
}
struct OutputBatch {
    channel: Channel,
    lines: Vec<String>,
    dropped_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub job_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: Status,
    pub cancel_requested: bool,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout_lines: VecDeque<String>,
    pub stderr_lines: VecDeque<String>,
    pub system_lines: VecDeque<String>,
    pub truncated: bool,
    pub dropped_lines: usize,
    pub error: Option<String>,
}
impl Snapshot {
    fn queued(request: CommandExecutionRequest) -> Self {
        Self {
            job_id: format!("ccr-command-{}", uuid::Uuid::new_v4()),
            command: request.command,
            args: request.args,
            status: Status::Queued,
            cancel_requested: false,
            started_at: Utc::now().to_rfc3339(),
            finished_at: None,
            duration_ms: None,
            exit_code: None,
            stdout_lines: VecDeque::new(),
            stderr_lines: VecDeque::new(),
            system_lines: VecDeque::from(["Job queued".into()]),
            truncated: false,
            dropped_lines: 0,
            error: None,
        }
    }
    fn push(&mut self, channel: Channel, text: String) {
        if self.status.is_terminal() {
            return;
        }
        let text = ccr_core::core::log_redact::redact_log_text(&text);
        let (text, clipped) = clip_line(text);
        self.truncated |= clipped;
        let lines = match channel {
            Channel::Stdout => &mut self.stdout_lines,
            Channel::Stderr => &mut self.stderr_lines,
            Channel::System => &mut self.system_lines,
        };
        lines.push_back(text);
        while lines.len() > MAX_LINES || lines.iter().map(String::len).sum::<usize>() > MAX_BYTES {
            lines.pop_front();
            self.truncated = true;
            self.dropped_lines += 1;
        }
    }
}

struct Entry {
    key: RequestKey,
    snapshot: Snapshot,
    cancel: CancellationToken,
    reporter: Option<ProgressReporter>,
    updates: broadcast::Sender<Delta>,
    sequence: u64,
}
#[derive(Default)]
pub struct CommandJobs {
    entries: Mutex<HashMap<String, Entry>>,
}
pub struct Admission {
    pub snapshot: Snapshot,
    pub ticket: Option<Ticket>,
}
/// A detachable observer. The retained runner remains owned by the application.
pub struct Observation {
    owner: Arc<CommandJobs>,
    updates: broadcast::Receiver<Delta>,
    finished: bool,
    job_id: String,
}
impl Observation {
    pub async fn next(&mut self) -> Result<Option<Snapshot>, Error> {
        if self.finished {
            return Ok(None);
        }
        match self.updates.recv().await {
            Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {
                // Deltas are wakeups; the owner snapshot is authoritative even
                // when output has overflowed the bounded notification channel.
                let snapshot = self.owner.status(&self.job_id)?;
                self.finished = snapshot.status.is_terminal();
                Ok(Some(snapshot))
            }
            Err(broadcast::error::RecvError::Closed) => Err(Error::Missing),
        }
    }
}
pub struct Ticket {
    job_id: String,
    cancel: CancellationToken,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Full,
    Missing,
    Unavailable,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Invalid => "Command or arguments are not allowed. / 命令或参数不受支持",
            Self::Full => "Command job capacity is full. / 命令任务已达上限",
            Self::Missing => "Command job is no longer available. / 命令任务不存在",
            Self::Unavailable => {
                "The trusted matching CCR sidecar is unavailable. / 可信 CCR sidecar 不可用"
            }
        }
    }
}

impl CommandJobs {
    /// Native blocking worker entry; consumes the application's existing runtime.
    pub fn run_blocking(
        &self,
        ticket: Ticket,
        app_cancel: CancellationToken,
        context: &SidecarContext,
    ) -> Result<Snapshot, Error> {
        let handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                self.abandon(ticket);
                return Err(Error::Unavailable);
            }
        };
        handle.block_on(self.run(ticket, app_cancel, context))
    }
    /// Trusted adapter fixture entry. The adapter must build a closed owned
    /// fixture program; renderer input must never choose the executable.
    pub fn run_fixture_blocking(
        &self,
        ticket: Ticket,
        app_cancel: CancellationToken,
        command: tokio::process::Command,
    ) -> Result<Snapshot, Error> {
        let handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                self.abandon(ticket);
                return Err(Error::Unavailable);
            }
        };
        handle.block_on(self.run_command(
            ticket,
            app_cancel,
            ProcessDescriptor::ccr_command(),
            command,
            Instant::now(),
        ))
    }
    pub fn admit(
        &self,
        key: RequestKey,
        request: CommandExecutionRequest,
    ) -> Result<Admission, Error> {
        validate_command_request(&request).map_err(|_| Error::Invalid)?;
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        prune(&mut entries, Utc::now());
        if let Some(entry) = entries.values().find(|entry| entry.key == key) {
            if entry.snapshot.command != request.command || entry.snapshot.args != request.args {
                return Err(Error::Invalid);
            }
            return Ok(Admission {
                snapshot: entry.snapshot.clone(),
                ticket: None,
            });
        }
        if entries.len() >= MAX_JOBS {
            let oldest = entries
                .iter()
                .filter(|(_, entry)| entry.snapshot.status.is_terminal())
                .min_by_key(|(_, entry)| entry.snapshot.finished_at.clone())
                .map(|(id, _)| id.clone());
            if let Some(oldest) = oldest {
                entries.remove(&oldest);
            }
        }
        if entries.len() >= MAX_JOBS {
            return Err(Error::Full);
        }
        let snapshot = Snapshot::queued(request);
        let cancel = CancellationToken::new();
        entries.insert(
            snapshot.job_id.clone(),
            Entry {
                key,
                snapshot: snapshot.clone(),
                cancel: cancel.clone(),
                reporter: None,
                updates: broadcast::channel(128).0,
                sequence: 0,
            },
        );
        Ok(Admission {
            ticket: Some(Ticket {
                job_id: snapshot.job_id.clone(),
                cancel,
            }),
            snapshot,
        })
    }
    pub fn status(&self, id: &str) -> Result<Snapshot, Error> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        prune(&mut entries, Utc::now());
        entries
            .get(id)
            .map(|entry| entry.snapshot.clone())
            .ok_or(Error::Missing)
    }
    pub fn subscribe(&self, id: &str) -> Result<broadcast::Receiver<Delta>, Error> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .map(|entry| entry.updates.subscribe())
            .ok_or(Error::Missing)
    }
    /// Subscribe and restore a snapshot atomically so immediate completion
    /// cannot fall between a status read and notification attachment.
    pub fn observe(self: &Arc<Self>, id: &str) -> Result<(Snapshot, Observation), Error> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        prune(&mut entries, Utc::now());
        let entry = entries.get(id).ok_or(Error::Missing)?;
        Ok((
            entry.snapshot.clone(),
            Observation {
                owner: self.clone(),
                updates: entry.updates.subscribe(),
                finished: entry.snapshot.status.is_terminal(),
                job_id: id.to_owned(),
            },
        ))
    }
    pub fn prune(&self) -> usize {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = entries.len();
        prune(&mut entries, Utc::now());
        before - entries.len()
    }
    /// In-memory notifications are coalesced by the existing app runtime.
    pub fn report_to(&self, id: &str, reporter: ProgressReporter) {
        self.update(id, |entry| entry.reporter = Some(reporter));
    }
    pub fn request_cancel(&self, id: &str) -> Result<Snapshot, Error> {
        self.update(id, |entry| {
            if !entry.snapshot.status.is_terminal() && !entry.snapshot.cancel_requested {
                entry.snapshot.cancel_requested = true;
                entry
                    .snapshot
                    .push(Channel::System, "Cancel requested".into());
                entry.cancel.cancel();
                publish(
                    entry,
                    Channel::System,
                    vec!["Cancel requested".into()],
                    0,
                    None,
                );
            }
        })
        .ok_or(Error::Missing)
    }
    /// Release an admission for which the app executor could not retain a runner.
    pub fn abandon(&self, ticket: Ticket) {
        let _ = self.abandon_snapshot(ticket);
    }
    /// Return the authoritative terminal while still holding the registry lock.
    /// Persistence must retain this value instead of looking up an evictable job.
    pub fn abandon_snapshot(&self, ticket: Ticket) -> Option<Snapshot> {
        self.complete(
            &ticket.job_id,
            Status::Unavailable,
            Instant::now(),
            None,
            Some("Command executor unavailable".into()),
        )
    }
    fn update(&self, id: &str, f: impl FnOnce(&mut Entry)) -> Option<Snapshot> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = entries.get_mut(id)?;
        f(entry);
        Some(entry.snapshot.clone())
    }
    fn complete(
        &self,
        id: &str,
        status: Status,
        started: Instant,
        code: Option<i32>,
        error: Option<String>,
    ) -> Option<Snapshot> {
        self.update(id, |entry| {
            if entry.snapshot.status.is_terminal() {
                return;
            }
            entry.snapshot.status = status;
            entry.snapshot.finished_at = Some(Utc::now().to_rfc3339());
            entry.snapshot.duration_ms =
                Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
            entry.snapshot.exit_code = code;
            entry.snapshot.error = error;
            entry.reporter = None;
            publish(entry, Channel::System, Vec::new(), 0, Some(status));
        })
    }
    pub async fn run(
        &self,
        ticket: Ticket,
        app_cancel: CancellationToken,
        context: &SidecarContext,
    ) -> Result<Snapshot, Error> {
        let started = Instant::now();
        if ticket.cancel.is_cancelled() || app_cancel.is_cancelled() {
            return self
                .complete(&ticket.job_id, Status::Cancelled, started, None, None)
                .ok_or(Error::Missing);
        }
        // Existing gateway owns sidecar identity/hash/version policy. Never PATH.
        if verify_sidecar(context).await.is_err() {
            return self
                .complete(
                    &ticket.job_id,
                    Status::Unavailable,
                    started,
                    None,
                    Some(Error::Unavailable.message().into()),
                )
                .ok_or(Error::Missing);
        }
        if ticket.cancel.is_cancelled() || app_cancel.is_cancelled() {
            return self
                .complete(&ticket.job_id, Status::Cancelled, started, None, None)
                .ok_or(Error::Missing);
        }
        let snapshot = self.status(&ticket.job_id)?;
        let descriptor = ProcessDescriptor::ccr_command();
        let mut command = match ProcessGateway::command_with_context(&descriptor, context) {
            Ok(command) => command,
            Err(_) => {
                return self
                    .complete(
                        &ticket.job_id,
                        Status::Unavailable,
                        started,
                        None,
                        Some(Error::Unavailable.message().into()),
                    )
                    .ok_or(Error::Missing);
            }
        };
        command.arg(snapshot.command).args(snapshot.args);
        self.run_command(ticket, app_cancel, descriptor, command, started)
            .await
    }
    /// Trusted native fixture/adaptor seam. The UI never supplies a program.
    /// Production CCR execution must enter through `run` and its sidecar check.
    pub async fn run_command(
        &self,
        ticket: Ticket,
        app_cancel: CancellationToken,
        descriptor: ProcessDescriptor,
        mut command: tokio::process::Command,
        started: Instant,
    ) -> Result<Snapshot, Error> {
        if ticket.cancel.is_cancelled() || app_cancel.is_cancelled() {
            return self
                .complete(&ticket.job_id, Status::Cancelled, started, None, None)
                .ok_or(Error::Missing);
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child =
            match ProcessGateway::spawn(command, &descriptor, ticket.cancel.clone(), Vec::new()) {
                Ok(child) => child,
                Err(_) => {
                    return self
                        .complete(
                            &ticket.job_id,
                            Status::Failed,
                            started,
                            None,
                            Some("Command could not start".into()),
                        )
                        .ok_or(Error::Missing);
                }
            };
        self.update(&ticket.job_id, |entry| {
            entry.snapshot.status = Status::Running;
            entry
                .snapshot
                .push(Channel::System, "Process started".into());
            publish(
                entry,
                Channel::System,
                vec!["Process started".into()],
                0,
                Some(Status::Running),
            );
        });
        let (tx, mut rx) = mpsc::channel(OUTPUT_CAPACITY);
        let stdout_dropped = Arc::new(AtomicUsize::new(0));
        let stderr_dropped = Arc::new(AtomicUsize::new(0));
        let mut readers = Vec::new();
        if let Some(stdout) = child.take_stdout() {
            readers.push(tokio::spawn(stream_reader(
                stdout,
                Channel::Stdout,
                tx.clone(),
                stdout_dropped.clone(),
            )));
        }
        if let Some(stderr) = child.take_stderr() {
            readers.push(tokio::spawn(stream_reader(
                stderr,
                Channel::Stderr,
                tx.clone(),
                stderr_dropped.clone(),
            )));
        }
        drop(tx);
        let deadline = tokio::time::sleep_until(tokio::time::Instant::now() + descriptor.timeout());
        tokio::pin!(deadline);
        let mut status = Status::Running;
        let mut code = None;
        let mut wait_finished = false;
        while !wait_finished || !rx.is_closed() || !rx.is_empty() {
            tokio::select! {
                biased;
                _ = ticket.cancel.cancelled(), if !status.is_terminal() => { status = Status::Cancelled; break; },
                _ = app_cancel.cancelled(), if !status.is_terminal() => { status = Status::Cancelled; break; },
                _ = &mut deadline => { status = Status::TimedOut; break; },
                event = rx.recv(), if !rx.is_closed() || !rx.is_empty() => { if let Some(event) = event { self.append_batch(&ticket.job_id, event); } },
                result = child.wait(), if !wait_finished => {
                    match result { Ok(exit) => code = Some(exit.code().unwrap_or(-1)), Err(_) => { status = Status::CleanupFailed; break; } }
                    wait_finished = true;
                },
            }
        }
        let cleanup_deadline = tokio::time::Instant::now() + CLEANUP;
        if !wait_finished {
            match tokio::time::timeout_at(cleanup_deadline, child.terminate_tree(CLEANUP)).await {
                Ok(Ok(exit)) => code = Some(exit.code().unwrap_or(-1)),
                _ => status = Status::CleanupFailed,
            }
        }
        let readers_clean = join_readers(readers, cleanup_deadline).await;
        if !readers_clean {
            status = Status::CleanupFailed;
        }
        while let Ok(event) = rx.try_recv() {
            self.append_batch(&ticket.job_id, event);
        }
        let dropped = stdout_dropped
            .load(Ordering::Relaxed)
            .saturating_add(stderr_dropped.load(Ordering::Relaxed));
        self.update(&ticket.job_id, |entry| {
            entry.snapshot.dropped_lines = entry.snapshot.dropped_lines.saturating_add(dropped);
            entry.snapshot.truncated |= dropped > 0;
        });
        if !status.is_terminal() {
            status = if code == Some(0) {
                Status::Success
            } else {
                Status::Failed
            };
        }
        let error = match status {
            Status::TimedOut => Some("Command timed out".into()),
            Status::CleanupFailed => Some("Command cleanup could not be confirmed".into()),
            Status::Failed => Some("Command exited unsuccessfully".into()),
            _ => None,
        };
        self.complete(&ticket.job_id, status, started, code, error)
            .ok_or(Error::Missing)
    }
    #[cfg(test)]
    fn append(&self, id: &str, channel: Channel, lines: Vec<String>) {
        self.append_batch(
            id,
            OutputBatch {
                channel,
                lines,
                dropped_count: 0,
            },
        );
    }
    fn append_batch(&self, id: &str, event: OutputBatch) {
        self.update(id, |entry| {
            if entry.snapshot.status.is_terminal() {
                return;
            }
            let emitted = event
                .lines
                .iter()
                .map(|line| ccr_core::core::log_redact::redact_log_text(line))
                .collect();
            for line in event.lines {
                entry.snapshot.push(event.channel, line);
            }
            entry.snapshot.dropped_lines = entry
                .snapshot
                .dropped_lines
                .saturating_add(event.dropped_count);
            entry.snapshot.truncated |= event.dropped_count > 0;
            publish(entry, event.channel, emitted, event.dropped_count, None);
            if let Some(reporter) = &entry.reporter {
                reporter.report(Progress {
                    completed: (entry.snapshot.stdout_lines.len()
                        + entry.snapshot.stderr_lines.len()
                        + entry.snapshot.dropped_lines) as u64,
                    total: None,
                });
            }
        });
    }
}

fn prune(entries: &mut HashMap<String, Entry>, now: DateTime<Utc>) {
    entries.retain(|_, entry| {
        !entry.snapshot.status.is_terminal()
            || !entry
                .snapshot
                .finished_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_some_and(|finished| {
                    now.signed_duration_since(finished).num_seconds() >= TTL.as_secs() as i64
                })
    });
}
fn clip_line(mut line: String) -> (String, bool) {
    if line.len() <= MAX_BYTES {
        return (line, false);
    }
    let mut end = MAX_BYTES - '…'.len_utf8();
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    line.truncate(end);
    line.push('…');
    (line, true)
}
async fn verify_sidecar(context: &SidecarContext) -> Result<(), Error> {
    let output = ProcessGateway::execute_with_context(
        &ProcessDescriptor::ccr_version_probe(),
        context,
        &[OsString::from("--version")],
    )
    .await
    .map_err(|_| Error::Unavailable)?;
    if output.timed_out
        || output.stdout_truncated
        || output.stderr_truncated
        || !output.status.success()
    {
        return Err(Error::Unavailable);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|ch| ch.is_ascii_digit()));
    if version != Some(context.expected_version.as_str()) {
        return Err(Error::Unavailable);
    }
    Ok(())
}
async fn stream_reader<R: AsyncRead + Unpin>(
    reader: R,
    channel: Channel,
    tx: mpsc::Sender<OutputBatch>,
    dropped: Arc<AtomicUsize>,
) -> bool {
    let mut reader = BufReader::new(reader);
    let mut batch = Vec::with_capacity(BATCH_LINES);
    let mut interval = tokio::time::interval(BATCH_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval.tick().await;
    loop {
        tokio::select! {
            next = read_bounded_line(&mut reader, MAX_BYTES) => match next {
                Ok(Some(line)) => {
                    let text = if line.truncated { dropped.fetch_add(1, Ordering::Relaxed); let (mut text,_) = clip_line(line.text); if !text.ends_with('…') { let mut end = text.len().saturating_sub('…'.len_utf8()); while !text.is_char_boundary(end) { end -= 1; } text.truncate(end); text.push('…'); } text } else { line.text };
                    if batch.len() < BATCH_LINES { batch.push(text); } else { dropped.fetch_add(1, Ordering::Relaxed); }
                },
                Ok(None) => { flush(&tx, channel, &mut batch, &dropped); return true; },
                Err(_) => { flush(&tx, channel, &mut batch, &dropped); return false; },
            },
            _ = interval.tick() => { if !flush(&tx, channel, &mut batch, &dropped) { return true; } },
        }
    }
}
fn flush(
    tx: &mpsc::Sender<OutputBatch>,
    channel: Channel,
    batch: &mut Vec<String>,
    dropped: &AtomicUsize,
) -> bool {
    if batch.is_empty() {
        return true;
    }
    let event = OutputBatch {
        channel,
        lines: std::mem::take(batch),
        dropped_count: dropped.swap(0, Ordering::AcqRel),
    };
    match tx.try_send(event) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Full(event)) => {
            dropped.fetch_add(
                event.lines.len().saturating_add(event.dropped_count),
                Ordering::Relaxed,
            );
            true
        }
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}
fn publish(
    entry: &mut Entry,
    channel: Channel,
    lines: Vec<String>,
    dropped_count: usize,
    status: Option<Status>,
) {
    let delta = Delta {
        job_id: entry.snapshot.job_id.clone(),
        seq: entry.sequence,
        channel,
        lines,
        dropped_count,
        status,
    };
    entry.sequence = entry.sequence.saturating_add(1);
    let _ = entry.updates.send(delta);
}
async fn join_readers(readers: Vec<JoinHandle<bool>>, deadline: tokio::time::Instant) -> bool {
    let mut clean = true;
    for mut reader in readers {
        match tokio::time::timeout_at(deadline, &mut reader).await {
            Ok(Ok(true)) => {}
            Ok(_) => clean = false,
            Err(_) => {
                clean = false;
                reader.abort();
                let _ = reader.await;
            }
        }
    }
    clean
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
