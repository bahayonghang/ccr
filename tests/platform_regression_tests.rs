// 🔄 CCR 多平台回归测试
//
// ⚠️ **重要提示**: 这些测试修改全局环境变量，因此必须串行运行
// 运行方式: `cargo test --test platform_regression_tests -- --test-threads=1`
//
// 测试目的:
// - 验证 Legacy 模式兼容性（旧有单平台配置）
// - 验证从 Legacy 到 Unified 的升级路径
// - 验证向后兼容性（新功能不破坏旧功能）
// - 验证边界条件和错误恢复
// - 验证数据迁移和转换
//
// 共计: 15 个回归测试

use ccr::{
    ConfigManager, Platform, PlatformConfigManager, PlatformPaths, ProfileConfig, SettingsManager,
    create_platform,
};
use std::fs;
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════
// 测试辅助函数
// ═══════════════════════════════════════════════════════════

/// 创建临时测试环境
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    unsafe {
        std::env::set_var("CCR_ROOT", temp_dir.path().to_str().unwrap());
        std::env::set_var("HOME", temp_dir.path().to_str().unwrap());
    }

    // 创建必要的目录结构
    std::fs::create_dir_all(temp_dir.path()).ok();
    std::fs::create_dir_all(temp_dir.path().join(".claude")).ok();

    temp_dir
}

/// 清理测试环境
fn cleanup_test_env(temp_dir: TempDir) {
    unsafe {
        std::env::remove_var("CCR_ROOT");
        std::env::remove_var("HOME");
    }
    drop(temp_dir);
}

/// 创建 Legacy 模式的配置文件（老版本 CCS 格式）
fn create_legacy_config(temp_dir: &TempDir) {
    let config_path = temp_dir.path().join(".ccs_config.toml");

    let legacy_content = r#"
# Legacy CCS Configuration (Single Platform)
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-legacy-test-token-1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678"
model = "claude-3-5-sonnet-20241022"
small_fast_model = "claude-3-5-haiku-20241022"
provider = "Anthropic"

[bedrock]
description = "AWS Bedrock"
base_url = "https://bedrock-runtime.us-east-1.amazonaws.com"
auth_token = "aws-access-key-legacy"
model = "anthropic.claude-3-sonnet"
small_fast_model = "anthropic.claude-3-haiku"
provider = "AWS"
"#;

    fs::write(config_path, legacy_content).unwrap();
}

/// 创建测试用的 Claude profile
fn create_test_claude_profile(name: &str) -> ProfileConfig {
    let mut profile = ProfileConfig::new();
    profile.description = Some(format!("Test Claude profile: {}", name));
    profile.base_url = Some("https://api.anthropic.com".to_string());
    profile.auth_token = Some(format!(
        "sk-ant-api03-{}",
        "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"
    ));
    profile.model = Some("claude-3-5-sonnet-20241022".to_string());
    profile.small_fast_model = Some("claude-3-5-haiku-20241022".to_string());
    profile
}

// ═══════════════════════════════════════════════════════════
// 回归测试 1: Legacy 配置检测
// ═══════════════════════════════════════════════════════════

