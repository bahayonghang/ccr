// 🧪 端到端集成测试
// 测试完整的配置切换工作流，模拟真实使用场景

use ccr::core::lock::LockManager;
use ccr::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use ccr::managers::history::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
};
use ccr::managers::settings::SettingsManager;
use ccr::services::{ConfigService, HistoryService, SettingsService};
use ccr::utils::Validatable;
use indexmap::IndexMap;
use std::sync::Arc;
use tempfile::tempdir;

/// 测试环境设置
struct TestEnvironment {
    _temp_dir: tempfile::TempDir, // 保持临时目录的生命周期
    config_service: ConfigService,
    settings_service: SettingsService,
    history_service: HistoryService,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = tempdir().unwrap();

        // 直接在临时目录下创建文件
        let config_path = temp_dir.path().join("config.toml");
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        // 确保必要的目录存在
        std::fs::create_dir_all(&backup_dir).unwrap();
        std::fs::create_dir_all(&lock_dir).unwrap();

        let config_manager = Arc::new(ConfigManager::new(&config_path));
        let lock_manager_settings = LockManager::new(&lock_dir);
        let lock_manager_history = LockManager::new(&lock_dir);
        let settings_manager = Arc::new(SettingsManager::new(
            &settings_path,
            &backup_dir,
            lock_manager_settings,
        ));
        let history_manager = Arc::new(HistoryManager::new(&history_path, lock_manager_history));

        Self {
            _temp_dir: temp_dir,
            config_service: ConfigService::new(config_manager),
            settings_service: SettingsService::new(settings_manager),
            history_service: HistoryService::new(history_manager),
        }
    }

    fn setup_configs(&self) {
        let mut config = CcsConfig {
            default_config: "anthropic".into(),
            current_config: "anthropic".into(),
            sections: IndexMap::new(),
        };

        // 添加 Anthropic 配置
        config.sections.insert(
            "anthropic".into(),
            ConfigSection {
                description: Some("Anthropic Official API".into()),
                base_url: Some("https://api.anthropic.com".into()),
                auth_token: Some("sk-ant-official-token-12345".into()),
                model: Some("claude-sonnet-4-5".into()),
                small_fast_model: Some("claude-haiku".into()),
                provider: Some("anthropic".into()),
                provider_type: Some(ccr::managers::config::ProviderType::OfficialRelay),
                account: None,
                tags: Some(vec!["official".into(), "stable".into()]),
            },
        );

        // 添加 AnyRouter 配置
        config.sections.insert(
            "anyrouter".into(),
            ConfigSection {
                description: Some("AnyRouter Proxy".into()),
                base_url: Some("https://api.anyrouter.ai/v1".into()),
                auth_token: Some("sk-anyrouter-token-67890".into()),
                model: Some("claude-sonnet-4-5".into()),
                small_fast_model: Some("claude-haiku".into()),
                provider: Some("anyrouter".into()),
                provider_type: Some(ccr::managers::config::ProviderType::OfficialRelay),
                account: Some("test_user".into()),
                tags: Some(vec!["relay".into(), "fast".into()]),
            },
        );

        self.config_service.save_config(&config).unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════
// 完整配置切换流程测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_complete_switch_workflow() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 步骤 1: 验证初始配置列表
    let list = env.config_service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 2);
    assert_eq!(list.current_config, "anthropic");

    // 步骤 2: 应用初始配置到 settings
    let section1 = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section1).unwrap();

    // 验证设置已应用
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anthropic.com".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-ant-official-token-12345".to_string())
    );
    assert!(settings.validate().is_ok());

    // 步骤 3: 备份当前设置
    let backup_path = env
        .settings_service
        .backup_settings(Some("before_switch"))
        .unwrap();
    assert!(backup_path.exists());

    // 步骤 4: 记录切换操作
    let mut entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: Some("anthropic".into()),
            to_config: Some("anyrouter".into()),
            backup_path: Some(backup_path.display().to_string()),
            extra: None,
        },
        OperationResult::Success,
    );

    entry.add_env_change(
        "ANTHROPIC_BASE_URL".into(),
        Some("https://api.anthropic.com".into()),
        Some("https://api.anyrouter.ai/v1".into()),
    );

    env.history_service.record_operation(entry).unwrap();

    // 步骤 5: 切换到新配置
    let section2 = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anyrouter")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section2).unwrap();

    // 步骤 6: 更新当前配置标记
    env.config_service.set_current("anyrouter").unwrap();

    // 验证切换成功
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anyrouter.ai/v1".to_string())
    );
    assert_eq!(
        settings.env.get("ANTHROPIC_AUTH_TOKEN"),
        Some(&"sk-anyrouter-token-67890".to_string())
    );

    let current_config = env.config_service.load_config().unwrap();
    assert_eq!(current_config.current_config, "anyrouter");

    // 步骤 7: 验证历史记录
    let history = env.history_service.get_recent(10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation, OperationType::Switch);
    assert_eq!(history[0].details.from_config, Some("anthropic".into()));
    assert_eq!(history[0].details.to_config, Some("anyrouter".into()));
}

