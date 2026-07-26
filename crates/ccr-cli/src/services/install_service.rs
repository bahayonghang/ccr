//! Top-level llmusage install orchestrator.
//!
//! Executable install plans remain canonical backend state. The renderer receives
//! only a short-lived, single-use [`PlanId`] plus an audit-safe preview.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, mpsc};
use tokio_util::sync::CancellationToken;

use crate::services::install_catalog;
use crate::services::install_detect;
use crate::services::install_exec;
use crate::services::install_plan::{self, GeneratedPlanOutcome};
use crate::services::install_ring_buffer::RingBufferHandle;
use crate::services::install_types::{
    AttemptId, CancelResult, DetectionResult, HostCapabilities, InstallAction, InstallEvent,
    InstallFlowError, InstallPlanConsumeErrorCode, InstallPlanView, ManualCatalog, PlanId,
    PlanOutcome, RingBufferSnapshot,
};

const INSTALL_PLAN_TTL: Duration = Duration::from_secs(120);
const EXPECTED_EFFECT: &str = "Install the llmusage executable for the current user";

type Clock = Arc<dyn Fn() -> ClockReading + Send + Sync>;
type HostProbe = Arc<dyn Fn() -> HostCapabilities + Send + Sync>;

#[derive(Debug, Clone, Copy)]
struct ClockReading {
    monotonic: Instant,
    unix_ms: u64,
}

/// The result of starting an install attempt.
#[derive(Debug)]
pub struct InstallAttempt {
    pub attempt_id: AttemptId,
    pub events: mpsc::Receiver<InstallEvent>,
}

struct AttemptSlot {
    attempt_id: AttemptId,
    cancel_token: CancellationToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostFingerprint(HostCapabilities);

impl From<&HostCapabilities> for HostFingerprint {
    fn from(value: &HostCapabilities) -> Self {
        Self(value.clone())
    }
}

#[derive(Debug)]
struct CanonicalInstallPlan {
    action: InstallAction,
    view: InstallPlanView,
    host: HostFingerprint,
    created_at: Instant,
    expires_at: Instant,
}

struct PlanTombstone {
    code: InstallPlanConsumeErrorCode,
    forget_at: Instant,
}

#[derive(Default)]
struct InstallPlanRegistry {
    entries: HashMap<PlanId, CanonicalInstallPlan>,
    tombstones: HashMap<PlanId, PlanTombstone>,
}

impl InstallPlanRegistry {
    fn insert(&mut self, plan: CanonicalInstallPlan, now: Instant) {
        self.prune(now);
        self.tombstones.remove(&plan.view.plan_id);
        self.entries.insert(plan.view.plan_id, plan);
    }

    fn consume(
        &mut self,
        plan_id: PlanId,
        host: &HostFingerprint,
        now: Instant,
    ) -> Result<CanonicalInstallPlan, InstallFlowError> {
        self.prune(now);

        if let Some(tombstone) = self.tombstones.get(&plan_id) {
            return Err(plan_error(tombstone.code, plan_id));
        }

        let Some(plan) = self.entries.remove(&plan_id) else {
            return Err(plan_error(InstallPlanConsumeErrorCode::Unknown, plan_id));
        };

        if now >= plan.expires_at {
            self.remember(plan_id, InstallPlanConsumeErrorCode::Expired, now);
            return Err(plan_error(InstallPlanConsumeErrorCode::Expired, plan_id));
        }

        if &plan.host != host {
            self.remember(plan_id, InstallPlanConsumeErrorCode::HostMismatch, now);
            return Err(plan_error(
                InstallPlanConsumeErrorCode::HostMismatch,
                plan_id,
            ));
        }

        self.remember(plan_id, InstallPlanConsumeErrorCode::Reused, now);
        Ok(plan)
    }

    fn remember(&mut self, plan_id: PlanId, code: InstallPlanConsumeErrorCode, now: Instant) {
        self.tombstones.insert(
            plan_id,
            PlanTombstone {
                code,
                forget_at: now + INSTALL_PLAN_TTL,
            },
        );
    }

