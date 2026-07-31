use super::*;

#[ccr_tauri_command_macros::command]
pub async fn claude_list_plugins(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let plugins = serde_json::to_value(&settings.plugins)
        .map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "plugins": plugins }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_plugin(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let mut plugin: ccr_types::Plugin = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid plugin config: {}", e))?;
    plugin.name = name;

    let result = update_settings(state.inner(), move |settings| {
        settings.plugins.push(plugin.clone());
        serde_json::to_value(&settings.plugins)
            .map_err(|error| format!("Serialization error: {error}"))
    })
    .await?;
    open_json(serde_json::json!({ "plugins": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_update_plugin(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let updated: ccr_types::Plugin = serde_json::from_value(config.into())
        .map_err(|e| format!("Invalid plugin config: {}", e))?;
    let result = update_settings(state.inner(), move |settings| {
        let pos = settings
            .plugins
            .iter()
            .position(|plugin| plugin.name == name)
            .ok_or_else(|| format!("Plugin '{name}' not found"))?;
        settings.plugins[pos] = updated.clone();
        serde_json::to_value(&settings.plugins)
            .map_err(|error| format!("Serialization error: {error}"))
    })
    .await?;
    open_json(serde_json::json!({ "plugins": result }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_delete_plugin(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    let deleted_name = name.clone();
    update_settings(state.inner(), move |settings| {
        let original_len = settings.plugins.len();
        settings.plugins.retain(|plugin| plugin.name != name);
        if settings.plugins.len() == original_len {
            return Err(format!("Plugin '{name}' not found"));
        }
        Ok(())
    })
    .await?;
    Ok(format!("Plugin '{}' deleted", deleted_name))
}
