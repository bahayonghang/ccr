// 📥 import 命令实现 - 导入配置
// 🔄 从备份文件恢复配置,支持合并和覆盖两种模式

#![allow(clippy::unused_async)]

use crate::application::profile_import::{
    ConflictStrategy, ImportModeKind, ImportProfilesRequest, commit_import_with_backup,
    preview_import,
};
use crate::commands::migration::canonical_profile_command_hint;
use crate::commands::platform::profile_exchange_cmd::{
    confirm_on_stderr, is_interactive_tty, read_regular_import_file, validate_regular_exchange_path,
};
use crate::managers::config::CcsConfig;
use crate::models::Platform;
use crate::services::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::{ColorOutput, OutputStatus};
use std::io::IsTerminal;
use std::path::Path;

/// 📋 导入模式
#[derive(Debug, Clone, Copy)]
pub enum ImportMode {
    /// 🔗 合并模式：保留现有配置,只添加新的
    Merge,
    /// 🔄 覆盖模式：完全替换现有配置
    Replace,
}

/// 📥 导入配置
///
/// 执行流程:
/// 1. ✅ 验证输入文件存在
/// 2. 🔍 解析配置文件
/// 3. 💾 备份当前配置(可选)
/// 4. 🔄 执行导入(根据模式)
/// 5. 📊 显示导入摘要
///
/// 参数:
/// - input: 输入文件路径
/// - mode: 导入模式(Merge/Replace)
/// - backup: 是否备份当前配置
/// - force: 跳过确认提示（危险操作）
pub async fn import_command(
    input: String,
    mode: ImportMode,
    backup: bool,
    force: bool,
    platform: &str,
) -> Result<()> {
    let parsed_platform = platform
        .parse::<Platform>()
        .ok()
        .filter(|p| p.supports_auth_profile());
    if let Some(p) = parsed_platform {
        return import_tri_platform_legacy(&input, mode, backup, force, p);
    }

    ColorOutput::title("导入配置");
    println!();

    // ⚡ 检查自动确认模式
    let config_service = ConfigService::for_platform(platform)?;
    let config = config_service.load_config().unwrap_or_else(|_| CcsConfig {
        default_config: String::new(),
        current_config: String::new(),
        settings: crate::managers::config::GlobalSettings::default(),
        sections: indexmap::IndexMap::new(),
    });
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("自动确认模式已启用，将跳过确认");
    }

    // 验证输入文件（拒绝符号链接/重解析点与非常规文件）
    ColorOutput::step("步骤 1/3: 验证输入文件");
    let input_path = Path::new(&input);
    let (_, exists) = validate_regular_exchange_path(input_path)?;
    if !exists {
        return Err(CcrError::ConfigMissing(input));
    }
    let content = read_regular_import_file(input_path)?;
    ColorOutput::success(&format!("找到配置文件: {}", input_path.display()));
    println!();

    // 使用不回显原始源码与内联密钥的安全解析器预校验
    ColorOutput::step("步骤 2/3: 解析配置文件");
    let import_config = ccr_config::platforms::base::parse_config_from_str(&content)?;
    ColorOutput::success(&format!(
        "成功解析,包含 {} 个配置节",
        import_config.sections.len()
    ));
    println!();

    // 🚨 Replace 模式需要确认（仅在三流均为 TTY 时在 stderr 提示，非交互环境直接拒绝）
    if matches!(mode, ImportMode::Replace) && !skip_confirmation {
        if !is_interactive_tty() {
            return Err(CcrError::ValidationError(
                "Replace 模式属于破坏性覆盖操作，非交互环境必须显式传入 --force 或 -y/--yes（或改用 --merge 合并导入）".into(),
            ));
        }

        let formatted = ColorOutput::format_status(
            OutputStatus::Warning,
            "Replace 模式将完全覆盖现有配置！（建议使用 --merge 保留现有配置）",
            std::io::stderr().is_terminal(),
        );
        eprintln!("{formatted}");
        if !confirm_on_stderr("确认执行 Replace 操作?")? {
            ColorOutput::info("已取消导入操作");
            return Ok(());
        }
        println!();
    }

    // 执行导入（在配置锁内完成覆盖前备份与保存，避免二次序列化）
    let step_msg = if skip_confirmation {
        "步骤 3/3: 执行导入 (⚡ 自动确认模式)"
    } else {
        "步骤 3/3: 执行导入"
    };
    ColorOutput::step(step_msg);
    let service_mode = match mode {
        ImportMode::Merge => crate::services::config_service::ImportMode::Merge,
        ImportMode::Replace => crate::services::config_service::ImportMode::Replace,
    };
    let should_backup = backup || matches!(mode, ImportMode::Replace);
    let raw_result = config_service.import_config(&content, service_mode, should_backup)?;
    let result = ImportResult {
        added: raw_result.added,
        updated: raw_result.updated,
        skipped: raw_result.skipped,
    };

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("配置导入成功");
    print_import_summary(&result, platform);

    Ok(())
}

