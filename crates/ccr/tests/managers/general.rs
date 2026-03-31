#![allow(clippy::unwrap_used)]
// 🧪 Manager 层集成测试
// 测试 ConfigManager, SettingsManager, HistoryManager 的核心功能

use ccr::LockManager;
use ccr::Validatable;
use ccr::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use ccr::managers::settings::{ClaudeSettings, SettingsManager};
use ccr::managers::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
};
use indexmap::IndexMap;
use tempfile::tempdir;

/// 创建测试用配置节
fn create_test_config_section(name: &str) -> ConfigSection {
    ConfigSection {
        description: Some(format!("Test config for {}", name)),
        base_url: Some(format!("https://api.{}.com", name)),
        auth_token: Some(format!("sk-test-token-{}", name)),
        model: Some("claude-sonnet-4".into()),
        small_fast_model: Some("claude-haiku".into()),
        provider: Some(name.to_string()),
        provider_type: None,
        account: None,
        tags: Some(vec!["test".into(), "integration".into()]),
        usage_count: Some(0),
        enabled: Some(true),
        other: IndexMap::new(),
    }
}

// ═══════════════════════════════════════════════════════════════
// ConfigManager 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_manager_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建配置
    let mut config = CcsConfig {
        default_config: "anthropic".into(),
        current_config: "anthropic".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("anthropic".into(), create_test_config_section("anthropic"));
    config
        .sections
        .insert("anyrouter".into(), create_test_config_section("anyrouter"));

    // 保存
    let manager = ConfigManager::new(&config_path);
    manager.save(&config).unwrap();
    assert!(config_path.exists());

    // 加载并验证
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.default_config, "anthropic");
    assert_eq!(loaded.current_config, "anthropic");
    assert_eq!(loaded.sections.len(), 2);
    assert!(loaded.get_section("anthropic").is_ok());
    assert!(loaded.get_section("anyrouter").is_ok());
}

#[test]
fn test_config_manager_section_operations() {
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
        .insert("test".into(), create_test_config_section("test"));

    let manager = ConfigManager::new(&config_path);
    manager.save(&config).unwrap();

    // 测试获取配置节
    let mut loaded = manager.load().unwrap();
    assert!(loaded.get_section("test").is_ok());
    assert!(loaded.get_section("nonexistent").is_err());

    // 测试添加配置节
    loaded
        .sections
        .insert("new".into(), create_test_config_section("new"));
    manager.save(&loaded).unwrap();
    let reloaded = manager.load().unwrap();
    assert_eq!(reloaded.sections.len(), 2);

    // 测试切换当前配置
    let mut config = manager.load().unwrap();
    assert!(config.set_current("new").is_ok());
    assert_eq!(config.current_config, "new");
    assert!(config.set_current("nonexistent").is_err());
}

#[test]
fn test_config_section_validation() {
    // 有效配置
    let valid = create_test_config_section("test");
    assert!(valid.validate().is_ok());

    // 无效：缺少 base_url
    let invalid = ConfigSection {
        description: Some("Invalid".into()),
        base_url: None,
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
    };
    assert!(invalid.validate().is_err());

    // 无效：空 base_url
    let invalid = ConfigSection {
        base_url: Some("".into()),
        auth_token: Some("token".into()),
        ..invalid
    };
    assert!(invalid.validate().is_err());

    // 无效：URL 格式错误
    let invalid = ConfigSection {
        base_url: Some("not-a-url".into()),
        auth_token: Some("token".into()),
        ..invalid
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn test_config_sorting_and_filtering() {
    let mut config = CcsConfig {
        default_config: "c".into(),
        current_config: "c".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };

    // 添加多个配置（无序）
    config
        .sections
        .insert("zebra".into(), create_test_config_section("zebra"));
    config
        .sections
        .insert("apple".into(), create_test_config_section("apple"));
    config
        .sections
        .insert("mango".into(), create_test_config_section("mango"));

    // 排序前
    let names_before: Vec<&String> = config.sections.keys().collect();
    assert_eq!(names_before, vec!["zebra", "apple", "mango"]);

    // 排序
    config.sort_sections();

    // 排序后
    let names_after: Vec<&String> = config.sections.keys().collect();
    assert_eq!(names_after, vec!["apple", "mango", "zebra"]);
}

// ═══════════════════════════════════════════════════════════════
// SettingsManager 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_settings_manager_atomic_operations() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(lock_dir);
    let manager = SettingsManager::new(&settings_path, &backup_dir, lock_manager);

    // 创建设置
    let mut settings = ClaudeSettings::new();
    settings.env.insert(
        "ANTHROPIC_BASE_URL".into(),
        "https://api.anthropic.com".into(),
    );
    settings
        .env
        .insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-test-token".into());

    // 原子保存
    manager.save_atomic(&settings).unwrap();
    assert!(settings_path.exists());

    // 加载并验证
    let loaded = manager.load().unwrap();
    assert_eq!(
        loaded.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anthropic.com".to_string())
    );
    assert_eq!(
        loaded.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-test-token".to_string())
    );
}

#[test]
fn test_settings_update_from_config() {
    let mut settings = ClaudeSettings::new();
    let config = create_test_config_section("test");

    // 更新前
    assert!(settings.env.is_empty());

    // 从配置更新
    settings.update_from_config(&config);

    // 验证更新后的值
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-test-token-test".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_MODEL"),
        Some(&"claude-sonnet-4".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_SMALL_FAST_MODEL"),
        Some(&"claude-haiku".to_string())
    );
}

