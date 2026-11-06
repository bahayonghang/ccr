//! Tauri adapter for the shared Codex agent owner.
use super::*;
use ccr_app::codex_agents::AgentStore;
pub use ccr_app::codex_agents::CodexAgentContextRequest;

#[ccr_tauri_command_macros::command]
pub async fn codex_list_agents(
    context: Option<CodexAgentContextRequest>,
) -> Result<OpenJsonValueDto, String> {
    let response = tokio::task::spawn_blocking(move || AgentStore::detect()?.list(context))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_add_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let response = tokio::task::spawn_blocking(move || {
        AgentStore::detect()?.add(context, name, config.into())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_update_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let response = tokio::task::spawn_blocking(move || {
        AgentStore::detect()?.update(context, name, config.into(), None)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_delete_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
) -> Result<String, String> {
    let response =
        tokio::task::spawn_blocking(move || AgentStore::detect()?.delete(context, name, None))
            .await
            .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[ccr_tauri_command_macros::command]
pub async fn codex_rename_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    new_name: String,
) -> Result<OpenJsonValueDto, String> {
    let response = tokio::task::spawn_blocking(move || {
        AgentStore::detect()?.rename(context, name, new_name, None)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_copy_agent(
    state: State<'_, AppState>,
    source_context: Option<CodexAgentContextRequest>,
    target_context: Option<CodexAgentContextRequest>,
    name: String,
    target_name: Option<String>,
) -> Result<OpenJsonValueDto, String> {
    let response = tokio::task::spawn_blocking(move || {
        AgentStore::detect()?.copy(source_context, target_context, name, target_name, None)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;
    invalidate_codex_dashboard_overview_cache(&state).await;
    response.try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn codex_validate_agent_toml(
    context: Option<CodexAgentContextRequest>,
    name: String,
) -> Result<OpenJsonValueDto, String> {
    let response =
        tokio::task::spawn_blocking(move || AgentStore::detect()?.validate(context, name))
            .await
            .map_err(|e| format!("任务执行失败: {e}"))??;
    response.try_into()
}
