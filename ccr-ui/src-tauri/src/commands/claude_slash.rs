use super::*;
use ccr_app::resources::claude_collections::{Collection, Mutation, apply_typed};

#[ccr_tauri_command_macros::command]
pub async fn claude_list_slash_commands(
    state: State<'_, AppState>,
) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let rows = serde_json::to_value(&settings.slash_commands)
        .map_err(|error| format!("Serialization error: {error}"))?;
    open_json(serde_json::json!({ "commands": rows }))
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_slash_command(
    state: State<'_, AppState>,
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::SlashCommands,
            &name,
            Mutation::Add(config.clone()),
        )
        .map_err(|error| error.to_string())
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
    let config: Value = config.into();
    let result = update_settings(state.inner(), move |settings| {
        apply_typed(
            settings,
            Collection::SlashCommands,
            &name,
            Mutation::Replace(config.clone()),
        )
        .map_err(|error| error.to_string())
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
        apply_typed(settings, Collection::SlashCommands, &name, Mutation::Delete)
            .map_err(|error| error.to_string())
    })
    .await?;
    Ok(format!("Slash command '{}' deleted", deleted_name))
}
