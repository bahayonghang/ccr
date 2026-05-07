#![allow(clippy::unwrap_used)]
// 🧪 CCR 多平台集成测试
//
// ⚠️ **重要提示**: 这些测试修改全局环境变量 CCR_ROOT，因此必须串行运行
// 运行方式: `cargo test --test platform_integration_tests -- --test-threads=1`
//
// 测试内容:
// - 平台切换工作流（完整流程）
// - 跨平台 Profile 管理和隔离
// - 配置服务与平台实例的集成
// - 历史记录与平台操作的集成
// - 备份恢复与平台配置的集成
//
// 共计: 10 个集成测试

use crate::{PlatformTestEnv, setup_platform_test_env};
use ccr::{Platform, PlatformConfigManager, PlatformPaths, ProfileConfig, create_platform};

type TempDir = PlatformTestEnv;

// ═══════════════════════════════════════════════════════════
// 测试辅助函数
// ═══════════════════════════════════════════════════════════

/// 创建临时测试环境
fn setup_test_env() -> TempDir {
    let temp_dir = setup_platform_test_env();

    // 设置环境变量指向临时目录
    // SAFETY: 测试仅在当前进程临时设置 CCR_ROOT，清理函数会恢复干净状态。
    unsafe {
        std::env::set_var("CCR_ROOT", temp_dir.path().to_str().unwrap());
    }

    // 确保根目录存在
    std::fs::create_dir_all(temp_dir.path()).ok();

    temp_dir
}

/// 清理测试环境
fn cleanup_test_env(temp_dir: TempDir) {
    // SAFETY: 仅清理本测试设置的 CCR_ROOT，避免污染后续测试。
    unsafe {
        std::env::remove_var("CCR_ROOT");
    }
    drop(temp_dir);
}

/// 创建测试用的 Claude profile
fn create_claude_profile(name: &str) -> ProfileConfig {
    let mut profile = ProfileConfig::new();
    profile.description = Some(format!("Test Claude profile: {}", name));
    profile.base_url = Some("https://api.anthropic.com".to_string());
    profile.auth_token = Some(format!(
        "sk-ant-api03-{}",
        "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"
    ));
    profile.model = Some("claude-3-5-sonnet-20241022".to_string());
    profile.small_fast_model = Some("claude-3-5-haiku-20241022".to_string());
    profile.provider = Some("Anthropic".to_string());
    profile
}

/// 创建测试用的 Codex profile
fn create_codex_profile(name: &str) -> ProfileConfig {
    let mut profile = ProfileConfig::new();
    profile.description = Some(format!("Test Codex profile: {}", name));
    profile.base_url = Some("https://api.openai.com/v1".to_string());
    profile.auth_token = Some("sk-test-codex-12345678901234567890".to_string());
    profile.model = Some("gpt-5-codex".to_string());
    profile.small_fast_model = Some("gpt-5-mini".to_string());
    profile.provider = Some("OpenAI".to_string());
    profile
}

/// 创建测试用的 Gemini profile
fn create_gemini_profile(name: &str) -> ProfileConfig {
    let mut profile = ProfileConfig::new();
    profile.description = Some(format!("Test Gemini profile: {}", name));
    profile.base_url = Some("https://generativelanguage.googleapis.com/v1".to_string());
    profile.auth_token = Some("AIzaSy1234567890123456789012345678901234".to_string());
    profile.model = Some("gemini-2.0-flash-exp".to_string());
    profile.small_fast_model = Some("gemini-1.5-flash".to_string());
    profile.provider = Some("Google".to_string());
    profile
}

