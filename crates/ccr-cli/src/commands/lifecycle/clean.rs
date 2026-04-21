// 🧹 clean 命令实现 - 清理旧备份文件
// 📅 根据时间策略删除过期的 .bak 备份文件

#![allow(clippy::unused_async)]

use crate::services::{BackupService, ConfigService};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;

/// 🧹 清理旧备份文件
///
/// 执行流程:
/// 1. 📁 扫描备份目录 (~/.claude/backups/)
/// 2. 🔍 识别 .bak 文件
/// 3. 📅 检查文件修改时间
/// 4. 🗑️ 删除超过指定天数的文件
/// 5. 📊 统计清理结果(文件数、释放空间)
///
/// 参数:
/// - days: 保留天数(删除 N 天前的文件)
/// - dry_run: 模拟运行(不实际删除)
/// - force: 跳过确认提示（危险操作）
pub async fn clean_command(days: u64, dry_run: bool, force: bool) -> Result<()> {
    ColorOutput::title("清理备份文件");
    println!();

    // ⚡ 检查自动确认模式：--force 参数 OR 配置文件中的 skip_confirmation
    let config_service = ConfigService::with_default()?;
    let config = config_service.load_config()?;
    let skip_confirmation = force || config.settings.skip_confirmation;

    if config.settings.skip_confirmation && !force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    // 使用 BackupService
    let service = BackupService::with_default()?;
    let backup_dir = service.backup_dir();

    if !backup_dir.exists() {
        ColorOutput::info("备份目录不存在,无需清理");
        return Ok(());
    }

    ColorOutput::info(&format!("备份目录: {}", backup_dir.display()));
    ColorOutput::info(&format!("清理策略: 删除 {} 天前的备份", days));

    if dry_run {
        ColorOutput::warning("⚠ 模拟运行模式(不会实际删除文件)");
    }

    // 🚨 非 dry-run 模式需要确认（除非 YOLO 模式）
    if !dry_run && !skip_confirmation {
        println!();
        ColorOutput::warning("⚠️  警告: 即将删除旧备份文件！");
        ColorOutput::info("提示: 使用 --dry-run 参数可以先预览将要删除的文件");
        println!();

        let confirmed = tokio::task::spawn_blocking(|| -> std::io::Result<bool> {
            use std::io::{self, Write};
            print!("确认执行清理操作? (y/N): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {}", e)))??;

        if !confirmed {
            ColorOutput::info("已取消清理操作");
            return Ok(());
        }
    }

    println!();
    ColorOutput::separator();
    println!();

    // 使用 BackupService 清理
    let status_msg = if skip_confirmation && !dry_run {
        "⚡ 执行清理 (自动确认模式)"
    } else {
        "执行清理"
    };
    if !dry_run {
        ColorOutput::step(status_msg);
    }
    let result = service.clean_old_backups(days, dry_run)?;

    println!();
    ColorOutput::separator();
    println!();

    // 显示结果
    if result.deleted_count > 0 || result.skipped_count > 0 {
        ColorOutput::title("清理摘要");
        println!();

        if result.deleted_count > 0 {
            if dry_run {
                ColorOutput::info(&format!("将删除文件: {} 个", result.deleted_count));
            } else {
                ColorOutput::success(&format!("✓ 已删除文件: {} 个", result.deleted_count));
            }
        }

        if result.skipped_count > 0 {
            ColorOutput::info(&format!("保留文件: {} 个", result.skipped_count));
        }

        if result.total_size > 0 {
            let size_mb = result.total_size as f64 / 1024.0 / 1024.0;
            if dry_run {
                ColorOutput::info(&format!("将释放空间: {:.2} MB", size_mb));
            } else {
                ColorOutput::success(&format!("✓ 释放空间: {:.2} MB", size_mb));
            }
        }
    } else {
        ColorOutput::success("✓ 没有需要清理的文件");
    }

    if dry_run {
        println!();
        ColorOutput::info("提示: 运行 'ccr clean' (不带 --dry-run) 执行实际清理");
    }

    Ok(())
}
