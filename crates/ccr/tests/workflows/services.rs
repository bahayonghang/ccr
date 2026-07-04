#![allow(clippy::unwrap_used)]
// 🧪 Service 层工作流集成测试
// 测试 ConfigService, SettingsService, HistoryService, BackupService 的业务流程

use ccr::LockManager;
use ccr::Validatable;
use ccr::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use ccr::managers::settings::SettingsManager;
use ccr::services::{BackupService, ConfigService, HistoryService, SettingsService};
use indexmap::IndexMap;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;

/// 创建测试用配置节
fn create_test_section(name: &str) -> ConfigSection {
    ConfigSection {
        description: Some(format!("Test config {}", name)),
        base_url: Some(format!("https://api.{}.com", name)),
        auth_token: Some(ccr_core::Secret::new(format!("sk-test-token-{}", name))),
        model: Some("claude-sonnet-4".into()),
        small_fast_model: Some("claude-haiku".into()),
        provider: Some(name.to_string()),
        provider_type: None,
        account: None,
        tags: Some(vec!["test".into()]),
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
        ..Default::default()
    }
}

// ═══════════════════════════════════════════════════════════════
// ConfigService 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_service_crud_workflow() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建初始配置
    let mut config = CcsConfig {
        default_config: "initial".into(),
        current_config: "initial".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("initial".into(), create_test_section("initial"));

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 测试添加配置
    service
        .add_config("new_config".into(), create_test_section("new"))
        .unwrap();

    // 验证列表
    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 2);
    assert!(list.configs.iter().any(|c| c.name == "new_config"));

    // 测试获取配置
    let config_info = service.get_config("new_config").unwrap();
    assert_eq!(config_info.name, "new_config");
    assert_eq!(config_info.description, "Test config new");

    // 测试更新配置
    let mut updated_section = create_test_section("updated");
    updated_section.description = Some("Updated description".into());
    service
        .update_config("new_config", "renamed_config".into(), updated_section)
        .unwrap();

    let list = service.list_configs().unwrap();
    assert!(list.configs.iter().any(|c| c.name == "renamed_config"));
    assert!(!list.configs.iter().any(|c| c.name == "new_config"));

    // 测试删除配置（不能删除当前配置或默认配置）
    assert!(service.delete_config("initial").is_err()); // 当前配置不能删除

    // 先切换当前配置，然后修改默认配置，最后才能删除
    service.set_current("renamed_config").unwrap();

    // 修改默认配置为新的配置
    let mut config = service.load_config().unwrap();
    config.default_config = "renamed_config".to_string();
    service.save_config(&config).unwrap();

    // 现在可以删除 initial 了
    service.delete_config("initial").unwrap();

    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 1);
    assert_eq!(list.configs[0].name, "renamed_config");
}

#[test]
fn test_config_service_validation() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建包含有效和无效配置的配置文件
    let mut config = CcsConfig {
        default_config: "valid".into(),
        current_config: "valid".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("valid".into(), create_test_section("valid"));

    // 添加无效配置
    let invalid_section = ConfigSection {
        description: Some("Invalid".into()),
        base_url: Some("".into()), // 空 URL
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
    config.sections.insert("invalid".into(), invalid_section);

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 验证所有配置
    let report = service.validate_all().unwrap();
    assert_eq!(report.valid_count, 1);
    assert_eq!(report.invalid_count, 1);

    // 检查结果
    let valid_result = report
        .results
        .iter()
        .find(|(name, _, _)| name == "valid")
        .unwrap();
    assert!(valid_result.1); // is_valid

    let invalid_result = report
        .results
        .iter()
        .find(|(name, _, _)| name == "invalid")
        .unwrap();
    assert!(!invalid_result.1); // is_invalid
    assert!(invalid_result.2.is_some()); // 有错误消息
}

#[test]
fn test_config_service_export_import() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建配置
    let mut config = CcsConfig {
        default_config: "test".into(),
        current_config: "test".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("test".into(), create_test_section("test"));

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 测试导出（包含密钥）
    let exported_with_secrets = service.export_config(true).unwrap();
    assert!(exported_with_secrets.contains("sk-test-token-test"));

    // 测试导出（不包含密钥）
    let exported_without_secrets = service.export_config(false).unwrap();
    assert!(!exported_without_secrets.contains("sk-test-token-test"));
    assert!(exported_without_secrets.contains("..."));

    // 测试导入（替换模式）
    let import_config = r#"
default_config = "imported"
current_config = "imported"

[imported]
description = "Imported config"
base_url = "https://api.imported.com"
auth_token = "sk-imported-token"
    "#;

    let result = service
        .import_config(
            import_config,
            ccr::services::config_service::ImportMode::Replace,
            false,
        )
        .unwrap();
    assert_eq!(result.added, 1);

    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 1);
    assert_eq!(list.configs[0].name, "imported");
}