#[test]
fn test_settings_backup_and_restore() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    let lock_manager = LockManager::new(lock_dir);
    let manager = SettingsManager::new(&settings_path, &backup_dir, lock_manager);

    // 创建原始设置
    let mut original_settings = ClaudeSettings::new();
    original_settings
        .env
        .insert("ANTHROPIC_BASE_URL".into(), "original-url".into());
    manager.save_atomic(&original_settings).unwrap();

    // 备份
    let backup_path = manager.backup(Some("test_backup")).unwrap();
    assert!(backup_path.exists());
    assert!(backup_path.to_string_lossy().contains("test_backup"));

    // 修改设置
    let mut modified_settings = ClaudeSettings::new();
    modified_settings
        .env
        .insert("ANTHROPIC_BASE_URL".into(), "modified-url".into());
    manager.save_atomic(&modified_settings).unwrap();

    // 验证修改
    let loaded = manager.load().unwrap();
    assert_eq!(
        loaded.env.get("ANTHROPIC_BASE_URL"),
        Some(&"modified-url".to_string())
    );

    // 从备份恢复
    manager.restore(&backup_path).unwrap();

    // 验证恢复
    let restored = manager.load().unwrap();
    assert_eq!(
        restored.env.get("ANTHROPIC_BASE_URL"),
        Some(&"original-url".to_string())
    );
}

#[test]
fn test_settings_validation() {
    // 有效设置
    let mut valid_settings = ClaudeSettings::new();
    valid_settings
        .env
        .insert("ANTHROPIC_BASE_URL".into(), "https://api.test.com".into());
    valid_settings
        .env
        .insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-token".into());
    assert!(valid_settings.validate().is_ok());

    // 无效：缺少 base_url
    let mut invalid_settings = ClaudeSettings::new();
    invalid_settings
        .env
        .insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-token".into());
    assert!(invalid_settings.validate().is_err());

    // 无效：缺少 auth_token
    let mut invalid_settings = ClaudeSettings::new();
    invalid_settings
        .env
        .insert("ANTHROPIC_BASE_URL".into(), "https://api.test.com".into());
    assert!(invalid_settings.validate().is_err());
}

// ═══════════════════════════════════════════════════════════════
// HistoryManager 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_history_manager_add_and_load() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();
    let manager = HistoryManager::new(db);

    // 添加第一条记录
    let entry1 = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: Some("anthropic".into()),
            to_config: Some("anyrouter".into()),
            backup_path: None,
            extra: None,
        },
        OperationResult::Success,
    );
    manager.add(entry1).unwrap();

    // 添加第二条记录
    let entry2 = HistoryEntry::new(
        OperationType::Backup,
        OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: Some("/tmp/backup.json".into()),
            extra: None,
        },
        OperationResult::Success,
    );
    manager.add(entry2).unwrap();

    // 加载并验证（历史记录按时间倒序排列，最新的在前）
    let entries = manager.load().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].operation, OperationType::Backup); // 第二个添加的，更新
    assert_eq!(entries[1].operation, OperationType::Switch); // 第一个添加的，较早
}

#[test]
fn test_history_manager_filtering() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();
    let manager = HistoryManager::new(db);

    let details = OperationDetails {
        from_config: None,
        to_config: None,
        backup_path: None,
        extra: None,
    };

    // 添加不同类型的记录
    manager
        .add(HistoryEntry::new(
            OperationType::Switch,
            details.clone(),
            OperationResult::Success,
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Backup,
            details.clone(),
            OperationResult::Success,
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Switch,
            details.clone(),
            OperationResult::Success,
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Validate,
            details,
            OperationResult::Success,
        ))
        .unwrap();

    // 按类型筛选
    let switch_ops = manager.filter_by_operation(OperationType::Switch).unwrap();
    assert_eq!(switch_ops.len(), 2);

    let backup_ops = manager.filter_by_operation(OperationType::Backup).unwrap();
    assert_eq!(backup_ops.len(), 1);

    let validate_ops = manager
        .filter_by_operation(OperationType::Validate)
        .unwrap();
    assert_eq!(validate_ops.len(), 1);
}

