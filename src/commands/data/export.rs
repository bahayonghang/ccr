// 📤 export 命令实现 - 导出配置
// 💾 将配置备份到文件,支持敏感信息脱敏

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::config::ConfigManager;
use std::fs;
use std::path::{Path, PathBuf};

/// 📤 导出配置到文件
///
/// 执行流程:
/// 1. 📖 读取当前配置
/// 2. 🔒 处理敏感信息(根据 include_secrets)
/// 3. 📝 序列化为 TOML
/// 4. 💾 保存到文件
///
/// 参数:
/// - output: 输出文件路径(默认: `ccs_config_export_<timestamp>.toml`)
/// - include_secrets: 是否包含 API 密钥等敏感信息
pub fn export_command(output: Option<String>, include_secrets: bool) -> Result<()> {
    ColorOutput::title("导出配置");
    println!();

    // 加载当前配置
    ColorOutput::step("步骤 1/3: 读取配置");
    let config_manager = ConfigManager::with_default()?;
    let config = config_manager.load()?;
    ColorOutput::success(&format!("已加载配置,共 {} 个配置节", config.sections.len()));
    println!();

    // 确定输出路径
    ColorOutput::step("步骤 2/3: 准备导出");
    let output_path = determine_output_path(output)?;
    ColorOutput::info(&format!("导出路径: {}", output_path.display()));
    println!();

    // 导出配置
    ColorOutput::step("步骤 3/3: 写入文件");
    export_to_file(&config_manager, &output_path, include_secrets)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ 配置导出成功");
    ColorOutput::info(&format!("导出文件: {}", output_path.display()));

    if include_secrets {
        println!();
        ColorOutput::warning("⚠ 已包含敏感信息(API密钥)");
        ColorOutput::info("提示: 请妥善保管导出文件,避免泄露");
        ColorOutput::info("提示: 使用 --no-secrets 参数可导出不含密钥的配置");
    } else {
        println!();
        ColorOutput::info("✓ 敏感信息已移除");
        ColorOutput::info("提示: 不使用 --no-secrets 可导出完整配置(包含密钥)");
    }

    Ok(())
}

/// 确定输出路径
fn determine_output_path(output: Option<String>) -> Result<PathBuf> {
    if let Some(path) = output {
        Ok(PathBuf::from(path))
    } else {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ccs_config_export_{}.toml", timestamp);
        Ok(PathBuf::from(filename))
    }
}

/// 导出到文件
fn export_to_file(
    config_manager: &ConfigManager,
    output_path: &Path,
    include_secrets: bool,
) -> Result<()> {
    let mut config = config_manager.load()?;

    if !include_secrets {
        for section in config.sections.values_mut() {
            if let Some(ref token) = section.auth_token {
                section.auth_token = Some(mask_token(token));
            }
        }
    }

    let content = toml::to_string_pretty(&config)
        .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

    fs::write(output_path, content)
        .map_err(|e| CcrError::ConfigError(format!("写入文件失败: {}", e)))?;

    ColorOutput::success(&format!("已导出到: {}", output_path.display()));
    Ok(())
}

/// 掩码处理 token
fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{} (已移除)", prefix, suffix)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_token() {
        assert_eq!(
            mask_token("sk-ant-1234567890abcdef"),
            "sk-a...cdef (已移除)"
        );
        assert_eq!(mask_token("short"), "*****");
    }

    #[test]
    fn test_determine_output_path() {
        let path = determine_output_path(Some("test.toml".to_string())).unwrap();
        assert_eq!(path, PathBuf::from("test.toml"));

        let path = determine_output_path(None).unwrap();
        assert!(path.to_string_lossy().starts_with("ccs_config_export_"));
        assert!(path.to_string_lossy().ends_with(".toml"));
    }
}
