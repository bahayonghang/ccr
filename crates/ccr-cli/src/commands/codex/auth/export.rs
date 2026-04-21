//! 📤 codex auth export 命令实现
//!
//! 导出所有账号到 JSON 文件 (交互式选择导出路径)。
//! 当包含敏感信息时，使用密码加密保护 (AES-256-GCM + Argon2id)。

#![allow(clippy::unused_async)]

use crate::services::CodexAuthService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use chrono::Local;
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// 密码最小长度
const MIN_PASSWORD_LEN: usize = 6;

/// 获取跨平台的 Downloads 目录
///
/// 支持 Windows、macOS 和 Linux
fn get_downloads_dir() -> Result<PathBuf> {
    dirs::download_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取 Downloads 目录路径".to_string()))
}

/// 获取默认导出文件路径
///
/// 返回 `Downloads/codex-auth-export-YYYY-MM-DD.json`
fn get_default_export_path() -> Result<PathBuf> {
    let downloads = get_downloads_dir()?;
    let date = Local::now().format("%Y-%m-%d");
    let filename = format!("codex-auth-export-{}.json", date);
    Ok(downloads.join(filename))
}

/// 读取用户输入的路径
fn read_user_path() -> Option<String> {
    print!("  → ");
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 交互式读取并确认密码
///
/// 返回 `Ok(password)` 或错误
fn read_and_confirm_password() -> Result<String> {
    println!();
    ColorOutput::info("🔐 导出包含敏感信息，需要设置加密密码");

    let password = rpassword::prompt_password("请输入密码: ")
        .map_err(|e| CcrError::FileIoError(format!("读取密码失败: {}", e)))?;

    if password.len() < MIN_PASSWORD_LEN {
        return Err(CcrError::ConfigError(format!(
            "密码长度不足: 最少 {} 个字符",
            MIN_PASSWORD_LEN
        )));
    }

    let confirm = rpassword::prompt_password("请再次确认: ")
        .map_err(|e| CcrError::FileIoError(format!("读取确认密码失败: {}", e)))?;

    if password != confirm {
        return Err(CcrError::ConfigError("两次输入的密码不一致".to_string()));
    }

    Ok(password)
}

/// 📤 导出所有账号到 JSON 文件
///
/// 将所有已保存的账号导出为 JSON 格式，支持交互式选择导出路径。
/// 当包含敏感信息时，使用密码加密保护。
///
/// # 参数
///
/// * `no_secrets` - 是否不包含敏感信息 (Token 等)
///
/// # 返回
///
/// * `Ok(())` - 导出成功
/// * `Err(CcrError)` - 导出失败
pub async fn export_command(no_secrets: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查是否有账号
    let accounts = service.list_accounts()?;
    let saved_count = accounts.iter().filter(|a| !a.is_virtual).count();

    if saved_count == 0 {
        ColorOutput::warning("没有已保存的账号可导出");
        println!();
        ColorOutput::info("提示:");
        println!("  • 使用 'ccr codex auth save <名称>' 保存当前登录");
        return Ok(());
    }

    // 获取默认导出路径
    let default_path = get_default_export_path()?;

    println!();
    ColorOutput::info(&format!(
        "默认导出路径: {}",
        default_path.display().to_string().bright_cyan()
    ));

    let default_path_for_task = default_path.clone();
    let export_path = tokio::task::spawn_blocking(move || -> Result<PathBuf> {
        print!("是否修改导出路径? [y/N]: ");
        io::stdout()
            .flush()
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        let mut confirm = String::new();
        io::stdin()
            .read_line(&mut confirm)
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        if confirm.trim().eq_ignore_ascii_case("y") || confirm.trim().eq_ignore_ascii_case("yes") {
            println!("请输入导出路径 (文件或目录):");
            match read_user_path() {
                Some(custom_path) => {
                    let path = PathBuf::from(&custom_path);
                    // 如果用户输入的是目录，则在目录下创建默认文件名
                    if path.is_dir() || custom_path.ends_with('/') || custom_path.ends_with('\\') {
                        let date = Local::now().format("%Y-%m-%d");
                        let filename = format!("codex-auth-export-{}.json", date);
                        Ok(path.join(filename))
                    } else {
                        Ok(path)
                    }
                }
                None => {
                    ColorOutput::info("使用默认路径");
                    Ok(default_path_for_task)
                }
            }
        } else {
            Ok(default_path_for_task)
        }
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取导出路径失败: {}", e)))??;

    // 确保父目录存在
    if let Some(parent) = export_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .map_err(|e| CcrError::FileIoError(format!("创建目录失败: {}", e)))?;
    }

    let include_secrets = !no_secrets;

    let json = if include_secrets {
        // 加密导出流程：提示密码 → 加密 → 写入
        let password = tokio::task::spawn_blocking(read_and_confirm_password)
            .await
            .map_err(|e| CcrError::FileIoError(format!("密码交互失败: {}", e)))??;

        service.export_accounts_encrypted(&password)?
    } else {
        // 明文导出（仅元数据）
        service.export_accounts(false)?
    };

    // 写入文件
    fs::write(&export_path, &json)
        .map_err(|e| CcrError::FileIoError(format!("写入文件失败: {}", e)))?;

    println!();
    ColorOutput::success(&format!(
        "已导出到: {}",
        export_path.display().to_string().bright_green()
    ));
    ColorOutput::info(&format!("账号数量: {}", saved_count));

    if include_secrets {
        println!();
        ColorOutput::success("🔐 导出文件已加密 (AES-256-GCM + Argon2id)");
        ColorOutput::info("导入时需要输入相同的密码");
    } else {
        ColorOutput::info("导出不包含敏感信息 (仅元数据)");
    }

    println!();
    ColorOutput::info("提示:");
    println!("  • 使用 'ccr codex auth import' 导入账号");

    Ok(())
}
