use super::*;

#[ccr_tauri_command_macros::command]
pub async fn claude_list_agents(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let agents = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "agents": agents }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_agent(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut agent: ccr_types::Agent = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid agent config: {}", e))?;
    agent.name = name;

    let result = update_settings(state.inner(), move |settings| {
        settings.agents.push(agent.clone());
        serde_json::to_value(&settings.agents)
            .map_err(|error| format!("Serialization error: {error}"))
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
    let updated: ccr_types::Agent = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid agent config: {}", e))?;
    let result = update_settings(state.inner(), move |settings| {
        let pos = settings
            .agents
            .iter()
            .position(|agent| agent.name == name)
            .ok_or_else(|| format!("Agent '{name}' not found"))?;
        settings.agents[pos] = updated.clone();
        serde_json::to_value(&settings.agents)
            .map_err(|error| format!("Serialization error: {error}"))
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
        let original_len = settings.agents.len();
        settings.agents.retain(|agent| agent.name != name);
        if settings.agents.len() == original_len {
            return Err(format!("Agent '{name}' not found"));
        }
        Ok(())
    })
    .await?;
    Ok(format!("Agent '{}' deleted", deleted_name))
}
