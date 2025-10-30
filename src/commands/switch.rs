// 🔄 switch 命令实现 - 切换配置
// 💎 这是 CCR 最核心的命令,负责完整的配置切换流程
// 🔄 支持平台感知: 在 unified 模式下从平台配置加载
//
// 执行流程(5 个步骤):
// 1. 📖 读取并验证目标配置 (Legacy 或 Unified 模式)
// 2. 💾 备份当前 settings.json
// 3. ✏️ 更新 Claude Code 设置
// 4. 📝 更新配置文件当前配置标记
// 5. 📚 记录操作历史(带环境变量变化)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::managers::config::{ConfigManager, ConfigSection};
use crate::managers::history::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
};
use crate::managers::settings::SettingsManager;
use crate::models::Platform;
use crate::platforms::create_platform;
use crate::utils::Validatable;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use std::str::FromStr;

/// 🔄 切换到指定配置
///
/// 这是一个原子性操作,确保配置切换的完整性和可追溯性
/// 支持 Legacy 和 Unified 两种模式
pub fn switch_command(config_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换配置: {}", config_name));
    println!();

    // 🔍 检测配置模式
    let unified_config = PlatformConfigManager::with_default()
        .ok()
        .and_then(|mgr| mgr.load().ok());
    let is_unified_mode = unified_config.is_some();

    // 📖 步骤 1: 读取并校验目标配置 (根据模式选择来源)
    ColorOutput::step("步骤 1/5: 读取配置文件");

    let target_section: ConfigSection = if is_unified_mode {
        // Unified 模式: 从平台配置加载
        let uc = unified_config
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Unified 配置未找到".to_string()))?;
        let platform_name = &uc.current_platform;
        let platform = Platform::from_str(platform_name)?;

        ColorOutput::info(&format!(
            "使用 {} 模式 (平台: {})",
            "Unified".bright_cyan(),
            platform_name.bright_yellow()
        ));

        // 从平台配置加载 profile
        let platform_config = create_platform(platform).map_err(|e| {
            CcrError::ConfigError(format!("创建平台 {} 失败: {}", platform_name, e))
        })?;

        // 加载所有 profiles
        let profiles = platform_config.load_profiles()?;

        // 查找目标 profile
        let profile = profiles.get(config_name).ok_or_else(|| {
            ColorOutput::error(&format!(
                "配置 '{}' 在平台 {} 中不存在",
                config_name, platform_name
            ));
            CcrError::ConfigSectionNotFound(config_name.to_string())
        })?;

        // 转换 ProfileConfig 为 ConfigSection
        ConfigSection {
            description: profile.description.clone(),
            base_url: profile.base_url.clone(),
            auth_token: profile.auth_token.clone(),
            model: profile.model.clone(),
            small_fast_model: profile.small_fast_model.clone(),
            provider: profile.provider.clone(),
            provider_type: profile.provider_type.as_ref().and_then(|pt| {
                // 尝试从字符串转换回 ProviderType
                use crate::managers::config::ProviderType;
                match pt.as_str() {
                    "official_relay" => Some(ProviderType::OfficialRelay),
                    "third_party_model" => Some(ProviderType::ThirdPartyModel),
                    _ => None,
                }
            }),
            account: profile.account.clone(),
            tags: profile.tags.clone(),
        }
    } else {
        // Legacy 模式: 从 ccs_config 加载
        ColorOutput::info(&format!("使用 {} 模式", "Legacy".bright_white()));

        let config_manager = ConfigManager::with_default()?;
        let config = config_manager.load()?;

        config
            .get_section(config_name)
            .map_err(|_| {
                ColorOutput::error(&format!("配置 '{}' 不存在", config_name));
                CcrError::ConfigSectionNotFound(config_name.to_string())
            })?
            .clone()
    };

    // 验证目标配置
    target_section.validate().map_err(|e| {
        ColorOutput::error(&format!("目标配置验证失败: {}", e));
        e
    })?;

    ColorOutput::success(&format!("✅ 目标配置 '{}' 验证通过", config_name));
    println!();

    // 💾 步骤 2: 备份当前设置
    ColorOutput::step("步骤 2/5: 备份当前设置");
    let settings_manager = SettingsManager::with_default()?;

    let backup_path = if settings_manager.settings_path().exists() {
        let path = settings_manager.backup(Some(config_name))?;
        ColorOutput::success(&format!("✅ 设置已备份: {}", path.display()));
        Some(path.display().to_string())
    } else {
        ColorOutput::info("📝 设置文件不存在,跳过备份(这可能是首次使用)");
        None
    };
    println!();

    // ✏️ 步骤 3: 更新 settings.json(清空旧 ANTHROPIC_* 后写入新值)
    ColorOutput::step("步骤 3/5: 更新 Claude Code 设置");

    // 📊 记录旧的环境变量状态(用于历史对比)
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

    // 📝 步骤 4: 更新配置文件 (根据模式选择目标)
    ColorOutput::step("步骤 4/5: 更新配置文件");

    let old_config_name: String = if is_unified_mode {
        // Unified 模式: 更新平台配置的 current_profile
        let uc = unified_config
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Unified 配置未找到".to_string()))?;
        let platform_name = &uc.current_platform;
        let platform = Platform::from_str(platform_name)?;

        let platform_config = create_platform(platform).map_err(|e| {
            CcrError::ConfigError(format!("创建平台 {} 失败: {}", platform_name, e))
        })?;

        let old_current = platform_config.get_current_profile()?.unwrap_or_default();

        // 应用 profile (这会设置当前profile并保存)
        platform_config.apply_profile(config_name)?;

        ColorOutput::success(&format!(
            "✅ 平台 {} 的当前配置已设置为: {}",
            platform_name, config_name
        ));

        old_current
    } else {
        // Legacy 模式: 更新 ccs_config 的 current_config
        let config_manager = ConfigManager::with_default()?;
        let mut config = config_manager.load()?;

        let old_current = config.current_config.clone();
        config.set_current(config_name)?;
        config_manager.save(&config)?;

        ColorOutput::success(&format!("✅ 当前配置已设置为: {}", config_name));

        old_current
    };

    println!();

    // 📚 步骤 5: 记录历史(包含环境变量变化的掩码记录)
    ColorOutput::step("步骤 5/5: 记录操作历史");
    let history_manager = HistoryManager::with_default()?;

    let mut history_entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: if old_config_name.is_empty() {
                None
            } else {
                Some(old_config_name.clone())
            },
            to_config: Some(config_name.to_string()),
            backup_path,
            extra: None,
        },
        OperationResult::Success,
    );

    // 记录环境变量变化
    let new_env = new_settings.anthropic_env_status();
    let new_env_display = new_env.clone(); // 克隆一份用于后续展示

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

    // === 配置详情表格 ===
    let mut config_table = Table::new();
    config_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("属性")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("新配置")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 配置名称
    config_table.add_row(vec![
        Cell::new("配置名称").fg(TableColor::Yellow),
        Cell::new(config_name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);

    // 描述
    config_table.add_row(vec![
        Cell::new("描述"),
        Cell::new(target_section.display_description()),
    ]);

    // 提供商类型（如果有）
    if let Some(provider_type) = target_section.provider_type.as_ref() {
        let type_display = match provider_type.to_string_value() {
            "official_relay" => "🔄 官方中转",
            "third_party_model" => "🤖 第三方模型",
            _ => provider_type.to_string_value(),
        };
        config_table.add_row(vec![
            Cell::new("提供商类型"),
            Cell::new(type_display).fg(TableColor::Cyan),
        ]);
    }

    // 提供商（如果有）
    if let Some(provider) = &target_section.provider {
        config_table.add_row(vec![
            Cell::new("提供商"),
            Cell::new(provider).fg(TableColor::Cyan),
        ]);
    }

    // Base URL
    if let Some(base_url) = &target_section.base_url {
        config_table.add_row(vec![
            Cell::new("Base URL")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    // Auth Token (脱敏)
    if let Some(auth_token) = &target_section.auth_token {
        config_table.add_row(vec![
            Cell::new("Auth Token")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(ColorOutput::mask_sensitive(auth_token)).fg(TableColor::DarkGrey),
        ]);
    }

    // Model
    if let Some(model) = &target_section.model {
        config_table.add_row(vec![
            Cell::new("主模型"),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    // Small Fast Model
    if let Some(small_model) = &target_section.small_fast_model {
        config_table.add_row(vec![
            Cell::new("快速小模型"),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    // 账号（如果有）
    if let Some(account) = &target_section.account {
        config_table.add_row(vec![
            Cell::new("账号标识"),
            Cell::new(format!("👤 {}", account)).fg(TableColor::Yellow),
        ]);
    }

    // 标签（如果有）
    if let Some(tags) = &target_section.tags
        && !tags.is_empty()
    {
        config_table.add_row(vec![
            Cell::new("标签"),
            Cell::new(format!("🏷️  {}", tags.join(", "))).fg(TableColor::Magenta),
        ]);
    }

    println!("{}", config_table);
    println!();

    // === 环境变量变化对比表格 ===
    ColorOutput::step("🔄 环境变量变化");
    println!();

    let mut env_changes_table = Table::new();
    env_changes_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("环境变量")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("变化")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 显示环境变量变化
    let env_vars = [
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_SMALL_FAST_MODEL",
    ];

    for var_name in env_vars {
        let old_val = old_env.get(var_name).and_then(|v| v.as_ref());
        let new_val = new_env_display.get(var_name).and_then(|v| v.as_ref());

        let is_sensitive = var_name.contains("TOKEN") || var_name.contains("KEY");

        let change_display = match (old_val, new_val) {
            (None, None) => "-".to_string(),
            (None, Some(new)) => {
                let new_display = if is_sensitive {
                    ColorOutput::mask_sensitive(new)
                } else if new.len() > 35 {
                    format!("{}...", &new[..32])
                } else {
                    new.to_string()
                };
                format!("➕ 新增: {}", new_display)
            }
            (Some(old), None) => {
                let old_display = if is_sensitive {
                    ColorOutput::mask_sensitive(old)
                } else if old.len() > 35 {
                    format!("{}...", &old[..32])
                } else {
                    old.to_string()
                };
                format!("➖ 删除: {}", old_display)
            }
            (Some(old), Some(new)) => {
                if old == new {
                    "○ 未变化".to_string()
                } else {
                    let old_display = if is_sensitive {
                        ColorOutput::mask_sensitive(old)
                    } else if old.len() > 20 {
                        format!("{}...", &old[..17])
                    } else {
                        old.to_string()
                    };
                    let new_display = if is_sensitive {
                        ColorOutput::mask_sensitive(new)
                    } else if new.len() > 20 {
                        format!("{}...", &new[..17])
                    } else {
                        new.to_string()
                    };
                    format!("🔄 {} → {}", old_display, new_display)
                }
            }
        };

        let change_cell = if change_display.starts_with("➕") {
            Cell::new(change_display).fg(TableColor::Green)
        } else if change_display.starts_with("➖") {
            Cell::new(change_display).fg(TableColor::Red)
        } else if change_display.starts_with("🔄") {
            Cell::new(change_display).fg(TableColor::Yellow)
        } else {
            Cell::new(change_display).fg(TableColor::DarkGrey)
        };

        env_changes_table.add_row(vec![Cell::new(var_name), change_cell]);
    }

    println!("{}", env_changes_table);
    println!();

    // 最终验证
    match new_settings.validate() {
        Ok(_) => {
            ColorOutput::success("✓ 配置已生效,Claude Code 可以使用新的 API 配置");
        }
        Err(e) => {
            ColorOutput::warning(&format!("⚠ 配置可能不完整: {}", e));
        }
    }

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::info(&format!(
        "💡 提示: 从 {} {} 切换到 {} {}",
        old_config_name.dimmed(),
        "→".dimmed(),
        config_name.bright_green().bold(),
        "✓".bright_green()
    ));

    ColorOutput::info("🔄 建议重启 Claude Code 以确保配置完全生效");

    Ok(())
}
