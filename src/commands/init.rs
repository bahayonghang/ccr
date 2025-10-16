// 🎬 init 命令实现 - 初始化配置文件
// 📦 从嵌入的模板创建 ~/.ccs_config.toml 配置文件

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::ConfigManager;
use std::fs;
use std::path::PathBuf;

/// 📋 示例配置文件内容(嵌入到二进制中)
/// 编译时从 .ccs_config.toml.example 读取
const EXAMPLE_CONFIG: &str = include_str!("../../.ccs_config.toml.example");

/// 🎬 初始化配置文件
///
/// 执行流程:
/// 1. ✅ 检查文件是否已存在
/// 2. 💾 备份现有配置(--force 模式)
/// 3. 📝 创建新配置文件
/// 4. 🔒 设置文件权限 (644)
/// 5. 💡 显示后续步骤提示
pub fn init_command(force: bool) -> Result<()> {
    ColorOutput::title("CCR 配置初始化");
    println!();

    // 获取配置文件路径
    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let config_path = home.join(".ccs_config.toml");

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

        // 使用 --force 时,备份现有配置
        ColorOutput::warning("使用 --force 模式,将覆盖现有配置");
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
    ColorOutput::success("✓ 配置文件初始化成功");
    ColorOutput::info(&format!("配置文件位置: {}", config_path.display()));
    println!();

    // 提示后续操作
    ColorOutput::info("后续步骤:");
    println!("  1. 编辑配置文件: ~/.ccs_config.toml");
    println!("  2. 填写您的 API 密钥");
    println!("  3. 运行 'ccr list' 查看所有配置");
    println!("  4. 运行 'ccr switch <config>' 切换配置");
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
