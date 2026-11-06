//! Benchmark-only retained S2 controls. The registry gate runs before admission.
use crate::{
    benchmark_s2,
    services::benchmark::{BenchmarkS2CancelRequest, BenchmarkS2SnapshotDto},
    state::AppState,
};
use tauri::{AppHandle, State};
#[ccr_tauri_command_macros::command]
pub async fn benchmark_s2_start(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<BenchmarkS2SnapshotDto, String> {
    benchmark_s2::seed()?;
    let permit = super::runtime_policy::take_background_admission("benchmark_s2_start")?;
    Ok(benchmark_s2::adapter(&state).start(app, permit)?.into())
}
#[ccr_tauri_command_macros::command]
pub async fn benchmark_s2_get_status(
    state: State<'_, AppState>,
) -> Result<BenchmarkS2SnapshotDto, String> {
    benchmark_s2::seed()?;
    Ok(benchmark_s2::adapter(&state).core.snapshot().into())
}
#[ccr_tauri_command_macros::command]
pub async fn benchmark_s2_cancel(
    state: State<'_, AppState>,
    request: BenchmarkS2CancelRequest,
) -> Result<BenchmarkS2SnapshotDto, String> {
    benchmark_s2::seed()?;
    Ok(benchmark_s2::adapter(&state)
        .core
        .request_cancel(&request.run_id)
        .map_err(str::to_string)?
        .into())
}
