use super::*;

/// 获取 Codex 完整配置（去掉 mcp_servers 和 profiles）
#[ccr_tauri_command_macros::command]
pub async fn codex_get_settings() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        Ok(codex_settings_to_json(&config))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 更新 Codex 配置（合并写入，不覆盖 mcp_servers/profiles）
#[ccr_tauri_command_macros::command]
pub async fn codex_update_settings(
    state: State<'_, AppState>,
    settings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let settings: Value = settings.into();
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let path = codex_config_path()?;
        update_codex_settings_at_path(&path, &settings)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    open_json(response)
}

fn update_codex_settings_at_path(
    path: &std::path::Path,
    settings: &Value,
) -> Result<Value, String> {
    use ccr_app::platform_settings::{Document, Format, SaveOutcome};
    let document = Document::new(
        path.to_path_buf(),
        None,
        ccr_core::BackupPolicy::None,
        None,
        Format::Toml,
    );
    // This legacy patch endpoint has no editor token. Replay only the submitted
    // dirty patch against a fresh snapshot; native drafts pass their original one.
    for _ in 0..3 {
        let snapshot = document.read().map_err(|error| error.to_string())?;
        match ccr_app::platform_settings::codex::save_patch(&document, &snapshot, settings)
            .map_err(|error| error.to_string())?
        {
            SaveOutcome::Saved { .. } => return Ok(json!({ "message": "Codex 配置已更新" })),
            SaveOutcome::Conflict => {}
        }
    }
    Err("settings_conflict".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_only_update_preserves_notifications_and_untouched_fields_on_disk() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("config.toml");
        let source = r#"model = "before-model"
model_reasoning_effort = "high"
custom_root_value = ["preserve", "array"]

[tui]
notification_condition = "always"
notification_method = "auto"
notifications = ["agent-turn-complete", "approval-requested"]
status_line = ["model-with-reasoning"]

[history]
persistence = "save-all"
max_bytes = 2048

[profiles.retained]
model = "profile-model"
custom_profile_value = "keep-profile"

[mcp_servers.fixture]
command = "fixture-mcp"
args = ["--stdio"]
custom_mcp_value = "keep-mcp"
"#;
        fs::write(&path, source).unwrap();
        let before: toml::Value = toml::from_str(source).unwrap();

        let response =
            update_codex_settings_at_path(&path, &json!({ "model": "after-model" })).unwrap();

        let saved: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(response, json!({ "message": "Codex 配置已更新" }));
        assert_eq!(saved["model"].as_str(), Some("after-model"));
        assert_eq!(
            saved["tui"]["notifications"],
            before["tui"]["notifications"]
        );
        let mut expected = before;
        expected["model"] = toml::Value::String("after-model".into());
        assert_eq!(saved, expected);
    }
}
