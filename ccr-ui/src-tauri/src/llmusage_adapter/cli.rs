use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncRead, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{
    AppPaths, JobEvent, SourceKind, error::LlmusageAdapterError, events::parse_ndjson_event,
};
use crate::process::{ProcessDescriptor, ProcessGateway, read_bounded_line};

/// stderr 摘要最长保留行数，避免 OOM 当上游疯狂打日志
const STDERR_TAIL_LINES: usize = 64;
const STDERR_MAX_LINE_BYTES: usize = 64 * 1024;
const STDOUT_MAX_LINE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct LlmusageCli {
    paths: AppPaths,
    binary: String,
}

impl LlmusageCli {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            binary: "llmusage".to_string(),
        }
    }

    pub fn command(&self) -> Result<(Command, ProcessDescriptor), String> {
        let descriptor = ProcessDescriptor::llmusage(&self.binary)?;
        let mut command = ProcessGateway::command(&descriptor)?;
        command.arg("--home").arg(&self.paths.root_dir);
        // 强行静音上游 tracing 日志：llmusage 0.5.3 logging.rs 默认 writer 是 stdout，
        // INFO 日志会跟 NDJSON 事件混在同一条流，把 sync 干掉。RUST_LOG=off 让
        // EnvFilter 直接 LevelFilter::OFF；NO_COLOR=1 兜底关 ANSI，万一上游又把
        // 日志放到我们要解析的流里也不会带控制码。
        command.env("RUST_LOG", "off");
        command.env("NO_COLOR", "1");
        Ok((command, descriptor))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyncCommandOptions {
    pub rebuild: bool,
    pub recent_days: Option<u32>,
    pub source: Option<SourceKind>,
    pub provider_map: Option<PathBuf>,
}

fn sync_option_args(options: &SyncCommandOptions) -> Vec<OsString> {
    let mut args = Vec::new();
    if options.rebuild {
        args.push(OsString::from("--rebuild"));
    }
    if let Some(days) = options.recent_days {
        args.push(OsString::from("--recent-days"));
        args.push(OsString::from(days.to_string()));
    }
    if let Some(source) = options.source {
        args.push(OsString::from("--source"));
        args.push(OsString::from(source.as_str()));
    }
    if let Some(provider_map) = options.provider_map.as_ref() {
        args.push(OsString::from("--provider-map"));
        args.push(provider_map.as_os_str().to_os_string());
    }
    args
}

fn append_sync_options(command: &mut Command, options: &SyncCommandOptions) {
    for arg in sync_option_args(options) {
        command.arg(arg);
    }
}

pub async fn run_sync_collect(
    cli: &LlmusageCli,
    options: SyncCommandOptions,
) -> Result<Vec<JobEvent>, LlmusageAdapterError> {
    let mut events = Vec::new();
    run_sync_stream(cli, options, CancellationToken::new(), |event| {
        events.push(event);
        std::future::ready(Ok(()))
    })
    .await?;
    Ok(events)
}

/// 启动 `llmusage sync --json-events` 子进程并把 NDJSON 事件回调出来。
///
/// 子进程的 stdout/stderr 都被 piped，**两条流必须并发消费**否则上游写满 pipe
/// 缓冲（典型 64KB）后会阻塞，导致整条 stream 死锁。
///
/// 取消语义：调用方通过 `cancel_token.cancel()` 触发后，本函数会立即 kill
/// 子进程（llmusage 0.5.3 没有 graceful contract），等子进程退出后返回
/// `LlmusageAdapterError::Cli("cancelled")`.
pub async fn run_sync_stream<F, Fut>(
    cli: &LlmusageCli,
    options: SyncCommandOptions,
    cancel_token: CancellationToken,
    on_event: F,
) -> Result<(), LlmusageAdapterError>
where
    F: FnMut(JobEvent) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let (mut command, descriptor) = cli
        .command()
        .map_err(|error| LlmusageAdapterError::Cli(error.to_string()))?;
    command.arg("sync").arg("--json-events");
    append_sync_options(&mut command, &options);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = ProcessGateway::spawn(command, &descriptor, cancel_token.clone(), Vec::new())
        .map_err(|error| {
        if error.contains("not found") || error.contains("os error 2") {
            LlmusageAdapterError::CliMissing
        } else {
            LlmusageAdapterError::Cli(error)
        }
    })?;

    let stdout = child
        .take_stdout()
        .ok_or_else(|| LlmusageAdapterError::Cli("failed to capture llmusage stdout".into()))?;
    let stderr = child
        .take_stderr()
        .ok_or_else(|| LlmusageAdapterError::Cli("failed to capture llmusage stderr".into()))?;

    let (stderr_tail, stderr_task) = spawn_stderr_drainer(stderr);

    let stdout_result = consume_stdout_events(stdout, cancel_token.clone(), on_event).await;

    let exit = match stdout_result {
        Ok(StdoutOutcome::Eof) => child.wait().await,
        Ok(StdoutOutcome::Cancelled) | Err(_) => {
            child
                .terminate_tree(std::time::Duration::from_secs(5))
                .await
        }
    };

    // 把 stderr drain 完，保证摘要完整。
    let _ = stderr_task.await;

    match stdout_result {
        Ok(StdoutOutcome::Cancelled) => return Err(LlmusageAdapterError::Cli("cancelled".into())),
        Err(error) => return Err(error),
        Ok(StdoutOutcome::Eof) => {}
    }

    let status = exit.map_err(|error| LlmusageAdapterError::Cli(error.to_string()))?;
    if !status.success() {
        let stderr_summary = stderr_tail
            .lock()
            .map(|buf| buf.join("\n").trim().to_string())
            .unwrap_or_default();
        return Err(LlmusageAdapterError::Cli(if stderr_summary.is_empty() {
            format!("llmusage sync exited with status {status}")
        } else {
            stderr_summary
        }));
    }

    Ok(())
}

