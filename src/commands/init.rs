// init 命令实现 - 初始化配置文件

use crate::error::{CcrError, Result};
use crate::logging::ColorOutput;
use std::fs;
use std::path::PathBuf;

/// 示例配置文件内容（嵌入到二进制中）
const EXAMPLE_CONFIG: &str = include_str!("../../.ccs_config.toml.example");

/// 初始化配置文件
pub fn init_command(force: bool) -> Result<()> {
    ColorOutput::title("CCR 配置初始化");
    println!();

    // 获取配置文件路径
    let home = dirs::home_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let config_path = home.join(".ccs_config.toml");

    // 检查文件是否已存在
    if config_path.exists() && !force {
        ColorOutput::warning(&format!("配置文件已存在: {}", config_path.display()));
        println!();
        
        if !ColorOutput::ask_confirmation("是否覆盖现有配置？", false) {
            ColorOutput::info("操作已取消");
            return Ok(());
        }
        
        // 备份现有配置
        println!();
        ColorOutput::step("备份现有配置");
        backup_existing_config(&config_path)?;
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
    fs::write(config_path, EXAMPLE_CONFIG).map_err(|e| {
        CcrError::ConfigError(format!("写入配置文件失败: {}", e))
    })?;

    // 设置文件权限为 644 (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o644);
        fs::set_permissions(config_path, permissions).map_err(|e| {
            CcrError::ConfigError(format!("设置文件权限失败: {}", e))
        })?;
    }

    ColorOutput::success(&format!("已创建配置文件: {}", config_path.display()));
    Ok(())
}

/// 备份现有配置
fn backup_existing_config(config_path: &PathBuf) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = config_path.with_extension(format!("toml.{}.bak", timestamp));

    fs::copy(config_path, &backup_path).map_err(|e| {
        CcrError::ConfigError(format!("备份配置文件失败: {}", e))
    })?;

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
        assert!(EXAMPLE_CONFIG.contains("[anyrouter]"));
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
                e.file_name()
                    .to_string_lossy()
                    .contains(".toml.")
                    && e.file_name().to_string_lossy().ends_with(".bak")
            })
            .collect();

        assert_eq!(backup_files.len(), 1);
    }
}