// ═══════════════════════════════════════════════════════════════
// SettingsService 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_settings_service_apply_and_get() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(lock_dir);
    let settings_manager = Arc::new(SettingsManager::new(
        &settings_path,
        &backup_dir,
        lock_manager,
    ));
    let service = SettingsService::new(settings_manager);

    // 应用配置
    let section = create_test_section("test");
    service.apply_config(&section).unwrap();

    // 获取当前设置
    let settings = service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-test-token-test".to_string())
    );
}

#[test]
fn test_settings_service_backup_workflow() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(lock_dir);
    let settings_manager = Arc::new(SettingsManager::new(
        &settings_path,
        &backup_dir,
        lock_manager,
    ));
    let service = SettingsService::new(Arc::clone(&settings_manager));

    // 创建初始设置
    service
        .apply_config(&create_test_section("initial"))
        .unwrap();

    // 备份（直接使用 settings_manager）
    let backup_path = settings_manager.backup(Some("test_backup")).unwrap();
    assert!(backup_path.exists());

    // 列出备份
    let backups = service.list_backups().unwrap();
    assert_eq!(backups.len(), 1);
    assert!(backups[0].filename.contains("test_backup"));

    // 修改设置
    service
        .apply_config(&create_test_section("modified"))
        .unwrap();

    // 恢复
    service.restore_settings(&backup_path).unwrap();

    // 验证恢复
    let settings = service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.initial.com".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════
// HistoryService 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_history_service_record_and_query() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();

    let history_manager = Arc::new(ccr::managers::HistoryManager::new(db));
    let service = HistoryService::new(history_manager);

    // 记录操作
    let entry = ccr::managers::HistoryEntry::new(
        ccr::managers::OperationType::Switch,
        ccr::managers::OperationDetails {
            from_config: Some("old".into()),
            to_config: Some("new".into()),
            backup_path: None,
            extra: None,
        },
        ccr::managers::OperationResult::Success,
    );

    service.record_operation(entry).unwrap();

    // 查询最近记录
    let recent = service.get_recent(10).unwrap();
    assert_eq!(recent.len(), 1);

    // 统计信息
    let stats = service.get_stats().unwrap();
    assert_eq!(stats.total_operations, 1);
    assert_eq!(stats.successful_operations, 1);
}

