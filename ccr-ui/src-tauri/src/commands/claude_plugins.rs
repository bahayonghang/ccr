use super::*;
use ccr_app::resources::claude_collections::{Collection, Mutation, apply_typed};

#[ccr_tauri_command_macros::command]
pub async fn claude_list_plugins(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let rows = serde_json::to_value(&settings.plugins)
        .map_err(|error| format!("Serialization error: {error}"))?;
    open_json(serde_json::json!({ "plugins": rows }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_plugin(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::Plugins,
            &name,
            Mutation::Add(config.clone()),
        )
        .map_err(|error| error.to_string())
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
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::Plugins,
            &name,
            Mutation::Replace(config.clone()),
        )
        .map_err(|error| error.to_string())
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
        apply_typed(settings, Collection::Plugins, &name, Mutation::Delete)
            .map_err(|error| error.to_string())
    })
    .await?;
    Ok(format!("Plugin '{}' deleted", deleted_name))
}
