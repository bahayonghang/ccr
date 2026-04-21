// 📚 history 命令实现 - 显示操作历史
// 🔍 展示所有操作的审计追踪,支持筛选和统计

#![allow(clippy::unused_async)]

use crate::managers::{OperationResult, OperationType};
use crate::services::HistoryService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::*;

/// 📚 显示操作历史
///
/// 显示内容:
/// - 📊 操作统计(总数、成功、失败、警告)
/// - 📋 历史记录列表(时间、操作、结果)
/// - 🌍 环境变量变化(已掩码)
/// - 📝 操作详情(from/to 配置、备份路径等)
///
/// 参数:
/// - limit: 显示记录数量(默认 20)
/// - filter_type: 按操作类型筛选(switch/backup/restore/validate/update)
pub async fn history_command(limit: Option<usize>, filter_type: Option<String>) -> Result<()> {
    ColorOutput::title("操作历史记录");
    println!();

    // 使用 HistoryService
    let service = HistoryService::with_default()?;

    // 获取历史记录
    let entries = if let Some(type_str) = filter_type {
        // 按类型筛选
        let op_type = match type_str.to_lowercase().as_str() {
            "switch" => OperationType::Switch,
            "backup" => OperationType::Backup,
            "restore" => OperationType::Restore,
            "validate" => OperationType::Validate,
            "update" => OperationType::Update,
            _ => {
                ColorOutput::error(&format!("未知的操作类型: {}", type_str));
                ColorOutput::info("支持的类型: switch, backup, restore, validate, update");
                return Ok(());
            }
        };
        service.filter_by_type_async(op_type).await?
    } else if let Some(n) = limit {
        service.get_recent_async(n).await?
    } else {
        service.get_recent_async(100).await?
    };

    if entries.is_empty() {
        ColorOutput::info("暂无历史记录");
        return Ok(());
    }

    // 显示统计信息
    let stats = service.get_stats_async().await?;
    ColorOutput::info(&format!("总操作数: {}", stats.total_operations));
    ColorOutput::info(&format!(
        "成功: {}, 失败: {}, 警告: {}",
        stats.successful_operations, stats.failed_operations, stats.warning_operations
    ));
    println!();

    ColorOutput::separator();
    println!();

    // 显示记录
    for (index, entry) in entries.iter().enumerate() {
        let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        let op_str = entry.operation.as_str();

        let result_str = match &entry.result {
            OperationResult::Success => "成功".green(),
            OperationResult::Failure(msg) => format!("失败: {}", msg).red(),
            OperationResult::Warning(msg) => format!("警告: {}", msg).yellow(),
        };

        println!("{}. [{}] {} - {}", index + 1, time_str, op_str, result_str);
        println!("   操作者: {}", entry.actor);

        // 显示详情
        if let Some(from) = &entry.details.from_config {
            println!("   从: {}", from);
        }
        if let Some(to) = &entry.details.to_config {
            println!("   到: {}", to);
        }
        if let Some(backup) = &entry.details.backup_path {
            println!("   备份: {}", backup);
        }

        // 显示环境变量变化
        if !entry.env_changes.is_empty() {
            println!("   环境变量变化:");
            for change in &entry.env_changes {
                let old_display = change.old_value.as_deref().unwrap_or("(无)").dimmed();
                let new_display = change.new_value.as_deref().unwrap_or("(无)");
                println!(
                    "     {} {} -> {}",
                    change.var_name, old_display, new_display
                );
            }
        }

        // 显示备注
        if let Some(notes) = &entry.notes {
            println!("   备注: {}", notes);
        }

        println!();
    }

    ColorOutput::info(&format!("显示了最近 {} 条记录", entries.len()));

    // 显示清理提示
    if stats.total_operations > 100 {
        println!();
        ColorOutput::warning(&format!(
            "历史记录较多 ({} 条),建议定期清理旧记录",
            stats.total_operations
        ));
        ColorOutput::info("提示: 使用 ccr history --clear 清理历史记录");
    }

    Ok(())
}
