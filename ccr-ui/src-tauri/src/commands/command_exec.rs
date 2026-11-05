//! Tauri wire/event adapter for shared command policy and app-owned jobs.
use crate::process::{ProcessDescriptor, ProcessGateway};
use ccr_app::{
    jobs::{
        commands as shared,
        policy::{self, CommandExecutionRequest, CommandPolicy, validate_command_request},
    },
    runtime::{EnvironmentId, Platform, RequestKey},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ffi::OsString,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU64, Ordering},
    },
};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

const EVENT_COMMAND_JOB_PROGRESS: &str = "commands:job-progress";
const EVENT_COMMAND_JOB_FINISHED: &str = "commands:job-finished";
const EVENT_COMMAND_JOB_CANCELLED: &str = "commands:job-cancelled";
static COMMAND_JOBS: LazyLock<Arc<shared::CommandJobs>> =
    LazyLock::new(|| Arc::new(shared::CommandJobs::default()));
static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandArgSchema {
    pub name: &'static str,
    pub label: &'static str,
    #[serde(rename = "type")]
    pub arg_type: &'static str,
    pub required: bool,
    pub placeholder: Option<&'static str>,
    pub source: Option<&'static str>,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandFlagSchema {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub label: &'static str,
    pub description: &'static str,
    #[serde(rename = "type")]
    pub flag_type: &'static str,
    pub takes_value: bool,
    pub default_value: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandInfo {
    pub name: &'static str,
    pub path: Vec<&'static str>,
    pub title: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub examples: Vec<&'static str>,
    pub category: &'static str,
    pub risk: &'static str,
    pub executable: bool,
    pub requires_confirmation: bool,
    pub args: Vec<CommandArgSchema>,
    pub flags: Vec<CommandFlagSchema>,
    pub aliases: Vec<&'static str>,
    pub related_route: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub enum CommandJobStatus {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
    CleanupFailed,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandJobSnapshot {
    pub job_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: CommandJobStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    #[ts(as = "Option<f64>")]
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    #[ts(as = "Vec<String>")]
    pub stdout_lines: VecDeque<String>,
    #[ts(as = "Vec<String>")]
    pub stderr_lines: VecDeque<String>,
    #[ts(as = "Vec<String>")]
    pub system_lines: VecDeque<String>,
    pub truncated: bool,
    pub dropped_lines: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct StartCommandJobResponse {
    pub job_id: String,
    pub snapshot: CommandJobSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub enum OutputChannel {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandJobDelta {
    pub job_id: String,
    #[ts(as = "f64")]
    pub seq: u64,
    pub channel: OutputChannel,
    pub lines: Vec<String>,
    pub dropped_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub status: Option<CommandJobStatus>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub output: String,
    pub error: String,
    pub exit_code: i32,
    #[ts(as = "f64")]
    pub duration_ms: u64,
    pub timed_out: bool,
    pub truncated: bool,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(transparent)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandCatalog(pub Vec<CommandInfo>);

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/command_exec/")]
pub struct CommandHelpResponse {
    pub command: String,
    pub help: String,
    pub stderr: String,
    pub success: bool,
    pub timed_out: bool,
    pub truncated: bool,
}

impl From<policy::CommandInfo> for CommandInfo {
    fn from(info: policy::CommandInfo) -> Self {
        Self {
            name: info.name,
            path: info.path,
            title: info.title,
            description: info.description,
            usage: info.usage,
            examples: info.examples,
            category: info.category,
            risk: info.risk,
            executable: info.executable,
            requires_confirmation: info.requires_confirmation,
            args: info
                .args
                .into_iter()
                .map(|arg| CommandArgSchema {
                    name: arg.name,
                    label: arg.label,
                    arg_type: arg.arg_type,
                    required: arg.required,
                    placeholder: arg.placeholder,
                    source: arg.source,
                    description: arg.description,
                })
                .collect(),
            flags: info
                .flags
                .into_iter()
                .map(|flag| CommandFlagSchema {
                    name: flag.name,
                    aliases: flag.aliases,
                    label: flag.label,
                    description: flag.description,
                    flag_type: flag.flag_type,
                    takes_value: flag.takes_value,
                    default_value: flag.default_value,
                })
                .collect(),
            aliases: info.aliases,
            related_route: info.related_route,
        }
    }
}
impl From<shared::Status> for CommandJobStatus {
    fn from(status: shared::Status) -> Self {
        match status {
            shared::Status::Queued => Self::Queued,
            shared::Status::Running => Self::Running,
            shared::Status::Success => Self::Success,
            shared::Status::Failed | shared::Status::TimedOut => Self::Failed,
            shared::Status::Cancelled => Self::Cancelled,
            shared::Status::CleanupFailed => Self::CleanupFailed,
            shared::Status::Unavailable => Self::Unavailable,
        }
    }
}
impl From<shared::Snapshot> for CommandJobSnapshot {
    fn from(snapshot: shared::Snapshot) -> Self {
        Self {
            job_id: snapshot.job_id,
            command: snapshot.command,
            args: snapshot.args,
            status: snapshot.status.into(),
            started_at: snapshot.started_at,
            finished_at: snapshot.finished_at,
            duration_ms: snapshot.duration_ms,
            exit_code: snapshot.exit_code,
            stdout_lines: snapshot.stdout_lines,
            stderr_lines: snapshot.stderr_lines,
            system_lines: snapshot.system_lines,
            truncated: snapshot.truncated,
            dropped_lines: snapshot.dropped_lines,
            error: if snapshot.status == shared::Status::TimedOut {
                Some("process_timeout".into())
            } else {
                snapshot.error
            },
        }
    }
}
impl From<shared::Delta> for CommandJobDelta {
    fn from(delta: shared::Delta) -> Self {
        Self {
            job_id: delta.job_id,
            seq: delta.seq,
            channel: match delta.channel {
                shared::Channel::Stdout => OutputChannel::Stdout,
                shared::Channel::Stderr => OutputChannel::Stderr,
                shared::Channel::System => OutputChannel::System,
            },
            lines: delta.lines,
            dropped_count: delta.dropped_count,
            status: delta.status.map(Into::into),
        }
    }
}
fn command_catalog() -> Vec<CommandInfo> {
    policy::command_catalog()
        .into_iter()
        .map(Into::into)
        .collect()
}
fn request_key() -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("local".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id: REQUEST_ID.fetch_add(1, Ordering::Relaxed),
    }
}
pub(crate) async fn prune_command_jobs() -> usize {
    COMMAND_JOBS.prune()
}
async fn request_command_job_cancel(job_id: String) -> Result<CommandJobSnapshot, String> {
    COMMAND_JOBS
        .request_cancel(&job_id)
        .map(Into::into)
        .map_err(|_| format!("Command job '{}' not found", job_id))
}

// The runtime-policy permit stays with the runner, through process and reader cleanup.
fn retain_admission(
    admission: tokio::sync::OwnedSemaphorePermit,
    runner: impl std::future::Future<Output = ()> + Send + 'static,
) {
    tauri::async_runtime::spawn(async move {
        runner.await;
        drop(admission);
    });
}
async fn emit_delta(app: &AppHandle, delta: shared::Delta) {
    // The final snapshot is emitted only after the runner has returned.
    if delta.status.is_some_and(shared::Status::is_terminal) {
        return;
    }
    if let Err(error) = app.emit(EVENT_COMMAND_JOB_PROGRESS, CommandJobDelta::from(delta)) {
        tracing::warn!(?error, "Failed to emit command job delta");
    }
}
async fn run_command_job(app: AppHandle, ticket: shared::Ticket, id: String) {
    let mut updates = COMMAND_JOBS.subscribe(&id).ok();
    let context = crate::process::gateway::sidecar_context();
    let runner = COMMAND_JOBS.run(ticket, CancellationToken::new(), &context);
    tokio::pin!(runner);
    let result = loop {
        tokio::select! {
            result = &mut runner => break result,
            update = async { match updates.as_mut() { Some(updates) => updates.recv().await,
                None => std::future::pending().await } } => match update {
                Ok(delta) => emit_delta(&app, delta).await,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                Err(tokio::sync::broadcast::error::RecvError::Closed) => updates = None,
            }
        }
    };
    if let Some(updates) = updates.as_mut() {
        while let Ok(delta) = updates.try_recv() {
            emit_delta(&app, delta).await;
        }
    }
    if let Ok(snapshot) = result {
        let event = if snapshot.status == shared::Status::Cancelled
            || (snapshot.cancel_requested && snapshot.status == shared::Status::CleanupFailed)
        {
            EVENT_COMMAND_JOB_CANCELLED
        } else {
            EVENT_COMMAND_JOB_FINISHED
        };
        if let Err(error) = app.emit(event, CommandJobSnapshot::from(snapshot)) {
            tracing::warn!(?error, job_id = %id, "Failed to emit command job terminal");
        }
    }
}

#[ccr_tauri_command_macros::command]
pub async fn start_ccr_command_job(
    app_handle: AppHandle,
    command: String,
    args: Option<Vec<String>>,
    confirmation_token: Option<String>,
) -> Result<StartCommandJobResponse, String> {
    let request = CommandExecutionRequest::background(command, args, confirmation_token);
    validate_command_request(&request)?;
    let permit = super::runtime_policy::take_background_admission("start_ccr_command_job")?;
    let admitted = COMMAND_JOBS
        .admit(request_key(), request)
        .map_err(|error| error.message().to_owned())?;
    let snapshot = CommandJobSnapshot::from(admitted.snapshot);
    let id = snapshot.job_id.clone();
    if let Some(ticket) = admitted.ticket {
        retain_admission(permit, run_command_job(app_handle, ticket, id.clone()));
    }
    Ok(StartCommandJobResponse {
        job_id: id,
        snapshot,
    })
}
#[ccr_tauri_command_macros::command]
pub async fn get_ccr_command_job_status(job_id: String) -> Result<CommandJobSnapshot, String> {
    COMMAND_JOBS
        .status(&job_id)
        .map(Into::into)
        .map_err(|_| format!("Command job '{}' not found", job_id))
}
#[ccr_tauri_command_macros::command]
pub async fn cancel_ccr_command_job(
    _app_handle: AppHandle,
    job_id: String,
) -> Result<CommandJobSnapshot, String> {
    request_command_job_cancel(job_id).await
}
#[ccr_tauri_command_macros::command]
pub async fn list_ccr_commands() -> Result<CommandCatalog, String> {
    Ok(CommandCatalog(command_catalog()))
}

async fn verify_ccr_sidecar_version() -> Result<(), String> {
    let descriptor = ProcessDescriptor::ccr_version_probe();
    let output = ProcessGateway::execute(&descriptor, &[OsString::from("--version")]).await?;
    if output.stdout_truncated || output.stderr_truncated {
        return Err("ccr_version_probe_truncated".into());
    }
    if output.timed_out {
        return Err("ccr_version_probe_timeout".to_string());
    }
    if !output.status.success() {
        return Err(format!(
            "ccr_version_probe_failed: exit code {:?}",
            output.status.code()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = parse_ccr_version_output(&stdout)
        .ok_or_else(|| "ccr_version_probe_invalid_output".to_string())?;
    let expected = env!("CARGO_PKG_VERSION");
    if actual != expected {
        return Err(format!(
            "ccr_version_mismatch: desktop {expected}, sidecar {actual}"
        ));
    }
    Ok(())
}

fn parse_ccr_version_output(output: &str) -> Option<&str> {
    output
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
}

/// 执行白名单内的 CCR CLI 子命令并返回输出
///
/// 返回 `{ success, stdout, stderr, output, error, exit_code, duration_ms }`
#[ccr_tauri_command_macros::command]
pub async fn execute_ccr_command(
    command: String,
    args: Option<Vec<String>>,
    confirmation_token: Option<String>,
) -> Result<CommandExecutionResult, String> {
    let request = CommandExecutionRequest::foreground(command, args, confirmation_token);
    validate_command_request(&request)?;

    verify_ccr_sidecar_version().await?;
    let descriptor = ProcessDescriptor::ccr_command();
    let command_args = std::iter::once(OsString::from(&request.command))
        .chain(request.args.iter().map(OsString::from))
        .collect::<Vec<_>>();
    let output = ProcessGateway::execute(&descriptor, &command_args).await?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let truncated = output.stdout_truncated || output.stderr_truncated;

    Ok(CommandExecutionResult {
        success: output.status.success() && !output.timed_out && !truncated,
        output: stdout.clone(),
        error: stderr.clone(),
        stdout,
        stderr,
        exit_code,
        duration_ms: output.duration.as_millis().min(u128::from(u64::MAX)) as u64,
        timed_out: output.timed_out,
        truncated,
        stdout_bytes: output.stdout_bytes,
        stderr_bytes: output.stderr_bytes,
    })
}

/// 执行 `ccr help <command>` 并返回帮助文本
#[ccr_tauri_command_macros::command]
pub async fn get_ccr_command_help(command: String) -> Result<CommandHelpResponse, String> {
    CommandPolicy::from_command(&command)?;

    verify_ccr_sidecar_version().await?;
    let output = ProcessGateway::execute(
        &ProcessDescriptor::ccr_command(),
        &[OsString::from("help"), OsString::from(&command)],
    )
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(CommandHelpResponse {
        command,
        help: stdout,
        stderr,
        success: output.status.success()
            && !output.timed_out
            && !output.stdout_truncated
            && !output.stderr_truncated,
        timed_out: output.timed_out,
        truncated: output.stdout_truncated || output.stderr_truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn admitted() -> shared::Admission {
        COMMAND_JOBS
            .admit(
                request_key(),
                CommandExecutionRequest::background(
                    "status".into(),
                    Some(vec!["--json".into()]),
                    None,
                ),
            )
            .expect("admit shared fixture")
    }
    fn no_spawn_command() -> tokio::process::Command {
        ccr_app::jobs::process::tokio_command("must-not-spawn-after-cancel")
    }
    async fn finish_cancelled(admission: shared::Admission) -> shared::Snapshot {
        COMMAND_JOBS
            .run_command(
                admission.ticket.expect("new ticket"),
                CancellationToken::new(),
                ProcessDescriptor::ccr_command(),
                no_spawn_command(),
                Instant::now(),
            )
            .await
            .expect("cancel before spawn")
    }
    #[test]
    fn catalog_and_wire_keep_existing_shapes_through_shared_owner() {
        let adapter = serde_json::to_value(command_catalog()).expect("wire");
        let owner = serde_json::to_value(policy::command_catalog()).expect("owner");
        assert_eq!(adapter, owner);
        for name in ["delete", "import", "restore", "add", "platform"] {
            assert!(
                validate_command_request(&CommandExecutionRequest::background(
                    name.into(),
                    Some(vec![]),
                    Some(format!("desktop-confirm:{name}"))
                ))
                .is_err()
            );
        }
        assert!(
            validate_command_request(&CommandExecutionRequest::foreground(
                "status".into(),
                Some(vec!["--delete-everything".into()]),
                None
            ))
            .is_err()
        );
    }
    #[tokio::test]
    async fn wire_snapshot_retains_bounds_and_terminal_vocabulary_without_private_cancel_field() {
        let admission = admitted();
        let id = admission.snapshot.job_id.clone();
        let pending = COMMAND_JOBS.request_cancel(&id).expect("cancel");
        let wire = serde_json::to_value(CommandJobSnapshot::from(pending)).expect("wire");
        assert_eq!(wire["status"], "queued");
        assert!(wire.get("cancel_requested").is_none());
        assert_eq!(
            wire["system_lines"]
                .as_array()
                .expect("lines")
                .last()
                .expect("cancel line"),
            "Cancel requested"
        );
        let mut snapshot = finish_cancelled(admission).await;
        assert_eq!(snapshot.status, shared::Status::Cancelled);
        snapshot.status = shared::Status::TimedOut;
        let wire = CommandJobSnapshot::from(snapshot);
        assert_eq!(wire.status, CommandJobStatus::Failed);
        assert_eq!(wire.error.as_deref(), Some("process_timeout"));
    }
    #[tokio::test]
    async fn foreground_barrier_does_not_block_existing_command_owner_controls() {
        use crate::commands::runtime_policy::execute;
        use tokio::sync::oneshot;
        let admission = admitted();
        let id = admission.snapshot.job_id.clone();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let foreground = tokio::spawn(execute("execute_ccr_command", async move {
            started_tx.send(()).expect("started");
            release_rx.await.expect("release");
            Ok::<_, String>(())
        }));
        started_rx.await.expect("execution permit held");
        let snapshot = tokio::time::timeout(
            Duration::from_secs(1),
            get_ccr_command_job_status(id.clone()),
        )
        .await
        .expect("status watchdog")
        .expect("status");
        assert_eq!(snapshot.job_id, id);
        tokio::time::timeout(
            Duration::from_secs(1),
            execute("cancel_ccr_command_job", request_command_job_cancel(id)),
        )
        .await
        .expect("cancel watchdog")
        .expect("cancel");
        assert!(!foreground.is_finished());
        assert_eq!(
            finish_cancelled(admission).await.status,
            shared::Status::Cancelled
        );
        release_tx.send(()).expect("release");
        foreground.await.expect("join").expect("result");
    }
    #[tokio::test]
    async fn command_owner_receives_cancel_before_cleanup_and_keeps_start_admission() {
        use crate::commands::runtime_policy::{execute, take_background_admission};
        use tokio::sync::oneshot;
        let admitted = admitted();
        let id = admitted.snapshot.job_id.clone();
        let runner_id = id.clone();
        let (cancel_ack, cancelled) = oneshot::channel();
        let (cleanup_release, cleanup) = oneshot::channel();
        let (owner_finished, finished) = oneshot::channel();
        execute("start_ccr_command_job", async move {
            let permit = take_background_admission("start_ccr_command_job")?;
            retain_admission(permit, async move {
                while !COMMAND_JOBS
                    .status(&runner_id)
                    .expect("retained")
                    .cancel_requested
                {
                    tokio::task::yield_now().await;
                }
                cancel_ack.send(()).expect("cancel acknowledged");
                cleanup.await.expect("release cleanup");
                assert_eq!(
                    finish_cancelled(admitted).await.status,
                    shared::Status::Cancelled
                );
                owner_finished.send(()).expect("finished");
            });
            Ok::<_, String>(())
        })
        .await
        .expect("start returned");
        assert_eq!(
            get_ccr_command_job_status(id.clone())
                .await
                .expect("status")
                .status,
            CommandJobStatus::Queued
        );
        assert!(request_command_job_cancel("wrong-id".into()).await.is_err());
        execute(
            "cancel_ccr_command_job",
            request_command_job_cancel(id.clone()),
        )
        .await
        .expect("cancel control");
        tokio::time::timeout(Duration::from_secs(1), cancelled)
            .await
            .expect("watchdog")
            .expect("ack");
        let mut contender = std::pin::pin!(execute("llmusage_install_plan", async {
            Ok::<_, String>(())
        }));
        std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(contender.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        cleanup_release.send(()).expect("cleanup release");
        finished.await.expect("owner finished");
        contender
            .await
            .expect("permit released only after completion");
        assert_eq!(
            get_ccr_command_job_status(id)
                .await
                .expect("terminal")
                .status,
            CommandJobStatus::Cancelled
        );
    }
    #[test]
    fn version_parser_keeps_existing_semver_checks() {
        assert_eq!(parse_ccr_version_output("ccr 7.5.0\n"), Some("7.5.0"));
        assert_eq!(parse_ccr_version_output("ccr"), None);
    }
}
