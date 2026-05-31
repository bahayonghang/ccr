use super::*;
use crate::commands::profile_lifecycle::{
    profiles_export_payload_from_path, resolve_profile_target_name,
};

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

fn update_codex_profile_payload(name: String, config: Value) -> Result<Value, String> {
    let platform = CodexPlatform::new().map_err(|e| format!("初始化 Codex 平台失败: {e}"))?;
    let profiles = platform
        .load_profiles()
        .map_err(|e| format!("读取 Codex profiles 失败: {e}"))?;
    let current_profile = platform
        .get_current_profile()
        .map_err(|e| format!("读取当前 Codex profile 失败: {e}"))?;
    let mut profile = profiles
        .get(&name)
        .cloned()
        .ok_or_else(|| format!("Codex Profile '{name}' 不存在"))?;
    let target_name = resolve_profile_target_name("Codex", &name, &config)?;

    if target_name != name && profiles.contains_key(&target_name) {
        return Err(format!("Codex Profile '{target_name}' 已存在"));
    }

    patch_profile_with_config(&mut profile, &config)?;
    platform
        .save_profile(&target_name, &profile)
        .map_err(|e| format!("更新 Codex Profile 失败: {e}"))?;

    if target_name != name {
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除旧 Codex Profile 失败: {e}"))?;

        if current_profile.as_deref() == Some(name.as_str()) {
            platform
                .apply_profile(&target_name)
                .map_err(|e| format!("同步当前 Codex Profile 失败: {e}"))?;
        }
    }

    Ok(json!({
        "message": format!("Codex Profile '{target_name}' 已更新"),
        "name": target_name,
    }))
}

