//! Top-level Install_Service orchestrator.
//!
//! Exposes typed operations: `detect`, `plan`, `execute`, `cancel`, `recent_events`,
//! and `manual_catalog`. Enforces single in-flight attempt via an `AttemptSlot`.

use tokio::sync::{RwLock, mpsc};
use tokio_util::sync::CancellationToken;

use crate::services::install_catalog;
use crate::services::install_detect;
use crate::services::install_exec;
use crate::services::install_plan;
use crate::services::install_ring_buffer::RingBufferHandle;
use crate::services::install_types::{
    AttemptId, CancelResult, DetectionResult, HostCapabilities, InstallEvent, InstallFlowError,
    InstallPlan, ManualCatalog, PlanOutcome, RingBufferSnapshot,
};

/// The result of starting an install attempt.
pub struct InstallAttempt {
    pub attempt_id: AttemptId,
    pub events: mpsc::Receiver<InstallEvent>,
}

/// Internal slot tracking the current in-flight install attempt.
struct AttemptSlot {
    attempt_id: AttemptId,
    cancel_token: CancellationToken,
}

/// Service managing llmusage detection, planning, installation, and cancellation.
pub struct InstallService {
    slot: RwLock<Option<AttemptSlot>>,
    ring: RingBufferHandle,
}

impl InstallService {
    pub fn new() -> Self {
        Self {
            slot: RwLock::new(None),
            ring: RingBufferHandle::new(),
        }
    }

    /// Detect whether `llmusage` is available on the host.
    ///
    /// Idempotent, side-effect free.
    pub async fn detect(&self) -> Result<DetectionResult, InstallFlowError> {
        install_detect::detect().await
    }

    /// Probe the host for available package managers.
    pub fn probe_capabilities(&self) -> HostCapabilities {
        install_detect::probe_host_capabilities()
    }

    /// Generate an install plan for the current host.
    ///
    /// Pure function: no I/O, no side effects.
    pub fn plan(
        &self,
        detection: &DetectionResult,
        caps: &HostCapabilities,
    ) -> Result<PlanOutcome, InstallFlowError> {
        install_plan::generate_plan(detection, caps)
    }

    /// Start an install attempt.
    ///
    /// Returns an `InstallAttempt` with the attempt ID and an event receiver.
    /// Fails with `AlreadyRunning` if an attempt is already in progress.
    pub async fn execute(&self, plan: InstallPlan) -> Result<InstallAttempt, InstallFlowError> {
        let mut slot = self.slot.write().await;

        if slot.is_some() {
            return Err(InstallFlowError::AlreadyRunning);
        }

        let attempt_id = AttemptId::new();
        let cancel_token = CancellationToken::new();

        // Clear ring buffer for new attempt.
        self.ring.clear();

        let rx =
            install_exec::run_attempt(plan, attempt_id, cancel_token.clone(), self.ring.clone());

        *slot = Some(AttemptSlot {
            attempt_id,
            cancel_token,
        });

        Ok(InstallAttempt {
            attempt_id,
            events: rx,
        })
    }

    /// Cancel the current in-flight install attempt.
    pub async fn cancel(&self, attempt_id: AttemptId) -> Result<CancelResult, InstallFlowError> {
        let slot = self.slot.read().await;

        let Some(current) = slot.as_ref() else {
            return Ok(CancelResult::NotRunning);
        };

        if current.attempt_id != attempt_id {
            return Ok(CancelResult::NotRunning);
        }

        let requested_at_ms = epoch_ms();
        current.cancel_token.cancel();

        Ok(CancelResult::Cancelled {
            attempt_id,
            requested_at_ms,
        })
    }

    /// Clear the attempt slot after a terminal event.
    ///
    /// Should be called by the event-forwarding layer when it observes a terminal event.
    pub async fn clear_slot(&self) {
        let mut slot = self.slot.write().await;
        *slot = None;
    }

    /// Check if an attempt is currently running.
    pub async fn is_running(&self) -> bool {
        self.slot.read().await.is_some()
    }

    /// Get the current attempt ID, if any.
    pub async fn current_attempt_id(&self) -> Option<AttemptId> {
        self.slot.read().await.as_ref().map(|s| s.attempt_id)
    }

    /// Read the most recent events from the ring buffer.
    pub fn recent_events(&self) -> RingBufferSnapshot {
        self.ring.snapshot()
    }

    /// Get the manual install catalog.
    pub fn manual_catalog(&self) -> Result<ManualCatalog, InstallFlowError> {
        install_catalog::build_catalog()
    }
}

impl Default for InstallService {
    fn default() -> Self {
        Self::new()
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::install_types::{AbsentReason, Platform};

    #[tokio::test]
    async fn detect_returns_result() {
        let svc = InstallService::new();
        let result = svc.detect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn plan_with_no_pm_returns_unsupported() {
        let svc = InstallService::new();
        let detection = DetectionResult::Absent {
            reason: AbsentReason::NotOnPath,
            data_root_warning: None,
        };
        let caps = HostCapabilities {
            platform: Platform::Linux,
            has_cargo: false,
            has_homebrew: false,
            has_scoop: false,
            has_winget: false,
            cargo_path: None,
            homebrew_path: None,
        };
        let result = svc.plan(&detection, &caps).expect("should not error");
        assert!(matches!(result, PlanOutcome::Unsupported { .. }));
    }

    #[tokio::test]
    async fn cancel_when_not_running_returns_not_running() {
        let svc = InstallService::new();
        let result = svc
            .cancel(AttemptId::new())
            .await
            .expect("should not error");
        assert!(matches!(result, CancelResult::NotRunning));
    }

    #[tokio::test]
    async fn manual_catalog_builds_successfully() {
        let svc = InstallService::new();
        let catalog = svc.manual_catalog().expect("catalog should build");
        assert!(!catalog.entries.is_empty());
        assert!(!catalog.docs_url.is_empty());
    }

    #[tokio::test]
    async fn execute_rejects_double_run() {
        use crate::services::install_types::{DurationClass, PackageManager};
        use std::collections::BTreeMap;

        let svc = InstallService::new();
        let plan = InstallPlan {
            platform: Platform::current(),
            package_manager: PackageManager::Cargo,
            command: "sleep".to_string(),
            args: vec!["60".to_string()],
            envs: BTreeMap::new(),
            elevation_required: false,
            duration_class: DurationClass::Slow,
            plan_id: uuid::Uuid::new_v4(),
        };

        let attempt = svc
            .execute(plan.clone())
            .await
            .expect("first execute should succeed");
        let second = svc.execute(plan).await;
        assert!(matches!(second, Err(InstallFlowError::AlreadyRunning)));

        // Clean up: cancel the first attempt.
        let _ = svc.cancel(attempt.attempt_id).await;
        // Give it a moment to terminate.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        svc.clear_slot().await;
    }
}
