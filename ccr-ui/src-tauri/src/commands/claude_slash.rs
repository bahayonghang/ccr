use super::*;

#[ccr_tauri_command_macros::command]
pub async fn claude_list_slash_commands(
    state: State<'_, AppState>,
) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let commands = serde_json::to_value(&settings.slash_commands)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "commands": commands }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut cmd: ccr_types::SlashCommand = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    cmd.name = name;

    let result = update_settings(state.inner(), move |settings| {
        settings.slash_commands.push(cmd.clone());
        serde_json::to_value(&settings.slash_commands)
            .map_err(|error| format!("Serialization error: {error}"))
    })
    .await?;
    open_json(serde_json::json!({ "commands": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_update_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let updated: ccr_types::SlashCommand = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid slash command config: {}", e))?;
    let result = update_settings(state.inner(), move |settings| {
        let pos = settings
            .slash_commands
            .iter()
            .position(|command| command.name == name)
            .ok_or_else(|| format!("Slash command '{name}' not found"))?;
        settings.slash_commands[pos] = updated.clone();
        serde_json::to_value(&settings.slash_commands)
            .map_err(|error| format!("Serialization error: {error}"))
    })
    .await?;
    open_json(serde_json::json!({ "commands": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_delete_slash_command(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let deleted_name = name.clone();
    update_settings(state.inner(), move |settings| {
        let original_len = settings.slash_commands.len();
        settings.slash_commands.retain(|command| command.name != name);
        if settings.slash_commands.len() == original_len {
            return Err(format!("Slash command '{name}' not found"));
        }
        Ok(())
    })
    .await?;
    Ok(format!("Slash command '{}' deleted", deleted_name))
}
