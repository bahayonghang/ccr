#![allow(clippy::unwrap_used)]
//! 🎯 同步内容选择功能集成测试
//!
//! 测试新的交互式内容选择功能的完整流程

use ccr::CONFIG_LOCK;
use ccr::commands::SyncContentSelector;
use ccr::commands::sync_content_selector::{SyncContentSelection, SyncContentType};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sync_content_selection_flow() {
    let _guard = CONFIG_LOCK.lock().expect("配置锁已中毒");
    // 创建临时测试环境
    let temp_dir = tempdir().unwrap();
    let ccr_root = temp_dir.path().join(".ccr");

    // 设置CCR_ROOT环境变量
    // SAFETY: 测试仅在当前进程临时设置 CCR_ROOT，结束前会显式清理。
    unsafe {
        std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
    }

    // 创建测试文件结构
    fs::create_dir_all(&ccr_root).unwrap();
    fs::write(
        ccr_root.join("config.toml"),
        "default_platform = 'claude'\n",
    )
    .unwrap();

    // 创建平台目录
    let platforms_dir = ccr_root.join("platforms");
    fs::create_dir_all(&platforms_dir).unwrap();

    let claude_dir = platforms_dir.join("claude");
    fs::create_dir_all(&claude_dir).unwrap();
    fs::write(claude_dir.join("settings.json"), "{\"env\": {}}\n").unwrap();

    // 测试内容选择器创建
    let selector = SyncContentSelector::new();

    // 验证可用类型（通过公共方法获取）
    let available_types = selector.get_available_types();
    assert!(available_types.contains(&SyncContentType::Config));
    assert!(available_types.contains(&SyncContentType::Claude));

    // 测试默认选择：应默认选择所有平台类型（Claude/Gemini/Qwen），不包含 Config
    let default_selection = SyncContentSelection::default();
    assert_eq!(default_selection.count(), 3);
    assert!(
        default_selection
            .selected_types
            .contains(&SyncContentType::Claude)
    );
    assert!(
        default_selection
            .selected_types
            .contains(&SyncContentType::Gemini)
    );
    assert!(
        default_selection
            .selected_types
            .contains(&SyncContentType::Qwen)
    );
    assert!(
        !default_selection
            .selected_types
            .contains(&SyncContentType::Config)
    );

    // 测试自定义选择
    let custom_selection =
        SyncContentSelection::custom(vec![SyncContentType::Config, SyncContentType::Claude]);
    assert_eq!(custom_selection.count(), 2);

    // 测试路径转换
    let paths = custom_selection.to_paths();
    assert!(paths.contains(&"config.toml".to_string()));
    assert!(paths.contains(&"platforms/claude".to_string()));

    // 清理环境变量
    // SAFETY: 仅清理本测试设置的 CCR_ROOT，避免影响其他测试。
    unsafe {
        std::env::remove_var("CCR_ROOT");
    }
}

#[test]
fn test_sync_content_type_detection() {
    let _guard = CONFIG_LOCK.lock().expect("配置锁已中毒");
    let temp_dir = tempdir().unwrap();
    let ccr_root = temp_dir.path().join(".ccr");

    // SAFETY: 测试仅在当前进程临时设置 CCR_ROOT，结束前会显式清理。
    unsafe {
        std::env::set_var("CCR_ROOT", ccr_root.to_str().unwrap());
    }

    // 只创建config文件
    fs::create_dir_all(&ccr_root).unwrap();
    fs::write(ccr_root.join("config.toml"), "test").unwrap();

    // 验证当前环境下各平台目录尚不存在
    assert!(!SyncContentType::Claude.exists());
    assert!(!SyncContentType::Gemini.exists());
    assert!(!SyncContentType::Qwen.exists());

    // 清理环境变量
    // SAFETY: 仅清理本测试设置的 CCR_ROOT，避免影响其他测试。
    unsafe {
        std::env::remove_var("CCR_ROOT");
    }
}

#[test]
fn test_sync_content_selection_empty() {
    // 测试空选择
    let empty_selection = SyncContentSelection::custom(vec![]);
    assert_eq!(empty_selection.count(), 0);
    assert!(empty_selection.to_paths().is_empty());
}