enum StdoutOutcome {
    Eof,
    Cancelled,
}

/// 并发 drain stderr：返回尾部行收集器和 join handle。
/// 收集器保留最后 [`STDERR_TAIL_LINES`] 行，用于子进程失败时拼摘要。
fn spawn_stderr_drainer<R>(reader: R) -> (Arc<Mutex<Vec<String>>>, JoinHandle<()>)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let tail: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let tail_clone = Arc::clone(&tail);
    let task = tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        while let Ok(Some(mut line)) = read_bounded_line(&mut reader, STDERR_MAX_LINE_BYTES).await {
            if line.truncated {
                while line.text.len() + '…'.len_utf8() > STDERR_MAX_LINE_BYTES {
                    line.text.pop();
                }
                line.text.push('…');
            }
            if let Ok(mut buf) = tail_clone.lock() {
                if buf.len() >= STDERR_TAIL_LINES {
                    buf.remove(0);
                }
                buf.push(line.text);
            }
        }
    });
    (tail, task)
}

/// 以 NDJSON 逐行消费 stdout，对每个有效 event 调用 `on_event`。
/// 在 cancel_token 触发时立刻返回 `StdoutOutcome::Cancelled`；自然 EOF 返回 `Eof`。
async fn consume_stdout_events<R, F, Fut>(
    reader: R,
    cancel_token: CancellationToken,
    mut on_event: F,
) -> Result<StdoutOutcome, LlmusageAdapterError>
where
    R: AsyncRead + Unpin,
    F: FnMut(JobEvent) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let mut reader = BufReader::new(reader);
    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => return Ok(StdoutOutcome::Cancelled),
            line = read_bounded_line(&mut reader, STDOUT_MAX_LINE_BYTES) => match line {
                Ok(Some(raw)) if raw.truncated => {
                    return Err(LlmusageAdapterError::Cli(
                        "llmusage_stdout_line_too_long".to_string(),
                    ));
                }
                Ok(Some(raw)) => match parse_ndjson_event(&raw.text) {
                    Ok(Some(event)) => {
                        on_event(event).await.map_err(LlmusageAdapterError::Cli)?;
                    }
                    Ok(None) => {}
                    Err(error) => return Err(LlmusageAdapterError::Cli(error)),
                },
                Ok(None) => return Ok(StdoutOutcome::Eof),
                Err(error) => return Err(LlmusageAdapterError::Cli(error.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn sync_options_defaults_to_all_sources_without_rebuild() {
        let options = SyncCommandOptions::default();
        assert!(!options.rebuild);
        assert_eq!(options.source, None);
        assert_eq!(options.provider_map, None);
    }

    #[test]
    fn sync_options_append_provider_map_argument() {
        let options = SyncCommandOptions {
            rebuild: true,
            recent_days: Some(7),
            source: Some(SourceKind::Codex),
            provider_map: Some(PathBuf::from("provider_activation.jsonl")),
        };
        let args = sync_option_args(&options)
            .into_iter()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                "--rebuild",
                "--recent-days",
                "7",
                "--source",
                "codex",
                "--provider-map",
                "provider_activation.jsonl"
            ]
        );
    }

    /// C1 回归：stderr 写 >64KB 不应阻塞 stdout NDJSON 的并发消费。
    ///
    /// 用 in-memory duplex 模拟子进程双流，stderr 灌入 ~128KB，stdout 灌入若干合法事件。
    /// 旧实现先 BufReader::lines() 消费 stdout 再 wait_with_output 拿 stderr，
    /// 在 stderr 缓冲（典型 64KB）满后写端会阻塞，整条 stream 死锁；
    /// 新实现并发 drain 两条流，此测试应在秒级完成。
    #[tokio::test]
    async fn stderr_flood_does_not_block_stdout_consumption() {
        let (mut stderr_tx, stderr_rx) = tokio::io::duplex(8 * 1024);
        let (mut stdout_tx, stdout_rx) = tokio::io::duplex(8 * 1024);

        // 推 128KB 噪音到 stderr —— 远超 64KB pipe 缓冲门槛。
        // 用 tokio::spawn 异步推，让 drainer 边读边推，避免本测试自己卡 8KB duplex。
        let writer_task = tokio::spawn(async move {
            let chunk = "noisy-stderr-line-with-some-padding\n".repeat(64);
            for _ in 0..64 {
                stderr_tx.write_all(chunk.as_bytes()).await.unwrap();
            }
            drop(stderr_tx);
        });
        let (tail, stderr_task) = spawn_stderr_drainer(stderr_rx);

        let stdout_writer = tokio::spawn(async move {
            // 推 3 个合法 NDJSON 事件 + 一个空行
            let events = vec![
                r#"{"event":"started","job_id":"job-1","files_total":10}"#,
                r#"{"event":"bootstrap_started"}"#,
                r#"{"event":"finished","summary":{"sources":1,"total_seen":3,"total_inserted":3}}"#,
            ];
            for line in events {
                stdout_tx
                    .write_all(line.as_bytes())
                    .await
                    .expect("write stdout line");
                stdout_tx.write_all(b"\n").await.expect("write newline");
            }
            drop(stdout_tx);
        });

        let mut collected: Vec<JobEvent> = Vec::new();
        let token = CancellationToken::new();
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            consume_stdout_events(stdout_rx, token, |event| {
                collected.push(event);
                std::future::ready(Ok(()))
            }),
        )
        .await
        .expect("stdout consumer must not hang while stderr is being drained")
        .expect("consumer should not error");

        stdout_writer.await.unwrap();
        writer_task.await.unwrap();
        stderr_task.await.unwrap();

        assert!(matches!(outcome, StdoutOutcome::Eof));
        assert_eq!(collected.len(), 3);
        let stderr_seen = tail
            .lock()
            .map(|buf| buf.len())
            .expect("stderr tail lock poisoned");
        assert!(
            stderr_seen > 0,
            "stderr drainer should have captured at least one line"
        );
        assert!(
            stderr_seen <= STDERR_TAIL_LINES,
            "stderr tail must be bounded; got {stderr_seen}"
        );
    }

    /// I7 回归：cancel_token 触发后 stdout consumer 立刻退出。
    #[tokio::test]
    async fn cancel_token_aborts_stdout_consumer() {
        let (_stdout_tx, stdout_rx) = tokio::io::duplex(8 * 1024);
        let token = CancellationToken::new();
        let cancel_clone = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            cancel_clone.cancel();
        });

        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            consume_stdout_events(stdout_rx, token, |_| std::future::ready(Ok(()))),
        )
        .await
        .expect("cancel must short-circuit the consumer")
        .expect("cancel path is not an error");

        assert!(matches!(outcome, StdoutOutcome::Cancelled));
    }

    #[tokio::test]
    async fn oversized_unterminated_stdout_line_fails_bounded() {
        let (mut writer, reader) = tokio::io::duplex(64 * 1024);
        let writer_task = tokio::spawn(async move {
            writer
                .write_all(&vec![b'x'; STDOUT_MAX_LINE_BYTES + 1024])
                .await
                .unwrap();
            writer.shutdown().await.unwrap();
        });

        let result = consume_stdout_events(reader, CancellationToken::new(), |_| {
            std::future::ready(Ok(()))
        })
        .await;
        writer_task.await.unwrap();

        let Err(error) = result else {
            panic!("oversized line must fail closed");
        };
        assert!(error.to_string().contains("llmusage_stdout_line_too_long"));
    }
}
