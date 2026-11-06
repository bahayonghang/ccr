use super::*;
use tokio::io::{AsyncWriteExt, DuplexStream};

const DEADLINE: Duration = Duration::from_millis(30);
const GRACE: Duration = Duration::from_millis(60);

#[test]
fn production_sync_deadline_remains_one_hour() {
    assert_eq!(
        ProcessDescriptor::llmusage("llmusage").unwrap().timeout(),
        Duration::from_secs(3600)
    );
}

#[derive(Clone, Copy)]
enum Cleanup {
    Success,
    Fail,
    Hang,
}

struct FakeProcess {
    stdout: Option<DuplexStream>,
    stderr: Option<DuplexStream>,
    stdout_writer: Option<DuplexStream>,
    stderr_writer: Option<DuplexStream>,
    exited: bool,
    cleanup: Cleanup,
}

impl FakeProcess {
    fn new(stdout_eof: bool, stderr_eof: bool, exited: bool, cleanup: Cleanup) -> Self {
        let (stdout_writer, stdout) = tokio::io::duplex(4096);
        let (stderr_writer, stderr) = tokio::io::duplex(4096);
        Self {
            stdout: Some(stdout),
            stderr: Some(stderr),
            stdout_writer: (!stdout_eof).then_some(stdout_writer),
            stderr_writer: (!stderr_eof).then_some(stderr_writer),
            exited,
            cleanup,
        }
    }

    async fn finished_stdout(&mut self) {
        self.stdout_writer.as_mut().unwrap().write_all(
            b"{\"event\":\"finished\",\"summary\":{\"sources\":1,\"total_seen\":3,\"total_inserted\":3}}\n",
        ).await.unwrap();
        self.stdout_writer.take();
    }
}

fn success() -> ExitStatus {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    ExitStatus::from_raw(0)
}

impl SyncProcess for FakeProcess {
    type Stdout = DuplexStream;
    type Stderr = DuplexStream;
    fn take_stdout(&mut self) -> Option<DuplexStream> {
        self.stdout.take()
    }
    fn take_stderr(&mut self) -> Option<DuplexStream> {
        self.stderr.take()
    }
    async fn wait(&mut self) -> io::Result<ExitStatus> {
        if self.exited {
            Ok(success())
        } else {
            std::future::pending().await
        }
    }
    async fn terminate_tree(&mut self, _: Duration) -> io::Result<ExitStatus> {
        match self.cleanup {
            Cleanup::Success => {
                self.stdout_writer.take();
                self.stderr_writer.take();
                Ok(success())
            }
            Cleanup::Fail => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected cleanup failure",
            )),
            Cleanup::Hang => std::future::pending().await,
        }
    }
}

async fn execute_fake(
    child: FakeProcess,
    token: CancellationToken,
) -> Result<SyncSummaryEvent, LlmusageAdapterError> {
    let start = tokio::time::Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        execute_sync_process(child, DEADLINE, GRACE, token, |_| {
            std::future::ready(Ok(()))
        }),
    )
    .await
    .expect("executor must finish within its deadline and cleanup budget");
    assert!(start.elapsed() < DEADLINE + GRACE + Duration::from_millis(250));
    result
}

#[tokio::test]
async fn silent_process_obeys_descriptor_deadline() {
    assert!(matches!(
        execute_fake(
            FakeProcess::new(false, false, false, Cleanup::Success),
            CancellationToken::new()
        )
        .await,
        Err(LlmusageAdapterError::TimedOut)
    ));
}

#[tokio::test]
async fn stdout_eof_does_not_disable_deadline_while_child_lives() {
    assert!(matches!(
        execute_fake(
            FakeProcess::new(true, true, false, Cleanup::Success),
            CancellationToken::new()
        )
        .await,
        Err(LlmusageAdapterError::TimedOut)
    ));
}

#[tokio::test]
async fn finished_event_waits_for_child_exit_and_stderr() {
    for exited in [false, true] {
        let mut child = FakeProcess::new(false, false, exited, Cleanup::Success);
        child.finished_stdout().await;
        assert!(matches!(
            execute_fake(child, CancellationToken::new()).await,
            Err(LlmusageAdapterError::TimedOut)
        ));
    }
}

#[tokio::test]
async fn successful_exit_returns_summary_after_readers_finish() {
    let mut child = FakeProcess::new(false, true, true, Cleanup::Success);
    child.finished_stdout().await;
    assert_eq!(
        execute_fake(child, CancellationToken::new())
            .await
            .unwrap()
            .total_inserted,
        3
    );
}

#[tokio::test]
async fn cleanup_error_and_hung_reap_override_cancelled() {
    for cleanup in [Cleanup::Fail, Cleanup::Hang] {
        let token = CancellationToken::new();
        token.cancel();
        assert!(matches!(
            execute_fake(FakeProcess::new(false, false, false, cleanup), token).await,
            Err(LlmusageAdapterError::CleanupFailed(_))
        ));
    }
}

#[tokio::test]
async fn cleanup_error_overrides_timeout_and_protocol_failure() {
    assert!(matches!(
        execute_fake(
            FakeProcess::new(false, true, false, Cleanup::Fail),
            CancellationToken::new()
        )
        .await,
        Err(LlmusageAdapterError::CleanupFailed(_))
    ));
    let mut child = FakeProcess::new(false, true, false, Cleanup::Fail);
    child
        .stdout_writer
        .as_mut()
        .unwrap()
        .write_all(b"{invalid-json}\n")
        .await
        .unwrap();
    assert!(matches!(
        execute_fake(child, CancellationToken::new()).await,
        Err(LlmusageAdapterError::CleanupFailed(_))
    ));
}

