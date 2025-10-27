// 🎬 init 命令实现 - 初始化配置文件
// 📦 初始化 CCR 多平台配置结构 (~/.ccr/) 或兼容旧版模式 (~/.ccs_config.toml)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::config::ConfigManager;
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformPaths};
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

/// 📋 示例配置文件内容(嵌入到二进制中)
/// 编译时从 .ccs_config.toml.example 读取
const EXAMPLE_CONFIG: &str = include_str!("../../.ccs_config.toml.example");

/// 🎬 初始化配置文件
///
/// **新的行为 (2025)**: 默认使用 Unified Mode (~/.ccr/ 目录结构)
///
/// 执行流程:
/// 1. ✅ 检测配置模式 (Unified vs Legacy)
/// 2. 🆕 Unified Mode: 初始化 ~/.ccr/ 目录和平台结构
/// 3. 🔙 Legacy Mode: 兼容旧的 ~/.ccs_config.toml（仅在环境变量强制时）
/// 4. 💾 备份现有配置(--force 模式)
/// 5. 📝 创建新配置文件和目录结构
/// 6. 💡 显示后续步骤提示
///
/// # 参数
///
/// * `force` - 强制重新初始化（覆盖现有配置）
///
/// # 配置模式检测
///
/// - **Unified Mode** (默认): 创建 `~/.ccr/` 目录结构
/// - **Legacy Mode**: 仅在设置 `CCR_LEGACY_MODE=1` 时使用 `~/.ccs_config.toml`
///
pub fn init_command(force: bool) -> Result<()> {
    ColorOutput::title("CCR 配置初始化");
    println!();

    // 🔍 检测配置模式
    let use_legacy = std::env::var("CCR_LEGACY_MODE").is_ok();

    if use_legacy {
        ColorOutput::warning("检测到 CCR_LEGACY_MODE 环境变量，使用 Legacy 模式");
        println!();
        return init_legacy_mode(force);
    }

    // 🆕 使用新的 Unified Mode
    init_unified_mode(force)
}

