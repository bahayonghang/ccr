use super::*;
use ccr_app::resources::claude_collections::{Collection, Mutation, apply_typed};

#[ccr_tauri_command_macros::command]
pub async fn claude_list_agents(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let rows = serde_json::to_value(&settings.agents)
        .map_err(|error| format!("Serialization error: {error}"))?;
    open_json(serde_json::json!({ "agents": rows }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_agent(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::Agents,
            &name,
            Mutation::Add(config.clone()),
        )
        .map_err(|error| error.to_string())
    })
    .await?;
    open_json(serde_json::json!({ "agents": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_update_agent(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::Agents,
            &name,
            Mutation::Replace(config.clone()),
        )
        .map_err(|error| error.to_string())
    })
    .await?;
    open_json(serde_json::json!({ "agents": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_delete_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let deleted_name = name.clone();
    update_settings(state.inner(), move |settings| {
        apply_typed(settings, Collection::Agents, &name, Mutation::Delete)
            .map_err(|error| error.to_string())
    })
    .await?;
    Ok(format!("Agent '{}' deleted", deleted_name))
}
