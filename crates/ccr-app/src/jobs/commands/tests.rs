use super::*;
use crate::runtime::{EnvironmentId, Platform};

fn key(request_id: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id,
    }
}
fn request() -> CommandExecutionRequest {
    CommandExecutionRequest::background("status".into(), Some(vec!["--json".into()]), None)
}
fn fixture(script: &str) -> tokio::process::Command {
    #[cfg(windows)]
    {
        let mut command = super::super::process::tokio_command("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        command
    }
    #[cfg(not(windows))]
    {
        let mut command = super::super::process::tokio_command("sh");
        command.args(["-c", script]);
        command
    }
}
fn descriptor(timeout: Duration) -> ProcessDescriptor {
    ProcessDescriptor::cli_probe("ccr", "unused-owned-fixture", timeout).unwrap()
}

#[tokio::test]
async fn observation_recovers_progress_and_terminal_after_detach_and_lag() {
    let jobs = Arc::new(CommandJobs::default());
    let admission = jobs.admit(key(51), request()).unwrap();
    let id = admission.snapshot.job_id;
    let (_, observer) = jobs.observe(&id).unwrap();
    drop(observer);
    jobs.append(&id, Channel::Stdout, vec!["while away".into()]);
    let (snapshot, mut observer) = jobs.observe(&id).unwrap();
    assert!(
        snapshot
            .stdout_lines
            .iter()
            .any(|line| line == "while away")
    );
    jobs.append(&id, Channel::Stdout, vec!["after return".into()]);
    let progress = observer.next().await.unwrap().unwrap();
    assert!(
        progress
            .stdout_lines
            .iter()
            .any(|line| line == "after return")
    );
    for _ in 0..200 {
        jobs.append(&id, Channel::Stdout, vec!["lag fixture".into()]);
    }
    let terminal = jobs
        .complete(&id, Status::Success, Instant::now(), Some(0), None)
        .unwrap();
    assert_eq!(observer.next().await.unwrap(), Some(terminal.clone()));
    assert_eq!(observer.next().await.unwrap(), None);
    let (restored, mut observer) = jobs.observe(&id).unwrap();
    assert_eq!(restored, terminal);
    assert_eq!(observer.next().await.unwrap(), None);
    assert!(matches!(jobs.observe("missing"), Err(Error::Missing)));
}

#[test]
fn foreground_and_help_keep_allowlist_and_argument_validation() {
    let jobs = CommandJobs::default();
    for command in ["delete", "unknown", "status; whoami"] {
        assert!(CommandExecutionRequest::help(command).is_err());
        assert!(matches!(
            jobs.admit(
                key(70),
                CommandExecutionRequest::foreground(command.into(), None, None)
            ),
            Err(Error::Invalid)
        ));
    }
    assert!(matches!(
        jobs.admit(
            key(71),
            CommandExecutionRequest::foreground(
                "status".into(),
                Some(vec!["--invalid".into()]),
                None
            )
        ),
        Err(Error::Invalid)
    ));
    let request = CommandExecutionRequest::help("status").unwrap();
    assert!(!request.background_job);
    assert_eq!(request.command, "help");
    assert_eq!(request.args, ["status"]);
    assert!(jobs.admit(key(72), request).is_ok());
    assert!(
        jobs.admit(
            key(73),
            CommandExecutionRequest::foreground("status".into(), Some(vec!["--json".into()]), None)
        )
        .is_ok()
    );
}

#[tokio::test]
async fn invalid_and_duplicate_admission_are_atomic_and_cancel_before_start_spawns_nothing() {
    let jobs = CommandJobs::default();
    let mut invalid = request();
    invalid.args.push("--unknown".into());
    assert!(matches!(jobs.admit(key(0), invalid), Err(Error::Invalid)));
    assert!(jobs.entries.lock().unwrap().is_empty());
    let first = jobs.admit(key(1), request()).unwrap();
    let duplicate = jobs.admit(key(1), request()).unwrap();
    assert_eq!(first.snapshot.job_id, duplicate.snapshot.job_id);
    assert!(duplicate.ticket.is_none());
    let pending = jobs.request_cancel(&first.snapshot.job_id).unwrap();
    assert_eq!(pending.status, Status::Queued);
    assert!(pending.cancel_requested);
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("must-not-exist");
    let command = if cfg!(windows) {
        fixture(&format!(
            "[IO.File]::WriteAllText('{}', 'spawned')",
            marker.display()
        ))
    } else {
        fixture(&format!("printf spawned > '{}'", marker.display()))
    };
    let final_state = jobs
        .run_command(
            first.ticket.unwrap(),
            CancellationToken::new(),
            descriptor(Duration::from_secs(3)),
            command,
            Instant::now(),
        )
        .await
        .unwrap();
    assert_eq!(final_state.status, Status::Cancelled);
    assert!(!marker.exists());
    assert_eq!(
        jobs.request_cancel(&final_state.job_id).unwrap(),
        final_state
    );
    jobs.append(
        &final_state.job_id,
        Channel::Stdout,
        vec!["late secret=should-not-appear".into()],
    );
    assert_eq!(jobs.status(&final_state.job_id).unwrap(), final_state);
}

#[tokio::test]
async fn real_owned_process_finishes_both_streams_and_nonzero_is_not_success() {
    let jobs = CommandJobs::default();
    let admission = jobs.admit(key(1), request()).unwrap();
    let command = if cfg!(windows) {
        fixture(
            "[Console]::Out.WriteLine('ready'); [Console]::Error.WriteLine('diagnostic'); exit 2",
        )
    } else {
        fixture("printf 'ready\\n'; printf 'diagnostic\\n' >&2; exit 2")
    };
    let result = jobs
        .run_command(
            admission.ticket.unwrap(),
            CancellationToken::new(),
            descriptor(Duration::from_secs(10)),
            command,
            Instant::now(),
        )
        .await
        .unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.exit_code, Some(2));
    assert!(result.stdout_lines.iter().any(|line| line == "ready"));
    assert!(result.stderr_lines.iter().any(|line| line == "diagnostic"));
    assert!(result.finished_at.is_some());
}

