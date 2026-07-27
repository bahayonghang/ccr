#![allow(clippy::unwrap_used)]
// 🧪 CCR 集成测试
// 测试核心功能的端到端工作流程

use ccr_cli::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use ccr_cli::services::{ConfigService, SettingsService};
use indexmap::IndexMap;
use std::sync::Arc;
use tempfile::tempdir;

/// 创建测试配置节
fn create_test_section(name: &str) -> ConfigSection {
    ConfigSection {
        description: Some(format!("Test config {}", name)),
        base_url: Some(format!("https://api.{}.com", name)),
        auth_token: Some(ccr_core::Secret::new(format!("sk-test-token-{}", name))),
        model: Some("test-model".into()),
        small_fast_model: Some("test-small".into()),
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    }
}

#[test]
fn test_config_service_workflow() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建初始配置
    let mut config = CcsConfig {
        default_config: "test1".into(),
        current_config: "test1".into(),
        settings: ccr_cli::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("test1".into(), create_test_section("test1"));

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    // 使用 ConfigService
    let service = ConfigService::new(manager);

    // 测试列出配置
    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 1);
    assert_eq!(list.current_config, "test1");

    // 测试添加配置
    service
        .add_config("test2".into(), create_test_section("test2"))
        .unwrap();

    // 验证添加成功
    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 2);

    // 测试获取配置
    let info = service.get_config("test2").unwrap();
    assert_eq!(info.name, "test2");
    assert_eq!(info.description, "Test config test2");

    // 测试删除配置
    service.delete_config("test2").unwrap();
    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 1);
}

#[test]
fn test_settings_service_workflow() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = ccr::LockManager::new(lock_dir);
    let settings_manager = Arc::new(ccr_cli::managers::settings::SettingsManager::new(
        settings_path,
        backup_dir,
        lock_manager,
    ));

    let service = SettingsService::new(Arc::clone(&settings_manager));

    // 测试应用配置
    let section = create_test_section("test");
    service.apply_config(&section).unwrap();

    // 验证设置
    let settings = service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-test-token-test".to_string())
    );

    // 测试备份（直接使用 settings_manager）
    let backup_path = settings_manager.backup(Some("test_backup")).unwrap();
    assert!(backup_path.exists());

    // 测试列出备份
    let backups = service.list_backups().unwrap();
    assert_eq!(backups.len(), 1);
}

#[test]
fn test_validation_trait() {
    use ccr::Validatable;

    // 测试有效配置
    let valid_section = create_test_section("valid");
    assert!(valid_section.validate().is_ok());

    // 测试无效配置(空 base_url)
    let invalid_section = ConfigSection {
        description: Some("Invalid".into()),
        base_url: Some("".into()),
        auth_token: Some("token".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    };
    assert!(invalid_section.validate().is_err());

    // 测试无效配置(无效 URL 格式)
    let invalid_section = ConfigSection {
        description: Some("Invalid".into()),
        base_url: Some("not-a-url".into()),
        auth_token: Some("token".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    };
    assert!(invalid_section.validate().is_err());
}