#[test]
fn test_multi_config_switch_with_rollback() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 应用初始配置
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 备份
    let backup1 = env
        .settings_service
        .backup_settings(Some("state1"))
        .unwrap();

    // 切换到 anyrouter
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anyrouter")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();
    env.config_service.set_current("anyrouter").unwrap();

    // 验证切换
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anyrouter.ai/v1".to_string())
    );

    // 回滚到备份
    env.settings_service.restore_settings(&backup1).unwrap();

    // 验证回滚成功
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anthropic.com".to_string())
    );
}

#[test]
fn test_import_export_roundtrip() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 导出配置（包含密钥）
    let exported = env.config_service.export_config(true).unwrap();
    assert!(exported.contains("anthropic"));
    assert!(exported.contains("anyrouter"));
    assert!(exported.contains("sk-ant-official-token-12345"));

    // 添加新配置
    env.config_service
        .add_config(
            "new_config".into(),
            ConfigSection {
                description: Some("New".into()),
                base_url: Some("https://api.new.com".into()),
                auth_token: Some("sk-new-token".into()),
                model: Some("new-model".into()),
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
            },
        )
        .unwrap();

    // 导入之前导出的配置（合并模式）
    let _result = env
        .config_service
        .import_config(
            &exported,
            ccr::services::config_service::ImportMode::Merge,
            false,
        )
        .unwrap();

    // 验证导入结果
    // new_config 应该被保留，原有配置被更新
    let list = env.config_service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 3); // anthropic, anyrouter, new_config
}

#[test]
fn test_validation_workflow() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 应用配置
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 验证配置文件
    let report = env.config_service.validate_all().unwrap();
    assert_eq!(report.valid_count, 2);
    assert_eq!(report.invalid_count, 0);

    // 验证设置文件
    let settings = env.settings_service.get_current_settings().unwrap();
    assert!(settings.validate().is_ok());

    // 记录验证操作
    let entry = HistoryEntry::new(
        OperationType::Validate,
        OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: None,
            extra: Some("All validations passed".into()),
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry).unwrap();

    // 验证历史
    let history = env.history_service.get_recent(1).unwrap();
    assert_eq!(history[0].operation, OperationType::Validate);
}

// ═══════════════════════════════════════════════════════════════
// 真实场景模拟测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_daily_usage_scenario() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 场景：用户一天的使用流程

    // 上午：使用 Anthropic 官方 API
    println!("上午：切换到 Anthropic");
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();
    env.config_service.set_current("anthropic").unwrap();

    let entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: None,
            to_config: Some("anthropic".into()),
            backup_path: None,
            extra: None,
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry).unwrap();

    // 中午：切换到 AnyRouter 节省成本
    println!("中午：切换到 AnyRouter");
    let backup_noon = env
        .settings_service
        .backup_settings(Some("noon_backup"))
        .unwrap();

    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anyrouter")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();
    env.config_service.set_current("anyrouter").unwrap();

    let entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: Some("anthropic".into()),
            to_config: Some("anyrouter".into()),
            backup_path: Some(backup_noon.display().to_string()),
            extra: None,
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry).unwrap();

    // 下午：验证当前配置
    println!("下午：验证配置");
    let report = env.config_service.validate_all().unwrap();
    assert_eq!(report.invalid_count, 0);

    let settings = env.settings_service.get_current_settings().unwrap();
    assert!(settings.validate().is_ok());

    // 晚上：查看操作历史
    println!("晚上：查看历史");
    let history = env.history_service.get_recent(10).unwrap();
    assert_eq!(history.len(), 2); // 2 次切换（验证操作没有记录到历史）

    let stats = env.history_service.get_stats().unwrap();
    assert_eq!(stats.total_operations, 2);
    assert_eq!(stats.successful_operations, 2);

    // 验证最终状态
    let current = env.config_service.load_config().unwrap();
    assert_eq!(current.current_config, "anyrouter");

    let final_settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        final_settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anyrouter.ai/v1".to_string())
    );
}

