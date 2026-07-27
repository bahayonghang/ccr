//! Completion-aware execution policy for registry-owned Tauri commands.

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
    let permit = acquire_permit(descriptor, deadline).await?;

    tracing::debug!(
        command = descriptor.id,
        module = descriptor.module,
        concurrency = ?descriptor.concurrency,
        timeout_enforcement = ?descriptor.timeout_enforcement,
        "tauri command admitted by runtime capability policy"
    );

    let result =
        match descriptor.timeout_enforcement {
            CommandTimeoutEnforcement::Cooperative => tokio::time::timeout_at(deadline, future)
                .await
                .map_err(|_| format!("command_timeout:{}", descriptor.id))?,
            CommandTimeoutEnforcement::CompletionAware
            | CommandTimeoutEnforcement::BusinessOwned => future.await,
        };

    drop(permit);
    result
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
