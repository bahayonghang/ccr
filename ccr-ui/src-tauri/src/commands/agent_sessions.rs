use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::services::agent_sessions::{
    AgentSessionDetailDto, AgentSessionDetailRequestDto, AgentSessionListRequestDto,
    AgentSessionPageDto, AgentSessionProviderStatusDto,
};
use crate::services::usage::StartSessionIndexJobResponse;
use crate::session_index_jobs::SessionIndexJobSnapshot;
use crate::state::AppState;

#[ccr_tauri_command_macros::command]
pub async fn agent_sessions_list(
    state: State<'_, AppState>,
    request: AgentSessionListRequestDto,
) -> Result<AgentSessionPageDto, String> {
    let pool = state.usage_db_pool.clone();
    tokio::task::spawn_blocking(move || {
        crate::services::agent_sessions::list_sessions(&pool, request)
    })
    .await
    .map_err(|error| format!("agent_session_task_failed:{error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn agent_sessions_get_detail(
    state: State<'_, AppState>,
    request: AgentSessionDetailRequestDto,
) -> Result<AgentSessionDetailDto, String> {
    let pool = state.usage_db_pool.clone();
    tokio::task::spawn_blocking(move || crate::services::agent_sessions::get_detail(&pool, request))
        .await
        .map_err(|error| format!("agent_session_task_failed:{error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn agent_sessions_get_provider_status()
-> Result<Vec<AgentSessionProviderStatusDto>, String> {
    tokio::task::spawn_blocking(crate::services::agent_sessions::provider_statuses)
        .await
        .map_err(|error| format!("agent_session_task_failed:{error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn agent_sessions_start_refresh(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<StartSessionIndexJobResponse, String> {
    if let Some(snapshot) = state.get_active_session_index_job().await {
        return Ok(StartSessionIndexJobResponse {
            job_id: snapshot.job_id.clone(),
            snapshot,
        });
    }
    let job_id = Uuid::new_v4().to_string();
    let snapshot = SessionIndexJobSnapshot::new(job_id.clone(), 8);
    state.insert_session_index_job(snapshot.clone()).await;
    let background_job_id = job_id.clone();
    tokio::spawn(async move {
        let app_state = app_handle.state::<AppState>();
        let pool = app_state.usage_db_pool.clone();
        let _ = app_state
            .update_session_index_job(&background_job_id, |job| job.mark_running(None, 0, 0))
            .await;
        let outcome = tokio::task::spawn_blocking(move || {
            crate::services::agent_sessions::refresh_archive(&pool)
        })
        .await;
        match outcome {
            Ok(Ok(report)) => {
                let _ = app_state
                    .update_session_index_job(&background_job_id, |job| {
                        job.platforms_completed = 8;
                        job.files_total = report.total.discovered;
                        job.files_scanned = report.total.fingerprinted;
                        job.sessions_added = report.total.upserted;
                        job.sessions_updated = report.total.upserted;
                        job.errors = report.total.errors;
                        job.discovered = report.total.discovered;
                        job.unchanged = report.total.unchanged;
                        job.fingerprinted = report.total.fingerprinted;
                        job.parsed = report.total.parsed;
                        job.upserted = report.total.upserted;
                        job.partial = report.total.partial;
                        job.locked = report.total.locked;
                        job.mark_finished();
                    })
                    .await;
            }
            Ok(Err(error)) => {
                let _ = app_state
                    .update_session_index_job(&background_job_id, |job| job.mark_failed(error))
                    .await;
            }
            Err(error) => {
                let _ = app_state
                    .update_session_index_job(&background_job_id, |job| {
                        job.mark_failed(format!("agent_session_task_failed:{error}"))
                    })
                    .await;
            }
        }
    });
    Ok(StartSessionIndexJobResponse { job_id, snapshot })
}

#[ccr_tauri_command_macros::command]
pub async fn agent_sessions_get_refresh_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<SessionIndexJobSnapshot, String> {
    if job_id.is_empty() || job_id.len() > 128 {
        return Err("agent_session_invalid_job_id".into());
    }
    state
        .get_session_index_job(&job_id)
        .await
        .ok_or_else(|| "agent_session_refresh_not_found".into())
}