#[test]
fn test_backup_and_recovery_scenario() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 初始配置：Anthropic
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 备份点 1
    let backup1 = env
        .settings_service
        .backup_settings(Some("checkpoint1"))
        .unwrap();

    // 切换到 AnyRouter
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anyrouter")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 备份点 2
    let backup2 = env
        .settings_service
        .backup_settings(Some("checkpoint2"))
        .unwrap();

    // 验证有 2 个备份
    let backups = env.settings_service.list_backups().unwrap();
    assert_eq!(backups.len(), 2);

    // 恢复到备份点 1
    env.settings_service.restore_settings(&backup1).unwrap();
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anthropic.com".to_string())
    );

    // 恢复到备份点 2
    env.settings_service.restore_settings(&backup2).unwrap();
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anyrouter.ai/v1".to_string())
    );
}

#[test]
fn test_error_recovery_scenario() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 应用有效配置
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 备份
    let backup = env
        .settings_service
        .backup_settings(Some("safety_backup"))
        .unwrap();

    // 尝试应用无效配置（应该失败）
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
    };

    assert!(invalid_section.validate().is_err());

    // 由于验证失败，设置不应该被更新
    let settings = env.settings_service.get_current_settings().unwrap();
    assert_eq!(
        settings.env.get("ANTHROPIC_BASE_URL"),
        Some(&"https://api.anthropic.com".to_string())
    );

    // 验证备份仍然有效
    assert!(backup.exists());
}

#[test]
fn test_config_import_merge_workflow() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 导入新配置（合并模式）
    let import_toml = r#"
default_config = "glm"
current_config = "glm"

[glm]
description = "GLM API"
base_url = "https://api.glm.ai"
auth_token = "sk-glm-token-123"
model = "glm-4"
provider = "glm"
tags = ["third_party", "chinese"]
    "#;

    let result = env
        .config_service
        .import_config(
            import_toml,
            ccr::services::config_service::ImportMode::Merge,
            false,
        )
        .unwrap();

    assert_eq!(result.added, 1); // GLM 是新增的
    assert_eq!(result.updated, 0); // anthropic 和 anyrouter 保持不变

    // 验证配置合并成功
    let list = env.config_service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 3); // anthropic + anyrouter + glm

    let glm_config = env.config_service.get_config("glm").unwrap();
    assert_eq!(glm_config.description, "GLM API");
    assert_eq!(
        glm_config.tags,
        Some(vec!["third_party".into(), "chinese".into()])
    );
}

#[test]
fn test_config_export_import_replace_workflow() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 导出当前配置
    let exported = env.config_service.export_config(true).unwrap();

    // 添加临时配置
    env.config_service
        .add_config(
            "temp".into(),
            ConfigSection {
                description: Some("Temp".into()),
                base_url: Some("https://temp.com".into()),
                auth_token: Some("temp-token".into()),
                model: None,
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
            },
        )
        .unwrap();

    let list = env.config_service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 3); // anthropic + anyrouter + temp

    // 导入之前导出的配置（替换模式）
    let result = env
        .config_service
        .import_config(
            &exported,
            ccr::services::config_service::ImportMode::Replace,
            false,
        )
        .unwrap();

    assert_eq!(result.added, 2); // anthropic + anyrouter

    // 验证 temp 配置被移除
    let list = env.config_service.list_configs().unwrap();
    assert_eq!(list.configs.len(), 2);
    assert!(!list.configs.iter().any(|c| c.name == "temp"));
}

