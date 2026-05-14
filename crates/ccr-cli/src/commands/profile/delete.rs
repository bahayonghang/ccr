// 🗑️ delete 命令实现 - 删除配置
// ⚠️ 删除指定的配置节，支持安全检查

#![allow(clippy::unused_async)]

use crate::services::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;

/// 🗑️ 删除指定配置
///
/// 执行流程:
/// 1. ✅ 检查配置是否存在
/// 2. ⚠️ 检查是否为当前配置
/// 3. 🤔 确认删除（除非 --force 或自动确认模式）
/// 4. 💾 执行删除
/// 5. 📊 显示结果
///
/// 参数:
/// - config_name: 要删除的配置名称
/// - force: 跳过确认提示
pub async fn delete_command(config_name: &str, force: bool) -> Result<()> {
    ColorOutput::title(&format!("删除配置: {}", config_name));
    println!();

    // 使用 ConfigService
    let service = ConfigService::with_default()?;
    let config = service.load_config()?;

    // ⚡ 检查自动确认模式: --force 参数 或 配置文件中的 skip_confirmation
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
        println!();
    }

    // 1. 检查配置是否存在
    ColorOutput::step("步骤 1/3: 检查配置");
    if !config.sections.contains_key(config_name) {
        return Err(CcrError::ConfigSectionNotFound(config_name.to_string()));
    }
    ColorOutput::success(&format!("✓ 配置 '{}' 存在", config_name));
    println!();

    // 2. 检查是否为当前配置或默认配置
    ColorOutput::step("步骤 2/3: 安全检查");
    let is_current = config.current_config == config_name;
    let is_default = config.default_config == config_name;

    if is_current {
        ColorOutput::warning(&format!("⚠ 配置 '{}' 是当前激活的配置", config_name));
        println!();
        ColorOutput::info("删除当前配置后，您需要:");
        println!("  1. 运行 'ccr list' 查看其他配置");
        println!("  2. 运行 'ccr switch <name>' 切换到其他配置");
        println!();
    }

    if is_default {
        ColorOutput::warning(&format!("⚠ 配置 '{}' 是默认配置", config_name));
        println!();
        ColorOutput::info("删除后，请记得编辑 ~/.ccs_config.toml 设置新的 default_config");
        println!();
    }

    ColorOutput::success("✓ 安全检查完成");
    println!();

    // 显示配置信息
    if let Ok(section) = config.get_section(config_name) {
        ColorOutput::step("将要删除的配置信息");
        println!();
        println!("  配置名称: {}", config_name.red().bold());
        if let Some(desc) = &section.description {
            println!("  描述: {}", desc);
        }
        if let Some(url) = &section.base_url {
            println!("  Base URL: {}", url);
        }
        if let Some(provider) = &section.provider {
            println!("  提供商: {}", provider);
        }
        println!();
    }

    // 3. 确认删除
    if !skip_confirmation {
        ColorOutput::step("步骤 3/3: 确认删除");
        ColorOutput::warning("此操作不可恢复！");
        println!();

        let config_name = config_name.to_string();
        let confirmed = tokio::task::spawn_blocking(move || -> Result<bool> {
            Ok(ColorOutput::ask_confirmation(
                &format!("确认删除配置 '{}'?", config_name),
                false,
            ))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

        if !confirmed {
            println!();
            ColorOutput::info("已取消删除");
            return Ok(());
        }
        println!();
    } else {
        let mode_text = if config.settings.skip_confirmation {
            "⚡ 自动确认模式"
        } else {
            "--force 模式"
        };
        ColorOutput::step(&format!("步骤 3/3: 执行删除 ({})", mode_text));
        ColorOutput::warning("跳过确认，直接删除");
        println!();
    }

    ColorOutput::separator();
    println!();

    // 4. 执行删除
    service.delete_config(config_name)?;

    ColorOutput::success(&format!("✓ 配置 '{}' 已删除", config_name));
    println!();

    // 5. 后续提示
    if is_current {
        println!();
        ColorOutput::warning("重要提示: 您刚刚删除了当前配置");
        ColorOutput::info("后续操作:");
        println!("  1. 运行 'ccr list' 查看剩余配置");
        println!("  2. 运行 'ccr switch <name>' 切换到其他配置");
    } else {
        ColorOutput::info("后续操作:");
        println!("  • 运行 'ccr list' 查看剩余配置");
    }
    println!();

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::managers::config::{CcsConfig, ConfigManager, ConfigSection};
    use indexmap::IndexMap;

    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small-model".into()),
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_delete_command_logic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        let mut config = CcsConfig {
            default_config: "default".into(),
            current_config: "default".into(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        config.sections.insert("test".into(), create_test_section());
        config
            .sections
            .insert("default".into(), create_test_section());

        let manager = ConfigManager::new(&config_path);
        manager.save(&config).unwrap();

        // 验证配置存在
        let loaded = manager.load().unwrap();
        assert!(loaded.sections.contains_key("test"));

        // 删除配置
        let mut updated = loaded;
        updated.sections.shift_remove("test");
        manager.save(&updated).unwrap();

        // 验证已删除
        let final_config = manager.load().unwrap();
        assert!(!final_config.sections.contains_key("test"));
    }
}
