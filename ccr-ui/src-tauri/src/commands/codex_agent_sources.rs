//! Tauri adapter for shared Codex source catalogs and tracked installs.
use super::{OpenJsonValueDto, invalidate_codex_dashboard_overview_cache};
use crate::state::AppState;
pub use ccr_app::codex_agents::sources::{
    CodexAgentSourceInstallActionRequest, CodexAgentSourceInstallRequest, CodexAgentSourceRequest,
    CodexAgentSourceSyncRequest,
};
use ccr_app::codex_agents::{AgentStore, sources::SourceStore};
use tauri::State;

#[ccr_tauri_command_macros::command]
pub async fn codex_list_agent_sources() -> Result<OpenJsonValueDto, String> {
    let client = reqwest::Client::new();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_list_agent_sources())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_add_agent_source(
    state: State<'_, AppState>,
    request: CodexAgentSourceRequest,
) -> Result<OpenJsonValueDto, String> {
    let client = state.http_client.clone();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_add_agent_source(request))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_remove_agent_source(source_id: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_remove_agent_source(source_id))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    Ok(())
}

#[ccr_tauri_command_macros::command]
pub async fn codex_sync_agent_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<OpenJsonValueDto, String> {
    let client = state.http_client.clone();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_sync_agent_source(source_id))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_get_agent_source_catalog(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<OpenJsonValueDto, String> {
    let client = state.http_client.clone();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_get_agent_source_catalog(source_id))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_install_source_agent(
    state: State<'_, AppState>,
    request: CodexAgentSourceInstallRequest,
) -> Result<OpenJsonValueDto, String> {
    let client = state.http_client.clone();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_install_source_agent(request))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_sync_source_install(
    state: State<'_, AppState>,
    request: CodexAgentSourceSyncRequest,
) -> Result<OpenJsonValueDto, String> {
    let client = state.http_client.clone();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_sync_source_install(request))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_accept_local_source_install(
    request: CodexAgentSourceInstallActionRequest,
) -> Result<OpenJsonValueDto, String> {
    let client = reqwest::Client::new();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_accept_local_source_install(request))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_untrack_source_install(
    request: CodexAgentSourceInstallActionRequest,
) -> Result<OpenJsonValueDto, String> {
    let client = reqwest::Client::new();
    let response = tokio::task::spawn_blocking(move || {
        let mut store = SourceStore::new(AgentStore::detect()?, client);
        tokio::runtime::Handle::current().block_on(store.codex_untrack_source_install(request))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}
