//! 命令执行模块 — CCR CLI 命令白名单执行。

use std::collections::HashMap;
use std::io;
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Instant;

use crate::process::tokio_command;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const EVENT_COMMAND_JOB_PROGRESS: &str = "commands:job-progress";
const EVENT_COMMAND_JOB_FINISHED: &str = "commands:job-finished";
const EVENT_COMMAND_JOB_CANCELLED: &str = "commands:job-cancelled";

/// 允许执行的 CCR 子命令白名单
const ALLOWED_COMMANDS: &[&str] = &[
    "list",
    "switch",
    "add",
    "delete",
    "rename",
    "duplicate",
    "show",
    "validate",
    "export",
    "import",
    "history",
    "version",
    "help",
    "backup",
    "restore",
    "diff",
    "status",
];

/// 每个白名单命令的简要描述
const COMMAND_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    ("list", "列出所有配置", "read"),
    ("switch", "切换到指定配置", "write"),
    ("add", "添加新配置", "write"),
    ("delete", "删除配置", "danger"),
    ("rename", "重命名配置", "write"),
    ("duplicate", "复制配置", "write"),
    ("show", "显示配置内容", "read"),
    ("validate", "校验配置文件", "read"),
    ("export", "导出配置", "read"),
    ("import", "导入配置", "danger"),
    ("history", "查看操作历史", "read"),
    ("version", "显示版本信息", "read"),
    ("help", "显示帮助信息", "read"),
    ("backup", "备份配置", "write"),
    ("restore", "恢复配置", "danger"),
    ("diff", "比较配置差异", "read"),
    ("status", "显示当前状态", "read"),
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommandJobStatus {
    Queued,
    Running,
    Success,
    Failed,
    Cancelled,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandJobSnapshot {
    pub job_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: CommandJobStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub system_lines: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartCommandJobResponse {
    pub job_id: String,
    pub snapshot: CommandJobSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputChannel {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug)]
struct OutputEvent {
    channel: OutputChannel,
    line: String,
}

#[derive(Default)]
struct CommandJobRegistry {
    jobs: Mutex<HashMap<String, CommandJobSnapshot>>,
    cancel_tokens: Mutex<HashMap<String, CancellationToken>>,
}

static COMMAND_JOBS: LazyLock<CommandJobRegistry> = LazyLock::new(CommandJobRegistry::default);

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

impl CommandJobSnapshot {
    fn queued(job_id: String, command: String, args: Vec<String>) -> Self {
        Self {
            job_id,
            command,
            args,
            status: CommandJobStatus::Queued,
            started_at: now_rfc3339(),
            finished_at: None,
            duration_ms: None,
            exit_code: None,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            system_lines: vec!["Job queued".to_string()],
            error: None,
        }
    }

    fn mark_running(&mut self) {
        self.status = CommandJobStatus::Running;
        self.system_lines.push("Process started".to_string());
    }

    fn push_line(&mut self, channel: OutputChannel, line: String) {
        match channel {
            OutputChannel::Stdout => self.stdout_lines.push(line),
            OutputChannel::Stderr => self.stderr_lines.push(line),
            OutputChannel::System => self.system_lines.push(line),
        }
    }

    fn mark_terminal(
        &mut self,
        status: CommandJobStatus,
        started: Instant,
        exit_code: Option<i32>,
        error: Option<String>,
    ) {
        self.status = status;
        self.finished_at = Some(now_rfc3339());
        self.duration_ms = Some(elapsed_ms(started));
        self.exit_code = exit_code;
        self.error = error;
    }
}

/// 校验子命令是否在白名单中
fn validate_command(command: &str) -> Result<(), String> {
    if ALLOWED_COMMANDS.contains(&command) {
        Ok(())
    } else {
        Err(format!(
            "命令 '{}' 不在允许列表中。允许的命令: {}",
            command,
            ALLOWED_COMMANDS.join(", ")
        ))
    }
}

#[cfg(test)]
fn split_output_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

async fn insert_job(snapshot: CommandJobSnapshot, cancel_token: CancellationToken) {
    COMMAND_JOBS
        .jobs
        .lock()
        .await
        .insert(snapshot.job_id.clone(), snapshot.clone());
    COMMAND_JOBS
        .cancel_tokens
        .lock()
        .await
        .insert(snapshot.job_id, cancel_token);
}

async fn update_job<F>(job_id: &str, updater: F) -> Option<CommandJobSnapshot>
where
    F: FnOnce(&mut CommandJobSnapshot),
{
    let mut jobs = COMMAND_JOBS.jobs.lock().await;
    let snapshot = jobs.get_mut(job_id)?;
    updater(snapshot);
    Some(snapshot.clone())
}

async fn get_job(job_id: &str) -> Option<CommandJobSnapshot> {
    COMMAND_JOBS.jobs.lock().await.get(job_id).cloned()
}

async fn remove_cancel_token(job_id: &str) {
    COMMAND_JOBS.cancel_tokens.lock().await.remove(job_id);
}

async fn emit_job_snapshot(app_handle: &AppHandle, event: &str, snapshot: &CommandJobSnapshot) {
    if let Err(error) = app_handle.emit(event, snapshot.clone()) {
        tracing::warn!(event, ?error, job_id = %snapshot.job_id, "Failed to emit command job event");
    }
}

async fn update_and_emit<F>(app_handle: &AppHandle, event: &str, job_id: &str, updater: F)
where
    F: FnOnce(&mut CommandJobSnapshot),
{
    if let Some(snapshot) = update_job(job_id, updater).await {
        emit_job_snapshot(app_handle, event, &snapshot).await;
    }
}

async fn stream_reader<R>(reader: R, channel: OutputChannel, tx: mpsc::UnboundedSender<OutputEvent>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                if tx.send(OutputEvent { channel, line }).is_err() {
                    break;
                }
            }
            Ok(None) => break,
            Err(error) => {
                let _ = tx.send(OutputEvent {
                    channel: OutputChannel::System,
                    line: format!("Output stream read error: {error}"),
                });
                break;
            }
        }
    }
}