#[tokio::test]
async fn real_timeout_and_cancellation_wait_for_managed_cleanup() {
    for cancel in [false, true] {
        let jobs = Arc::new(CommandJobs::default());
        let admission = jobs.admit(key(1), request()).unwrap();
        let id = admission.snapshot.job_id;
        let app_cancel = CancellationToken::new();
        let command = if cfg!(windows) {
            fixture("Start-Sleep -Seconds 30")
        } else {
            fixture("sleep 30")
        };
        let owner = jobs.clone();
        let cancel_clone = app_cancel.clone();
        let task = tokio::spawn(async move {
            owner
                .run_command(
                    admission.ticket.unwrap(),
                    cancel_clone,
                    descriptor(if cancel {
                        Duration::from_secs(10)
                    } else {
                        Duration::from_millis(200)
                    }),
                    command,
                    Instant::now(),
                )
                .await
                .unwrap()
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            while jobs.status(&id).unwrap().status != Status::Running {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if cancel {
            let pending = jobs.request_cancel(&id).unwrap();
            assert!(pending.cancel_requested);
            assert_eq!(pending.status, Status::Running);
        }
        let result = tokio::time::timeout(Duration::from_secs(7), task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            result.status,
            if cancel {
                Status::Cancelled
            } else {
                Status::TimedOut
            }
        );
        assert!(result.finished_at.is_some());
    }
}

#[tokio::test]
async fn output_is_bounded_with_visible_drops_and_cleanup_failure_wins() {
    let jobs = CommandJobs::default();
    let admission = jobs.admit(key(1), request()).unwrap();
    let command = if cfg!(windows) {
        fixture("for($i=0;$i -lt 20000;$i++){[Console]::Out.WriteLine('bounded fixture output')} ")
    } else {
        fixture(
            "i=0; while [ $i -lt 20000 ]; do printf 'bounded fixture output\\n'; i=$((i+1)); done",
        )
    };
    let result = jobs
        .run_command(
            admission.ticket.unwrap(),
            CancellationToken::new(),
            descriptor(Duration::from_secs(15)),
            command,
            Instant::now(),
        )
        .await
        .unwrap();
    assert_eq!(result.status, Status::Success);
    assert!(result.stdout_lines.len() <= MAX_LINES);
    assert!(result.stdout_lines.iter().map(String::len).sum::<usize>() <= MAX_BYTES);
    assert!(result.truncated);
    assert!(result.dropped_lines > 0);

    // A held reader must complete its bounded join before a cleanup_failed claim.
    let held = tokio::spawn(std::future::pending::<bool>());
    assert!(
        !join_readers(
            vec![held],
            tokio::time::Instant::now() + Duration::from_millis(10)
        )
        .await
    );
    let queued = jobs.admit(key(2), request()).unwrap();
    let failed = jobs
        .complete(
            &queued.snapshot.job_id,
            Status::CleanupFailed,
            Instant::now(),
            None,
            Some("Command cleanup could not be confirmed".into()),
        )
        .unwrap();
    jobs.complete(
        &failed.job_id,
        Status::Cancelled,
        Instant::now(),
        None,
        None,
    );
    assert_eq!(jobs.status(&failed.job_id).unwrap(), failed);
}

#[test]
fn capacity_preserves_active_jobs_and_terminal_ttl_removes_only_finished() {
    let jobs = CommandJobs::default();
    let mut ids = Vec::new();
    for i in 0..MAX_JOBS {
        ids.push(
            jobs.admit(key(i as u64), request())
                .unwrap()
                .snapshot
                .job_id,
        );
    }
    assert!(matches!(jobs.admit(key(1000), request()), Err(Error::Full)));
    jobs.complete(&ids[0], Status::Success, Instant::now(), Some(0), None);
    assert!(jobs.admit(key(1001), request()).is_ok());
    assert!(matches!(jobs.status(&ids[0]), Err(Error::Missing)));
    assert!(jobs.status(&ids[1]).is_ok());
    let mut entries = jobs.entries.lock().unwrap();
    let active = entries.get_mut(&ids[1]).unwrap();
    active.snapshot.finished_at = Some((Utc::now() - chrono::Duration::hours(1)).to_rfc3339());
    prune(&mut entries, Utc::now());
    assert!(entries.contains_key(&ids[1]));
}

#[test]
fn snapshot_redacts_secret_output_and_caps_long_utf8_lines() {
    let mut snapshot = Snapshot::queued(request());
    snapshot.push(Channel::Stdout, "api_key=fixture-secret-value".into());
    assert!(!snapshot.stdout_lines[0].contains("fixture-secret-value"));
    snapshot.push(Channel::Stderr, "密".repeat(MAX_BYTES));
    assert!(snapshot.stderr_lines[0].len() <= MAX_BYTES);
    assert!(snapshot.stderr_lines[0].ends_with('…'));
}

#[tokio::test]
async fn oversized_multibyte_stream_keeps_valid_utf8_and_reports_truncation() {
    let text = "🔑".repeat(MAX_BYTES / 4 + 1);
    let (tx, mut rx) = mpsc::channel(1);
    let dropped = Arc::new(AtomicUsize::new(0));
    assert!(stream_reader(text.as_bytes(), Channel::Stdout, tx, dropped).await);
    let event = rx.recv().await.expect("bounded batch");
    assert_eq!(event.lines.len(), 1);
    assert!(event.lines[0].len() <= MAX_BYTES);
    assert!(event.lines[0].ends_with('…'));
    assert_eq!(event.dropped_count, 1);
}