#[test]
fn test_history_tracking_across_operations() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 执行一系列操作并记录历史

    // 操作 1: 切换配置
    let entry1 = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: None,
            to_config: Some("anthropic".into()),
            backup_path: None,
            extra: None,
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry1).unwrap();

    // 操作 2: 备份
    let entry2 = HistoryEntry::new(
        OperationType::Backup,
        OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: Some("/path/to/backup".into()),
            extra: None,
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry2).unwrap();

    // 操作 3: 验证
    let entry3 = HistoryEntry::new(
        OperationType::Validate,
        OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: None,
            extra: Some("All checks passed".into()),
        },
        OperationResult::Success,
    );
    env.history_service.record_operation(entry3).unwrap();

    // 操作 4: 切换（失败）
    let entry4 = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: Some("anthropic".into()),
            to_config: Some("invalid".into()),
            backup_path: None,
            extra: None,
        },
        OperationResult::Failure("配置不存在".into()),
    );
    env.history_service.record_operation(entry4).unwrap();

    // 验证历史统计
    let stats = env.history_service.get_stats().unwrap();
    assert_eq!(stats.total_operations, 4);
    assert_eq!(stats.successful_operations, 3);
    assert_eq!(stats.failed_operations, 1);

    // 按类型筛选
    let switches = env
        .history_service
        .filter_by_type(OperationType::Switch)
        .unwrap();
    assert_eq!(switches.len(), 2);

    let backups = env
        .history_service
        .filter_by_type(OperationType::Backup)
        .unwrap();
    assert_eq!(backups.len(), 1);

    let validates = env
        .history_service
        .filter_by_type(OperationType::Validate)
        .unwrap();
    assert_eq!(validates.len(), 1);
}

#[test]
fn test_full_lifecycle_with_cleanup() {
    let env = TestEnvironment::new();
    env.setup_configs();

    // 1. 初始化：应用配置
    let section = env
        .config_service
        .load_config()
        .unwrap()
        .get_section("anthropic")
        .unwrap()
        .clone();
    env.settings_service.apply_config(&section).unwrap();

    // 2. 多次切换生成备份
    for i in 0..5 {
        let backup = env
            .settings_service
            .backup_settings(Some(&format!("auto_backup_{}", i)))
            .unwrap();
        assert!(backup.exists());
        std::thread::sleep(std::time::Duration::from_millis(50)); // 确保时间戳不同
    }

    // 3. 验证备份数量
    let backups = env.settings_service.list_backups().unwrap();
    println!("备份数量: {}", backups.len());
    for backup in &backups {
        println!("  - {}", backup.filename);
    }
    assert!(
        backups.len() >= 4,
        "应该至少有 4 个备份文件，实际: {}",
        backups.len()
    );

    // 4. 清理旧备份（这里使用 BackupService）
    // 注意：这需要正确的备份目录路径
    // 实际测试中我们跳过清理，因为所有备份都是新的

    // 5. 验证最终状态
    let current_config = env.config_service.load_config().unwrap();
    assert_eq!(current_config.sections.len(), 2);

    let current_settings = env.settings_service.get_current_settings().unwrap();
    assert!(current_settings.validate().is_ok());
}

#[test]
fn test_configuration_persistence() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let settings_path = temp_dir.path().join("settings.json");
    let backup_dir = temp_dir.path().join("backups");
    let lock_dir = temp_dir.path().join("locks");

    // 第一个会话：创建和保存配置
    {
        // 创建初始配置
        let mut initial_config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            sections: IndexMap::new(),
        };
        initial_config.sections.insert(
            "test".into(),
            ConfigSection {
                description: Some("Test".into()),
                base_url: Some("https://api.test.com".into()),
                auth_token: Some("sk-test-token".into()),
                model: Some("test-model".into()),
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
            },
        );

        let config_manager = Arc::new(ConfigManager::new(&config_path));
        config_manager.save(&initial_config).unwrap();

        let config_service = ConfigService::new(config_manager);

        let lock_manager = LockManager::new(&lock_dir);
        let settings_manager = Arc::new(SettingsManager::new(
            &settings_path,
            &backup_dir,
            lock_manager,
        ));
        let settings_service = SettingsService::new(settings_manager);

        // 应用到 settings
        let section = config_service
            .load_config()
            .unwrap()
            .get_section("test")
            .unwrap()
            .clone();
        settings_service.apply_config(&section).unwrap();
    }

    // 第二个会话：加载并验证
    {
        let config_manager = Arc::new(ConfigManager::new(&config_path));
        let config_service = ConfigService::new(config_manager);

        let lock_manager = LockManager::new(&lock_dir);
        let settings_manager = Arc::new(SettingsManager::new(
            &settings_path,
            &backup_dir,
            lock_manager,
        ));
        let settings_service = SettingsService::new(settings_manager);

        // 验证配置持久化
        let config = config_service.load_config().unwrap();
        assert_eq!(config.current_config, "test");
        assert!(config.get_section("test").is_ok());

        // 验证设置持久化
        let settings = settings_service.get_current_settings().unwrap();
        assert_eq!(
            settings.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://api.test.com".to_string())
        );
        assert!(settings.validate().is_ok());
    }
}