async fn run_command_job(app_handle: AppHandle, job_id: String, cancel_token: CancellationToken) {
    let started = Instant::now();
    let Some(initial) = get_job(&job_id).await else {
        return;
    };

    update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
        job.mark_running();
    })
    .await;

    let mut cmd = tokio_command("ccr");
    cmd.arg(&initial.command)
        .args(&initial.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            let (status, message) = if error.kind() == io::ErrorKind::NotFound {
                (
                    CommandJobStatus::Unavailable,
                    "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string(),
                )
            } else {
                (CommandJobStatus::Failed, format!("执行失败: {error}"))
            };
            update_and_emit(&app_handle, EVENT_COMMAND_JOB_FINISHED, &job_id, |job| {
                job.push_line(OutputChannel::System, message.clone());
                job.mark_terminal(status, started, None, Some(message));
            })
            .await;
            remove_cancel_token(&job_id).await;
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<OutputEvent>();
    if let Some(stdout) = child.stdout.take() {
        tauri::async_runtime::spawn(stream_reader(stdout, OutputChannel::Stdout, tx.clone()));
    }
    if let Some(stderr) = child.stderr.take() {
        tauri::async_runtime::spawn(stream_reader(stderr, OutputChannel::Stderr, tx.clone()));
    }
    drop(tx);

    let mut wait_finished = false;
    let mut final_code: Option<i32> = None;
    let mut cancel_requested = false;

    while !wait_finished || !rx.is_closed() {
        tokio::select! {
            event = rx.recv(), if !rx.is_closed() => {
                if let Some(event) = event {
                    update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
                        job.push_line(event.channel, event.line);
                    }).await;
                }
            }
            status = child.wait(), if !wait_finished => {
                match status {
                    Ok(status) => {
                        final_code = Some(status.code().unwrap_or(-1));
                    }
                    Err(error) => {
                        update_and_emit(&app_handle, EVENT_COMMAND_JOB_PROGRESS, &job_id, |job| {
                            job.push_line(OutputChannel::System, format!("Process wait error: {error}"));
                        }).await;
                        final_code = Some(-1);
                    }
                }
                wait_finished = true;
            }
            _ = cancel_token.cancelled(), if !cancel_requested && !wait_finished => {
                cancel_requested = true;
                let kill_result = child.kill().await;
                update_and_emit(&app_handle, EVENT_COMMAND_JOB_CANCELLED, &job_id, |job| {
                    if let Err(error) = kill_result {
                        job.push_line(OutputChannel::System, format!("Cancel signal failed: {error}"));
                    } else {
                        job.push_line(OutputChannel::System, "Cancel signal sent".to_string());
                    }
                    job.mark_terminal(
                        CommandJobStatus::Cancelled,
                        started,
                        None,
                        Some("Command cancelled".to_string()),
                    );
                }).await;
            }
        }

        if wait_finished && rx.is_closed() {
            break;
        }
    }

    remove_cancel_token(&job_id).await;

    if cancel_requested {
        return;
    }

    let exit_code = final_code.unwrap_or(-1);
    let success = exit_code == 0;
    let event = if success {
        EVENT_COMMAND_JOB_FINISHED
    } else {
        EVENT_COMMAND_JOB_FINISHED
    };
    update_and_emit(&app_handle, event, &job_id, |job| {
        job.mark_terminal(
            if success {
                CommandJobStatus::Success
            } else {
                CommandJobStatus::Failed
            },
            started,
            Some(exit_code),
            if success {
                None
            } else {
                Some(format!("Command exited with code {exit_code}"))
            },
        );
    })
    .await;
}

