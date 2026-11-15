// 📤 export 命令实现 - 导出配置
// 💾 将配置备份到文件,支持敏感信息脱敏

#![allow(clippy::unused_async)]

use crate::application::profile_export::export_legacy_flat_catalog;
use crate::commands::platform::profile_exchange_cmd::write_secret_export_file;
use crate::models::Platform;
use crate::services::ConfigService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::{ColorOutput, OutputStatus};
use std::io::{IsTerminal, Write};
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
pub async fn export_command(
    output: Option<String>,
    include_secrets: bool,
    platform: &str,
    stdout: bool,
) -> Result<()> {
    let parsed_platform = platform
        .parse::<Platform>()
        .ok()
        .filter(|p| p.supports_auth_profile());

    if stdout {
        let (mut content, _, warnings) = if let Some(p) = parsed_platform {
            export_legacy_flat_catalog(p, include_secrets)?
        } else {
            ConfigService::for_platform(platform)?.export_config_with_warnings(include_secrets)?
        };
        for warning in &warnings {
            let formatted = ColorOutput::format_status(
                OutputStatus::Warning,
                warning,
                std::io::stderr().is_terminal(),
            );
            eprintln!("{formatted}");
        }
        if !content.ends_with('\n') {
            content.push('\n');
        }
        let mut out = std::io::stdout().lock();
        out.write_all(content.as_bytes())
            .map_err(|error| CcrError::FileIoError(format!("写入标准输出失败: {error}")))?;
        return Ok(());
    }

    ColorOutput::title("导出配置");
    println!();

    // 加载当前配置
    ColorOutput::step("步骤 1/3: 读取配置");
    let (content, section_count, warnings) = if let Some(p) = parsed_platform {
        export_legacy_flat_catalog(p, include_secrets)?
    } else {
        ConfigService::for_platform(platform)?.export_config_with_warnings(include_secrets)?
    };
    ColorOutput::success(&format!("已加载配置,共 {} 个配置节", section_count));
    for warning in &warnings {
        ColorOutput::warning(warning);
    }
    println!();

    // 确定输出路径
    ColorOutput::step("步骤 2/3: 准备导出");
    let output_path = determine_output_path(output)?;
    ColorOutput::key_value("导出路径", &output_path.display().to_string(), 2);
    println!();

    // 导出配置
    ColorOutput::step("步骤 3/3: 写入文件");
    export_to_file(&content, &output_path)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("配置导出成功");
    ColorOutput::key_value("导出文件", &output_path.display().to_string(), 2);

    if include_secrets {
        println!();
        ColorOutput::warning("已包含敏感信息(API密钥)");
        ColorOutput::info("请妥善保管导出文件,避免泄露");
        ColorOutput::info("使用 --no-secrets 参数可导出不含密钥的配置");
    } else {
        println!();
        ColorOutput::info("敏感信息已移除");
        ColorOutput::info("不使用 --no-secrets 可导出完整配置(包含密钥)");
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

/// 导出到文件（符号链接检查 + 0600 原子写入）
fn export_to_file(content: &str, output_path: &Path) -> Result<()> {
    let mut normalized = content.to_string();
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    write_secret_export_file(output_path, normalized.as_bytes())?;
    ColorOutput::success(&format!("已导出到: {}", output_path.display()));
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_output_path() {
        let path = determine_output_path(Some("test.toml".to_string())).unwrap();
        assert_eq!(path, PathBuf::from("test.toml"));

        let path = determine_output_path(None).unwrap();
        assert!(path.to_string_lossy().starts_with("ccs_config_export_"));
        assert!(path.to_string_lossy().ends_with(".toml"));
    }
}