#[tokio::test]
async fn wire_cancel_event_returns_cancelled_only_after_cleanup() {
    let mut child = FakeProcess::new(false, true, false, Cleanup::Success);
    child
        .stdout_writer
        .as_mut()
        .unwrap()
        .write_all(b"{\"event\":\"cancelled\"}\n")
        .await
        .unwrap();
    assert!(matches!(
        execute_fake(child, CancellationToken::new()).await,
        Err(LlmusageAdapterError::Cancelled)
    ));
}

#[tokio::test]
async fn missing_stderr_still_requires_tree_cleanup() {
    for cleanup in [Cleanup::Success, Cleanup::Fail] {
        let mut child = FakeProcess::new(false, true, false, cleanup);
        child.stderr.take();
        let result = execute_fake(child, CancellationToken::new()).await;
        match cleanup {
            Cleanup::Success => assert!(matches!(result, Err(LlmusageAdapterError::Cli(_)))),
            _ => assert!(matches!(
                result,
                Err(LlmusageAdapterError::CleanupFailed(_))
            )),
        }
    }
}

#[tokio::test]
async fn externally_held_stderr_has_bounded_join_and_cleanup_error() {
    let mut child = FakeProcess::new(false, false, false, Cleanup::Success);
    let _external_writer = child.stderr_writer.take();
    let token = CancellationToken::new();
    token.cancel();
    let result = execute_fake(child, token).await;
    assert!(
        matches!(result, Err(LlmusageAdapterError::CleanupFailed(ref error)) if error == "stderr_reader_join_timeout")
    );
}

#[tokio::test]
async fn blocked_event_callback_remains_cancellable() {
    let mut child = FakeProcess::new(false, true, false, Cleanup::Success);
    child
        .stdout_writer
        .as_mut()
        .unwrap()
        .write_all(b"{\"event\":\"bootstrap_started\"}\n")
        .await
        .unwrap();
    let result = execute_sync_process(child, DEADLINE, GRACE, CancellationToken::new(), |_| {
        std::future::pending::<Result<(), String>>()
    })
    .await;
    assert!(matches!(result, Err(LlmusageAdapterError::TimedOut)));
}

#[tokio::test]
async fn stdout_flood_without_eof_fails_at_one_mebibyte() {
    let (mut writer, reader) = tokio::io::duplex(8192);
    let task = tokio::spawn(async move {
        let chunk = [b'x'; 8192];
        while writer.write_all(&chunk).await.is_ok() {}
    });
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        consume_stdout_events(reader, CancellationToken::new(), |_| {
            std::future::ready(Ok(()))
        }),
    )
    .await
    .unwrap();
    assert!(
        matches!(result, Err(LlmusageAdapterError::Cli(ref message)) if message == "llmusage_stdout_line_too_long")
    );
    task.await.unwrap();
}

#[tokio::test]
async fn stderr_retains_at_most_sixty_four_capped_lines() {
    let (mut writer, reader) = tokio::io::duplex(8192);
    let writer_task = tokio::spawn(async move {
        let line = [vec![b'x'; STDERR_MAX_LINE_BYTES + 10], vec![b'\n']].concat();
        for _ in 0..70 {
            writer.write_all(&line).await.unwrap();
        }
    });
    let (tail, drainer) = spawn_stderr_drainer(reader);
    writer_task.await.unwrap();
    drainer.await.unwrap().unwrap();
    let tail = tail.lock().unwrap();
    assert_eq!(tail.len(), STDERR_TAIL_LINES);
    assert!(tail.iter().all(|line| line.len() <= STDERR_MAX_LINE_BYTES));
}

#[cfg(windows)]
#[tokio::test]
async fn native_windows_silent_child_times_out_and_is_reaped() {
    let descriptor =
        ProcessDescriptor::cli_probe("ccr", "powershell.exe", Duration::from_millis(200)).unwrap();
    let mut command = ProcessGateway::command(&descriptor).unwrap();
    let temp = tempfile::tempdir().unwrap();
    command
        .current_dir(temp.path())
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 30",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child =
        ProcessGateway::spawn(command, &descriptor, CancellationToken::new(), vec![]).unwrap();
    let start = tokio::time::Instant::now();
    let result = execute_sync_process(
        child,
        descriptor.timeout(),
        Duration::from_secs(2),
        CancellationToken::new(),
        |_| std::future::ready(Ok(())),
    )
    .await;
    assert!(matches!(result, Err(LlmusageAdapterError::TimedOut)));
    assert!(start.elapsed() < Duration::from_millis(2700));
}

#[cfg(windows)]
#[tokio::test]
async fn native_windows_running_cancel_reaps_the_child() {
    let descriptor =
        ProcessDescriptor::cli_probe("ccr", "powershell.exe", Duration::from_secs(5)).unwrap();
    let mut command = ProcessGateway::command(&descriptor).unwrap();
    let temp = tempfile::tempdir().unwrap();
    command.current_dir(temp.path()).args(["-NoProfile", "-NonInteractive", "-Command",
        "[Console]::Out.WriteLine('{\"event\":\"bootstrap_started\"}'); [Console]::Out.Flush(); Start-Sleep -Seconds 30"])
        .stdout(Stdio::piped()).stderr(Stdio::piped());
    let token = CancellationToken::new();
    let child = ProcessGateway::spawn(command, &descriptor, token.clone(), vec![]).unwrap();
    let result = execute_sync_process(
        child,
        descriptor.timeout(),
        Duration::from_secs(2),
        token.clone(),
        |_| {
            token.cancel();
            std::future::ready(Ok(()))
        },
    )
    .await;
    assert!(matches!(result, Err(LlmusageAdapterError::Cancelled)));
}