    fn prune(&mut self, now: Instant) {
        self.tombstones
            .retain(|_, tombstone| tombstone.forget_at > now);

        let expired = self
            .entries
            .iter()
            .filter_map(|(plan_id, plan)| (plan.expires_at <= now).then_some(*plan_id))
            .collect::<Vec<_>>();

        for plan_id in expired {
            self.entries.remove(&plan_id);
            self.remember(plan_id, InstallPlanConsumeErrorCode::Expired, now);
        }
    }
}

fn plan_error(code: InstallPlanConsumeErrorCode, plan_id: PlanId) -> InstallFlowError {
    InstallFlowError::PlanUnavailable { code, plan_id }
}

/// Service managing llmusage detection, planning, installation, and cancellation.
pub struct InstallService {
    slot: RwLock<Option<AttemptSlot>>,
    ring: RingBufferHandle,
    registry: Mutex<InstallPlanRegistry>,
    clock: Clock,
    host_probe: HostProbe,
}

impl InstallService {
    pub fn new() -> Self {
        Self::with_environment(
            Arc::new(system_clock),
            Arc::new(install_detect::probe_host_capabilities),
        )
    }

    fn with_environment(clock: Clock, host_probe: HostProbe) -> Self {
        Self {
            slot: RwLock::new(None),
            ring: RingBufferHandle::new(),
            registry: Mutex::new(InstallPlanRegistry::default()),
            clock,
            host_probe,
        }
    }

    /// Detect whether `llmusage` is available on the host.
    pub async fn detect(&self) -> Result<DetectionResult, InstallFlowError> {
        install_detect::detect().await
    }

    /// Probe the host for available package managers.
    pub fn probe_capabilities(&self) -> HostCapabilities {
        (self.host_probe)()
    }

    /// Validate renderer hints against fresh backend detection and register a plan.
    pub async fn plan(
        &self,
        detection_hint: &DetectionResult,
        capabilities_hint: &HostCapabilities,
    ) -> Result<PlanOutcome, InstallFlowError> {
        let detection = self.detect().await?;
        self.plan_with_detection(detection_hint, capabilities_hint, &detection)
    }

    fn plan_with_detection(
        &self,
        detection_hint: &DetectionResult,
        capabilities_hint: &HostCapabilities,
        detection: &DetectionResult,
    ) -> Result<PlanOutcome, InstallFlowError> {
        if detection_hint != detection {
            return Err(InstallFlowError::InvalidPayload {
                field: "detection".to_string(),
                reason: "llmusage detection changed; check again".to_string(),
            });
        }

        let host = self.probe_capabilities();
        if capabilities_hint != &host {
            return Err(InstallFlowError::InvalidPayload {
                field: "capabilities".to_string(),
                reason: "host capabilities changed; probe again".to_string(),
            });
        }

        let generated = match install_plan::generate_plan(detection, &host)? {
            GeneratedPlanOutcome::Plan(generated) => generated,
            GeneratedPlanOutcome::Unsupported { reason } => {
                return Ok(PlanOutcome::Unsupported { reason });
            }
        };

        let now = (self.clock)();
        let plan_id = PlanId::new();
        let view = InstallPlanView {
            plan_id,
            platform: generated.platform,
            package_manager: generated.package_manager,
            action: generated.action.kind(),
            expected_effects: vec![EXPECTED_EFFECT.to_string()],
            elevation_required: false,
            duration_class: generated.duration_class,
            expires_at_ms: now
                .unix_ms
                .saturating_add(INSTALL_PLAN_TTL.as_millis() as u64),
        };
        let canonical = CanonicalInstallPlan {
            action: generated.action,
            view: view.clone(),
            host: HostFingerprint::from(&host),
            created_at: now.monotonic,
            expires_at: now.monotonic + INSTALL_PLAN_TTL,
        };

        self.registry
            .lock()
            .map_err(|_| InstallFlowError::Internal {
                message: "install plan registry lock poisoned".to_string(),
            })?
            .insert(canonical, now.monotonic);

        tracing::info!(
            %plan_id,
            action = ?view.action,
            expires_at_ms = view.expires_at_ms,
            "registered install plan"
        );

        Ok(PlanOutcome::Plan(view))
    }

    /// Consume a canonical plan and start an install attempt.
    pub async fn execute(&self, plan_id: PlanId) -> Result<InstallAttempt, InstallFlowError> {
        let mut slot = self.slot.write().await;
        if slot.is_some() {
            return Err(InstallFlowError::AlreadyRunning);
        }

        let now = (self.clock)();
        let host = HostFingerprint::from(&self.probe_capabilities());
        let plan = self
            .registry
            .lock()
            .map_err(|_| InstallFlowError::Internal {
                message: "install plan registry lock poisoned".to_string(),
            })?
            .consume(plan_id, &host, now.monotonic)?;

        let attempt_id = AttemptId::new();
        let cancel_token = CancellationToken::new();
        let plan_age_ms = now
            .monotonic
            .saturating_duration_since(plan.created_at)
            .as_millis() as u64;

        tracing::info!(
            %attempt_id,
            %plan_id,
            action = ?plan.view.action,
            plan_age_ms,
            "consumed install plan"
        );

        self.ring.clear();
        let events = install_exec::run_attempt(
            plan.action,
            plan.view,
            attempt_id,
            cancel_token.clone(),
            self.ring.clone(),
        );

        *slot = Some(AttemptSlot {
            attempt_id,
            cancel_token,
        });

        Ok(InstallAttempt { attempt_id, events })
    }

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