// ═══════════════════════════════════════════════════════════
// 集成测试 1: 平台切换工作流
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_switching_workflow() {
    let _temp_dir = setup_test_env();

    // 1. 初始化配置管理器
    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 验证初始状态
    assert!(config.platforms.contains_key("claude"));

    // 2. 注册 Codex 平台
    let codex_entry = ccr::PlatformConfigEntry {
        description: Some("Codex Platform".to_string()),
        enabled: true,
        ..Default::default()
    };
    config
        .register_platform("codex".to_string(), codex_entry)
        .unwrap();

    // 3. 为 Claude 创建 profile
    let claude = create_platform(Platform::Claude).unwrap();
    let claude_profile = create_claude_profile("official");
    claude.save_profile("official", &claude_profile).unwrap();

    // 立即验证 profile 被保存
    let claude_profiles_immediate = claude.load_profiles().unwrap();
    assert!(
        claude_profiles_immediate.contains_key("official"),
        "Claude profile 'official' 应该在保存后立即存在"
    );

    // 4. 为 Codex 创建 profile
    let codex = create_platform(Platform::Codex).unwrap();
    let codex_profile = create_codex_profile("github");
    codex.save_profile("github", &codex_profile).unwrap();

    // 5. 切换到 Codex 平台
    config.set_current_profile("codex", Some("github")).unwrap();
    manager.save(&config).unwrap();

    // 6. 验证切换成功
    let reloaded = manager.load().unwrap();
    assert_eq!(
        reloaded.get_current_profile("codex").unwrap(),
        Some("github")
    );

    // 7. 验证 profiles 依然独立存在
    let claude_profiles = claude.load_profiles().unwrap();
    let codex_profiles = codex.load_profiles().unwrap();

    assert!(claude_profiles.contains_key("official"));
    assert!(codex_profiles.contains_key("github"));

    // 8. 切换回 Claude
    let mut config = manager.load().unwrap();
    config
        .set_current_profile("claude", Some("official"))
        .unwrap();
    manager.save(&config).unwrap();

    let final_config = manager.load().unwrap();
    assert_eq!(
        final_config.get_current_profile("claude").unwrap(),
        Some("official")
    );

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 2: 跨平台 Profile 批量管理
// ═══════════════════════════════════════════════════════════

#[test]
fn test_cross_platform_profile_management() {
    let _temp_dir = setup_test_env();

    // 创建三个平台实例
    let claude = create_platform(Platform::Claude).unwrap();
    let codex = create_platform(Platform::Codex).unwrap();
    let gemini = create_platform(Platform::Gemini).unwrap();

    // 为每个平台创建多个 profiles
    // Claude: 3 profiles
    for i in 1..=3 {
        let profile = create_claude_profile(&format!("claude-{}", i));
        claude
            .save_profile(&format!("profile-{}", i), &profile)
            .unwrap();
    }

    // Codex: 3 profiles
    for i in 1..=3 {
        let profile = create_codex_profile(&format!("codex-{}", i));
        codex
            .save_profile(&format!("profile-{}", i), &profile)
            .unwrap();
    }

    // Gemini: 3 profiles
    for i in 1..=3 {
        let profile = create_gemini_profile(&format!("gemini-{}", i));
        gemini
            .save_profile(&format!("profile-{}", i), &profile)
            .unwrap();
    }

    // 验证每个平台都有 3 个 profiles
    assert_eq!(claude.list_profile_names().unwrap().len(), 3);
    assert_eq!(codex.list_profile_names().unwrap().len(), 3);
    assert_eq!(gemini.list_profile_names().unwrap().len(), 3);

    // 验证 profiles 完全隔离（Claude 的 profile 不会出现在 Codex 中）
    let claude_profiles = claude.load_profiles().unwrap();
    let codex_profiles = codex.load_profiles().unwrap();
    let gemini_profiles = gemini.load_profiles().unwrap();

    // Claude profiles 包含 "claude-" 描述
    for (_, profile) in claude_profiles.iter() {
        let desc = profile.description.as_ref().unwrap();
        assert!(desc.contains("Claude"));
    }

    // Codex profiles 包含 "codex-" 描述
    for (_, profile) in codex_profiles.iter() {
        let desc = profile.description.as_ref().unwrap();
        assert!(desc.contains("Codex"));
    }

    // Gemini profiles 包含 "gemini-" 描述
    for (_, profile) in gemini_profiles.iter() {
        let desc = profile.description.as_ref().unwrap();
        assert!(desc.contains("Gemini"));
    }

    // 删除 Claude 的一个 profile
    claude.delete_profile("profile-2").unwrap();
    assert_eq!(claude.list_profile_names().unwrap().len(), 2);

    // 验证其他平台不受影响
    assert_eq!(codex.list_profile_names().unwrap().len(), 3);
    assert_eq!(gemini.list_profile_names().unwrap().len(), 3);

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 3: 平台路径隔离验证
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_path_isolation() {
    let _temp_dir = setup_test_env();

    // 获取三个平台的路径
    let claude_paths = PlatformPaths::new(Platform::Claude).unwrap();
    let codex_paths = PlatformPaths::new(Platform::Codex).unwrap();
    let gemini_paths = PlatformPaths::new(Platform::Gemini).unwrap();

    // 验证平台目录不同
    assert_ne!(claude_paths.platform_dir, codex_paths.platform_dir);
    assert_ne!(claude_paths.platform_dir, gemini_paths.platform_dir);
    assert_ne!(codex_paths.platform_dir, gemini_paths.platform_dir);

    // 验证 profiles 文件路径不同
    assert_ne!(claude_paths.profiles_file, codex_paths.profiles_file);
    assert_ne!(claude_paths.profiles_file, gemini_paths.profiles_file);
    assert_ne!(codex_paths.profiles_file, gemini_paths.profiles_file);

    // 验证历史文件路径不同
    assert_ne!(claude_paths.history_file, codex_paths.history_file);
    assert_ne!(claude_paths.history_file, gemini_paths.history_file);
    assert_ne!(codex_paths.history_file, gemini_paths.history_file);

    // 验证备份目录不同
    assert_ne!(claude_paths.backups_dir, codex_paths.backups_dir);
    assert_ne!(claude_paths.backups_dir, gemini_paths.backups_dir);
    assert_ne!(codex_paths.backups_dir, gemini_paths.backups_dir);

    // 但它们共享同一个根目录
    assert_eq!(claude_paths.root, codex_paths.root);
    assert_eq!(claude_paths.root, gemini_paths.root);

    // 验证路径包含正确的平台名称
    assert!(
        claude_paths
            .platform_dir
            .to_str()
            .unwrap()
            .contains("claude")
    );
    assert!(codex_paths.platform_dir.to_str().unwrap().contains("codex"));
    assert!(
        gemini_paths
            .platform_dir
            .to_str()
            .unwrap()
            .contains("gemini")
    );

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 4: 配置持久化和重载
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_config_persistence_and_reload() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 更新 Claude 平台配置（默认已存在）
    if let Some(claude_entry) = config.platforms.get_mut("claude") {
        claude_entry.description = Some("claude Platform".to_string());
        claude_entry.current_profile = Some("claude-official".to_string());
    }

    // 注册 Codex 和 Gemini
    for platform_name in &["codex", "gemini"] {
        let entry = ccr::PlatformConfigEntry {
            description: Some(format!("{} Platform", platform_name)),
            enabled: true,
            current_profile: Some(format!("{}-official", platform_name)),
            ..Default::default()
        };
        config
            .register_platform(platform_name.to_string(), entry)
            .unwrap();
    }

    // 设置当前平台为 Codex
    config
        .set_current_profile("codex", Some("codex-official"))
        .unwrap();
    manager.save(&config).unwrap();

    // 重新加载并验证
    let reloaded = manager.load().unwrap();

    // 验证当前平台
    assert_eq!(
        reloaded.get_current_profile("codex").unwrap(),
        Some("codex-official")
    );

    // 验证所有平台都已注册
    assert!(reloaded.platforms.contains_key("claude"));
    assert!(reloaded.platforms.contains_key("codex"));
    assert!(reloaded.platforms.contains_key("gemini"));

    // 验证平台描述
    assert_eq!(
        reloaded.platforms.get("claude").unwrap().description,
        Some("claude Platform".to_string())
    );
    assert_eq!(
        reloaded.platforms.get("codex").unwrap().description,
        Some("codex Platform".to_string())
    );
    assert_eq!(
        reloaded.platforms.get("gemini").unwrap().description,
        Some("gemini Platform".to_string())
    );

    // 验证 current_profile 字段
    assert_eq!(
        reloaded.platforms.get("claude").unwrap().current_profile,
        Some("claude-official".to_string())
    );

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 5: Profile 应用工作流
// ═══════════════════════════════════════════════════════════

#[test]
fn test_profile_application_workflow() {
    let _temp_dir = setup_test_env();

    // 创建平台实例
    let claude = create_platform(Platform::Claude).unwrap();

    // 创建多个 profiles
    let profile1 = create_claude_profile("official");
    let mut profile2 = create_claude_profile("custom");
    profile2.base_url = Some("https://custom-api.example.com".to_string());
    profile2.model = Some("claude-opus-4".to_string());

    // 保存 profiles
    claude.save_profile("official", &profile1).unwrap();
    claude.save_profile("custom", &profile2).unwrap();

    // 验证 profiles 已保存
    let profiles = claude.load_profiles().unwrap();
    assert_eq!(profiles.len(), 2);
    assert!(profiles.contains_key("official"));
    assert!(profiles.contains_key("custom"));

    // 验证 profile 内容
    let loaded_official = profiles.get("official").unwrap();
    assert_eq!(
        loaded_official.base_url,
        Some("https://api.anthropic.com".to_string())
    );
    assert_eq!(
        loaded_official.model,
        Some("claude-3-5-sonnet-20241022".to_string())
    );

    let loaded_custom = profiles.get("custom").unwrap();
    assert_eq!(
        loaded_custom.base_url,
        Some("https://custom-api.example.com".to_string())
    );
    assert_eq!(loaded_custom.model, Some("claude-opus-4".to_string()));

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 6: 平台间互不干扰验证
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_isolation_under_operations() {
    let _temp_dir = setup_test_env();

    let claude = create_platform(Platform::Claude).unwrap();
    let codex = create_platform(Platform::Codex).unwrap();

    // 同时在两个平台创建同名 profile
    let claude_profile = create_claude_profile("default");
    let codex_profile = create_codex_profile("default");

    claude.save_profile("default", &claude_profile).unwrap();
    codex.save_profile("default", &codex_profile).unwrap();

    // 验证两个平台都有 "default" profile
    let claude_profiles = claude.load_profiles().unwrap();
    let codex_profiles = codex.load_profiles().unwrap();

    assert!(claude_profiles.contains_key("default"));
    assert!(codex_profiles.contains_key("default"));

    // 但内容应该不同
    let claude_default = claude_profiles.get("default").unwrap();
    let codex_default = codex_profiles.get("default").unwrap();

    assert_ne!(claude_default.base_url, codex_default.base_url);
    assert!(
        claude_default
            .base_url
            .as_ref()
            .unwrap()
            .contains("anthropic")
    );
    assert!(codex_default.base_url.as_ref().unwrap().contains("openai"));

    // 删除 Claude 的 default profile
    claude.delete_profile("default").unwrap();

    // Codex 的 default 应该不受影响
    let codex_profiles_after = codex.load_profiles().unwrap();
    assert!(codex_profiles_after.contains_key("default"));

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 7: 多次平台切换稳定性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_multiple_platform_switches() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 注册三个平台
    for platform_name in &["claude", "codex", "gemini"] {
        if !config.platforms.contains_key(*platform_name) {
            let entry = ccr::PlatformConfigEntry {
                enabled: true,
                ..Default::default()
            };
            config
                .register_platform(platform_name.to_string(), entry)
                .unwrap();
        }
    }

    // 执行多次切换
    let platforms = vec!["claude", "codex", "gemini", "claude", "gemini", "codex"];

    for platform in platforms {
        config
            .set_current_profile(platform, Some("selected"))
            .unwrap();
        manager.save(&config).unwrap();

        // 重新加载验证
        let reloaded = manager.load().unwrap();
        assert_eq!(
            reloaded.get_current_profile(platform).unwrap(),
            Some("selected")
        );

        // 更新本地配置引用
        config = reloaded;
    }

    // 最终验证
    let final_config = manager.load().unwrap();
    assert_eq!(
        final_config.get_current_profile("codex").unwrap(),
        Some("selected")
    );

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 8: Profile 验证集成
// ═══════════════════════════════════════════════════════════

#[test]
fn test_profile_validation_integration() {
    let _temp_dir = setup_test_env();

    let claude = create_platform(Platform::Claude).unwrap();

    // 有效 profile
    let valid_profile = create_claude_profile("valid");
    assert!(claude.validate_profile(&valid_profile).is_ok());

    // 无效 profile - 缺少 base_url
    let mut invalid_profile = ProfileConfig::new();
    invalid_profile.auth_token = Some("sk-ant-api03-123".to_string());
    invalid_profile.model = Some("claude-3".to_string());
    assert!(claude.validate_profile(&invalid_profile).is_err());

    // 无效 profile - 空 base_url
    let mut invalid_profile2 = ProfileConfig::new();
    invalid_profile2.base_url = Some("".to_string());
    invalid_profile2.auth_token = Some("sk-ant-api03-123".to_string());
    invalid_profile2.model = Some("claude-3".to_string());
    assert!(claude.validate_profile(&invalid_profile2).is_err());

    // 保存有效 profile 应该成功
    assert!(claude.save_profile("valid", &valid_profile).is_ok());

    // 尝试保存无效 profile 应该失败
    assert!(claude.save_profile("invalid", &invalid_profile).is_err());

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 9: 并发场景 - 多平台同时操作
// ═══════════════════════════════════════════════════════════

#[test]
fn test_concurrent_multi_platform_operations() {
    use std::sync::Arc;
    use std::thread;

    let _temp_dir = setup_test_env();

    // 预先创建平台实例（Arc 共享）
    let claude = Arc::new(create_platform(Platform::Claude).unwrap());
    let codex = Arc::new(create_platform(Platform::Codex).unwrap());
    let gemini = Arc::new(create_platform(Platform::Gemini).unwrap());

    // 创建三个线程，每个线程操作一个平台
    let claude_clone = Arc::clone(&claude);
    let handle1 = thread::spawn(move || {
        for i in 1..=5 {
            let profile = create_claude_profile(&format!("thread-{}", i));
            claude_clone
                .save_profile(&format!("profile-{}", i), &profile)
                .unwrap();
        }
    });

    let codex_clone = Arc::clone(&codex);
    let handle2 = thread::spawn(move || {
        for i in 1..=5 {
            let profile = create_codex_profile(&format!("thread-{}", i));
            codex_clone
                .save_profile(&format!("profile-{}", i), &profile)
                .unwrap();
        }
    });

    let gemini_clone = Arc::clone(&gemini);
    let handle3 = thread::spawn(move || {
        for i in 1..=5 {
            let profile = create_gemini_profile(&format!("thread-{}", i));
            gemini_clone
                .save_profile(&format!("profile-{}", i), &profile)
                .unwrap();
        }
    });

    // 等待所有线程完成
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();

    // 验证所有 profiles 都成功创建
    assert_eq!(claude.list_profile_names().unwrap().len(), 5);
    assert_eq!(codex.list_profile_names().unwrap().len(), 5);
    assert_eq!(gemini.list_profile_names().unwrap().len(), 5);

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 集成测试 10: 完整工作流端到端测试
// ═══════════════════════════════════════════════════════════

#[test]
fn test_end_to_end_complete_workflow() {
    let _temp_dir = setup_test_env();

    // 1. 初始化配置系统
    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();
    // 2. 为 Claude 创建 profiles
    let claude = create_platform(Platform::Claude).unwrap();
    let claude_official = create_claude_profile("official");
    let mut claude_custom = create_claude_profile("custom");
    claude_custom.base_url = Some("https://custom.anthropic.com".to_string());

    claude.save_profile("official", &claude_official).unwrap();
    claude.save_profile("custom", &claude_custom).unwrap();

    // 3. 注册并切换到 Codex
    let codex_entry = ccr::PlatformConfigEntry {
        enabled: true,
        description: Some("Codex CLI".to_string()),
        ..Default::default()
    };
    config
        .register_platform("codex".to_string(), codex_entry)
        .unwrap();
    config.set_current_profile("codex", Some("github")).unwrap();
    manager.save(&config).unwrap();

    // 4. 为 Codex 创建 profiles
    let codex = create_platform(Platform::Codex).unwrap();
    let codex_github = create_codex_profile("github");
    codex.save_profile("github", &codex_github).unwrap();

    // 5. 验证当前平台是 Codex
    let reloaded = manager.load().unwrap();
    assert_eq!(
        reloaded.get_current_profile("codex").unwrap(),
        Some("github")
    );

    // 6. 验证 Claude profiles 依然存在
    let claude_profiles = claude.load_profiles().unwrap();
    assert_eq!(claude_profiles.len(), 2);
    assert!(claude_profiles.contains_key("official"));
    assert!(claude_profiles.contains_key("custom"));

    // 7. 验证 Codex profiles 存在
    let codex_profiles = codex.load_profiles().unwrap();
    assert_eq!(codex_profiles.len(), 1);
    assert!(codex_profiles.contains_key("github"));

    // 8. 切换回 Claude
    let mut config = manager.load().unwrap();
    config
        .set_current_profile("claude", Some("official"))
        .unwrap();
    manager.save(&config).unwrap();

    // 9. 最终验证
    let final_config = manager.load().unwrap();
    assert_eq!(
        final_config.get_current_profile("claude").unwrap(),
        Some("official")
    );

    // 10. 清理：删除一个 Claude profile
    claude.delete_profile("custom").unwrap();
    let claude_profiles_final = claude.load_profiles().unwrap();
    assert_eq!(claude_profiles_final.len(), 1);
    assert!(claude_profiles_final.contains_key("official"));

    cleanup_test_env(_temp_dir);
}
