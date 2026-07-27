use super::*;

#[tauri::command]
pub async fn claude_list_slash_commands(
    state: State<'_, AppState>,
) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let commands = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "commands": commands }))
}

#[tauri::command]
pub async fn claude_add_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut cmd: ccr_types::SlashCommand = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    cmd.name = name;

    settings.slash_commands.push(cmd);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "commands": result }))
}

#[tauri::command]
pub async fn claude_update_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .slash_commands
        .iter()
        .position(|c| c.name == name)
        .ok_or_else(|| format!("Slash command '{}' not found", name))?;

    let updated: ccr_types::SlashCommand = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    settings.slash_commands[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "commands": result }))
}

#[tauri::command]
pub async fn claude_delete_slash_command(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.slash_commands.len();
    settings.slash_commands.retain(|c| c.name != name);

    if settings.slash_commands.len() >= original_len {
        return Err(format!("Slash command '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Slash command '{}' deleted", name))
}