    pub async fn clear_slot(&self) {
        let mut slot = self.slot.write().await;
        *slot = None;
    }

    pub async fn is_running(&self) -> bool {
        self.slot.read().await.is_some()
    }

    pub async fn current_attempt_id(&self) -> Option<AttemptId> {
        self.slot.read().await.as_ref().map(|slot| slot.attempt_id)
    }

    pub fn recent_events(&self) -> RingBufferSnapshot {
        self.ring.snapshot()
    }

    pub fn manual_catalog(&self) -> Result<ManualCatalog, InstallFlowError> {
        install_catalog::build_catalog()
    }
}

impl Default for InstallService {
    fn default() -> Self {
        Self::new()
    }
}

fn system_clock() -> ClockReading {
    ClockReading {
        monotonic: Instant::now(),
        unix_ms: epoch_ms(),
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
    use std::path::PathBuf;
    use std::sync::Barrier;

    struct FakeClock {
        reading: Mutex<ClockReading>,
    }

    impl FakeClock {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                reading: Mutex::new(ClockReading {
                    monotonic: Instant::now(),
                    unix_ms: 1_700_000_000_000,
                }),
            })
        }

        fn read(self: &Arc<Self>) -> Clock {
            let fake = Arc::clone(self);
            Arc::new(move || *fake.reading.lock().expect("fake clock lock"))
        }

