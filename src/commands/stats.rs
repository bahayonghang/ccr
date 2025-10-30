// 📊 CCR 统计命令实现
// 提供成本、使用情况等统计功能

use crate::core::ColorOutput;
use crate::core::error::{CcrError, Result};
use crate::managers::CostTracker;
use chrono::{Datelike, Duration, Utc};
use clap::{Args, Subcommand};

/// 📊 统计命令
#[derive(Args)]
pub struct StatsArgs {
    #[command(subcommand)]
    pub command: StatsSubcommand,
}

/// 📋 统计子命令
#[derive(Subcommand)]
pub enum StatsSubcommand {
    /// 💰 成本统计
    Cost(CostStatsArgs),
}

/// 💰 成本统计参数
#[derive(Args)]
pub struct CostStatsArgs {
    /// 📅 时间范围: today, week, month
    #[arg(long, default_value = "today")]
    pub range: String,

    /// 🤖 按模型分组
    #[arg(long)]
    pub by_model: bool,

    /// 📁 按项目分组
    #[arg(long)]
    pub by_project: bool,

    /// 🏆 显示 Top N 会话
    #[arg(long)]
    pub top: Option<usize>,

    /// 📊 显示详细信息
    #[arg(long)]
    pub details: bool,

    /// 📤 导出到文件
    #[arg(long)]
    pub export: Option<String>,
}

/// 执行统计命令
pub async fn stats_command(args: StatsArgs, _color_output: &mut ColorOutput) -> Result<()> {
    match args.command {
        StatsSubcommand::Cost(cost_args) => cost_stats_command(cost_args).await,
    }
}

/// 执行成本统计命令
async fn cost_stats_command(args: CostStatsArgs) -> Result<()> {
    let storage_dir = CostTracker::default_storage_dir()?;
    let tracker = CostTracker::new(storage_dir)?;

    // 解析时间范围
    let (start, end) = match args.range.as_str() {
        "today" => {
            let now = Utc::now();
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
            (start, now)
        }
        "week" => {
            let now = Utc::now();
            let start = now - Duration::days(7);
            (start, now)
        }
        "month" => {
            let now = Utc::now();
            let start = now
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            (start, now)
        }
        _ => {
            return Err(CcrError::ValidationError(
                "无效的时间范围，请使用: today, week, month".to_string(),
            ));
        }
    };

    // 生成统计
    let stats = tracker.generate_stats(start, end)?;

    // 显示基本统计
    ColorOutput::title(&format!("📊 成本统计 - {}", args.range));
    println!();

    ColorOutput::info(&format!("💰 总成本: ${:.4}", stats.total_cost));
    ColorOutput::info(&format!("📊 记录数: {}", stats.record_count));
    println!();

    // Token 统计
    ColorOutput::success("🎫 Token 使用:");
    println!(
        "  📥 输入: {} tokens",
        format_number(stats.token_stats.total_input_tokens)
    );
    println!(
        "  📤 输出: {} tokens",
        format_number(stats.token_stats.total_output_tokens)
    );
    println!(
        "  💾 Cache: {} tokens",
        format_number(stats.token_stats.total_cache_tokens)
    );
    println!(
        "  📊 Cache 效率: {:.2}%",
        stats.token_stats.cache_efficiency
    );
    println!();

    // 按模型分组
    if args.by_model || args.details {
        ColorOutput::success("🤖 按模型分组:");
        let mut models: Vec<_> = stats.by_model.iter().collect();
        models.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        for (model, cost) in models {
            let short_model = shorten_model_name(model);
            println!("  • {}: ${:.4}", short_model, cost);
        }
        println!();
    }

    // 按项目分组
    if args.by_project || args.details {
        ColorOutput::success("📁 按项目分组:");
        let mut projects: Vec<_> = stats.by_project.iter().collect();
        projects.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        for (project, cost) in projects.iter().take(10) {
            let short_project = shorten_path(project);
            println!("  • {}: ${:.4}", short_project, cost);
        }
        println!();
    }

    // Top 会话
    if let Some(limit) = args.top {
        ColorOutput::success(&format!("🏆 成本最高的 {} 个会话:", limit));
        let top_sessions = tracker.get_top_sessions(limit)?;

        for (i, (session_id, cost)) in top_sessions.iter().enumerate() {
            let short_id = shorten_id(session_id);
            println!("  {}. {}: ${:.4}", i + 1, short_id, cost);
        }
        println!();
    }

    // 趋势数据
    if args.details
        && let Some(trend) = &stats.trend
    {
        ColorOutput::success("📈 每日趋势:");
        for daily in trend.iter().rev().take(7).rev() {
            println!("  {} - ${:.4} ({} 次)", daily.date, daily.cost, daily.count);
        }
        println!();
    }

    // 导出
    if let Some(export_path) = args.export {
        export_stats(&stats, &export_path)?;
        ColorOutput::success(&format!("📤 已导出到: {}", export_path));
    }

    Ok(())
}

/// 格式化大数字
fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// 缩短模型名称
fn shorten_model_name(model: &str) -> String {
    if model.starts_with("claude-") {
        model.replace("claude-", "")
    } else {
        model.to_string()
    }
}

/// 缩短路径
fn shorten_path(path: &str) -> String {
    if let Some(name) = std::path::Path::new(path).file_name() {
        name.to_string_lossy().to_string()
    } else {
        path.to_string()
    }
}

/// 缩短 ID
fn shorten_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}...{}", &id[..6], &id[id.len() - 6..])
    } else {
        id.to_string()
    }
}

/// 导出统计数据
fn export_stats(stats: &crate::models::stats::CostStats, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(stats)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_shorten_model_name() {
        assert_eq!(
            shorten_model_name("claude-3-5-sonnet-20241022"),
            "3-5-sonnet-20241022"
        );
    }

    #[test]
    fn test_shorten_id() {
        assert_eq!(shorten_id("abc"), "abc");
        assert_eq!(shorten_id("abcdefghijklmnop"), "abcdef...klmnop");
    }
}
