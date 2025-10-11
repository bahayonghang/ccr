// 🔄 switch 命令实现 - 切换配置
// 💎 这是 CCR 最核心的命令，负责完整的配置切换流程
//
// 执行流程（5 个步骤）:
// 1. 📖 读取并验证目标配置
// 2. 💾 备份当前 settings.json
// 3. ✏️ 更新 Claude Code 设置
// 4. 📝 更新 ccs_config 当前配置标记
// 5. 📚 记录操作历史（带环境变量变化）

use crate::config::ConfigManager;
use crate::error::{CcrError, Result};
use crate::history::{HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType};
use crate::logging::ColorOutput;
use crate::settings::SettingsManager;

/// 🔄 切换到指定配置
/// 
/// 这是一个原子性操作，确保配置切换的完整性和可追溯性
pub fn switch_command(config_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换配置: {}", config_name));
    println!();

    // 📖 步骤 1: 读取并校验目标配置
    ColorOutput::step("步骤 1/5: 读取配置文件");
    let config_manager = ConfigManager::default()?;
    let mut config = config_manager.load()?;

    let target_section = config
        .get_section(config_name)
        .map_err(|_| {
            ColorOutput::error(&format!("配置 '{}' 不存在", config_name));
            CcrError::ConfigSectionNotFound(config_name.to_string())
        })?
        .clone();

    // 验证目标配置
    target_section.validate().map_err(|e| {
        ColorOutput::error(&format!("目标配置验证失败: {}", e));
        e
    })?;

    ColorOutput::success(&format!("✅ 目标配置 '{}' 验证通过", config_name));
    println!();

    // 💾 步骤 2: 备份当前设置
    ColorOutput::step("步骤 2/5: 备份当前设置");
    let settings_manager = SettingsManager::default()?;

    let backup_path = if settings_manager.settings_path().exists() {
        let path = settings_manager.backup(Some(config_name))?;
        ColorOutput::success(&format!("✅ 设置已备份: {}", path.display()));
        Some(path.display().to_string())
    } else {
        ColorOutput::info("📝 设置文件不存在，跳过备份（这可能是首次使用）");
        None
    };
    println!();

    // ✏️ 步骤 3: 更新 settings.json（清空旧 ANTHROPIC_* 后写入新值）
    ColorOutput::step("步骤 3/5: 更新 Claude Code 设置");

    // 📊 记录旧的环境变量状态（用于历史对比）
    let old_settings = settings_manager.load().ok();
    let old_env = old_settings
        .as_ref()
        .map(|s| s.anthropic_env_status())
        .unwrap_or_default();

    // 🔄 应用新配置
    let mut new_settings = old_settings.unwrap_or_default();
    new_settings.update_from_config(&target_section);

    // 💾 原子性保存
    settings_manager.save_atomic(&new_settings)?;
    ColorOutput::success("✅ Claude Code 设置已更新");
    println!();

    // 📝 步骤 4: 更新 ccs_config 的 current_config 标记
    ColorOutput::step("步骤 4/5: 更新配置文件");
    let old_config = config.current_config.clone();
    config.set_current(config_name)?;
    config_manager.save(&config)?;
    ColorOutput::success(&format!("✅ 当前配置已设置为: {}", config_name));
    println!();

    // 📚 步骤 5: 记录历史（包含环境变量变化的掩码记录）
    ColorOutput::step("步骤 5/5: 记录操作历史");
    let history_manager = HistoryManager::default()?;

    let mut history_entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: if old_config.is_empty() {
                None
            } else {
                Some(old_config.clone())
            },
            to_config: Some(config_name.to_string()),
            backup_path,
            extra: None,
        },
        OperationResult::Success,
    );

    // 记录环境变量变化
    let new_env = new_settings.anthropic_env_status();
    for (var_name, new_value) in new_env {
        let old_value = old_env.get(&var_name).and_then(|v| v.clone());
        history_entry.add_env_change(var_name, old_value, new_value);
    }

    history_manager.add(history_entry)?;
    ColorOutput::success("✅ 操作历史已记录");
    println!();

    // 📋 输出新配置细节与校验结果
    ColorOutput::separator();
    println!();
    ColorOutput::title("🎉 配置切换成功");
    println!();
    ColorOutput::key_value("📝 配置名称", config_name, 2);
    ColorOutput::key_value("📄 描述", &target_section.display_description(), 2);
    if let Some(base_url) = &target_section.base_url {
        ColorOutput::key_value("Base URL", base_url, 2);
    }
    if let Some(auth_token) = &target_section.auth_token {
        ColorOutput::key_value_sensitive("Auth Token", auth_token, 2);
    }
    if let Some(model) = &target_section.model {
        ColorOutput::key_value("Model", model, 2);
    }
    if let Some(small_model) = &target_section.small_fast_model {
        ColorOutput::key_value("Small Fast Model", small_model, 2);
    }
    println!();

    // 最终验证
    match new_settings.validate() {
        Ok(_) => ColorOutput::success("✓ 配置已生效，Claude Code 可以使用新的 API 配置"),
        Err(e) => ColorOutput::warning(&format!("⚠ 配置可能不完整: {}", e)),
    }

    println!();
    ColorOutput::info("提示: 重启 Claude Code 以确保配置完全生效");

    Ok(())
}