fn import_tri_platform_legacy(
    input: &str,
    mode: ImportMode,
    backup: bool,
    force: bool,
    platform: Platform,
) -> Result<()> {
    ColorOutput::title("导入配置");
    println!();

    ColorOutput::step("步骤 1/3: 验证输入文件");
    let input_path = Path::new(input);
    let (_, exists) = validate_regular_exchange_path(input_path)?;
    if !exists {
        return Err(CcrError::ConfigMissing(input.to_string()));
    }
    let content = read_regular_import_file(input_path)?;
    ColorOutput::success(&format!("找到配置文件: {}", input_path.display()));
    println!();

    let (mode_kind, on_conflict) = match mode {
        ImportMode::Replace => (ImportModeKind::Replace, ConflictStrategy::Skip),
        ImportMode::Merge => (ImportModeKind::Merge, ConflictStrategy::Overwrite),
    };
    let request = ImportProfilesRequest {
        platform,
        content,
        mode: mode_kind,
        on_conflict,
        dedup: false,
        dry_run: false,
        no_apply: false,
        expected_revision: None,
    };

    ColorOutput::step("步骤 2/3: 校验导入内容");
    let preview = preview_import(&request)?;
    ColorOutput::success(&format!(
        "成功解析,包含 {} 个配置节",
        preview.counts.total_incoming
    ));
    println!();

    if matches!(mode, ImportMode::Replace) && !force {
        if !is_interactive_tty() {
            return Err(CcrError::ValidationError(
                "Replace 模式属于破坏性覆盖操作，非交互环境必须显式传入 --force 或 -y/--yes（或改用 --merge 合并导入）".into(),
            ));
        }

        let formatted = ColorOutput::format_status(
            OutputStatus::Warning,
            &format!(
                "Replace 模式将以导入文件为准覆盖本地 {} 配置目录（将移除 {} 个本地条目）",
                platform.short_name(),
                preview.counts.removed
            ),
            std::io::stderr().is_terminal(),
        );
        eprintln!("{formatted}");
        if !confirm_on_stderr("确认执行 Replace 操作?")? {
            ColorOutput::info("已取消导入操作");
            return Ok(());
        }
        println!();
    }

    ColorOutput::step("步骤 3/3: 执行导入");
    let report = commit_import_with_backup(request, backup)?;
    if let Some(ref backup_path) = report.backup_path {
        ColorOutput::success(&format!("已备份到: {backup_path}"));
    }

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("配置导入成功");
    let summary = ImportResult {
        added: report.counts.added,
        updated: report.counts.updated,
        skipped: report.counts.skipped + report.counts.unchanged,
    };
    print_import_summary(&summary, platform.short_name());
    if report.counts.removed > 0 {
        ColorOutput::key_value("移除配置", &report.counts.removed.to_string(), 2);
    }
    for warning in &report.warnings {
        ColorOutput::warning(warning);
    }

    Ok(())
}

/// 合并配置（仅测试使用）
#[cfg(test)]
fn merge_configs(current: &mut CcsConfig, import: CcsConfig) -> Result<ImportResult> {
    let mut result = ImportResult {
        added: 0,
        updated: 0,
        skipped: 0,
    };

    for (name, section) in import.sections {
        if current.sections.contains_key(&name) {
            current.sections.insert(name, section);
            result.updated += 1;
        } else {
            current.sections.insert(name, section);
            result.added += 1;
        }
    }

    current.default_config = import.default_config;

    let config_manager = crate::managers::config::ConfigManager::with_default()?;
    config_manager.save(current)?;

    Ok(result)
}

/// 导入结果
struct ImportResult {
    added: usize,
    updated: usize,
    skipped: usize,
}

/// 打印导入摘要
fn print_import_summary(result: &ImportResult, platform: &str) {
    println!();
    ColorOutput::info("导入摘要:");
    if result.added > 0 {
        ColorOutput::key_value("新增配置", &result.added.to_string(), 2);
    }
    if result.updated > 0 {
        ColorOutput::key_value("更新配置", &result.updated.to_string(), 2);
    }
    if result.skipped > 0 {
        ColorOutput::key_value("跳过配置", &result.skipped.to_string(), 2);
    }
    let next_cmd = canonical_profile_command_hint(platform, "list", "ccr current");
    crate::commands::common::print_next_steps(&[("查看配置", next_cmd.as_str())]);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::config::ConfigSection;
    use crate::test_support::TestHome;
    use ccr_core::core::lock::CONFIG_LOCK;

    #[test]
    fn test_merge_configs() {
        let _guard = CONFIG_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _test_home = TestHome::new();

        let mut current = CcsConfig {
            default_config: "old_default".to_string(),
            current_config: "test1".to_string(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: indexmap::IndexMap::new(),
        };

        current.sections.insert(
            "test1".to_string(),
            ConfigSection {
                description: Some("Old".into()),
                base_url: Some("http://old.com".into()),
                auth_token: Some("old_token".into()),
                model: None,
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
                usage_count: Some(5),
                enabled: Some(true),
                other: indexmap::IndexMap::new(),
                ..Default::default()
            },
        );

        let mut import = CcsConfig {
            default_config: "new_default".to_string(),
            current_config: "test2".to_string(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: indexmap::IndexMap::new(),
        };

        import.sections.insert(
            "test1".to_string(),
            ConfigSection {
                description: Some("New".into()),
                base_url: Some("http://new.com".into()),
                auth_token: Some("new_token".into()),
                model: None,
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
                usage_count: Some(0),
                enabled: Some(true),
                other: indexmap::IndexMap::new(),
                ..Default::default()
            },
        );

        import.sections.insert(
            "test2".to_string(),
            ConfigSection {
                description: Some("Test2".into()),
                base_url: Some("http://test2.com".into()),
                auth_token: Some("test2_token".into()),
                model: None,
                small_fast_model: None,
                provider: None,
                provider_type: None,
                account: None,
                tags: None,
                usage_count: Some(0),
                enabled: Some(true),
                other: indexmap::IndexMap::new(),
                ..Default::default()
            },
        );

        let result = merge_configs(&mut current, import).unwrap();

        assert_eq!(result.added, 1);
        assert_eq!(result.updated, 1);
        assert_eq!(current.default_config, "new_default");
    }
}
