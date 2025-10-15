// 📥 import 命令实现 - 导入配置
// 🔄 从备份文件恢复配置,支持合并和覆盖两种模式

use crate::managers::config::{CcsConfig, ConfigManager};
use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
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
pub fn import_command(input: String, mode: ImportMode, backup: bool) -> Result<()> {
    ColorOutput::title("导入配置");
    println!();

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
        let config_manager = ConfigManager::default()?;
        if config_manager.config_path().exists() {
            backup_current_config(&config_manager)?;
        } else {
            ColorOutput::info("当前无配置文件,跳过备份");
        }
        println!();
    }

    // 执行导入
    ColorOutput::step(if backup {
        "步骤 4/4: 执行导入"
    } else {
        "步骤 3/3: 执行导入"
    });
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
        .map_err(|e| CcrError::ConfigError(format!("读取文件失败: {}", e)))?;

    let config: CcsConfig = toml::from_str(&content)
        .map_err(|e| CcrError::ConfigFormatInvalid(format!("解析 TOML 失败: {}", e)))?;

    Ok(config)
}

/// 备份当前配置
fn backup_current_config(config_manager: &ConfigManager) -> Result<()> {
    let backup_path = config_manager.backup(Some("import_backup"))?;
    ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
    Ok(())
}

/// 根据模式导入配置
fn import_config_with_mode(import_config: CcsConfig, mode: ImportMode) -> Result<ImportResult> {
    let config_manager = ConfigManager::default()?;

    let result = match mode {
        ImportMode::Merge => {
            // 合并模式
            if config_manager.config_path().exists() {
                let mut current_config = config_manager.load()?;
                merge_configs(&mut current_config, import_config)?
            } else {
                // 没有现有配置,直接使用导入的
                config_manager.save(&import_config)?;
                ImportResult {
                    added: import_config.sections.len(),
                    updated: 0,
                    skipped: 0,
                }
            }
        }
        ImportMode::Replace => {
            // 替换模式
            let count = import_config.sections.len();
            config_manager.save(&import_config)?;
            ImportResult {
                added: count,
                updated: 0,
                skipped: 0,
            }
        }
    };

    Ok(result)
}

/// 合并配置
fn merge_configs(current: &mut CcsConfig, import: CcsConfig) -> Result<ImportResult> {
    let mut result = ImportResult {
        added: 0,
        updated: 0,
        skipped: 0,
    };

    for (name, section) in import.sections {
        if current.sections.contains_key(&name) {
            // 已存在,更新
            current.sections.insert(name, section);
            result.updated += 1;
        } else {
            // 不存在,添加
            current.sections.insert(name, section);
            result.added += 1;
        }
    }

    // 如果导入配置中有 default_config,也更新它
    // 但保持 current_config 不变
    current.default_config = import.default_config;

    let config_manager = ConfigManager::default()?;
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
mod tests {
    use super::*;
    use crate::managers::config::ConfigSection;

    #[test]
    fn test_merge_configs() {
        let mut current = CcsConfig {
            default_config: "old_default".to_string(),
            current_config: "test1".to_string(),
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
            },
        );

        let mut import = CcsConfig {
            default_config: "new_default".to_string(),
            current_config: "test2".to_string(),
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
            },
        );

        let result = merge_configs(&mut current, import).unwrap();

        assert_eq!(result.added, 1); // test2 是新增的
        assert_eq!(result.updated, 1); // test1 被更新了
        assert_eq!(current.default_config, "new_default");
    }
}
