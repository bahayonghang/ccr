// 🔄 migrate 命令实现 - 配置迁移
// 📦 将 Legacy 模式配置迁移到 Unified 模式

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::{
    ConfigManager, MigrationStatus, PlatformConfigEntry, PlatformConfigManager, UnifiedConfig,
};
use crate::models::{Platform, PlatformPaths, ProfileConfig};
use colored::Colorize;
use std::io::{self, Write};

/// 🔄 迁移配置到统一模式
pub fn migrate_command(dry_run: bool, platform_filter: Option<&str>) -> Result<()> {
    if dry_run {
        ColorOutput::title("配置迁移检查 (dry-run 模式)");
    } else {
        ColorOutput::title("配置迁移");
    }

    // 1. 检查迁移状态
    let config_manager = ConfigManager::with_default()?;
    let migration_status = config_manager.get_migration_status();

    display_migration_status(&migration_status)?;

    // 2. 判断是否需要迁移
    if !migration_status.should_migrate {
        println!();
        if migration_status.is_unified_mode {
            ColorOutput::info("已经在统一模式下运行，无需迁移。");
        } else if !migration_status.legacy_config_exists {
            ColorOutput::info("未找到 Legacy 配置文件，无需迁移。");
        } else {
            ColorOutput::info("配置节较少（< 2 个），建议继续使用 Legacy 模式。");
            println!();
            ColorOutput::info("如果仍要迁移，可以使用 --force 参数。");
        }
        return Ok(());
    }

    // 3. Dry-run 模式只显示迁移计划
    if dry_run {
        println!();
        ColorOutput::success("✓ 迁移检查通过");
        println!();
        display_migration_plan(&migration_status, platform_filter)?;
        println!();
        ColorOutput::info("提示: 移除 --check 参数以执行实际迁移");
        return Ok(());
    }

    // 4. 确认迁移
    println!();
    display_migration_plan(&migration_status, platform_filter)?;
    println!();

    if !confirm_migration()? {
        ColorOutput::info("迁移已取消。");
        return Ok(());
    }

    // 5. 执行迁移
    println!();
    ColorOutput::info("开始迁移...");

    execute_migration(&config_manager, platform_filter)?;

    println!();
    ColorOutput::success("✓ 迁移完成！");
    println!();
    display_post_migration_instructions();

    Ok(())
}

/// 显示迁移状态
fn display_migration_status(status: &MigrationStatus) -> Result<()> {
    println!();
    ColorOutput::info("当前状态:");

    println!(
        "  配置模式: {}",
        if status.is_unified_mode {
            "Unified (统一模式)".green()
        } else {
            "Legacy (传统模式)".yellow()
        }
    );

    println!(
        "  Legacy 配置: {}",
        if status.legacy_config_exists {
            format!("存在 ({} 个配置节)", status.legacy_section_count).yellow()
        } else {
            "不存在".bright_black()
        }
    );

    if let Some(unified_path) = &status.unified_config_path {
        println!(
            "  Unified 配置: {}",
            format!("存在 ({})", unified_path.display()).green()
        );
    } else {
        println!("  Unified 配置: {}", "不存在".bright_black());
    }

    println!(
        "  建议迁移: {}",
        if status.should_migrate {
            "是 ✓".green()
        } else {
            "否".bright_black()
        }
    );

    Ok(())
}

/// 显示迁移计划
fn display_migration_plan(status: &MigrationStatus, platform_filter: Option<&str>) -> Result<()> {
    ColorOutput::info("迁移计划:");

    let config_manager = ConfigManager::with_default()?;
    let legacy_config = config_manager.load()?;

    println!();
    println!("  将迁移以下配置节:");

    let mut count = 0;
    for (name, section) in &legacy_config.sections {
        if platform_filter.is_some() && platform_filter != Some("claude") {
            continue;
        }

        count += 1;
        println!(
            "    {} {} - {}",
            "•".bright_cyan(),
            name.bright_yellow(),
            section.display_description()
        );
    }

    if count == 0 {
        println!("    {}", "(无需迁移的配置)".bright_black());
    }

    println!();
    println!("  迁移后的结构:");
    println!("    ~/.ccr/");
    println!("      ├── config.toml          (统一配置注册表)");
    println!("      └── platforms/");
    println!("          └── claude/");
    println!("              ├── profiles.toml (配置 profiles)");
    println!("              ├── settings.json (平台设置)");
    println!("              ├── history.json  (操作历史)");
    println!("              └── backups/      (备份目录)");

    println!();
    println!("  Legacy 配置文件: {}", status.legacy_config_path.display());
    println!("    {} 迁移后不会删除，将保留作为备份", "注意:".yellow());

    Ok(())
}

