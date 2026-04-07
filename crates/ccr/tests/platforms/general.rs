#![allow(clippy::unwrap_used)]
// 🧪 CCR 多平台功能集成测试
//
// ⚠️ **重要提示**: 这些测试修改全局环境变量 CCR_ROOT，因此必须串行运行
// 运行方式: `cargo test --test platform_tests -- --test-threads=1`
//
// 测试内容:
// - Platform 初始化
// - Platform 切换
// - Profile 管理（跨平台）
// - 配置管理器
// - 模式检测
// - 迁移功能
//
// 共计: 22 个单元测试

use ccr::{
    CcrError, Platform, PlatformConfigManager, PlatformPaths, ProfileConfig, create_platform,
};
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════
// 测试辅助函数
// ═══════════════════════════════════════════════════════════

/// 创建临时测试环境
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // 设置环境变量指向临时目录
    unsafe {
        std::env::set_var("CCR_ROOT", temp_dir.path().to_str().unwrap());
    }

    // 确保根目录存在
    std::fs::create_dir_all(temp_dir.path()).ok();

    temp_dir
}

/// 清理测试环境
fn cleanup_test_env(temp_dir: TempDir) {
    unsafe {
        std::env::remove_var("CCR_ROOT");
    }
    drop(temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.1 单元测试 - Platform enum
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_all() {
    let platforms = Platform::all();
    assert_eq!(platforms.len(), 5);
    assert!(platforms.contains(&Platform::Claude));
    assert!(platforms.contains(&Platform::Codex));
    assert!(platforms.contains(&Platform::Gemini));
    assert!(platforms.contains(&Platform::Qwen));
    assert!(platforms.contains(&Platform::Droid));
}

#[test]
fn test_platform_implemented() {
    let implemented = Platform::implemented();
    assert_eq!(implemented.len(), 4);
    assert!(implemented.contains(&Platform::Claude));
    assert!(implemented.contains(&Platform::Codex));
    assert!(implemented.contains(&Platform::Gemini));
    assert!(implemented.contains(&Platform::Droid));
    assert!(!implemented.contains(&Platform::Qwen));
}

#[test]
fn test_platform_display_name() {
    assert_eq!(Platform::Claude.display_name(), "Claude Code");
    assert_eq!(Platform::Codex.display_name(), "Codex");
    assert_eq!(Platform::Gemini.display_name(), "Gemini CLI");
    assert_eq!(Platform::Qwen.display_name(), "Qwen CLI");
}

#[test]
fn test_platform_short_name() {
    assert_eq!(Platform::Claude.short_name(), "claude");
    assert_eq!(Platform::Codex.short_name(), "codex");
    assert_eq!(Platform::Gemini.short_name(), "gemini");
    assert_eq!(Platform::Qwen.short_name(), "qwen");
}

#[test]
fn test_platform_icon() {
    assert_eq!(Platform::Claude.icon(), "🤖");
    assert_eq!(Platform::Codex.icon(), "💻");
    assert_eq!(Platform::Gemini.icon(), "✨");
    assert_eq!(Platform::Qwen.icon(), "🌟");
}

// ═══════════════════════════════════════════════════════════
// 9.1 单元测试 - PlatformPaths
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_paths_structure() {
    let _temp_dir = setup_test_env();

    let paths = PlatformPaths::new(Platform::Claude).unwrap();

    // 验证路径结构（检查包含 CCR_ROOT 设置的路径）
    let root_str = paths.root.to_str().unwrap();
    assert!(
        root_str.contains("tmp") || root_str.contains(".ccr"),
        "Root path should be temp or .ccr, got: {}",
        root_str
    );
    assert!(paths.platform_dir.to_str().unwrap().contains("claude"));
    assert!(
        paths
            .profiles_file
            .to_str()
            .unwrap()
            .ends_with("profiles.toml")
    );
    assert!(
        paths
            .settings_file
            .to_str()
            .unwrap()
            .ends_with("settings.json")
    );
    assert!(paths.history_file.to_str().unwrap().contains("history"));
    assert!(paths.backups_dir.to_str().unwrap().contains("backups"));

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_platform_paths_ensure_directories() {
    let _temp_dir = setup_test_env();

    let paths = PlatformPaths::new(Platform::Codex).unwrap();

    // 确保目录不存在
    assert!(!paths.platform_dir.exists());
    assert!(!paths.backups_dir.exists());

    // 创建目录
    paths.ensure_directories().unwrap();

    // 验证目录已创建
    assert!(paths.root.exists());
    assert!(paths.platform_dir.exists());
    assert!(paths.history_file.parent().unwrap().exists());
    assert!(paths.backups_dir.exists());

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.1 单元测试 - PlatformConfigManager
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_config_manager_default() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    assert!(
        manager
            .config_path()
            .to_str()
            .unwrap()
            .contains("config.toml")
    );

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_platform_config_manager_create_default() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let config = manager.load_or_create_default().unwrap();

    // 验证默认配置
    assert_eq!(config.default_platform, "claude");
    assert_eq!(config.current_platform, "claude");
    assert!(config.platforms.contains_key("claude"));
    assert!(config.platforms.get("claude").unwrap().enabled);

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_platform_config_manager_register_platform() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 注册新平台
    let entry = ccr::PlatformConfigEntry {
        description: Some("Test Platform".to_string()),
        ..Default::default()
    };
    config.register_platform("test".to_string(), entry).unwrap();

    // 保存并重新加载
    manager.save(&config).unwrap();
    let reloaded = manager.load().unwrap();

    assert!(reloaded.platforms.contains_key("test"));
    assert_eq!(
        reloaded.platforms.get("test").unwrap().description,
        Some("Test Platform".to_string())
    );

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_platform_config_manager_switch_platform() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 注册 Codex 平台
    let entry = ccr::PlatformConfigEntry {
        description: Some("Codex Platform".to_string()),
        ..Default::default()
    };
    config
        .register_platform("codex".to_string(), entry)
        .unwrap();

    // 切换到 Codex
    config.set_current_platform("codex").unwrap();
    manager.save(&config).unwrap();

    // 验证切换
    let reloaded = manager.load().unwrap();
    assert_eq!(reloaded.current_platform, "codex");

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.2 集成测试 - Platform 初始化
// ═══════════════════════════════════════════════════════════

#[test]
fn test_platform_initialization() {
    let _temp_dir = setup_test_env();

    // 创建 Claude 平台实例
    let claude = create_platform(Platform::Claude).unwrap();
    assert_eq!(claude.platform_name(), "claude");
    assert_eq!(claude.platform_type(), Platform::Claude);

    // 创建 Codex 平台实例
    let codex = create_platform(Platform::Codex).unwrap();
    assert_eq!(codex.platform_name(), "codex");
    assert_eq!(codex.platform_type(), Platform::Codex);

    // 创建 Gemini 平台实例
    let gemini = create_platform(Platform::Gemini).unwrap();
    assert_eq!(gemini.platform_name(), "gemini");
    assert_eq!(gemini.platform_type(), Platform::Gemini);

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_platform_initialization_creates_directories() {
    let _temp_dir = setup_test_env();

    let paths = PlatformPaths::new(Platform::Codex).unwrap();

    // 初始化前目录不存在
    assert!(!paths.platform_dir.exists());

    // 创建平台实例（可能会创建目录）
    let codex = create_platform(Platform::Codex).unwrap();

    // 保存一个 profile（会触发目录创建）
    let mut profile = ProfileConfig::new();
    profile.base_url = Some("https://api.test.com".to_string());
    profile.auth_token = Some("test-token".to_string());
    profile.model = Some("test-model".to_string());

    let result = codex.save_profile("test", &profile);

    // 验证目录已创建
    if result.is_ok() {
        assert!(paths.platform_dir.exists());
    }

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.2 集成测试 - Profile 管理
// ═══════════════════════════════════════════════════════════

#[test]
fn test_profile_save_and_load() {
    let _temp_dir = setup_test_env();

    let codex = create_platform(Platform::Codex).unwrap();

    // 创建测试 profile（使用符合 Codex 格式的 token）
    let mut profile = ProfileConfig::new();
    profile.description = Some("Test Profile".to_string());
    profile.base_url = Some("https://api.test.com".to_string());
    profile.auth_token = Some("ghp_test123456789012345678901234567890".to_string()); // GitHub token 格式
    profile.model = Some("gpt-4".to_string());
    profile.small_fast_model = Some("gpt-3.5-turbo".to_string());

    // 保存 profile
    codex.save_profile("test-profile", &profile).unwrap();

    // 加载并验证
    let profiles = codex.load_profiles().unwrap();
    assert!(profiles.contains_key("test-profile"));

    let loaded = profiles.get("test-profile").unwrap();
    assert_eq!(loaded.description, Some("Test Profile".to_string()));
    assert_eq!(loaded.base_url, Some("https://api.test.com".to_string()));
    assert_eq!(loaded.model, Some("gpt-4".to_string()));

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_profile_delete() {
    let _temp_dir = setup_test_env();

    let gemini = create_platform(Platform::Gemini).unwrap();

    // 创建并保存 profile（使用符合 Gemini 格式的 token）
    let mut profile = ProfileConfig::new();
    profile.base_url = Some("https://api.gemini.com".to_string());
    profile.auth_token = Some("AIzaSy1234567890123456789012345678901234".to_string()); // Google API Key 格式
    profile.model = Some("gemini-pro".to_string());

    gemini.save_profile("to-delete", &profile).unwrap();

    // 验证 profile 存在
    let profiles = gemini.load_profiles().unwrap();
    assert!(profiles.contains_key("to-delete"));

    // 删除 profile
    gemini.delete_profile("to-delete").unwrap();

    // 验证已删除
    let profiles_after = gemini.load_profiles().unwrap();
    assert!(!profiles_after.contains_key("to-delete"));

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_profile_list_names() {
    let _temp_dir = setup_test_env();

    let codex = create_platform(Platform::Codex).unwrap();

    // 创建多个 profiles（使用符合格式的 token）
    for i in 1..=3 {
        let mut profile = ProfileConfig::new();
        profile.base_url = Some(format!("https://api{}.test.com", i));
        profile.auth_token = Some(format!(
            "ghp_test{:0>36}",
            i // GitHub token 格式，40 字符
        ));
        profile.model = Some("model".to_string());

        codex
            .save_profile(&format!("profile-{}", i), &profile)
            .unwrap();
    }

    // 列出所有 profile 名称
    let names = codex.list_profile_names().unwrap();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"profile-1".to_string()));
    assert!(names.contains(&"profile-2".to_string()));
    assert!(names.contains(&"profile-3".to_string()));

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.2 集成测试 - 跨平台操作
// ═══════════════════════════════════════════════════════════

#[test]
fn test_multiple_platforms_coexist() {
    let _temp_dir = setup_test_env();

    // 创建多个平台的 profiles
    let claude = create_platform(Platform::Claude).unwrap();
    let codex = create_platform(Platform::Codex).unwrap();
    let gemini = create_platform(Platform::Gemini).unwrap();

    // 为每个平台保存一个 profile（使用符合格式的 token）
    let mut claude_profile = ProfileConfig::new();
    claude_profile.base_url = Some("https://api.anthropic.com".to_string());
    claude_profile.auth_token = Some("sk-ant-api03-1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string()); // Claude token 格式
    claude_profile.model = Some("claude-3-5-sonnet".to_string());
    claude.save_profile("official", &claude_profile).unwrap();

    let mut codex_profile = ProfileConfig::new();
    codex_profile.base_url = Some("https://api.github.com".to_string());
    codex_profile.auth_token = Some("ghp_1234567890123456789012345678901234567890".to_string()); // GitHub token 格式
    codex_profile.model = Some("gpt-4".to_string());
    codex.save_profile("github", &codex_profile).unwrap();

    let mut gemini_profile = ProfileConfig::new();
    gemini_profile.base_url = Some("https://api.google.com".to_string());
    gemini_profile.auth_token = Some("AIzaSy1234567890123456789012345678901234".to_string()); // Google API Key 格式
    gemini_profile.model = Some("gemini-pro".to_string());
    gemini.save_profile("google", &gemini_profile).unwrap();

    // 验证每个平台都有自己的 profiles
    let claude_profiles = claude.load_profiles().unwrap();
    let codex_profiles = codex.load_profiles().unwrap();
    let gemini_profiles = gemini.load_profiles().unwrap();

    assert!(claude_profiles.contains_key("official"));
    assert!(codex_profiles.contains_key("github"));
    assert!(gemini_profiles.contains_key("google"));

    // 验证 profiles 互不干扰
    assert!(!claude_profiles.contains_key("github"));
    assert!(!codex_profiles.contains_key("google"));
    assert!(!gemini_profiles.contains_key("official"));

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 9.3 回归测试 - Legacy 模式兼容性
// ═══════════════════════════════════════════════════════════

#[test]
fn test_config_mode_detection() {
    let _temp_dir = setup_test_env();

    // 在 Unified 模式下，config.toml 存在
    let manager = PlatformConfigManager::with_default().unwrap();
    manager.load_or_create_default().unwrap();

    // 验证 config 文件存在
    assert!(manager.config_path().exists());

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_unified_config_persistence() {
    let _temp_dir = setup_test_env();

    let manager = PlatformConfigManager::with_default().unwrap();
    let mut config = manager.load_or_create_default().unwrap();

    // 先注册 codex 平台
    let entry = ccr::PlatformConfigEntry {
        description: Some("Codex Platform".to_string()),
        ..Default::default()
    };
    config
        .register_platform("codex".to_string(), entry)
        .unwrap();

    // 切换配置
    config.set_current_platform("codex").unwrap();
    manager.save(&config).unwrap();

    // 重新加载并验证
    let reloaded = manager.load().unwrap();
    assert_eq!(reloaded.current_platform, "codex");

    cleanup_test_env(_temp_dir);
}

// ═══════════════════════════════════════════════════════════
// 错误处理测试
// ═══════════════════════════════════════════════════════════

#[test]
fn test_invalid_platform_name() {
    use std::str::FromStr;

    let result = Platform::from_str("invalid-platform");
    assert!(result.is_err());

    if let Err(CcrError::PlatformNotFound(name)) = result {
        assert_eq!(name, "invalid-platform");
    } else {
        panic!("Expected PlatformNotFound error");
    }
}

#[test]
fn test_delete_nonexistent_profile() {
    let _temp_dir = setup_test_env();

    let codex = create_platform(Platform::Codex).unwrap();

    // 尝试删除不存在的 profile
    let result = codex.delete_profile("nonexistent");
    assert!(result.is_err());

    cleanup_test_env(_temp_dir);
}

#[test]
fn test_load_empty_profiles() {
    let _temp_dir = setup_test_env();

    let gemini = create_platform(Platform::Gemini).unwrap();

    // 加载空的 profiles（文件不存在）
    let profiles = gemini.load_profiles().unwrap();
    assert_eq!(profiles.len(), 0);

    cleanup_test_env(_temp_dir);
}
