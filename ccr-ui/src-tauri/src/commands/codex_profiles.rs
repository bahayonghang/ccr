use super::*;
use std::path::Path;

/// 列出 CCR Codex profiles（~/.ccr/platforms/codex/profiles.toml）
#[tauri::command]
pub async fn codex_list_profiles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Codex profile 失败: {e}"))?;
        let credential_store = CodexAuthService::new()
            .map(|service| service.get_auth_state().store.as_str().to_string())
            .ok();
        let profiles: Vec<Value> = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?
            .into_iter()
            .map(|(name, profile)| {
                profile_to_json(
                    &platform,
                    current_profile.as_deref(),
                    credential_store.as_deref(),
                    name,
                    profile,
                )
            })
            .collect();

        Ok(json!({ "profiles": profiles, "current_profile": current_profile }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 列出 Codex 可选模型（内置 + 自定义）
#[tauri::command]
pub async fn codex_list_models() -> Result<Value, String> {
    tokio::task::spawn_blocking(codex_list_models_payload)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存 Codex 自定义模型
#[tauri::command]
pub async fn codex_add_custom_model(model: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let normalized =
            normalize_model_name(&model).ok_or_else(|| "模型名称不能为空".to_string())?;
        let path = codex_custom_models_path()?;
        let mut file = read_codex_custom_models(&path)?;
        let mut custom_models = sanitize_custom_models(std::mem::take(&mut file.models));
        if !custom_models.iter().any(|item| item == &normalized)
            && !CODEX_BUILTIN_MODELS
                .iter()
                .any(|builtin| *builtin == normalized)
        {
            custom_models.push(normalized.clone());
        }
        file.models = custom_models.clone();
        write_codex_custom_models(&path, &file)?;
        Ok(json!({
            "model": normalized,
            "models": merge_codex_models(&custom_models),
            "message": "Codex 自定义模型已保存",
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 新增 Codex profile（写入 CCR profiles.toml）
#[tauri::command]
pub async fn codex_add_profile(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;
        if profiles.contains_key(&name) {
            return Err(format!("Codex Profile '{name}' 已存在"));
        }

        let profile = build_profile_from_config(&config)?;
        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Codex Profile 失败: {e}"))?;

        Ok(json!({ "message": format!("Codex Profile '{name}' 已添加") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 更新 Codex profile（核心字段覆盖 + extra/platform_data 整体替换）
#[tauri::command]
pub async fn codex_update_profile(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;
        let mut profile = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Codex Profile '{name}' 不存在"))?;

        patch_profile_with_config(&mut profile, &config)?;
        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("更新 Codex Profile 失败: {e}"))?;

        Ok(json!({ "message": format!("Codex Profile '{name}' 已更新") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 删除 Codex profile
#[tauri::command]
pub async fn codex_delete_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 Codex Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Codex Profile '{name}' 已删除") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 应用 Codex profile
#[tauri::command]
pub async fn codex_apply_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 Codex Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Codex Profile '{name}' 已应用") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 获取 Codex profile 导出的环境变量与 shell 脚本
fn codex_profiles_export_payload_from_path(
    profiles_file: &Path,
    filename_prefix: &str,
    include_secrets: bool,
) -> Result<Value, String> {
    if !include_secrets {
        return Err("Redacted profiles export is not supported".to_string());
    }

    let content = fs::read_to_string(profiles_file)
        .map_err(|e| format!("Failed to read profiles.toml: {e}"))?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{filename_prefix}-{timestamp}.toml");

    Ok(json!({
        "content": content,
        "filename": filename,
    }))
}

fn codex_profiles_export_payload(include_secrets: bool) -> Result<Value, String> {
    let paths = PlatformPaths::new(Platform::Codex)
        .map_err(|e| format!("Failed to resolve Codex Profiles path: {e}"))?;
    codex_profiles_export_payload_from_path(
        &paths.profiles_file,
        "ccr-codex-profiles",
        include_secrets,
    )
}

#[tauri::command]
pub async fn codex_export_profiles(include_secrets: bool) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || codex_profiles_export_payload(include_secrets))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[cfg(test)]
mod export_tests {
    use super::*;

    #[test]
    fn codex_export_profiles_reads_raw_toml_and_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        let content = "[profiles.demo]\nauth_token = \"secret\"\n";
        fs::write(&profiles_file, content).unwrap();

        let payload =
            codex_profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", true)
                .unwrap();
        let filename = payload["filename"].as_str().unwrap();

        assert_eq!(payload["content"].as_str(), Some(content));
        assert!(filename.starts_with("ccr-codex-profiles-"));
        assert!(filename.ends_with(".toml"));
    }

    #[test]
    fn codex_export_profiles_reports_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("missing.toml");

        let error =
            codex_profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", true)
                .unwrap_err();

        assert!(error.contains("Failed to read profiles.toml"));
    }

    #[test]
    fn codex_export_profiles_rejects_redacted_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        fs::write(&profiles_file, "[profiles.demo]\n").unwrap();

        let error =
            codex_profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", false)
                .unwrap_err();

        assert_eq!(error, "Redacted profiles export is not supported");
    }
}

#[tauri::command]
pub async fn codex_get_profile_env(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
        let env_export = platform
            .export_profile_env(&name)
            .map_err(|e| format!("导出 Codex Profile 环境变量失败: {e}"))?;
        let shell_export_script = platform
            .export_profile_shell_script(&name)
            .map_err(|e| format!("生成 Codex Profile shell 导出脚本失败: {e}"))?;

        Ok(json!({
            "name": name,
            "env_export": env_export,
            "shell_export_script": shell_export_script,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}
