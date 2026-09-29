//! Completion-aware execution policy for registry-owned Tauri commands.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;

use super::handler_registry::{
    CommandConcurrency, CommandDescriptor, CommandTimeoutEnforcement, command_descriptor,
    command_descriptors,
};

static MODULE_GATES: LazyLock<HashMap<&'static str, Arc<Semaphore>>> = LazyLock::new(|| {
    command_descriptors()
        .map(|descriptor| descriptor.module)
        .map(|module| (module, Arc::new(Semaphore::new(1))))
        .collect()
});

static SINGLETON_GATE: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(1)));

/// OAuth checks idempotent reuse before acquiring the original module gate.
/// The controller transfers this permit directly to the new login owner.
pub(crate) async fn acquire_oauth_admission() -> Result<OwnedSemaphorePermit, String> {
    let descriptor =
        command_descriptor("codex_oauth_login_start").ok_or("oauth_descriptor_missing")?;
    acquire_permit(
        descriptor,
        Instant::now() + Duration::from_millis(descriptor.timeout_ms),
    )
    .await?
    .ok_or_else(|| "oauth_admission_missing".into())
}

tokio::task_local! {
    static ADMISSION: RefCell<(&'static str, Option<OwnedSemaphorePermit>)>;
}

/// Move, never reacquire, the wrapper's execution permit into a job owner.
pub(crate) fn take_background_admission(
    command: &'static str,
) -> Result<OwnedSemaphorePermit, String> {
    ADMISSION
        .try_with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.0 != command {
                return Err(format!("command_admission_owner_mismatch:{command}"));
            }
            slot.1
                .take()
                .ok_or_else(|| format!("command_admission_already_transferred:{command}"))
        })
        .map_err(|_| format!("command_admission_missing:{command}"))?
}

pub(crate) async fn execute<T, F>(command: &'static str, future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    let descriptor = command_descriptor(command)
        .ok_or_else(|| format!("command_runtime_policy_missing:{command}"))?;
    execute_with_descriptor(descriptor, future).await
}

async fn execute_with_descriptor<T, F>(
    descriptor: CommandDescriptor,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    let deadline = Instant::now() + Duration::from_millis(descriptor.timeout_ms);
    let permit = if descriptor.id == "codex_oauth_login_start" {
        // New login execution still acquires this exact module gate inside
        // the owner after its short idempotency check.
        None
    } else {
        acquire_permit(descriptor, deadline).await?
    };

    tracing::debug!(
        command = descriptor.id,
        module = descriptor.module,
        concurrency = ?descriptor.concurrency,
        timeout_enforcement = ?descriptor.timeout_enforcement,
        "tauri command admitted by runtime capability policy"
    );

    ADMISSION
        .scope(RefCell::new((descriptor.id, permit)), async {
            match descriptor.timeout_enforcement {
                CommandTimeoutEnforcement::Cooperative => tokio::time::timeout_at(deadline, future)
                    .await
                    .map_err(|_| format!("command_timeout:{}", descriptor.id))?,
                CommandTimeoutEnforcement::CompletionAware
                | CommandTimeoutEnforcement::BusinessOwned => future.await,
            }
        })
        .await
}

