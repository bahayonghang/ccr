use super::*;

#[tauri::command]
pub async fn claude_list_agents(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let agents = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "agents": agents }))
}

#[tauri::command]
pub async fn claude_add_agent(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut agent: ccr_types::Agent = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid agent config: {}", e))?;
    agent.name = name;

    settings.agents.push(agent);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "agents": result }))
}

#[tauri::command]
pub async fn claude_update_agent(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .agents
        .iter()
        .position(|a| a.name == name)
        .ok_or_else(|| format!("Agent '{}' not found", name))?;

    let updated: ccr_types::Agent = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid agent config: {}", e))?;
    settings.agents[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.agents)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "agents": result }))
}

#[tauri::command]
pub async fn claude_delete_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.agents.len();
    settings.agents.retain(|a| a.name != name);

    if settings.agents.len() >= original_len {
        return Err(format!("Agent '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Agent '{}' deleted", name))
}