#[tauri::command]
pub async fn codex_update_profile(
    state: State<'_, AppState>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || update_codex_profile_payload(name, config))
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

fn codex_profiles_export_payload(include_secrets: bool) -> Result<Value, String> {
    let paths = PlatformPaths::new(Platform::Codex)
        .map_err(|e| format!("Failed to resolve Codex Profiles path: {e}"))?;
    profiles_export_payload_from_path(&paths.profiles_file, "ccr-codex-profiles", include_secrets)
}

#[tauri::command]
pub async fn codex_export_profiles(include_secrets: bool) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || codex_profiles_export_payload(include_secrets))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod export_tests {
    use super::*;

    #[test]
    fn codex_export_profiles_reads_raw_toml_and_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        let content = "[profiles.demo]\nauth_token = \"secret\"\n";
        fs::write(&profiles_file, content).unwrap();

        let payload =
            profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", true).unwrap();
        let filename = payload["filename"].as_str().unwrap();

        assert_eq!(payload["content"].as_str(), Some(content));
        assert!(filename.starts_with("ccr-codex-profiles-"));
        assert!(filename.ends_with(".toml"));
    }

    #[test]
    fn codex_export_profiles_reports_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("missing.toml");

        let error = profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", true)
            .unwrap_err();

        assert!(error.contains("Failed to read profiles.toml"));
    }

    #[test]
    fn codex_export_profiles_rejects_redacted_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_file = temp_dir.path().join("profiles.toml");
        fs::write(&profiles_file, "[profiles.demo]\n").unwrap();

        let error = profiles_export_payload_from_path(&profiles_file, "ccr-codex-profiles", false)
            .unwrap_err();

        assert_eq!(error, "Redacted profiles export is not supported");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod update_tests {
    use super::*;
    use ccr_config::{PlatformConfig, get_current_profile_from_registry, load_profiles_from_toml};
    use serde_json::json;
    use std::path::Path;

    fn restore_env_var(key: &str, previous: Option<String>) {
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    fn write_file_store_config(codex_dir: &Path) {
        fs::write(
            codex_dir.join("config.toml"),
            "cli_auth_credentials_store = \"file\"\n",
        )
        .unwrap();
    }

    fn build_provider_env_profile(secret: &str) -> ProfileConfig {
        let mut profile = ProfileConfig {
            description: Some("Relay profile".to_string()),
            base_url: Some("https://relay.example/v1".to_string()),
            auth_token: Some(secret.to_string()),
            model: Some("gpt-5.4".to_string()),
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: Some("third_party_model".to_string()),
            account: None,
            tags: Some(vec!["stable".to_string()]),
            usage_count: Some(0),
            enabled: Some(true),
            platform_data: Default::default(),
            ..Default::default()
        };
        profile
            .platform_data
            .insert("wire_api".into(), json!("responses"));
        profile
            .platform_data
            .insert("env_key".into(), json!("MISTRAL_API_KEY"));
        profile
            .platform_data
            .insert("auth_mode".into(), json!("provider_env_key"));
        profile
    }

    #[test]
    fn codex_update_profile_rename_migrates_profile_secret_and_current_profile() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr-root");
        let codex_dir = temp_dir.path().join("codex-home");
        fs::create_dir_all(&ccr_root).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_codex_dir = std::env::var("CCR_CODEX_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", &ccr_root);
            std::env::set_var("CCR_CODEX_DIR", &codex_dir);
        }

        let result = (|| -> Result<(), String> {
            write_file_store_config(&codex_dir);

            let platform = CodexPlatform::new().map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
            platform
                .save_profile("ice", &build_provider_env_profile("sk-old"))
                .map_err(|e| format!("保存初始 profile 失败: {e}"))?;
            platform
                .apply_profile("ice")
                .map_err(|e| format!("应用初始 profile 失败: {e}"))?;

            let response = update_codex_profile_payload(
                "ice".to_string(),
                json!({
                    "name": "ice-renamed",
                    "auth_token": "sk-new"
                }),
            )?;

            assert_eq!(response["name"].as_str(), Some("ice-renamed"));

            let paths = PlatformPaths::new(Platform::Codex)
                .map_err(|e| format!("解析 Codex 路径失败: {e}"))?;
            let raw_profiles = load_profiles_from_toml(&paths.profiles_file)
                .map_err(|e| format!("读取 profiles.toml 失败: {e}"))?;
            assert!(!raw_profiles.contains_key("ice"));
            assert!(raw_profiles.contains_key("ice-renamed"));

            let secret_store =
                fs::read_to_string(ccr_root.join("platforms/codex/profile_secrets.json"))
                    .map_err(|e| format!("读取 secret store 失败: {e}"))?;
            assert!(!secret_store.contains("\"ice\""));
            assert!(secret_store.contains("ice-renamed"));
            assert!(secret_store.contains("sk-new"));

            let profiles_file = fs::read_to_string(ccr_root.join("platforms/codex/profiles.toml"))
                .map_err(|e| format!("读取 profiles.toml 原文失败: {e}"))?;
            assert!(profiles_file.contains("current_config = \"ice-renamed\""));

            assert_eq!(
                get_current_profile_from_registry("codex")
                    .map_err(|e| format!("读取注册表 current_profile 失败: {e}"))?,
                Some("ice-renamed".to_string())
            );
            assert_eq!(
                platform
                    .get_current_profile()
                    .map_err(|e| format!("读取平台 current_profile 失败: {e}"))?,
                Some("ice-renamed".to_string())
            );

            let env_export = platform
                .export_profile_env("ice-renamed")
                .map_err(|e| format!("导出环境变量失败: {e}"))?;
            assert_eq!(
                env_export.get("MISTRAL_API_KEY"),
                Some(&"sk-new".to_string())
            );

            let shell_export = platform
                .export_profile_shell_script("ice-renamed")
                .map_err(|e| format!("导出 shell 脚本失败: {e}"))?;
            assert!(shell_export.contains("MISTRAL_API_KEY"));
            assert!(shell_export.contains("sk-new"));

            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_CODEX_DIR", previous_codex_dir);
        result.unwrap();
    }

    #[test]
    fn codex_update_profile_rename_rejects_conflicting_target_without_writing_partial_state() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr-root");
        let codex_dir = temp_dir.path().join("codex-home");
        fs::create_dir_all(&ccr_root).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_codex_dir = std::env::var("CCR_CODEX_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", &ccr_root);
            std::env::set_var("CCR_CODEX_DIR", &codex_dir);
        }

        let result = (|| -> Result<(), String> {
            write_file_store_config(&codex_dir);

            let platform = CodexPlatform::new().map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
            platform
                .save_profile("ice", &build_provider_env_profile("sk-old"))
                .map_err(|e| format!("保存旧 profile 失败: {e}"))?;
            platform
                .save_profile("ice-renamed", &build_provider_env_profile("sk-existing"))
                .map_err(|e| format!("保存冲突 profile 失败: {e}"))?;

            let error = update_codex_profile_payload(
                "ice".to_string(),
                json!({
                    "name": "ice-renamed"
                }),
            )
            .unwrap_err();
            assert!(error.contains("已存在"));

            let paths = PlatformPaths::new(Platform::Codex)
                .map_err(|e| format!("解析 Codex 路径失败: {e}"))?;
            let raw_profiles = load_profiles_from_toml(&paths.profiles_file)
                .map_err(|e| format!("读取 profiles.toml 失败: {e}"))?;
            assert!(raw_profiles.contains_key("ice"));
            assert!(raw_profiles.contains_key("ice-renamed"));

            let secret_store =
                fs::read_to_string(ccr_root.join("platforms/codex/profile_secrets.json"))
                    .map_err(|e| format!("读取 secret store 失败: {e}"))?;
            assert!(secret_store.contains("ice"));
            assert!(secret_store.contains("ice-renamed"));
            assert!(secret_store.contains("sk-old"));
            assert!(secret_store.contains("sk-existing"));

            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_CODEX_DIR", previous_codex_dir);
        result.unwrap();
    }

    #[test]
    fn codex_update_profile_same_target_name_updates_in_place() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr-root");
        let codex_dir = temp_dir.path().join("codex-home");
        fs::create_dir_all(&ccr_root).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_codex_dir = std::env::var("CCR_CODEX_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", &ccr_root);
            std::env::set_var("CCR_CODEX_DIR", &codex_dir);
        }

        let result = (|| -> Result<(), String> {
            write_file_store_config(&codex_dir);

            let platform = CodexPlatform::new().map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
            platform
                .save_profile("ice", &build_provider_env_profile("sk-old"))
                .map_err(|e| format!("保存旧 profile 失败: {e}"))?;

            let response = update_codex_profile_payload(
                "ice".to_string(),
                json!({
                    "name": "ice",
                    "auth_token": "sk-new"
                }),
            )?;
            assert_eq!(response["name"].as_str(), Some("ice"));

            let paths = PlatformPaths::new(Platform::Codex)
                .map_err(|e| format!("解析 Codex 路径失败: {e}"))?;
            let raw_profiles = load_profiles_from_toml(&paths.profiles_file)
                .map_err(|e| format!("读取 profiles.toml 失败: {e}"))?;
            assert_eq!(raw_profiles.len(), 1);
            assert!(raw_profiles.contains_key("ice"));

            let env_export = platform
                .export_profile_env("ice")
                .map_err(|e| format!("导出环境变量失败: {e}"))?;
            assert_eq!(
                env_export.get("MISTRAL_API_KEY"),
                Some(&"sk-new".to_string())
            );

            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_CODEX_DIR", previous_codex_dir);
        result.unwrap();
    }

    #[test]
    fn codex_update_profile_rejects_blank_target_name() {
        let _guard = crate::test_support::lock_env();
        let temp_dir = tempfile::tempdir().unwrap();
        let ccr_root = temp_dir.path().join("ccr-root");
        let codex_dir = temp_dir.path().join("codex-home");
        fs::create_dir_all(&ccr_root).unwrap();
        fs::create_dir_all(&codex_dir).unwrap();

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_codex_dir = std::env::var("CCR_CODEX_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", &ccr_root);
            std::env::set_var("CCR_CODEX_DIR", &codex_dir);
        }

        let result = (|| -> Result<(), String> {
            write_file_store_config(&codex_dir);

            let platform = CodexPlatform::new().map_err(|e| format!("创建 Codex 平台失败: {e}"))?;
            platform
                .save_profile("ice", &build_provider_env_profile("sk-old"))
                .map_err(|e| format!("保存初始 profile 失败: {e}"))?;

            let error = update_codex_profile_payload(
                "ice".to_string(),
                json!({
                    "name": "   "
                }),
            )
            .unwrap_err();
            assert!(error.contains("不能为空"));

            let paths = PlatformPaths::new(Platform::Codex)
                .map_err(|e| format!("解析 Codex 路径失败: {e}"))?;
            let raw_profiles = load_profiles_from_toml(&paths.profiles_file)
                .map_err(|e| format!("读取 profiles.toml 失败: {e}"))?;
            assert!(raw_profiles.contains_key("ice"));

            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_CODEX_DIR", previous_codex_dir);
        result.unwrap();
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
