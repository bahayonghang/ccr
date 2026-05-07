// ❌ disable 命令实现 - 禁用配置
// 🔒 将指定配置标记为禁用状态，暂时不可使用

#![allow(clippy::unused_async)]

use crate::services::config_service::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;

/// ❌ 禁用指定配置
///
/// 将配置的 `enabled` 字段设置为 `false`，使其暂时不可用。
/// 禁用的配置在列表中显示为灰色/禁用状态，不能被切换使用。
///
/// # 参数
///
/// * `config_name` - 要禁用的配置名称
/// * `force` - 是否强制禁用（即使是当前正在使用的配置）
///
/// # 返回
///
/// * `Ok(())` - 成功禁用配置
/// * `Err(CcrError::ConfigNotFound)` - 配置不存在
/// * `Err(CcrError::ConfigError)` - 配置文件操作失败
pub async fn disable_command(config_name: &str, force: bool) -> Result<()> {
    ColorOutput::title("禁用配置");
    println!();

    // 创建配置服务
    let config_service = ConfigService::with_default()?;

    // 检查是否是当前配置
    let current = config_service.get_current()?;
    let is_current = current.name == config_name;

    if is_current && !force {
        println!();
        ColorOutput::warning(&format!("⚠️  警告: '{}' 是当前正在使用的配置", config_name));
        println!();

        // 询问确认
        let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
            use std::io::{self, Write};
            print!("{}", "确认禁用当前配置? (y/N): ".bright_yellow().bold());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

        if !confirmed {
            ColorOutput::info("已取消禁用操作");
            return Ok(());
        }
        println!();
    }

    // 禁用配置
    config_service.disable_config(config_name)?;

    println!();
    ColorOutput::success(&format!("✓ 配置 '{}' 已禁用", config_name));
    println!();

    // 显示后续操作提示
    ColorOutput::info("💡 提示:");
    println!("  • 禁用的配置不会被删除，只是暂时不可用");
    println!("  • 使用 'ccr enable {}' 重新启用", config_name);
    if is_current {
        println!("  • 使用 'ccr switch <other>' 切换到其他配置");
    }
    println!();

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::managers::config::{CcsConfig, ConfigManager, ConfigSection, GlobalSettings};
    use crate::services::ConfigService;
    use indexmap::IndexMap;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                match self.previous.take() {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    fn create_test_config_with_enabled() -> CcsConfig {
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
                enabled: Some(true), // 初始为启用状态
                other: IndexMap::new(),
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
    fn test_disable_config() {
        let _env_guard = crate::test_support::env_lock();
        let temp_dir = tempdir().unwrap();
        let _lock_dir = EnvVarGuard::set_path("CCR_LOCK_DIR", &temp_dir.path().join("locks"));
        let config_path = temp_dir.path().join(".ccs_config.toml");

        // 创建测试配置
        {
            let config_manager = ConfigManager::new(&config_path);
            let config = create_test_config_with_enabled();
            config_manager.save(&config).unwrap();

            // 验证初始状态
            let initial_config = config_manager.load().unwrap();
            let initial_section = initial_config.get_section("test1").unwrap();
            assert!(initial_section.is_enabled(), "初始状态应该是启用的");
        }

        // 直接使用服务层测试
        {
            let config_manager = Arc::new(ConfigManager::new(&config_path));
            let service = ConfigService::new(config_manager);
            service.disable_config("test1").unwrap();
        }

        // 重新创建 config_manager 并验证配置已禁用
        let fresh_config_manager = ConfigManager::new(&config_path);
        let updated_config = fresh_config_manager.load().unwrap();
        let section = updated_config.get_section("test1").unwrap();
        assert!(!section.is_enabled(), "禁用后应该是禁用状态");
    }
}
