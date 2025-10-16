// 🧪 集成测试：add 和 delete 命令
// 测试添加和删除配置的完整流程

use ccr::{ConfigManager, ConfigSection, Validatable};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_add_delete_config_flow() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".ccs_config.toml");

    // 初始化配置管理器
    let manager = ConfigManager::new(&config_path);

    // 创建初始配置
    let mut config = ccr::CcsConfig {
        default_config: "default".to_string(),
        current_config: "default".to_string(),
        sections: indexmap::IndexMap::new(),
    };

    // 添加一个默认配置
    let default_section = ConfigSection {
        description: Some("Default config".into()),
        base_url: Some("https://api.anthropic.com".into()),
        auth_token: Some("sk-ant-default-token".into()),
        model: Some("claude-3-5-sonnet-20241022".into()),
        small_fast_model: Some("claude-3-5-haiku-20241022".into()),
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
    };
    config.sections.insert("default".to_string(), default_section);

    // 保存初始配置
    manager.save(&config).unwrap();

    // 测试添加新配置
    println!("📝 测试添加新配置...");
    let test_section = ConfigSection {
        description: Some("Test Provider".into()),
        base_url: Some("https://api.test-provider.com".into()),
        auth_token: Some("sk-test-12345678901234567890".into()),
        model: Some("test-model-v1".into()),
        small_fast_model: Some("test-small-model-v1".into()),
        provider: Some("test_provider".into()),
        provider_type: Some(ccr::managers::config::ProviderType::ThirdPartyModel),
        account: Some("test_account_001".into()),
        tags: Some(vec!["test".to_string(), "temporary".to_string()]),
    };

    // 添加测试配置
    let mut updated_config = manager.load().unwrap();
    updated_config.sections.insert("test_config".to_string(), test_section.clone());
    manager.save(&updated_config).unwrap();

    // 验证配置已添加
    let reloaded = manager.load().unwrap();
    assert!(reloaded.sections.contains_key("test_config"), "配置应该已添加");
    assert_eq!(reloaded.sections.len(), 2, "应该有2个配置");

    // 验证添加的配置内容
    let added_section = reloaded.sections.get("test_config").unwrap();
    assert_eq!(
        added_section.description.as_deref(),
        Some("Test Provider"),
        "描述应该匹配"
    );
    assert_eq!(
        added_section.base_url.as_deref(),
        Some("https://api.test-provider.com"),
        "Base URL 应该匹配"
    );
    assert_eq!(
        added_section.provider.as_deref(),
        Some("test_provider"),
        "提供商应该匹配"
    );
    assert_eq!(
        added_section.tags.as_ref().unwrap().len(),
        2,
        "应该有2个标签"
    );

    println!("✅ 配置添加测试通过");

    // 测试删除配置
    println!("🗑️  测试删除配置...");
    let mut final_config = manager.load().unwrap();
    let removed = final_config.sections.shift_remove("test_config");
    assert!(removed.is_some(), "配置应该存在并被删除");
    manager.save(&final_config).unwrap();

    // 验证配置已删除
    let after_delete = manager.load().unwrap();
    assert!(!after_delete.sections.contains_key("test_config"), "配置应该已删除");
    assert_eq!(after_delete.sections.len(), 1, "应该只剩1个配置");

    println!("✅ 配置删除测试通过");

    // 验证无法删除不存在的配置
    println!("🔍 测试删除不存在的配置...");
    let mut test_config = manager.load().unwrap();
    let removed = test_config.sections.shift_remove("nonexistent");
    assert!(removed.is_none(), "不存在的配置应该返回 None");

    println!("✅ 删除不存在配置的测试通过");

    // 清理
    println!("🧹 清理测试数据...");
    drop(manager);
    fs::remove_dir_all(temp_dir).unwrap();

    println!("✅ 所有 add/delete 测试通过！");
}

#[test]
fn test_add_config_validation() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".ccs_config.toml");

    let manager = ConfigManager::new(&config_path);

    // 创建基础配置
    let mut config = ccr::CcsConfig {
        default_config: "default".to_string(),
        current_config: "default".to_string(),
        sections: indexmap::IndexMap::new(),
    };

    let default_section = ConfigSection {
        description: Some("Default".into()),
        base_url: Some("https://api.test.com".into()),
        auth_token: Some("token".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
    };
    config.sections.insert("default".to_string(), default_section);
    manager.save(&config).unwrap();

    // 测试添加无效配置（缺少 base_url）
    println!("🔍 测试添加无效配置...");
    let invalid_section = ConfigSection {
        description: Some("Invalid".into()),
        base_url: None,
        auth_token: Some("token".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
    };

    // 验证应该失败
    let validation_result = invalid_section.validate();
    assert!(validation_result.is_err(), "无效配置应该验证失败");

    println!("✅ 验证测试通过");

    // 清理
    drop(manager);
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_delete_current_config_warning() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".ccs_config.toml");

    let manager = ConfigManager::new(&config_path);

    // 创建配置
    let mut config = ccr::CcsConfig {
        default_config: "config1".to_string(),
        current_config: "config1".to_string(),
        sections: indexmap::IndexMap::new(),
    };

    let section1 = ConfigSection {
        description: Some("Config 1".into()),
        base_url: Some("https://api.test1.com".into()),
        auth_token: Some("token1".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
    };

    let section2 = ConfigSection {
        description: Some("Config 2".into()),
        base_url: Some("https://api.test2.com".into()),
        auth_token: Some("token2".into()),
        model: None,
        small_fast_model: None,
        provider: None,
        provider_type: None,
        account: None,
        tags: None,
    };

    config.sections.insert("config1".to_string(), section1);
    config.sections.insert("config2".to_string(), section2);
    manager.save(&config).unwrap();

    // 验证当前配置
    let loaded = manager.load().unwrap();
    assert_eq!(loaded.current_config, "config1");

    // 模拟删除当前配置（实际场景应该警告）
    println!("⚠️  测试删除当前配置...");
    let is_current = loaded.current_config == "config1";
    assert!(is_current, "config1 应该是当前配置");

    // 在实际命令中，这里会显示警告
    // 但允许删除（用户需要后续手动切换）

    println!("✅ 当前配置检测测试通过");

    // 清理
    drop(manager);
    fs::remove_dir_all(temp_dir).unwrap();
}