async fn acquire_permit(
    descriptor: CommandDescriptor,
    deadline: Instant,
) -> Result<Option<OwnedSemaphorePermit>, String> {
    let gate = match descriptor.concurrency {
        CommandConcurrency::Parallel => return Ok(None),
        CommandConcurrency::ModuleExclusive => MODULE_GATES
            .get(descriptor.module)
            .cloned()
            .ok_or_else(|| format!("command_runtime_module_missing:{}", descriptor.module))?,
        CommandConcurrency::Singleton => Arc::clone(&SINGLETON_GATE),
    };

    let permit = tokio::time::timeout_at(deadline, gate.acquire_owned())
        .await
        .map_err(|_| format!("command_queue_timeout:{}", descriptor.id))?
        .map_err(|_| format!("command_runtime_gate_closed:{}", descriptor.id))?;
    Ok(Some(permit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    // The watchdog protects a failed test. The execution barrier stays closed
    // until every control body has acknowledged delivery.
    async fn assert_control_delivery(blocker: &'static str, controls: &[&'static str]) {
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let running = tokio::spawn(execute(blocker, async move {
            let _ = started_tx.send(());
            let _ = release_rx.await;
            Ok::<_, String>(())
        }));
        started_rx.await.expect("foreground started");
        for &control in controls {
            let result = tokio::time::timeout(
                Duration::from_millis(500),
                execute(control, async { Ok::<_, String>(control) }),
            )
            .await;
            assert_eq!(
                result,
                Ok(Ok(control)),
                "owner acknowledgement for {control} while {blocker} holds its execution permit"
            );
        }
        release_tx.send(()).expect("release foreground");
        running
            .await
            .expect("foreground join")
            .expect("foreground result");
    }

    #[tokio::test]
    async fn control_matrix_command_and_install_delivery() {
        assert_control_delivery(
            "execute_ccr_command",
            &[
                "get_ccr_command_job_status",
                "cancel_ccr_command_job",
                "llmusage_install_cancel",
                "llmusage_install_recent",
                "llmusage_install_detect",
                "llmusage_install_probe_capabilities",
                "llmusage_install_manual_catalog",
                "llmusage_install_check",
            ],
        )
        .await;
        assert_control_delivery(
            "llmusage_install_plan",
            &["cancel_ccr_command_job", "llmusage_install_cancel"],
        )
        .await;
    }

    #[tokio::test]
    async fn control_matrix_usage_delivery() {
        for blocker in ["import_usage_v2", "import_all_usage_v2"] {
            assert_control_delivery(
                blocker,
                &[
                    "get_usage_import_job_status_v2",
                    "cancel_usage_import_job_v2",
                ],
            )
            .await;
        }
    }

    #[tokio::test]
    async fn control_matrix_oauth_delivery() {
        assert_control_delivery(
            "codex_save_auth",
            &[
                "codex_oauth_login_completed",
                "codex_oauth_login_cancel",
                "codex_oauth_submit_callback_url",
                "codex_release_oauth_port",
                "codex_is_oauth_port_in_use",
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn background_permit_transfer_is_single_use_and_preserves_shared_process_exclusion() {
        for start in ["start_ccr_command_job", "llmusage_install_execute"] {
            let (cleanup_tx, cleanup_rx) = oneshot::channel();
            let owner = execute(start, async move {
                assert!(take_background_admission("wrong_owner").is_err());
                let admission = take_background_admission(start)?;
                assert!(take_background_admission(start).is_err());
                Ok(tokio::spawn(async move {
                    let _ = cleanup_rx.await;
                    drop(admission);
                }))
            })
            .await
            .expect("start handler returned");
            for contender in [
                "execute_ccr_command",
                "start_ccr_command_job",
                "llmusage_install_plan",
                "llmusage_install_execute",
            ] {
                let mut descriptor = command_descriptor(contender).expect("descriptor");
                descriptor.timeout_ms = 10;
                let result = execute_with_descriptor(descriptor, async {
                    Err::<(), _>("execution_started_while_resource_owned".to_string())
                })
                .await;
                assert_eq!(result, Err(format!("command_queue_timeout:{contender}")));
            }
            cleanup_tx.send(()).expect("cleanup release");
            owner.await.expect("owner joined");
            execute("execute_ccr_command", async { Ok::<_, String>(()) })
                .await
                .expect("resource released after cleanup");
        }
    }

    #[tokio::test]
    async fn transfer_survives_wrapper_abort_and_early_error_releases_untransferred_permit() {
        let (transferred_tx, transferred_rx) = oneshot::channel();
        let (cleanup_tx, cleanup_rx) = oneshot::channel();
        let handler = tokio::spawn(execute("start_ccr_command_job", async move {
            let admission = take_background_admission("start_ccr_command_job")?;
            let owner = tokio::spawn(async move {
                let _ = cleanup_rx.await;
                drop(admission);
            });
            transferred_tx
                .send(owner)
                .map_err(|_| "test_receiver_closed")?;
            std::future::pending::<Result<(), String>>().await
        }));
        let owner = transferred_rx.await.expect("owner published");
        handler.abort();
        assert!(handler.await.expect_err("wrapper aborted").is_cancelled());
        assert_eq!(SINGLETON_GATE.available_permits(), 0);
        cleanup_tx.send(()).expect("cleanup release");
        owner.await.expect("owner joined");
        let result = execute("llmusage_install_execute", async {
            Err::<(), _>("plan_rejected".to_string())
        })
        .await;
        assert_eq!(result, Err("plan_rejected".into()));
        execute("execute_ccr_command", async { Ok::<_, String>(()) })
            .await
            .expect("early error releases admission");
    }

    #[tokio::test]
    async fn install_owner_cancel_and_recent_reach_attempt_before_cleanup() {
        use ccr_cli::services::install_service::InstallService;
        use ccr_cli::services::install_types::{AttemptId, CancelResult, InstallEvent, PlanId};
        use std::sync::Mutex;
        let (cancel_ack, cancelled) = oneshot::channel();
        let (cleanup_release, cleanup) = oneshot::channel();
        let fixture = Mutex::new(Some((cancel_ack, cleanup)));
        let (service, plan) = InstallService::test_with_runner(move |attempt_id, token, ring| {
            let (cancel_ack, cleanup) = fixture
                .lock()
                .expect("fixture")
                .take()
                .expect("one attempt");
            let (tx, rx) = tokio::sync::mpsc::channel(2);
            let completion = tokio::spawn(async move {
                token.cancelled().await;
                cancel_ack.send(()).expect("owner ack");
                // A terminal-looking UI event cannot release the owner's slot.
                let event = InstallEvent::Cancelled {
                    attempt_id,
                    requested_at_ms: 0,
                };
                ring.record(&event);
                tx.send(event).await.expect("event receiver");
                cleanup.await.expect("cleanup release");
            });
            (rx, completion)
        })
        .expect("install fixture");
        let mut attempt = execute("llmusage_install_execute", async {
            service
                .execute_with_admission(
                    plan,
                    Some(take_background_admission("llmusage_install_execute")?),
                )
                .await
                .map_err(|e| e.to_string())
        })
        .await
        .expect("start handler returned");
        let wrong = execute("llmusage_install_cancel", async {
            service
                .cancel(AttemptId::new())
                .await
                .map_err(|e| e.to_string())
        })
        .await
        .expect("wrong ID result");
        assert!(matches!(wrong, CancelResult::NotRunning));
        execute("llmusage_install_cancel", async {
            service
                .cancel(attempt.attempt_id)
                .await
                .map_err(|e| e.to_string())
        })
        .await
        .expect("cancel delivered");
        tokio::time::timeout(Duration::from_secs(1), cancelled)
            .await
            .expect("watchdog")
            .expect("owner ack before cleanup");
        assert!(attempt.events.recv().await.expect("event").is_terminal());
        assert!(service.is_running().await);
        let recent = execute("llmusage_install_recent", async {
            Ok::<_, String>(service.recent_events())
        })
        .await
        .expect("recent");
        assert!(recent.terminal.is_some());
        assert!(service.execute(PlanId::new()).await.is_err());
        let mut contender = std::pin::pin!(execute("execute_ccr_command", async {
            Ok::<_, String>(())
        }));
        std::future::poll_fn(|cx| {
            assert!(Future::poll(contender.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        cleanup_release.send(()).expect("release cleanup");
        contender
            .await
            .expect("shared permit released after attempt join");
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn usage_owner_controls_reach_existing_job_during_each_foreground_import() {
        use crate::llmusage_adapter::error::LlmusageAdapterError;
        use crate::usage_jobs::{
            UsageImportCompletion, UsageImportJobSnapshot, UsageImportJobStatus, UsageImportJobs,
        };
        for foreground in ["import_usage_v2", "import_all_usage_v2"] {
            let owner = Arc::new(tokio::sync::Mutex::new(UsageImportJobs::default()));
            let (_, cancellation) = owner.lock().await.admit(UsageImportJobSnapshot::new(
                "existing".into(),
                "all".into(),
                30,
            ));
            let cancellation = cancellation.expect("token registered before start returns");
            let (started_tx, started_rx) = oneshot::channel();
            let (release_tx, release_rx) = oneshot::channel();
            let running = tokio::spawn(execute(foreground, async move {
                started_tx.send(()).expect("started");
                release_rx.await.expect("execution release");
                Ok::<_, String>(())
            }));
            started_rx.await.expect("foreground holds permit");
            let status = tokio::time::timeout(
                Duration::from_secs(1),
                execute("get_usage_import_job_status_v2", async {
                    Ok::<_, String>(owner.lock().await.get("existing"))
                }),
            )
            .await
            .expect("status watchdog")
            .expect("status")
            .expect("snapshot");
            assert_eq!(status.job_id, "existing");
            let cancelled = tokio::time::timeout(
                Duration::from_secs(1),
                execute("cancel_usage_import_job_v2", async {
                    Ok::<_, String>(owner.lock().await.request_cancel("existing"))
                }),
            )
            .await
            .expect("cancel watchdog")
            .expect("cancel")
            .expect("owner ack");
            assert!(cancellation.is_cancelled());
            assert_eq!(cancelled.status, UsageImportJobStatus::CancelRequested);
            assert!(owner.lock().await.request_cancel("wrong-id").is_none());
            let (reused, token) = owner.lock().await.admit(UsageImportJobSnapshot::new(
                "second".into(),
                "all".into(),
                30,
            ));
            assert_eq!(reused.job_id, "existing");
            assert!(token.is_none());
            release_tx
                .send(())
                .expect("release after owner acknowledgements");
            running
                .await
                .expect("foreground join")
                .expect("foreground result");
            owner.lock().await.complete(
                "existing",
                UsageImportCompletion::Error(LlmusageAdapterError::CleanupFailed(
                    "synthetic cleanup failure".into(),
                )),
            );
            assert_eq!(
                owner.lock().await.get("existing").expect("terminal").status,
                UsageImportJobStatus::CleanupFailed
            );
            assert!(
                owner
                    .lock()
                    .await
                    .admit(UsageImportJobSnapshot::new(
                        "after-cleanup".into(),
                        "all".into(),
                        30
                    ))
                    .1
                    .is_some()
            );
        }
    }

    #[tokio::test]
    async fn foreground_install_plan_does_not_block_existing_attempt_owner() {
        use ccr_cli::services::install_service::InstallService;
        use std::sync::Mutex;
        let (ack, acknowledged) = oneshot::channel();
        let owner_ack = Mutex::new(Some(ack));
        let (service, plan) = InstallService::test_with_runner(move |_, token, _| {
            let ack = owner_ack.lock().expect("ack").take().expect("one owner");
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            let owner = tokio::spawn(async move {
                token.cancelled().await;
                ack.send(()).expect("owner ack");
            });
            (rx, owner)
        })
        .expect("fixture");
        let attempt = service.execute(plan).await.expect("existing attempt");
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let foreground = tokio::spawn(execute("llmusage_install_plan", async move {
            started_tx.send(()).expect("started");
            release_rx.await.expect("release");
            Ok::<_, String>(())
        }));
        started_rx.await.expect("foreground holds shared gate");
        tokio::time::timeout(
            Duration::from_secs(1),
            execute("llmusage_install_cancel", async {
                service
                    .cancel(attempt.attempt_id)
                    .await
                    .map_err(|error| error.to_string())
            }),
        )
        .await
        .expect("cancel watchdog")
        .expect("cancel control");
        tokio::time::timeout(Duration::from_secs(1), acknowledged)
            .await
            .expect("owner watchdog")
            .expect("owner ack before release");
        assert!(!foreground.is_finished());
        release_tx.send(()).expect("release after ack");
        foreground
            .await
            .expect("foreground join")
            .expect("foreground result");
    }

    #[tokio::test]
    async fn cooperative_deadline_cancels_the_command_future() {
        let Some(mut descriptor) = command_descriptor("test_webdav_config") else {
            panic!("test_webdav_config descriptor missing");
        };
        descriptor.timeout_ms = 10;
        descriptor.timeout_enforcement = CommandTimeoutEnforcement::Cooperative;

        let result = execute_with_descriptor(descriptor, async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, String>(())
        })
        .await;

        assert_eq!(
            result,
            Err("command_timeout:test_webdav_config".to_string())
        );
    }

    #[tokio::test]
    async fn module_permit_is_held_until_the_command_future_completes() {
        let Some(mut descriptor) = command_descriptor("delete_config") else {
            panic!("delete_config descriptor missing");
        };
        descriptor.timeout_ms = 1_000;

        let (first_started_tx, first_started_rx) = oneshot::channel();
        let (first_release_tx, first_release_rx) = oneshot::channel();
        let first = tokio::spawn(execute_with_descriptor(descriptor, async move {
            let _ = first_started_tx.send(());
            let _ = first_release_rx.await;
            Ok::<_, String>(())
        }));
        assert!(first_started_rx.await.is_ok());

        let (second_started_tx, second_started_rx) = oneshot::channel();
        let second = tokio::spawn(execute_with_descriptor(descriptor, async move {
            let _ = second_started_tx.send(());
            Ok::<_, String>(())
        }));

        assert!(
            tokio::time::timeout(Duration::from_millis(25), second_started_rx)
                .await
                .is_err(),
            "second command must wait for the first command's permit"
        );
        let _ = first_release_tx.send(());

        assert!(matches!(first.await, Ok(Ok(()))));
        assert!(matches!(second.await, Ok(Ok(()))));
    }

    #[test]
    fn risk_classes_choose_explicit_timeout_ownership() {
        let read = command_descriptor("get_system_info");
        let cooperative = command_descriptor("test_webdav_config");
        let mutation = command_descriptor("delete_config");
        let process = command_descriptor("execute_ccr_command");

        assert_eq!(
            read.map(|descriptor| descriptor.timeout_enforcement),
            Some(CommandTimeoutEnforcement::CompletionAware)
        );
        assert_eq!(
            cooperative.map(|descriptor| descriptor.timeout_enforcement),
            Some(CommandTimeoutEnforcement::Cooperative)
        );
        assert_eq!(
            mutation.map(|descriptor| descriptor.timeout_enforcement),
            Some(CommandTimeoutEnforcement::CompletionAware)
        );
        assert_eq!(
            process.map(|descriptor| descriptor.timeout_enforcement),
            Some(CommandTimeoutEnforcement::BusinessOwned)
        );
    }
}