// ═══════════════════════════════════════════════════════════════
// BackupService 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_backup_service_clean_workflow() {
    let temp_dir = tempdir().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    // 创建多个备份文件
    let old_backup = backup_dir.join("old_backup.bak");
    let new_backup = backup_dir.join("new_backup.bak");
    let other_file = backup_dir.join("other.txt");

    fs::File::create(&old_backup)
        .unwrap()
        .write_all(b"old")
        .unwrap();
    fs::File::create(&new_backup)
        .unwrap()
        .write_all(b"new")
        .unwrap();
    fs::File::create(&other_file)
        .unwrap()
        .write_all(b"other")
        .unwrap();

    // 设置旧文件的时间为 10 天前
    let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(10 * 24 * 60 * 60);
    filetime::set_file_mtime(&old_backup, filetime::FileTime::from_system_time(old_time)).unwrap();

    let service = BackupService::new(backup_dir);

    // Dry run
    let result = service.clean_old_backups(7, true).unwrap();
    assert_eq!(result.deleted_count, 1); // old_backup 应该被标记删除
    assert_eq!(result.skipped_count, 1); // new_backup 应该被保留
    assert!(old_backup.exists()); // dry run 不应实际删除

    // 实际清理
    let result = service.clean_old_backups(7, false).unwrap();
    assert_eq!(result.deleted_count, 1);
    assert!(!old_backup.exists()); // 应该被删除
    assert!(new_backup.exists()); // 应该保留
    assert!(other_file.exists()); // 非 .bak 文件应该保留
}

#[test]
fn test_backup_service_scan() {
    let temp_dir = tempdir().unwrap();
    let backup_dir = temp_dir.path().to_path_buf();

    // 创建多个备份文件
    for i in 0..5 {
        let filename = format!("backup{}.bak", i);
        fs::File::create(backup_dir.join(&filename))
            .unwrap()
            .write_all(format!("content{}", i).as_bytes())
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10)); // 确保时间戳不同
    }

    let service = BackupService::new(backup_dir.clone());
    let backups = service.scan_backup_directory().unwrap();

    assert_eq!(backups.len(), 5);
    // 验证按修改时间倒序排列（最新的在前）
    for i in 0..backups.len() - 1 {
        assert!(backups[i].modified >= backups[i + 1].modified);
    }
}

// ═══════════════════════════════════════════════════════════════
// 跨 Service 集成测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_complete_config_switch_workflow() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    // 初始化配置
    let mut config = CcsConfig {
        default_config: "config1".into(),
        current_config: "config1".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("config1".into(), create_test_section("config1"));
    config
        .sections
        .insert("config2".into(), create_test_section("config2"));

    let config_manager = Arc::new(ConfigManager::new(&config_path));
    config_manager.save(&config).unwrap();

    // 初始化服务
    let config_service = ConfigService::new(config_manager);

    let lock_manager = LockManager::new(&lock_dir);
    let settings_manager = Arc::new(SettingsManager::new(
        &settings_path,
        &backup_dir,
        lock_manager,
    ));
    let settings_service = SettingsService::new(Arc::clone(&settings_manager));

    // 步骤 1: 应用 config1
    let section1 = config_service
        .load_config()
        .unwrap()
        .get_section("config1")
        .unwrap()
        .clone();
    settings_service.apply_config(&section1).unwrap();

    let settings = settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.config1.com".to_string())
    );

    // 步骤 2: 备份当前设置（直接使用 settings_manager）
    let backup_path = settings_manager.backup(Some("before_switch")).unwrap();
    assert!(backup_path.exists());

    // 步骤 3: 切换到 config2
    let section2 = config_service
        .load_config()
        .unwrap()
        .get_section("config2")
        .unwrap()
        .clone();
    settings_service.apply_config(&section2).unwrap();

    let settings = settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.config2.com".to_string())
    );

    // 步骤 4: 验证备份存在
    let backups = settings_service.list_backups().unwrap();
    assert_eq!(backups.len(), 1);

    // 步骤 5: 恢复到之前的配置
    settings_service.restore_settings(&backup_path).unwrap();

    let settings = settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.config1.com".to_string())
    );
}