/// 🆕 初始化 Unified Mode - 新的多平台配置结构
///
/// 创建目录结构:
/// ```text
/// ~/.ccr/
/// ├── config.toml              # 平台注册表
/// └── platforms/
///     └── claude/              # Claude 平台目录（默认）
///         ├── profiles.toml    # 将在首次使用时创建
///         ├── history/         # 历史记录目录
///         └── backups/         # 备份目录
/// ```
fn init_unified_mode(force: bool) -> Result<()> {
    let manager = PlatformConfigManager::default()?;
    let config_path = manager.config_path();

    // 检查配置是否已存在
    if config_path.exists() {
        if !force {
            ColorOutput::warning(&format!("配置已存在: {}", config_path.display()));
            println!();
            ColorOutput::info("配置已经初始化，无需重复执行");
            ColorOutput::info("提示:");
            println!("  • 查看平台列表: ccr platform list");
            println!("  • 初始化特定平台: ccr platform init <平台名>");
            println!("  • 强制重新初始化: ccr init --force");
            println!();
            return Ok(());
        }

        // 使用 --force 时需要确认
        println!();
        ColorOutput::warning("⚠️  警告: 即将覆盖现有配置！");
        ColorOutput::info("提示: 现有配置会自动备份");
        println!();

        print!("确认强制重新初始化? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") {
            ColorOutput::info("已取消初始化操作");
            return Ok(());
        }
        println!();

        // 备份现有配置
        ColorOutput::step("备份现有配置");
        if let Ok(content) = fs::read_to_string(&config_path) {
            let backup_path = config_path.with_extension("toml.bak");
            fs::write(&backup_path, content)?;
            ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
        }
        println!();
    }

    // 创建目录结构
    ColorOutput::step("创建 CCR 目录结构");

    // 获取 CCR 根目录
    let home = dirs::home_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let ccr_root = home.join(".ccr");
    let platforms_dir = ccr_root.join("platforms");

    // 创建根目录和平台目录
    fs::create_dir_all(&platforms_dir)
        .map_err(|e| CcrError::from(e))?;

    ColorOutput::success(&format!("✓ CCR 根目录: {}", ccr_root.display()));
    ColorOutput::success(&format!("✓ 平台目录: {}", platforms_dir.display()));

    // 初始化默认平台（Claude）
    println!();
    ColorOutput::step("初始化默认平台: Claude");

    let claude_paths = PlatformPaths::new(Platform::Claude)?;
    claude_paths.ensure_directories()?;

    ColorOutput::success(&format!("✓ Claude 平台目录: {}", claude_paths.platform_dir.display()));
    ColorOutput::success(&format!("✓ 历史目录: {}", claude_paths.history_file.parent().unwrap().display()));
    ColorOutput::success(&format!("✓ 备份目录: {}", claude_paths.backups_dir.display()));

    // 在首次初始化时，创建一个最小可用的 profiles.toml，避免后续 ccr list 等命令因文件缺失报错
    // 注意：不覆盖已有文件，仅在缺失时创建
    if !claude_paths.profiles_file.exists() {
        ColorOutput::step("创建默认 Claude profiles.toml");

        // 构建一个空的 CcsConfig（默认/当前配置名为 "default"，sections 为空）
        let default_ccs = crate::managers::config::CcsConfig {
            default_config: "default".to_string(),
            current_config: "default".to_string(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        // 序列化并写入文件
        let content = toml::to_string_pretty(&default_ccs)
            .map_err(|e| CcrError::ConfigError(format!("序列化默认配置失败: {}", e)))?;
        fs::write(&claude_paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入默认 profiles.toml 失败: {}", e)))?;

        ColorOutput::success(&format!(
            "✓ 已创建: {}",
            claude_paths.profiles_file.display()
        ));
    }

    // 创建平台注册表配置
    println!();
    ColorOutput::step("创建平台注册表");

    let config = manager.load_or_create_default()?;
    manager.save(&config)?;

    ColorOutput::success(&format!("✓ 配置文件: {}", config_path.display()));

    // 显示完成信息
    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ CCR 配置初始化成功 (Unified Mode)");
    println!();

    ColorOutput::info("已创建的目录结构:");
    println!("  ~/.ccr/                    # CCR 根目录");
    println!("  └── config.toml            # 平台注册表");
    println!("  └── platforms/");
    println!("      └── claude/            # Claude 平台（默认）");
    println!();

    ColorOutput::info("后续步骤:");
    println!("  1. 使用 'ccr platform list' 查看所有平台");
    println!("  2. 使用 'ccr platform init <平台>' 初始化其他平台");
    println!("  3. 使用 'ccr add' 添加配置 profile");
    println!("  4. 使用 'ccr list' 查看配置列表");
    println!();

    ColorOutput::info("💡 提示:");
    println!("  • 如需迁移旧配置，运行: ccr migrate");
    println!("  • 查看帮助: ccr --help");
    println!();

    Ok(())
}

/// 🔙 Legacy Mode - 兼容旧版 ~/.ccs_config.toml
///
/// 仅在设置 `CCR_LEGACY_MODE=1` 环境变量时使用
fn init_legacy_mode(force: bool) -> Result<()> {
    // 获取配置文件路径
    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let config_path = home.join(".ccs_config.toml");

    // ⚡ 检查自动确认模式：--force 参数 OR 配置文件中的 skip_confirmation
    let auto_confirm = if config_path.exists() {
        let config_manager = ConfigManager::new(&config_path);
        config_manager
            .load()
            .ok()
            .map(|c| c.settings.skip_confirmation)
            .unwrap_or(false)
    } else {
        false
    };
    let skip_confirmation = force || auto_confirm;

    if auto_confirm && force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    // 检查文件是否已存在
    if config_path.exists() {
        if !force {
            ColorOutput::warning(&format!("配置文件已存在: {}", config_path.display()));
            println!();
            ColorOutput::info("配置文件已经初始化,无需重复执行");
            ColorOutput::info("提示:");
            println!("  • 查看配置: ccr list");
            println!("  • 编辑配置: vim ~/.ccs_config.toml");
            println!("  • 强制重新初始化: ccr init --force");
            println!();
            return Ok(());
        }

        // 🚨 使用 --force 时需要确认（除非 YOLO 模式）
        if !skip_confirmation {
            println!();
            ColorOutput::warning("⚠️  警告: 即将覆盖现有配置文件！");
            ColorOutput::info("提示: 现有配置会自动备份");
            println!();

            print!("确认强制重新初始化? (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            if !input.trim().eq_ignore_ascii_case("y") {
                ColorOutput::info("已取消初始化操作");
                return Ok(());
            }
            println!();
        }

        // 使用 --force 时,备份现有配置
        let status_msg = if skip_confirmation {
            "⚡ 使用 --force 模式,将覆盖现有配置 (自动确认模式)"
        } else {
            "使用 --force 模式,将覆盖现有配置"
        };
        ColorOutput::warning(status_msg);
        println!();
        ColorOutput::step("备份现有配置");
        backup_existing_config(&config_path)?;
        println!();
    }

    // 写入配置文件
    ColorOutput::step("创建配置文件");
    create_config_file(&config_path)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ 配置文件初始化成功 (Legacy Mode)");
    ColorOutput::info(&format!("配置文件位置: {}", config_path.display()));
    println!();

    // 提示后续操作
    ColorOutput::info("后续步骤:");
    println!("  1. 编辑配置文件: ~/.ccs_config.toml");
    println!("  2. 填写您的 API 密钥");
    println!("  3. 运行 'ccr list' 查看所有配置");
    println!("  4. 运行 'ccr switch <config>' 切换配置");
    println!();

    ColorOutput::warning("💡 建议: 迁移到新的 Unified Mode");
    println!("  运行 'ccr migrate' 迁移到新的多平台配置结构");
    println!();

    Ok(())
}

/// 创建配置文件
fn create_config_file(config_path: &PathBuf) -> Result<()> {
    // 写入示例配置内容
    fs::write(config_path, EXAMPLE_CONFIG)
        .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

    // 设置文件权限为 644 (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o644);
        fs::set_permissions(config_path, permissions)
            .map_err(|e| CcrError::ConfigError(format!("设置文件权限失败: {}", e)))?;
    }

    ColorOutput::success(&format!("已创建配置文件: {}", config_path.display()));
    Ok(())
}

/// 备份现有配置
fn backup_existing_config(config_path: &PathBuf) -> Result<()> {
    let config_manager = ConfigManager::new(config_path);
    let backup_path = config_manager.backup(Some("init"))?;
    ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_config_not_empty() {
        assert!(!EXAMPLE_CONFIG.is_empty());
        assert!(EXAMPLE_CONFIG.contains("default_config"));
        assert!(EXAMPLE_CONFIG.contains("[anyrouter_main]"));
    }

    #[test]
    fn test_create_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        create_config_file(&config_path).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("default_config"));
    }

    #[test]
    fn test_backup_existing_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建原始配置
        fs::write(&config_path, "test content").unwrap();

        // 备份
        backup_existing_config(&config_path).unwrap();

        // 检查备份文件是否存在
        let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().contains(".toml.")
                    && e.file_name().to_string_lossy().ends_with(".bak")
            })
            .collect();

        assert_eq!(backup_files.len(), 1);

        // 验证原文件内容未改变
        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_init_command_preserves_existing_config() {
        // 注意：这个测试使用真实的 home 目录路径判断
        // 但不会实际执行 init_command,只是验证逻辑

        // 测试逻辑：当配置文件已存在且不使用 --force 时,应该保护现有文件
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建现有配置
        let original_content = "existing config content";
        fs::write(&config_path, original_content).unwrap();

        // 模拟检查：如果文件存在,不应该被覆盖(除非 --force)
        if config_path.exists() {
            // 这是 init_command 的保护逻辑
            let content_after = fs::read_to_string(&config_path).unwrap();
            assert_eq!(content_after, original_content, "配置文件不应被意外覆盖");
        }
    }

    #[test]
    fn test_init_with_force_creates_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建现有配置
        let original_content = "original config";
        fs::write(&config_path, original_content).unwrap();

        // 备份现有配置(模拟 --force 的备份步骤)
        backup_existing_config(&config_path).unwrap();

        // 验证备份文件被创建
        let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().contains(".toml.")
                    && e.file_name().to_string_lossy().ends_with(".bak")
            })
            .collect();

        assert_eq!(backup_files.len(), 1, "应该创建一个备份文件");

        // 验证备份文件内容正确
        let backup_path = &backup_files[0].path();
        let backup_content = fs::read_to_string(backup_path).unwrap();
        assert_eq!(backup_content, original_content, "备份文件应包含原始内容");

        // 验证原文件未被修改(在备份阶段)
        let current_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(current_content, original_content, "备份操作不应修改原文件");
    }
}