        fn advance(&self, duration: Duration) {
            let mut reading = self.reading.lock().expect("fake clock lock");
            reading.monotonic += duration;
            reading.unix_ms += duration.as_millis() as u64;
        }
    }

    fn absent() -> DetectionResult {
        DetectionResult::Absent {
            reason: AbsentReason::NotOnPath,
            data_root_warning: None,
        }
    }

    fn linux_caps() -> HostCapabilities {
        HostCapabilities {
            platform: Platform::Linux,
            has_cargo: true,
            has_homebrew: false,
            has_scoop: false,
            has_winget: false,
            cargo_path: Some(PathBuf::from("/usr/bin/cargo")),
            homebrew_path: None,
        }
    }

    fn service(
        clock: &Arc<FakeClock>,
        capabilities: &Arc<Mutex<HostCapabilities>>,
    ) -> InstallService {
        let host = Arc::clone(capabilities);
        InstallService::with_environment(
            clock.read(),
            Arc::new(move || host.lock().expect("host lock").clone()),
        )
    }

    fn registered_plan(svc: &InstallService, caps: &HostCapabilities) -> InstallPlanView {
        match svc
            .plan_with_detection(&absent(), caps, &absent())
            .expect("plan should register")
        {
            PlanOutcome::Plan(plan) => plan,
            PlanOutcome::Unsupported { .. } => panic!("expected install plan"),
        }
    }

    #[tokio::test]
    async fn detect_returns_result() {
        assert!(InstallService::new().detect().await.is_ok());
    }

    #[test]
    fn plan_with_no_pm_returns_unsupported() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(HostCapabilities {
            platform: Platform::Linux,
            has_cargo: false,
            has_homebrew: false,
            has_scoop: false,
            has_winget: false,
            cargo_path: None,
            homebrew_path: None,
        }));
        let svc = service(&clock, &caps);
        let hint = caps.lock().expect("host lock").clone();

        let result = svc
            .plan_with_detection(&absent(), &hint, &absent())
            .expect("should not error");
        assert!(matches!(result, PlanOutcome::Unsupported { .. }));
    }

    #[test]
    fn plan_rejects_renderer_modified_capabilities() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let mut forged = linux_caps();
        forged.cargo_path = Some(PathBuf::from("renderer-controlled"));

        let error = svc
            .plan_with_detection(&absent(), &forged, &absent())
            .expect_err("forged host hint must fail");
        assert!(matches!(
            error,
            InstallFlowError::InvalidPayload { ref field, .. } if field == "capabilities"
        ));
    }

    #[test]
    fn plan_rejects_stale_detection_hint() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let backend_detection = DetectionResult::Available {
            path: PathBuf::from("/usr/bin/llmusage"),
            version: Some("1.0.0".to_string()),
            data_root_warning: None,
        };

        let error = svc
            .plan_with_detection(&absent(), &linux_caps(), &backend_detection)
            .expect_err("stale detection hint must fail");
        assert!(matches!(
            error,
            InstallFlowError::InvalidPayload { ref field, .. } if field == "detection"
        ));
    }

    #[tokio::test]
    async fn unknown_plan_id_has_stable_typed_error() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let plan_id = PlanId::new();

        let error = svc
            .execute(plan_id)
            .await
            .expect_err("unknown plan must fail");
        assert_eq!(
            error,
            plan_error(InstallPlanConsumeErrorCode::Unknown, plan_id)
        );
        assert!(error.to_string().contains("install_plan_unknown"));
    }

    #[tokio::test]
    async fn expired_plan_cannot_execute() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let plan = registered_plan(&svc, &linux_caps());
        clock.advance(INSTALL_PLAN_TTL + Duration::from_millis(1));

        let error = svc
            .execute(plan.plan_id)
            .await
            .expect_err("expired plan must fail");
        assert!(matches!(
            error,
            InstallFlowError::PlanUnavailable {
                code: InstallPlanConsumeErrorCode::Expired,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn plan_is_bound_to_host_snapshot() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let plan = registered_plan(&svc, &linux_caps());
        caps.lock().expect("host lock").platform = Platform::Windows;

        let error = svc
            .execute(plan.plan_id)
            .await
            .expect_err("host mismatch must fail");
        assert!(matches!(
            error,
            InstallFlowError::PlanUnavailable {
                code: InstallPlanConsumeErrorCode::HostMismatch,
                ..
            }
        ));
    }

    #[test]
    fn consumed_plan_is_rejected_as_reused() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let plan = registered_plan(&svc, &linux_caps());
        let now = (svc.clock)();
        let host = HostFingerprint::from(&linux_caps());
        let mut registry = svc.registry.lock().expect("registry lock");

        registry
            .consume(plan.plan_id, &host, now.monotonic)
            .expect("first consume wins");
        let error = registry
            .consume(plan.plan_id, &host, now.monotonic)
            .expect_err("second consume must fail");
        assert!(matches!(
            error,
            InstallFlowError::PlanUnavailable {
                code: InstallPlanConsumeErrorCode::Reused,
                ..
            }
        ));
    }

    #[test]
    fn concurrent_consumption_has_exactly_one_winner() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = Arc::new(service(&clock, &caps));
        let plan = registered_plan(&svc, &linux_caps());
        let barrier = Arc::new(Barrier::new(3));

        let handles = (0..2)
            .map(|_| {
                let svc = Arc::clone(&svc);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let now = (svc.clock)();
                    svc.registry
                        .lock()
                        .expect("registry lock")
                        .consume(
                            plan.plan_id,
                            &HostFingerprint::from(&linux_caps()),
                            now.monotonic,
                        )
                        .is_ok()
                })
            })
            .collect::<Vec<_>>();

        barrier.wait();
        let winners = handles
            .into_iter()
            .map(|handle| handle.join().expect("consumer thread"))
            .filter(|won| *won)
            .count();
        assert_eq!(winners, 1);
    }

    #[test]
    fn registering_a_new_plan_prunes_expired_entries() {
        let clock = FakeClock::new();
        let caps = Arc::new(Mutex::new(linux_caps()));
        let svc = service(&clock, &caps);
        let expired = registered_plan(&svc, &linux_caps());
        clock.advance(INSTALL_PLAN_TTL + Duration::from_millis(1));
        let current = registered_plan(&svc, &linux_caps());
        let registry = svc.registry.lock().expect("registry lock");

        assert!(!registry.entries.contains_key(&expired.plan_id));
        assert!(registry.entries.contains_key(&current.plan_id));
        assert_eq!(
            registry
                .tombstones
                .get(&expired.plan_id)
                .map(|tombstone| tombstone.code),
            Some(InstallPlanConsumeErrorCode::Expired)
        );
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

    #[test]
    fn manual_catalog_builds_successfully() {
        let svc = InstallService::new();
        let catalog = svc.manual_catalog().expect("catalog should build");
        assert!(!catalog.entries.is_empty());
        assert!(!catalog.docs_url.is_empty());
    }

    #[tokio::test]
    async fn execute_rejects_when_attempt_slot_is_occupied() {
        let svc = InstallService::new();
        *svc.slot.write().await = Some(AttemptSlot {
            attempt_id: AttemptId::new(),
            cancel_token: CancellationToken::new(),
        });

        let error = svc
            .execute(PlanId::new())
            .await
            .expect_err("occupied slot must reject execution");
        assert_eq!(error, InstallFlowError::AlreadyRunning);
    }
}