/// 确认迁移
fn confirm_migration() -> Result<bool> {
    print!(
        "{}",
        "确认执行迁移? 这将创建新的统一配置结构。(y/N): "
            .bright_yellow()
            .bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

/// 执行迁移
fn execute_migration(config_manager: &ConfigManager, platform_filter: Option<&str>) -> Result<()> {
    // 1. 备份 Legacy 配置
    ColorOutput::info("1/5 备份 Legacy 配置...");
    let backup_path = config_manager.backup(Some("pre_migration"))?;
    ColorOutput::success(&format!("  ✓ 备份已创建: {}", backup_path.display()));

    // 2. 加载 Legacy 配置
    ColorOutput::info("2/5 加载 Legacy 配置...");
    let legacy_config = config_manager.load()?;
    ColorOutput::success(&format!(
        "  ✓ 已加载 {} 个配置节",
        legacy_config.sections.len()
    ));

    // 3. 创建统一配置结构
    ColorOutput::info("3/5 创建统一配置结构...");

    let platform_manager = PlatformConfigManager::with_default()?;
    let mut unified_config = UnifiedConfig::default();

    // 注册 Claude 平台
    let claude_registry = PlatformConfigEntry {
        description: Some("Claude Code AI Assistant".to_string()),
        current_profile: Some(legacy_config.current_config.clone()),
        ..Default::default()
    };

    unified_config.register_platform("claude".to_string(), claude_registry)?;
    unified_config.current_platform = "claude".to_string();
    unified_config.default_platform = "claude".to_string();

    // 保存统一配置
    platform_manager.save(&unified_config)?;
    ColorOutput::success(&format!(
        "  ✓ 统一配置已创建: {}",
        platform_manager.config_path().display()
    ));

    // 4. 迁移 Claude profiles
    ColorOutput::info("4/5 迁移配置 profiles...");

    if platform_filter.is_none() || platform_filter == Some("claude") {
        migrate_claude_profiles(&legacy_config)?;
    }

    // 5. 创建目录结构
    ColorOutput::info("5/5 创建平台目录结构...");
    create_platform_directories()?;

    Ok(())
}

/// 迁移 Claude profiles
fn migrate_claude_profiles(legacy_config: &crate::managers::CcsConfig) -> Result<()> {
    use crate::platforms::create_platform;

    let claude_platform = create_platform(Platform::Claude)?;

    let mut migrated_count = 0;
    for (name, section) in &legacy_config.sections {
        let profile = ProfileConfig {
            description: section.description.clone(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            provider: section.provider.clone(),
            provider_type: section
                .provider_type
                .as_ref()
                .map(|t| t.to_string_value().to_string()),
            account: section.account.clone(),
            tags: section.tags.clone(),
            usage_count: section.usage_count,
            enabled: section.enabled,
            platform_data: indexmap::IndexMap::new(),
        };

        claude_platform.validate_profile(&profile)?;
        claude_platform.save_profile(name, &profile)?;

        migrated_count += 1;
    }

    ColorOutput::success(&format!("  ✓ 已迁移 {} 个 Claude profiles", migrated_count));

    let current_profile = &legacy_config.current_config;
    claude_platform.apply_profile(current_profile)?;
    ColorOutput::success(&format!(
        "  ✓ 当前 profile 已应用: {}",
        current_profile.bright_cyan()
    ));

    Ok(())
}

/// 创建平台目录结构
fn create_platform_directories() -> Result<()> {
    let paths = PlatformPaths::new(Platform::Claude)?;

    std::fs::create_dir_all(&paths.platform_dir)
        .map_err(|e| CcrError::ConfigError(format!("创建平台目录失败: {}", e)))?;

    std::fs::create_dir_all(&paths.backups_dir)
        .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败: {}", e)))?;

    ColorOutput::success("  ✓ Claude 平台目录已创建");
    ColorOutput::success(&format!("    平台目录: {}", paths.platform_dir.display()));
    ColorOutput::success(&format!("    备份目录: {}", paths.backups_dir.display()));

    Ok(())
}

/// 显示迁移后说明
fn display_post_migration_instructions() {
    ColorOutput::info("迁移后说明:");
    println!();
    println!("  1. 原 Legacy 配置文件已保留在:");
    println!("     ~/.ccs_config.toml");
    println!();
    println!("  2. 新的统一配置位于:");
    println!("     ~/.ccr/config.toml");
    println!();
    println!("  3. Claude profiles 已迁移到:");
    println!("     ~/.ccr/platforms/claude/profiles.toml");
    println!();
    println!("  4. 您现在可以:");
    println!("     • 使用 'ccr list' 查看所有配置");
    println!("     • 使用 'ccr switch <profile>' 切换配置");
    println!("     • 使用 'ccr platform list' 查看所有平台");
    println!();
    println!(
        "  {}",
        "注意: 如需回退到 Legacy 模式，只需删除 ~/.ccr/ 目录".yellow()
    );
}

/// 🔍 检查迁移状态（不执行迁移）
pub fn migrate_check_command() -> Result<()> {
    ColorOutput::title("迁移状态检查");

    let config_manager = ConfigManager::with_default()?;
    let migration_status = config_manager.get_migration_status();

    display_migration_status(&migration_status)?;

    println!();

    if migration_status.should_migrate {
        ColorOutput::success("✓ 建议进行迁移");
        println!();
        ColorOutput::info("执行迁移:");
        println!("  ccr migrate          # 执行迁移");
        println!("  ccr migrate --check  # 查看迁移计划（不实际执行）");
    } else if migration_status.is_unified_mode {
        ColorOutput::info("✓ 已在统一模式下运行");
    } else {
        ColorOutput::info("当前使用 Legacy 模式,无需迁移");
    }

    Ok(())
}