#[test]
fn test_history_manager_recent_limit() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();
    let manager = HistoryManager::new(db);

    let details = OperationDetails {
        from_config: None,
        to_config: None,
        backup_path: None,
        extra: None,
    };

    // 添加 10 条记录
    for _ in 0..10 {
        manager
            .add(HistoryEntry::new(
                OperationType::Switch,
                details.clone(),
                OperationResult::Success,
            ))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10)); // 确保时间戳不同
    }

    // 获取最近 5 条
    let recent = manager.get_recent(5).unwrap();
    assert_eq!(recent.len(), 5);

    // 验证是按时间倒序排列（最新的在前）
    for i in 0..recent.len() - 1 {
        assert!(recent[i].timestamp >= recent[i + 1].timestamp);
    }
}

#[test]
fn test_history_manager_stats() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = ccr::Database::init(&db_path).unwrap();
    let manager = HistoryManager::new(db);

    let details = OperationDetails {
        from_config: None,
        to_config: None,
        backup_path: None,
        extra: None,
    };

    // 添加不同结果的记录
    manager
        .add(HistoryEntry::new(
            OperationType::Switch,
            details.clone(),
            OperationResult::Success,
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Switch,
            details.clone(),
            OperationResult::Success,
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Backup,
            details.clone(),
            OperationResult::Failure("error".into()),
        ))
        .unwrap();
    manager
        .add(HistoryEntry::new(
            OperationType::Validate,
            details,
            OperationResult::Warning("warning".into()),
        ))
        .unwrap();

    // 获取统计
    let stats = manager.stats().unwrap();
    assert_eq!(stats.total_operations, 4);
    assert_eq!(stats.successful_operations, 2);
    assert_eq!(stats.failed_operations, 1);
    assert_eq!(stats.warning_operations, 1);
    assert!(stats.last_operation.is_some());
}

#[test]
fn test_history_entry_env_changes() {
    let details = OperationDetails {
        from_config: Some("old".into()),
        to_config: Some("new".into()),
        backup_path: None,
        extra: None,
    };

    let mut entry = HistoryEntry::new(OperationType::Switch, details, OperationResult::Success);

    // 添加环境变量变更
    entry.add_env_change(
        "ANTHROPIC_AUTH_TOKEN".into(),
        Some("sk-old-token-1234567890".into()),
        Some("sk-new-token-9876543210".into()),
    );

    entry.add_env_change(
        "ANTHROPIC_BASE_URL".into(),
        Some("https://old-api.com".into()),
        Some("https://new-api.com".into()),
    );

    // 验证变更记录
    assert_eq!(entry.env_changes.len(), 2);

    // 验证敏感信息被掩码
    let token_change = &entry.env_changes[0];
    assert_eq!(token_change.var_name, "ANTHROPIC_AUTH_TOKEN");
    assert!(token_change.old_value.as_ref().unwrap().contains("..."));
    assert!(token_change.new_value.as_ref().unwrap().contains("..."));

    // 验证非敏感信息未被掩码
    let url_change = &entry.env_changes[1];
    assert_eq!(url_change.var_name, "ANTHROPIC_BASE_URL");
    assert_eq!(url_change.old_value, Some("https://old-api.com".into()));
    assert_eq!(url_change.new_value, Some("https://new-api.com".into()));
}

// ═══════════════════════════════════════════════════════════════
// 跨 Manager 集成测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_and_settings_integration() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    // 创建配置
    let mut config = CcsConfig {
        default_config: "test".into(),
        current_config: "test".into(),
        settings: ccr::managers::config::GlobalSettings::default(),
        sections: IndexMap::new(),
    };
    config
        .sections
        .insert("test".into(), create_test_config_section("test"));

    let config_manager = ConfigManager::new(&config_path);
    config_manager.save(&config).unwrap();

    // 从配置创建设置
    let loaded_config = config_manager.load().unwrap();
    let section = loaded_config.get_current_section().unwrap();

    let mut settings = ClaudeSettings::new();
    settings.update_from_config(section);

    // 保存设置
    let lock_manager = LockManager::new(lock_dir);
    let settings_manager = SettingsManager::new(&settings_path, &backup_dir, lock_manager);
    settings_manager.save_atomic(&settings).unwrap();

    // 验证设置
    let loaded_settings = settings_manager.load().unwrap();
    assert!(loaded_settings.validate().is_ok());
    assert_eq!(
        loaded_settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.test.com".to_string())
    );
}
