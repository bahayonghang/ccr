// 📥 import 命令实现 - 导入配置
// 🔄 从备份文件恢复配置,支持合并和覆盖两种模式

#![allow(clippy::unused_async)]

use crate::managers::config::CcsConfig;
use crate::services::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use std::fs;
use std::path::PathBuf;

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
) -> Result<()> {
    ColorOutput::title("导入配置");
    println!();

    // ⚡ 检查自动确认模式
    let config_service = ConfigService::with_default()?;
    let config = config_service.load_config().unwrap_or_else(|_| CcsConfig {
        default_config: String::new(),
        current_config: String::new(),
        settings: crate::managers::config::GlobalSettings::default(),
        sections: indexmap::IndexMap::new(),
    });
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    // 🚨 Replace 模式需要确认
    if matches!(mode, ImportMode::Replace) && !skip_confirmation {
        println!();
        ColorOutput::warning("⚠️  警告: Replace 模式将完全覆盖现有配置！");
        ColorOutput::info("建议: 使用 --merge 参数保留现有配置");
        println!();

        let confirmed = tokio::task::spawn_blocking(|| -> std::io::Result<bool> {
            use std::io::{self, Write};
            print!("确认执行 Replace 操作? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))??;

        if !confirmed {
            ColorOutput::info("已取消导入操作");
            return Ok(());
        }
        println!();
    }

    // 验证输入文件
    ColorOutput::step("步骤 1/4: 验证输入文件");
    let input_path = PathBuf::from(&input);
    if !input_path.exists() {
        return Err(CcrError::ConfigMissing(input.clone()));
    }
    ColorOutput::success(&format!("找到配置文件: {}", input_path.display()));
    println!();

    // 读取并解析导入文件
    ColorOutput::step("步骤 2/4: 解析配置文件");
    let import_config = parse_import_file(&input_path)?;
    ColorOutput::success(&format!(
        "成功解析,包含 {} 个配置节",
        import_config.sections.len()
    ));
    println!();

    // 备份现有配置(如果需要)
    if backup {
        ColorOutput::step("步骤 3/4: 备份当前配置");
        if config_service.config_manager().config_path().exists() {
            backup_current_config(&config_service)?;
        } else {
            ColorOutput::info("当前无配置文件,跳过备份");
        }
        println!();
    }

    // 执行导入
    let step_msg = if backup {
        if skip_confirmation {
            "步骤 4/4: 执行导入 (⚡ 自动确认模式)"
        } else {
            "步骤 4/4: 执行导入"
        }
    } else if skip_confirmation {
        "步骤 3/3: 执行导入 (⚡ 自动确认模式)"
    } else {
        "步骤 3/3: 执行导入"
    };
    ColorOutput::step(step_msg);
    let result = import_config_with_mode(import_config, mode)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ 配置导入成功");
    print_import_summary(&result);

    Ok(())
}

/// 解析导入文件
fn parse_import_file(path: &PathBuf) -> Result<CcsConfig> {
    let content = fs::read_to_string(path)
        .map_err(|e| CcrError::FileIoError(format!("读取文件失败: {}", e)))?;

    let config: CcsConfig = toml::from_str(&content)
        .map_err(|e| CcrError::ConfigFormatInvalid(format!("解析 TOML 失败: {}", e)))?;

    Ok(config)
}

/// 备份当前配置
fn backup_current_config(config_service: &ConfigService) -> Result<()> {
    let backup_path = config_service.backup_config(Some("import_backup"))?;
    ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
    Ok(())
}

/// 根据模式导入配置
fn import_config_with_mode(import_config: CcsConfig, mode: ImportMode) -> Result<ImportResult> {
    let service = ConfigService::with_default()?;
    let content = toml::to_string_pretty(&import_config)
        .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

    let service_mode = match mode {
        ImportMode::Merge => crate::services::config_service::ImportMode::Merge,
        ImportMode::Replace => crate::services::config_service::ImportMode::Replace,
    };

    let result = service.import_config(&content, service_mode, false)?;

    Ok(ImportResult {
        added: result.added,
        updated: result.updated,
        skipped: result.skipped,
    })
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
fn print_import_summary(result: &ImportResult) {
    println!();
    ColorOutput::info("导入摘要:");
    if result.added > 0 {
        println!("  ✓ 新增配置: {}", result.added);
    }
    if result.updated > 0 {
        println!("  ✓ 更新配置: {}", result.updated);
    }
    if result.skipped > 0 {
        println!("  ○ 跳过配置: {}", result.skipped);
    }
    println!();
    ColorOutput::info("提示: 运行 'ccr list' 查看所有配置");
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::config::ConfigSection;
    use ccr_core::core::lock::CONFIG_LOCK;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_merge_configs() {
        let _env_guard = crate::test_support::env_lock();
        let _guard = CONFIG_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempdir().unwrap();
        let temp_root = temp_dir.path().to_path_buf();

        let prev_root = env::var("CCR_ROOT").ok();
        let prev_config_path = env::var("CCR_CONFIG_PATH").ok();

        // SAFETY: 测试仅在当前进程临时覆写导入目标路径，结束后会按原值恢复。
        unsafe {
            env::set_var("CCR_ROOT", &temp_root);
            env::remove_var("CCR_CONFIG_PATH");
        }

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

        // SAFETY: 恢复当前测试先前保存的环境变量，避免污染其他测试用例。
        unsafe {
            match prev_root {
                Some(val) => env::set_var("CCR_ROOT", val),
                None => env::remove_var("CCR_ROOT"),
            }
            match prev_config_path {
                Some(val) => env::set_var("CCR_CONFIG_PATH", val),
                None => env::remove_var("CCR_CONFIG_PATH"),
            }
        }
    }
}