#[test]
fn test_config_service_list_with_classification() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建带分类信息的配置
    let mut config = CcsConfig {
        default_config: "anthropic".into(),
        current_config: "anthropic".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };

    let mut anthropic = create_test_section("anthropic");
    anthropic.provider = Some("anthropic".into());
    anthropic.provider_type = Some(ccr::managers::config::ProviderType::OfficialRelay);
    anthropic.tags = Some(vec!["official".into(), "stable".into()]);

    let mut anyrouter = create_test_section("anyrouter");
    anyrouter.provider = Some("anyrouter".into());
    anyrouter.provider_type = Some(ccr::managers::config::ProviderType::OfficialRelay);
    anyrouter.tags = Some(vec!["relay".into(), "fast".into()]);

    config.sections.insert("anthropic".into(), anthropic);
    config.sections.insert("anyrouter".into(), anyrouter);

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 列出配置
    let list = service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 2);

    // 验证分类信息
    let anthropic_info = list.configs.iter().find(|c| c.name == "anthropic").unwrap();
    assert_eq!(anthropic_info.provider, Some("anthropic".into()));
    assert_eq!(anthropic_info.provider_type, Some("official_relay".into()));
    assert_eq!(
        anthropic_info.tags,
        Some(vec!["official".into(), "stable".into()])
    );
}

#[test]
fn test_settings_service_multiple_switches() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(&lock_dir);
    let settings_manager = Arc::new(SettingsManager::new(
        &settings_path,
        &backup_dir,
        lock_manager,
    ));
    let service = SettingsService::new(settings_manager);

    // 多次切换配置
    for i in 1..=5 {
        let section = create_test_section(&format!("config{}", i));
        service.apply_config(&section).unwrap();

        let settings = service.get_current_settings().unwrap();
        assert_eq!(
            settings.env.get("ANTHROPIC_BASE_URL"),
            Some(&format!("https://api.config{}.com", i))
        );

        // 每次切换都验证配置有效
        assert!(settings.validate().is_ok());
    }

    // 验证备份数量
    let backups = service.list_backups().unwrap();
    // 注意：apply_config 不自动备份，所以备份数应该是 0
    // 除非在切换前手动备份
    assert_eq!(backups.len(), 0);
}

#[test]
fn test_history_service_workflow() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();

    let history_manager = Arc::new(ccr::managers::HistoryManager::new(db));
    let service = HistoryService::new(history_manager);

    // 记录多个操作
    for i in 0..10 {
        let entry = ccr::managers::HistoryEntry::new(
            if i % 2 == 0 {
                ccr::managers::OperationType::Switch
            } else {
                ccr::managers::OperationType::Backup
            },
            ccr::managers::OperationDetails {
                from_config: Some(format!("config{}", i)),
                to_config: Some(format!("config{}", i + 1)),
                backup_path: None,
                extra: None,
            },
            ccr::managers::OperationResult::Success,
        );
        service.record_operation(entry).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5)); // 确保时间戳不同
    }

    // 测试查询最近记录
    let recent = service.get_recent(5).unwrap();
    assert_eq!(recent.len(), 5);

    // 测试按类型筛选
    let switch_ops = service
        .filter_by_type(ccr::managers::OperationType::Switch)
        .unwrap();
    assert_eq!(switch_ops.len(), 5);

    let backup_ops = service
        .filter_by_type(ccr::managers::OperationType::Backup)
        .unwrap();
    assert_eq!(backup_ops.len(), 5);

    // 测试统计
    let stats = service.get_stats().unwrap();
    assert_eq!(stats.total_operations, 10);
    assert_eq!(stats.successful_operations, 10);
    assert!(stats.last_operation.is_some());
}