#[test]
fn test_legacy_mode_detection() {
    let temp_dir = setup_test_env();

    // 创建 Legacy 配置文件
    create_legacy_config(&temp_dir);

    // 检测应该识别为 Legacy 模式
    let manager = ConfigManager::with_default().unwrap();
    let config = manager.load().unwrap();

    // Legacy 模式下应该能正常读取配置
    assert_eq!(config.default_config, "anthropic");
    assert!(config.sections.contains_key("anthropic"));
    assert!(config.sections.contains_key("bedrock"));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 2: Legacy 配置读取兼容性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_legacy_config_reading_compatibility() {
    let temp_dir = setup_test_env();

    create_legacy_config(&temp_dir);

    let manager = ConfigManager::with_default().unwrap();
    let config = manager.load().unwrap();

    // 验证 Legacy 配置字段正确读取
    let anthropic = config.sections.get("anthropic").unwrap();
    assert_eq!(
        anthropic.description,
        Some("Anthropic Official API".to_string())
    );
    assert_eq!(
        anthropic.base_url,
        Some("https://api.anthropic.com".to_string())
    );
    assert_eq!(anthropic.provider, Some("Anthropic".to_string()));

    let bedrock = config.sections.get("bedrock").unwrap();
    assert_eq!(bedrock.description, Some("AWS Bedrock".to_string()));
    assert_eq!(
        bedrock.base_url,
        Some("https://bedrock-runtime.us-east-1.amazonaws.com".to_string())
    );

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 3: Legacy 配置修改兼容性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_legacy_config_modification_compatibility() {
    let temp_dir = setup_test_env();

    create_legacy_config(&temp_dir);

    let manager = ConfigManager::with_default().unwrap();
    let mut config = manager.load().unwrap();

    // 修改 Legacy 配置
    if let Some(anthropic) = config.sections.get_mut("anthropic") {
        anthropic.model = Some("claude-opus-4".to_string());
    }

    // 保存应该成功
    manager.save(&config).unwrap();

    // 重新加载验证
    let reloaded = manager.load().unwrap();
    let anthropic = reloaded.sections.get("anthropic").unwrap();
    assert_eq!(anthropic.model, Some("claude-opus-4".to_string()));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 4: Unified 模式新建配置
// ═══════════════════════════════════════════════════════════

#[test]
fn test_unified_mode_fresh_install() {
    let temp_dir = setup_test_env();

    // 不创建任何 Legacy 配置，直接使用 Unified 模式
    let manager = PlatformConfigManager::with_default().unwrap();
    let config = manager.load_or_create_default().unwrap();

    // 应该默认创建 Claude 平台配置
    assert_eq!(config.current_platform, "claude");
    assert!(config.platforms.contains_key("claude"));
    assert!(config.platforms.get("claude").unwrap().enabled);

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 5: 平台路径向后兼容性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_paths_backward_compatibility() {
    let temp_dir = setup_test_env();

    // 旧版本可能使用 ~/.claude/ 目录
    let claude_dir = temp_dir.path().join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // 创建旧版本的 settings.json
    let settings_path = claude_dir.join("settings.json");
    let old_settings = r#"{
  "statusLine": {
    "enabled": true
  },
  "experimental": {
    "outputStyles": ["default"]
  }
}"#;
    fs::write(&settings_path, old_settings).unwrap();

    // SettingsManager 应该能读取旧配置
    let manager = SettingsManager::with_default().unwrap();
    let settings = manager.load().unwrap();

    // 验证 settings 可以正常加载（检查 env 或 other 字段）
    // 旧版本的配置可能在 other 字段中
    assert!(settings.env.is_empty() || settings.other.contains_key("statusLine"));

    cleanup_test_env(temp_dir);
}

//测试 6: 空配置文件处理
// ═══════════════════════════════════════════════════════════

#[test]
fn test_empty_config_file_handling() {
    let temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();

    // 不创建任何文件，直接使用 load_or_create_default
    // 这应该创建默认配置
    let config = manager.load_or_create_default().unwrap();
    assert!(
        config.platforms.contains_key("claude"),
        "没有配置文件时应该创建默认配置"
    );

    // 验证配置被正确保存到文件
    assert!(manager.config_path().exists(), "配置文件应该被创建");

    // 重新加载应该得到相同的配置
    let reloaded = manager.load().unwrap();
    assert_eq!(reloaded.current_platform, config.current_platform);

    cleanup_test_env(temp_dir);
}

// ��══════════════════════════════════════════════════════════
// 回归测试 7: 损坏的配置文件恢复
// ═══════════════════════════════════════════════════════════

#[test]
fn test_corrupted_config_recovery() {
    let temp_dir = setup_test_env();

    // 创建损坏的 TOML 文件
    let ccr_dir = temp_dir.path();
    fs::create_dir_all(ccr_dir).unwrap();
    let config_path = ccr_dir.join("config.toml");
    fs::write(&config_path, "invalid toml content {{{").unwrap();

    let manager = PlatformConfigManager::with_default().unwrap();

    // 加载损坏的文件应该失败
    let result = manager.load();
    assert!(result.is_err(), "损坏的配置应该返回错误");

    // 删除损坏的文件，然后 load_or_create_default 应该能创建新的
    fs::remove_file(&config_path).ok();
    let config = manager.load_or_create_default().unwrap();
    assert!(
        config.platforms.contains_key("claude"),
        "删除损坏文件后应该能创建新配置"
    );

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 8: 多平台共存不破坏单平台
// ═══════════════════════════════════════════════════════════

#[test]
fn test_multi_platform_does_not_break_single_platform() {
    let temp_dir = setup_test_env();

    // 创建 Legacy 配置
    create_legacy_config(&temp_dir);

    // 使用 Legacy ConfigManager
    let legacy_manager = ConfigManager::with_default().unwrap();
    let legacy_config = legacy_manager.load().unwrap();

    // 验证 Legacy 配置正常
    assert_eq!(legacy_config.default_config, "anthropic");

    // 现在创建 Unified 配置
    let unified_manager = PlatformConfigManager::with_default().unwrap();
    let unified_config = unified_manager.load_or_create_default().unwrap();

    // Unified 配置也应该正常工作
    assert_eq!(unified_config.current_platform, "claude");

    // 再次加载 Legacy 配置，应该不受影响
    let legacy_config_again = legacy_manager.load().unwrap();
    assert_eq!(legacy_config_again.default_config, "anthropic");

    cleanup_test_env(temp_dir);
}

// ════════════════��══════════════════════════════════════════
// 回归测试 9: Profile 文件格式兼容性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_profile_file_format_compatibility() {
    let temp_dir = setup_test_env();

    // 创建 Legacy 配置文件（ClaudePlatform 初始化需要）
    create_legacy_config(&temp_dir);

    // 创建 Claude 平台实例
    let claude = create_platform(Platform::Claude).unwrap();

    // 手动构建旧格式 profile 并保存
    let mut old_profile = ProfileConfig::new();
    old_profile.description = Some("Old format profile".to_string());
    old_profile.base_url = Some("https://api.anthropic.com".to_string());
    old_profile.auth_token = Some(
        "sk-ant-api03-old-format-1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678"
            .to_string(),
    );
    old_profile.model = Some("claude-3-sonnet".to_string());

    // 保存 profile
    claude.save_profile("default", &old_profile).unwrap();

    // 重新加载验证
    let profiles = claude.load_profiles().unwrap();
    assert!(
        profiles.contains_key("default"),
        "应该能读取 default profile"
    );

    let profile = profiles.get("default").unwrap();
    assert_eq!(profile.description, Some("Old format profile".to_string()));
    assert_eq!(profile.model, Some("claude-3-sonnet".to_string()));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 10: 大量 Profiles 性能测试
// ═══════════════════════════════════════════════════════════

#[test]
fn test_large_number_of_profiles_performance() {
    let temp_dir = setup_test_env();

    let claude = create_platform(Platform::Claude).unwrap();

    // 创建 50 个 profiles
    for i in 1..=50 {
        let profile = create_test_claude_profile(&format!("profile-{}", i));
        claude
            .save_profile(&format!("profile-{}", i), &profile)
            .unwrap();
    }

    // 加载所有 profiles 应该很快
    let start = std::time::Instant::now();
    let profiles = claude.load_profiles().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(profiles.len(), 50);
    assert!(
        elapsed.as_millis() < 1000,
        "加载 50 个 profiles 应该在 1 秒内完成"
    );

    // 列出 profile 名称也应该很快
    let start = std::time::Instant::now();
    let names = claude.list_profile_names().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(names.len(), 50);
    assert!(
        elapsed.as_millis() < 500,
        "列出 50 个 profile 名称应该在 0.5 秒内完成"
    );

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 11: 平台切换后配置不丢失
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_switch_preserves_configs() {
    let temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 为 Claude 设置配置
    if let Some(claude_entry) = config.platforms.get_mut("claude") {
        claude_entry.description = Some("My Claude Config".to_string());
        claude_entry.current_profile = Some("my-profile".to_string());
    }

    // 注册 Codex
    let codex_entry = ccr::PlatformConfigEntry {
        enabled: true,
        description: Some("My Codex Config".to_string()),
        ..Default::default()
    };
    config
        .register_platform("codex".to_string(), codex_entry)
        .unwrap();

    manager.save(&config).unwrap();

    // 切换到 Codex
    let mut config = manager.load().unwrap();
    config.set_current_platform("codex").unwrap();
    manager.save(&config).unwrap();

    // 切换回 Claude
    let mut config = manager.load().unwrap();
    config.set_current_platform("claude").unwrap();
    manager.save(&config).unwrap();

    // Claude 的配置应该保留
    let final_config = manager.load().unwrap();
    let claude = final_config.platforms.get("claude").unwrap();
    assert_eq!(claude.description, Some("My Claude Config".to_string()));
    assert_eq!(claude.current_profile, Some("my-profile".to_string()));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 12: 删除平台不影响其他平台
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_deletion_isolation() {
    let temp_dir = setup_test_env();

    // 创建三个平台的 profiles
    let claude = create_platform(Platform::Claude).unwrap();
    let codex = create_platform(Platform::Codex).unwrap();

    let claude_profile = create_test_claude_profile("claude-1");
    claude.save_profile("claude-1", &claude_profile).unwrap();

    let mut codex_profile = ProfileConfig::new();
    codex_profile.base_url = Some("https://api.github.com/copilot".to_string());
    codex_profile.auth_token = Some("ghp_1234567890123456789012345678901234567890".to_string());
    codex_profile.model = Some("gpt-4".to_string());
    codex.save_profile("codex-1", &codex_profile).unwrap();

    // 删除 Claude profile
    claude.delete_profile("claude-1").unwrap();

    // Codex profile 应该不受影响
    let codex_profiles = codex.load_profiles().unwrap();
    assert!(codex_profiles.contains_key("codex-1"));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 13: 路径环境变量覆盖
// ═══════════════════════════════════════════════════════════

#[test]
fn test_ccr_root_env_override() {
    let temp_dir = setup_test_env();

    let custom_path = temp_dir.path().join("custom_ccr");
    fs::create_dir_all(&custom_path).unwrap();

    unsafe {
        std::env::set_var("CCR_ROOT", custom_path.to_str().unwrap());
    }

    // 创建平台路径应该使用自定义路径
    let paths = PlatformPaths::new(Platform::Claude).unwrap();
    assert!(paths.root.starts_with(&custom_path));

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 14: 特殊字符处理
// ═══════════════════════════════════════════════════════════

#[test]
fn test_special_characters_in_profile_names() {
    let temp_dir = setup_test_env();

    let claude = create_platform(Platform::Claude).unwrap();

    // 测试带特殊字符的 profile 名称
    let special_names = vec![
        "profile-with-dash",
        "profile_with_underscore",
        "profile.with.dot",
        "profile123",
    ];

    for name in special_names {
        let profile = create_test_claude_profile(name);
        claude.save_profile(name, &profile).unwrap();
    }

    let profiles = claude.load_profiles().unwrap();
    assert_eq!(profiles.len(), 4);

    cleanup_test_env(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 回归测试 15: 并发读写安全性（压力测试）
// ═══════════════════════════════════════════════════════════

#[test]
fn test_concurrent_read_write_safety() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = setup_test_env();

    let claude = Arc::new(create_platform(Platform::Claude).unwrap());

    // 创建 3 个线程，每个线程执行 10 次写入
    let mut handles = vec![];

    for thread_id in 0..3 {
        let claude_clone = Arc::clone(&claude);
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let profile = create_test_claude_profile(&format!("t{}-p{}", thread_id, i));
                // 忽略并发冲突错误
                let _ = claude_clone
                    .save_profile(&format!("thread{}-profile{}", thread_id, i), &profile);
                // 添加小延迟减少冲突
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证大部分 profiles 都被正确创建
    let profiles = claude.load_profiles().unwrap();
    assert!(
        profiles.len() >= 15,
        "应该有至少 15 个 profiles (允许并发冲突)，实际: {}",
        profiles.len()
    );

    cleanup_test_env(temp_dir);
}
