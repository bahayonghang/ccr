use super::*;
use crate::commands::profile_lifecycle::{
    profiles_export_payload_from_path, profiles_raw_payload_from_paths,
    resolve_profile_target_name, save_profiles_raw_to_paths,
};
use crate::commands::settings_raw::ensure_local_env;
use ccr_cli::application::{needs_login_prep, profile_off_for_platform};

/// 列出所有 Claude Code Profiles（~/.ccr/platforms/claude/profiles.toml）。
#[ccr_tauri_command_macros::command]
pub async fn claude_list_profiles() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profiles: Vec<Value> = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .into_iter()
            .map(|(name, profile)| profile_to_json(current_profile.as_deref(), name, profile))
            .collect();

        // can_off 仅用于横幅；探测失败时仍返回列表，避免 settings 异常拖垮整个 Profiles 页。
        let can_off = needs_login_prep(Platform::Claude).unwrap_or(false);

        Ok(json!({
            "profiles": profiles,
            "current_profile": current_profile,
            "can_off": can_off,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 获取单个 Profile 详情。
#[ccr_tauri_command_macros::command]
pub async fn claude_get_profile(name: String) -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profile = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .shift_remove(&name)
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 创建新 Profile。
#[ccr_tauri_command_macros::command]
pub async fn claude_add_profile(request: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    let request: Value = request.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "Missing 'name' field".to_string())?
            .to_string();

        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        if profiles.contains_key(&name) {
            return Err(format!("Claude Profile '{name}' 已存在"));
        }

        let profile = build_profile_from_config(&request)?;

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Claude Profile 失败: {e}"))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 更新 Profile。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_profile(
    name: String,
    request: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let request: Value = request.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let existing = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        let target_name = resolve_profile_target_name("Claude", &name, &request)?;
        if target_name != name && profiles.contains_key(&target_name) {
            return Err(format!("Claude Profile '{target_name}' 已存在"));
        }

        let mut profile = existing;
        patch_profile_with_config(&mut profile, &request)?;

        platform
            .save_profile(&target_name, &profile)
            .map_err(|e| format!("更新 Claude Profile 失败: {e}"))?;

        if target_name != name {
            platform
                .delete_profile(&name)
                .map_err(|e| format!("删除旧 Claude Profile 失败: {e}"))?;

            if current_profile.as_deref() == Some(name.as_str()) {
                platform
                    .apply_profile(&target_name)
                    .map_err(|e| format!("同步当前 Claude Profile 失败: {e}"))?;
            }
        }

        let latest_current = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(
            latest_current.as_deref(),
            target_name,
            profile,
        ))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 删除 Profile。
#[ccr_tauri_command_macros::command]
pub async fn claude_delete_profile(name: String) -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 Claude Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Claude Profile '{name}' 已删除") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_profiles_raw(
    state: State<'_, AppState>,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let paths = PlatformPaths::new(Platform::Claude)
        .map_err(|error| format!("解析 Claude Profiles 路径失败: {error}"))?;
    tokio::task::spawn_blocking(move || profiles_raw_payload_from_paths(&paths))
        .await
        .map_err(|error| format!("读取 Claude Profiles 后台任务失败: {error}"))??
        .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_save_profiles_raw(
    state: State<'_, AppState>,
    content: String,
    token: String,
    force: bool,
) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    let paths = PlatformPaths::new(Platform::Claude)
        .map_err(|error| format!("解析 Claude Profiles 路径失败: {error}"))?;
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform =
            ClaudePlatform::new().map_err(|error| format!("初始化 Claude 平台失败: {error}"))?;
        let current = platform
            .get_current_profile()
            .map_err(|error| format!("读取当前 Claude profile 失败: {error}"))?;
        save_profiles_raw_to_paths(&paths, current.as_deref(), &content, &token, force)
    })
    .await
    .map_err(|error| format!("写入 Claude Profiles 后台任务失败: {error}"))??
    .try_into()
}

/// 构建 Claude profile 导出 payload。
fn claude_profiles_export_payload(include_secrets: bool) -> Result<Value, String> {
    let paths = PlatformPaths::new(Platform::Claude)
        .map_err(|e| format!("Failed to resolve Claude Profiles path: {e}"))?;
    profiles_export_payload_from_path(&paths.profiles_file, "ccr-claude-profiles", include_secrets)
}

#[ccr_tauri_command_macros::command]
pub async fn claude_export_profiles(include_secrets: bool) -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(move || claude_profiles_export_payload(include_secrets))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))??
        .try_into()
}

#[cfg(test)]
mod export_tests {
    use super::*;

    #[test]
    fn claude_export_profiles_reads_raw_toml_and_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        let content = "[profiles.demo]\nauth_token = \"secret\"\n";
        fs::write(&profiles_file, content).unwrap();

        let payload =
            profiles_export_payload_from_path(&profiles_file, "ccr-claude-profiles", true).unwrap();
        let filename = payload["filename"].as_str().unwrap();

        assert_eq!(payload["content"].as_str(), Some(content));
        assert!(filename.starts_with("ccr-claude-profiles-"));
        assert!(filename.ends_with(".toml"));
    }

    #[test]
    fn claude_export_profiles_reports_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("missing.toml");

        let error = profiles_export_payload_from_path(&profiles_file, "ccr-claude-profiles", true)
            .unwrap_err();

        assert!(error.contains("Failed to read profiles.toml"));
    }

    #[test]
    fn claude_export_profiles_rejects_redacted_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        fs::write(&profiles_file, "[profiles.demo]\n").unwrap();

        let error = profiles_export_payload_from_path(&profiles_file, "ccr-claude-profiles", false)
            .unwrap_err();

        assert_eq!(error, "Redacted profiles export is not supported");
    }
}

#[ccr_tauri_command_macros::command]
pub async fn claude_profile_off(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    if let Some(response) = ensure_local_env(state.inner()).await {
        return response.try_into();
    }
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let result = profile_off_for_platform(Platform::Claude)
            .map_err(|error| format!("退出 Claude profile 模式失败: {error}"))?;
        let outcome = result.auth_outcome.unwrap_or_default();
        Ok(json!({
            "ok": true,
            "changed": result.changed,
            "previous_profile": result.previous_profile,
            "runtime_mode": result.runtime_mode,
            "warnings": result.warnings,
            "remaining_suppressors": outcome.remaining_suppressors,
            "cleared_managed_sources": outcome.cleared_managed_sources,
        }))
    })
    .await
    .map_err(|error| format!("退出 Claude profile 模式后台任务失败: {error}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_apply_profile(name: String) -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 Claude Profile 失败: {e}"))?;
        Ok(json!({
            "success": true,
            "applied_profile": name,
            "message": "Claude Profile 已应用",
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}