// ═══════════════════════════════════════════════════════════════
// 服务层错误处理测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_service_error_handling() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = CcsConfig {
        default_config: "test".into(),
        current_config: "test".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("test".into(), create_test_section("test"));

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 测试添加重复配置
    let result = service.add_config("test".into(), create_test_section("duplicate"));
    assert!(result.is_err());

    // 测试删除当前配置
    let result = service.delete_config("test");
    assert!(result.is_err());

    // 测试删除默认配置
    let result = service.delete_config("test");
    assert!(result.is_err());

    // 测试获取不存在的配置
    let result = service.get_config("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_settings_service_error_handling() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(&lock_dir);
    let settings_manager = Arc::new(SettingsManager::new(
        &settings_path,
        &backup_dir,
        lock_manager,
    ));
    let service = SettingsService::new(Arc::clone(&settings_manager));

    // 测试备份不存在的设置（直接使用 settings_manager）
    let result = settings_manager.backup(Some("test"));
    assert!(result.is_err());

    // 测试恢复不存在的备份
    let result = service.restore_settings(temp_dir.path().join("nonexistent.bak").as_path());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════
// Enable/Disable 功能测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_enable_disable_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建初始配置
    let mut config = CcsConfig {
        default_config: "test".into(),
        current_config: "test".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };

    let mut section = create_test_section("test");
    section.enabled = Some(true); // 初始状态为启用
    config.sections.insert("test".into(), section);

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 测试禁用配置
    service.disable_config("test").unwrap();

    // 验证配置已被禁用
    let config_info = service.get_config("test").unwrap();
    assert!(!config_info.enabled);

    // 测试启用配置
    service.enable_config("test").unwrap();

    // 验证配置已被启用
    let config_info = service.get_config("test").unwrap();
    assert!(config_info.enabled);
}

#[test]
fn test_usage_count_increment() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建初始配置
    let mut config = CcsConfig {
        default_config: "config1".into(),
        current_config: "config1".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };

    let mut section1 = create_test_section("config1");
    section1.usage_count = Some(0);
    config.sections.insert("config1".into(), section1);

    let mut section2 = create_test_section("config2");
    section2.usage_count = Some(0);
    config.sections.insert("config2".into(), section2);

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 切换到 config2，应该自动递增 usage_count
    service.set_current("config2").unwrap();

    let config_info = service.get_config("config2").unwrap();
    assert_eq!(config_info.usage_count, 1);

    // 再次切换到 config2
    service.set_current("config1").unwrap();
    service.set_current("config2").unwrap();

    let config_info = service.get_config("config2").unwrap();
    assert_eq!(config_info.usage_count, 2);
}

#[test]
fn test_auto_complete_missing_fields() {
    use ccr::AutoCompletable;

    // 创建一个缺少新字段的配置节
    let mut section = ConfigSection {
        description: Some("Test".into()),
        base_url: Some("https://api.test.com".into()),
        auth_token: Some("sk-test".into()),
        model: Some("model".into()),
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
        usage_count: None, // 缺失字段
        enabled: None,     // 缺失字段
        other: IndexMap::new(),
        ..Default::default()
    };

    // 调用自动补全
    let modified = section.auto_complete();

    // 验证字段已被自动补全
    assert!(modified); // 应该返回 true 表示有字段被修改
    assert_eq!(section.usage_count, Some(0)); // 默认值
    assert_eq!(section.enabled, Some(true)); // 默认值

    // 再次调用不应修改
    let modified_again = section.auto_complete();
    assert!(!modified_again); // 应该返回 false，因为已经完整
}

#[test]
fn test_disable_prevents_switch() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建初始配置
    let mut config = CcsConfig {
        default_config: "config1".into(),
        current_config: "config1".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };

    config
        .sections
        .insert("config1".into(), create_test_section("config1"));

    let mut section2 = create_test_section("config2");
    section2.enabled = Some(false); // 禁用状态
    config.sections.insert("config2".into(), section2);

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 尝试切换到被禁用的配置，应该失败
    let result = service.set_current("config2");
    assert!(result.is_err());

    // 验证当前配置没有改变
    let current = service.get_current().unwrap();
    assert_eq!(current.name, "config1");
}

#[test]
fn test_enable_disable_nonexistent_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = CcsConfig {
        default_config: "test".into(),
        current_config: "test".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("test".into(), create_test_section("test"));

    let manager = Arc::new(ConfigManager::new(&config_path));
    manager.save(&config).unwrap();

    let service = ConfigService::new(manager);

    // 测试启用不存在的配置
    let result = service.enable_config("nonexistent");
    assert!(result.is_err());

    // 测试禁用不存在的配置
    let result = service.disable_config("nonexistent");
    assert!(result.is_err());
}
