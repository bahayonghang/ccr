use super::*;

#[tauri::command]
pub async fn claude_list_plugins(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let plugins = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "plugins": plugins }))
}

#[tauri::command]
pub async fn claude_add_plugin(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let mut plugin: ccr_types::Plugin =
        serde_json::from_value(config.into()).map_err(|e| format!("Invalid plugin config: {}", e))?;
    plugin.name = name;

    settings.plugins.push(plugin);
    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "plugins": result }))
}

#[tauri::command]
pub async fn claude_update_plugin(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut settings = load_settings(state.inner()).await?;

    let pos = settings
        .plugins
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| format!("Plugin '{}' not found", name))?;

    let updated: ccr_types::Plugin =
        serde_json::from_value(config.into()).map_err(|e| format!("Invalid plugin config: {}", e))?;
    settings.plugins[pos] = updated;

    save_settings(state.inner(), &settings).await?;

    let result = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "plugins": result }))
}

#[tauri::command]
pub async fn claude_delete_plugin(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let mut settings = load_settings(state.inner()).await?;

    let original_len = settings.plugins.len();
    settings.plugins.retain(|p| p.name != name);

    if settings.plugins.len() >= original_len {
        return Err(format!("Plugin '{}' not found", name));
    }

    save_settings(state.inner(), &settings).await?;
    Ok(format!("Plugin '{}' deleted", name))
}
