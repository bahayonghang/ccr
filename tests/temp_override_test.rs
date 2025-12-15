// 🎯 临时Token功能集成测试
// 测试临时配置覆盖功能的完整工作流程

use ccr::{ConfigManager, ConfigSection, SettingsManager, TempOverride, TempOverrideManager};
use tempfile::TempDir;

/// 创建测试环境
fn setup_test_env() -> (TempDir, ConfigManager, SettingsManager, TempOverrideManager) {
    let temp_dir = tempfile::tempdir().unwrap();

    // 创建配置管理器
    let config_path = temp_dir.path().join(".ccs_config.toml");
    let config_manager = ConfigManager::new(&config_path);

    // 创建设置管理器
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");
    let lock_manager = ccr::LockManager::new(lock_dir);
    let settings_manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

    // 创建临时配置管理器
    let override_path = temp_dir.path().join("temp_override.json");
    let temp_manager = TempOverrideManager::new(override_path);

    (temp_dir, config_manager, settings_manager, temp_manager)
}

/// 创建测试配置节
fn create_test_config() -> ConfigSection {
    ConfigSection {
        description: Some("Test Config".to_string()),
        base_url: Some("https://api.test.com".to_string()),
        auth_token: Some("sk-test-original".to_string()),
        model: Some("test-model".to_string()),
        small_fast_model: Some("test-small".to_string()),
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: indexmap::IndexMap::new(),
    }
}

#[test]
fn test_temp_override_basic_workflow() {
    let (_temp_dir, _config_manager, settings_manager, temp_manager) = setup_test_env();

    // 1. 创建原始设置
    let config = create_test_config();
    let mut settings = ccr::ClaudeSettings::new();
    settings.update_from_config(&config);
    settings_manager.save_atomic(&settings).unwrap();

    // 验证原始token
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-test-original".to_string())
    );

    // 2. 设置临时token
    let temp_override = TempOverride::new("sk-temp-free".to_string());
    temp_manager.save(&temp_override).unwrap();

    // 3. 模拟 switch 命令：加载临时覆盖并应用
    let loaded_temp = temp_manager.load().unwrap();
    assert!(loaded_temp.is_some());

    let mut new_settings = settings_manager.load().unwrap();
    if let Some(temp) = loaded_temp
        && let Some(temp_token) = &temp.auth_token
    {
        new_settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".to_string(), temp_token.clone());
    }
    settings_manager.save_atomic(&new_settings).unwrap();

    // 4. 验证临时token已生效
    let final_settings = settings_manager.load().unwrap();
    assert_eq!(
        final_settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-temp-free".to_string())
    );

    // 5. 清除临时配置（模拟一次性使用）
    temp_manager.clear().unwrap();
    assert!(!temp_manager.override_path().exists());
}

#[test]
fn test_temp_override_multiple_fields() {
    let (_temp_dir, _config_manager, settings_manager, temp_manager) = setup_test_env();

    // 创建原始设置
    let config = create_test_config();
    let mut settings = ccr::ClaudeSettings::new();
    settings.update_from_config(&config);
    settings_manager.save_atomic(&settings).unwrap();

    // 创建多字段临时覆盖
    let mut temp_override = TempOverride::new("sk-temp-token".to_string());
    temp_override.base_url = Some("https://api.temp.com".to_string());
    temp_override.model = Some("temp-model".to_string());
    temp_manager.save(&temp_override).unwrap();

    // 应用临时覆盖
    let mut new_settings = settings_manager.load().unwrap();
    if let Some(temp) = temp_manager.load().unwrap() {
        if let Some(token) = &temp.auth_token {
            new_settings
                .env
                .insert("ANTHROPIC_AUTH_TOKEN".to_string(), token.clone());
        }
        if let Some(url) = &temp.base_url {
            new_settings
                .env
                .insert("ANTHROPIC_BASE_URL".to_string(), url.clone());
        }
        if let Some(model) = &temp.model {
            new_settings
                .env
                .insert("ANTHROPIC_MODEL".to_string(), model.clone());
        }
    }
    settings_manager.save_atomic(&new_settings).unwrap();

    // 验证所有字段都被覆盖
    let final_settings = settings_manager.load().unwrap();
    assert_eq!(
        final_settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-temp-token".to_string())
    );
    assert_eq!(
        final_settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.temp.com".to_string())
    );
    assert_eq!(
        final_settings.env.get("ANTHROPIC_MODEL"),
        Some(&"temp-model".to_string())
    );
}

#[test]
fn test_temp_override_no_interference_with_other_vars() {
    let (_temp_dir, _config_manager, settings_manager, temp_manager) = setup_test_env();

    // 创建包含其他环境变量的设置
    let mut settings = ccr::ClaudeSettings::new();
    settings.env.insert(
        "ANTHROPIC_AUTH_TOKEN".to_string(),
        "sk-original".to_string(),
    );
    settings.env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        "https://original.com".to_string(),
    );
    settings
        .env
        .insert("OTHER_VAR".to_string(), "keep-this".to_string());
    settings_manager.save_atomic(&settings).unwrap();

    // 只覆盖token
    let temp_override = TempOverride::new("sk-temp".to_string());
    temp_manager.save(&temp_override).unwrap();

    // 应用覆盖
    let mut new_settings = settings_manager.load().unwrap();
    if let Some(temp) = temp_manager.load().unwrap()
        && let Some(token) = &temp.auth_token
    {
        new_settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".to_string(), token.clone());
    }
    settings_manager.save_atomic(&new_settings).unwrap();

    // 验证：token被覆盖，其他变量保持不变
    let final_settings = settings_manager.load().unwrap();
    assert_eq!(
        final_settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-temp".to_string())
    );
    assert_eq!(
        final_settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://original.com".to_string())
    );
    assert_eq!(
        final_settings.env.get("OTHER_VAR"),
        Some(&"keep-this".to_string())
    );
}

#[test]
fn test_temp_override_count() {
    let temp1 = TempOverride::new("sk-test".to_string());
    assert_eq!(temp1.override_count(), 1);

    let mut temp2 = TempOverride::new("sk-test".to_string());
    temp2.base_url = Some("https://test.com".to_string());
    assert_eq!(temp2.override_count(), 2);

    let mut temp3 = TempOverride::new("sk-test".to_string());
    temp3.base_url = Some("https://test.com".to_string());
    temp3.model = Some("test-model".to_string());
    temp3.small_fast_model = Some("test-small".to_string());
    assert_eq!(temp3.override_count(), 4);
}
