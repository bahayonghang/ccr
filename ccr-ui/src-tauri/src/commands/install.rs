//! Tauri command wrappers for the llmusage install flow.
//!
//! Thin delegation layer — all business logic lives in `ccr_cli::services::install_service`.

use std::sync::Arc;

use ccr_cli::services::install_detect;
use ccr_cli::services::install_service::InstallService;
use ccr_cli::services::install_types::{
    AttemptId, CancelResult, DetectionResult, HostCapabilities, InstallPlan, ManualCatalog,
    PlanOutcome, RingBufferSnapshot,
};
use tauri::{AppHandle, Emitter, State};

/// Detect whether `llmusage` is available on the host.
#[tauri::command]
pub async fn llmusage_install_detect(
    svc: State<'_, Arc<InstallService>>,
) -> Result<DetectionResult, String> {
    svc.detect().await.map_err(|e| e.to_string())
}

/// Probe the host for available package managers and platform info.
#[tauri::command]
pub async fn llmusage_install_probe_capabilities() -> Result<HostCapabilities, String> {
    Ok(install_detect::probe_host_capabilities())
}

/// Generate an install plan for the current host.
#[tauri::command]
pub async fn llmusage_install_plan(
    svc: State<'_, Arc<InstallService>>,
    detection: DetectionResult,
    capabilities: HostCapabilities,
) -> Result<PlanOutcome, String> {
    svc.plan(&detection, &capabilities)
        .map_err(|e| e.to_string())
}

/// Start an install attempt. Returns the attempt ID.
///
/// Events are streamed to the `"llmusage.install"` event channel.
#[tauri::command]
pub async fn llmusage_install_execute(
    app: AppHandle,
    svc: State<'_, Arc<InstallService>>,
    plan: InstallPlan,
) -> Result<AttemptId, String> {
    let attempt = svc.execute(plan).await.map_err(|e| e.to_string())?;
    let attempt_id = attempt.attempt_id;
    let mut rx = attempt.events;

    // Spawn a task to forward events to the Tauri event channel.
    let svc_clone = Arc::clone(svc.inner());
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let is_terminal = event.is_terminal();
            // Best-effort emit to frontend.
            let _ = app.emit("llmusage.install", &event);
            if is_terminal {
                // Clear the attempt slot so a new attempt can start.
                svc_clone.clear_slot().await;
                break;
            }
        }
    });

    Ok(attempt_id)
}

/// Cancel the current in-flight install attempt.
#[tauri::command]
pub async fn llmusage_install_cancel(
    svc: State<'_, Arc<InstallService>>,
    attempt_id: AttemptId,
) -> Result<CancelResult, String> {
    svc.cancel(attempt_id).await.map_err(|e| e.to_string())
}

/// Read the most recent events from the ring buffer.
#[tauri::command]
pub async fn llmusage_install_recent(
    svc: State<'_, Arc<InstallService>>,
) -> Result<RingBufferSnapshot, String> {
    Ok(svc.recent_events())
}

/// Get the manual install catalog (copy-able commands + docs link).
#[tauri::command]
pub async fn llmusage_install_manual_catalog(
    svc: State<'_, Arc<InstallService>>,
) -> Result<ManualCatalog, String> {
    svc.manual_catalog().map_err(|e| e.to_string())
}

/// Convenience: detect + probe capabilities in one call.
/// Returns both the detection result and host capabilities.
#[tauri::command]
pub async fn llmusage_install_check(
    svc: State<'_, Arc<InstallService>>,
) -> Result<(DetectionResult, HostCapabilities), String> {
    let detection = svc.detect().await.map_err(|e| e.to_string())?;
    let caps = install_detect::probe_host_capabilities();
    Ok((detection, caps))
}
