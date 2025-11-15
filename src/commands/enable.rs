// ✅ enable 命令实现 - 启用配置
// 🔓 将指定配置标记为启用状态，使其可以被正常使用

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::config_service::ConfigService;

/// ✅ 启用指定配置
///
/// 将配置的 `enabled` 字段设置为 `true`，使其可以被正常使用。
/// 启用的配置在列表中显示为正常状态，可以被切换使用。
///
/// # 参数
///
/// * `config_name` - 要启用的配置名称
///
/// # 返回
///
/// * `Ok(())` - 成功启用配置
/// * `Err(CcrError::ConfigNotFound)` - 配置不存在
/// * `Err(CcrError::ConfigError)` - 配置文件操作失败
///
/// # 示例
///
/// ```bash
/// ccr enable anthropic
/// ```
///
/// # 输出示例
///
/// ```text
/// ╭─────────────────────────────────────╮
/// │           启用配置                  │
/// ╰─────────────────────────────────────╯
///
/// ✓ 配置 'anthropic' 已启用
///
/// 💡 提示:
///   • 使用 'ccr list' 查看所有配置
///   • 使用 'ccr switch anthropic' 切换到该配置
/// ```
pub fn enable_command(config_name: &str) -> Result<()> {
    ColorOutput::title("启用配置");
    println!();

    // 创建配置服务
    let config_service = ConfigService::with_default()?;

    // 启用配置
    config_service.enable_config(config_name)?;

    println!();
    ColorOutput::success(&format!("✓ 配置 '{}' 已启用", config_name));
    println!();

    // 显示后续操作提示
    ColorOutput::info("💡 提示:");
    println!("  • 使用 'ccr list' 查看所有配置");
    println!("  • 使用 'ccr switch {}' 切换到该配置", config_name);
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::config::{CcsConfig, ConfigManager, ConfigSection, GlobalSettings};
    use indexmap::IndexMap;
    use tempfile::tempdir;

    fn create_test_config_with_disabled() -> CcsConfig {
        let mut sections = IndexMap::new();
        sections.insert(
            "test1".to_string(),
            ConfigSection {
                description: Some("Test 1".to_string()),
                base_url: Some("https://api.test1.com".to_string()),
                auth_token: Some("token1".to_string()),
                model: Some("model1".to_string()),
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
                usage_count: Some(0),
                enabled: Some(false), // 初始为禁用状态
            },
        );

        CcsConfig {
            default_config: "test1".to_string(),
            current_config: "test1".to_string(),
            settings: GlobalSettings::default(),
            sections,
        }
    }

    #[test]
    fn test_enable_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        let config_manager = ConfigManager::new(&config_path);
        let config = create_test_config_with_disabled();
        config_manager.save(&config).unwrap();

        // 临时设置环境变量
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        // 执行启用命令
        let result = enable_command("test1");
        assert!(result.is_ok());

        // 验证配置已启用
        let updated_config = config_manager.load().unwrap();
        let section = updated_config.get_section("test1").unwrap();
        assert!(section.is_enabled());
    }

    #[test]
    fn test_enable_nonexistent_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建空配置
        let config_manager = ConfigManager::new(&config_path);
        let config = create_test_config_with_disabled();
        config_manager.save(&config).unwrap();

        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        // 尝试启用不存在的配置
        let result = enable_command("nonexistent");
        assert!(result.is_err());
    }
}