/// 执行白名单内的 CCR CLI 子命令并返回输出
///
/// 返回 `{ success, stdout, stderr, output, error, exit_code, duration_ms }`
#[tauri::command]
pub async fn execute_ccr_command(
    command: String,
    args: Option<Vec<String>>,
) -> Result<Value, String> {
    validate_command(&command)?;

    let started = Instant::now();
    let mut cmd = tokio_command("ccr");
    cmd.arg(&command);
    if let Some(extra_args) = args {
        cmd.args(&extra_args);
    }

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
        } else {
            format!("执行失败: {e}")
        }
    })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "success": output.status.success(),
        "stdout": stdout,
        "stderr": stderr,
        "output": stdout,
        "error": stderr,
        "exit_code": exit_code,
        "duration_ms": elapsed_ms(started),
    }))
}

/// 启动一个 app session 内可跟踪、可取消的 CCR CLI 后台任务。
#[tauri::command]
pub async fn start_ccr_command_job(
    app_handle: AppHandle,
    command: String,
    args: Option<Vec<String>>,
) -> Result<Value, String> {
    validate_command(&command)?;

    let job_id = format!("ccr-command-{}", Uuid::new_v4());
    let snapshot = CommandJobSnapshot::queued(job_id.clone(), command, args.unwrap_or_default());
    let cancel_token = CancellationToken::new();
    insert_job(snapshot.clone(), cancel_token.clone()).await;

    tauri::async_runtime::spawn(run_command_job(app_handle, job_id.clone(), cancel_token));

    serde_json::to_value(StartCommandJobResponse { job_id, snapshot })
        .map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn get_ccr_command_job_status(job_id: String) -> Result<Value, String> {
    let snapshot = get_job(&job_id)
        .await
        .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

#[tauri::command]
pub async fn cancel_ccr_command_job(
    app_handle: AppHandle,
    job_id: String,
) -> Result<Value, String> {
    if let Some(token) = COMMAND_JOBS
        .cancel_tokens
        .lock()
        .await
        .get(&job_id)
        .cloned()
    {
        token.cancel();
    }

    let snapshot = update_job(&job_id, |job| {
        if !matches!(
            job.status,
            CommandJobStatus::Success
                | CommandJobStatus::Failed
                | CommandJobStatus::Cancelled
                | CommandJobStatus::Unavailable
        ) {
            job.push_line(OutputChannel::System, "Cancel requested".to_string());
            job.status = CommandJobStatus::Cancelled;
            job.finished_at = Some(now_rfc3339());
            job.error = Some("Command cancelled".to_string());
        }
    })
    .await
    .ok_or_else(|| format!("Command job '{}' not found", job_id))?;

    emit_job_snapshot(&app_handle, EVENT_COMMAND_JOB_CANCELLED, &snapshot).await;
    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {e}"))
}

/// 返回白名单命令列表及其描述
///
/// 返回 `[{ name, description, usage, examples, category }, ...]`
#[tauri::command]
pub async fn list_ccr_commands() -> Result<Value, String> {
    let commands: Vec<Value> = COMMAND_DESCRIPTIONS
        .iter()
        .map(|(name, description, category)| {
            serde_json::json!({
                "name": name,
                "description": description,
                "usage": format!("ccr {name}"),
                "examples": [format!("ccr {name}")],
                "category": category,
            })
        })
        .collect();

    Ok(Value::Array(commands))
}

/// 执行 `ccr help <command>` 并返回帮助文本
#[tauri::command]
pub async fn get_ccr_command_help(command: String) -> Result<Value, String> {
    validate_command(&command)?;

    let output = tokio_command("ccr")
        .args(["help", &command])
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "CCR 二进制未找到，请确认已安装并在 PATH 中".to_string()
            } else {
                format!("执行失败: {e}")
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(serde_json::json!({
        "command": command,
        "help": stdout,
        "stderr": stderr,
        "success": output.status.success(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_command_rejects_non_whitelisted_command() {
        assert!(validate_command("status").is_ok());
        let error = validate_command("platform").expect_err("platform must stay blocked");
        assert!(error.contains("不在允许列表"));
    }

    #[test]
    fn split_output_lines_preserves_channel_text_without_blank_noise() {
        assert_eq!(
            split_output_lines("first\n\nsecond\r\n"),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn command_job_snapshot_serializes_required_status_values() {
        let snapshot = CommandJobSnapshot::queued(
            "job-1".to_string(),
            "status".to_string(),
            vec!["--json".to_string()],
        );
        let value = serde_json::to_value(snapshot).expect("snapshot serializes");

        assert_eq!(value["job_id"], "job-1");
        assert_eq!(value["command"], "status");
        assert_eq!(value["args"][0], "--json");
        assert_eq!(value["status"], "queued");
        assert!(value["stdout_lines"].is_array());
        assert!(value["stderr_lines"].is_array());
        assert!(value["system_lines"].is_array());
    }

    #[test]
    fn terminal_snapshot_keeps_non_zero_exit_as_failed_with_output() {
        let started = Instant::now();
        let mut snapshot =
            CommandJobSnapshot::queued("job-2".to_string(), "validate".to_string(), Vec::new());

        snapshot.push_line(OutputChannel::Stdout, "partial stdout".to_string());
        snapshot.push_line(OutputChannel::Stderr, "validation failed".to_string());
        snapshot.mark_terminal(
            CommandJobStatus::Failed,
            started,
            Some(2),
            Some("Command exited with code 2".to_string()),
        );

        assert_eq!(snapshot.status, CommandJobStatus::Failed);
        assert_eq!(snapshot.exit_code, Some(2));
        assert_eq!(snapshot.stdout_lines, vec!["partial stdout"]);
        assert_eq!(snapshot.stderr_lines, vec!["validation failed"]);
        assert!(snapshot.duration_ms.is_some());
        assert!(snapshot.error.as_deref().unwrap_or_default().contains('2'));
    }
}
