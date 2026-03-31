//! 📥 codex auth import 命令实现
//!
//! 从 JSON 文件导入账号 (交互式选择导入文件)。

#![allow(clippy::unused_async)]

use crate::models::ImportMode;
use crate::services::CodexAuthService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// 获取跨平台的 Downloads 目录
fn get_downloads_dir() -> Result<PathBuf> {
    dirs::download_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取 Downloads 目录路径".to_string()))
}

/// 扫描 Downloads 目录中的导出文件
fn scan_downloads_for_exports() -> Result<Vec<PathBuf>> {
    let downloads = get_downloads_dir()?;

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = fs::read_dir(&downloads)
        .map_err(|e| CcrError::FileIoError(format!("读取 Downloads 目录失败: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("codex-auth-export-") && name_str.ends_with(".json")
        })
        .filter_map(|entry| {
            let path = entry.path();
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((path, modified))
        })
        .collect();

    files.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(files.into_iter().map(|(path, _)| path).collect())
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

/// 📥 从 JSON 文件导入账号
pub async fn import_command(replace: bool, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    let exports = scan_downloads_for_exports()?;
    let downloads_dir = get_downloads_dir()?;
    let default_file = exports.first().cloned();

    println!();
    if let Some(ref file) = default_file {
        ColorOutput::info(&format!(
            "默认导入文件: {}",
            file.display().to_string().bright_cyan()
        ));
        if exports.len() > 1 {
            ColorOutput::info(&format!(
                "(在 Downloads 中找到 {} 个导出文件，已选择最新的)",
                exports.len()
            ));
        }
    } else {
        ColorOutput::info(&format!(
            "默认导入目录: {}",
            downloads_dir.display().to_string().bright_cyan()
        ));
        ColorOutput::warning("未在 Downloads 中找到导出文件");
    }

    let default_file_for_task = default_file.clone();
    let import_path = tokio::task::spawn_blocking(move || -> Result<PathBuf> {
        print!("是否修改导入路径? [y/N]: ");
        io::stdout()
            .flush()
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        let mut confirm = String::new();
        io::stdin()
            .read_line(&mut confirm)
            .map_err(|e| CcrError::FileIoError(e.to_string()))?;

        if confirm.trim().eq_ignore_ascii_case("y") || confirm.trim().eq_ignore_ascii_case("yes") {
            println!("请输入导入文件路径 (JSON 文件):");
            match read_user_path() {
                Some(custom_path) => Ok(PathBuf::from(custom_path)),
                None => {
                    if let Some(file) = default_file_for_task {
                        ColorOutput::info("使用默认文件");
                        Ok(file)
                    } else {
                        ColorOutput::error("未指定文件且无默认文件可用");
                        Ok(PathBuf::new())
                    }
                }
            }
        } else {
            match default_file_for_task {
                Some(file) => Ok(file),
                None => {
                    ColorOutput::error("在 Downloads 目录中未找到导出文件");
                    println!();
                    ColorOutput::info("提示:");
                    println!("  • 先使用 'ccr codex auth export' 导出账号");
                    println!("  • 或输入 'y' 手动指定文件路径");
                    Ok(PathBuf::new())
                }
            }
        }
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取导入路径失败: {}", e)))??;

    if import_path.as_os_str().is_empty() {
        return Ok(());
    }

    if !import_path.exists() {
        ColorOutput::error(&format!("文件不存在: {}", import_path.display()));
        return Ok(());
    }

    if import_path.extension().is_some_and(|ext| ext != "json") {
        ColorOutput::warning("警告: 文件扩展名不是 .json，继续尝试导入...");
    }

    let content = fs::read_to_string(&import_path)
        .map_err(|e| CcrError::FileIoError(format!("读取文件失败: {}", e)))?;

    let mode = if replace {
        ImportMode::Replace
    } else {
        ImportMode::Merge
    };

    match service.import_accounts(&content, mode, force) {
        Ok(result) => {
            println!();
            ColorOutput::success("导入完成！");
            println!();

            if result.added > 0 {
                ColorOutput::info(&format!(
                    "新增账号: {}",
                    result.added.to_string().bright_green()
                ));
            }
            if result.updated > 0 {
                ColorOutput::info(&format!(
                    "更新账号: {}",
                    result.updated.to_string().bright_yellow()
                ));
            }
            if result.skipped > 0 {
                ColorOutput::info(&format!(
                    "跳过账号: {}",
                    result.skipped.to_string().bright_cyan()
                ));
            }
            if !result.overwritten.is_empty() {
                ColorOutput::warning(&format!(
                    "覆盖账号: {}",
                    result.overwritten.len().to_string().bright_magenta()
                ));
                for name in &result.overwritten {
                    println!("  • {}", name.bright_magenta());
                }
            }

            let total = result.added + result.updated + result.skipped;
            if total == 0 {
                ColorOutput::warning("没有账号被导入");
            }

            println!();
            match mode {
                ImportMode::Merge => {
                    if force {
                        ColorOutput::info("模式: 合并 (强制覆盖已存在的账号)");
                    } else {
                        ColorOutput::info("模式: 合并 (跳过已存在的账号)");
                    }
                }
                ImportMode::Replace => {
                    ColorOutput::info("模式: 替换 (覆盖同名账号)");
                }
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
        }
        Err(e) => {
            ColorOutput::error(&format!("导入失败: {}", e));
            let err_msg = e.to_string();
            if err_msg.contains("解析") {
                println!();
                ColorOutput::info("提示: 请确保文件是有效的 JSON 格式");
            }
        }
    }

    Ok(())
}
