// 📊 CCR 统计命令实现
// 提供成本、使用情况等统计功能

use crate::core::ColorOutput;
use crate::core::error::{CcrError, Result};
use crate::managers::CostTracker;
use crate::models::stats::{CostRecord, TokenUsage};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use clap::{Args, Subcommand};
use std::fs;
use std::path::PathBuf;

/// 📊 统计命令
#[derive(Args)]
pub struct StatsArgs {
    #[command(subcommand)]
    pub command: StatsSubcommand,
}

/// 📋 统计子命令
#[derive(Subcommand)]
pub enum StatsSubcommand {
    /// 📊 显示成本统计摘要
    ///
    /// 示例:
    ///   ccr stats summary
    ///   ccr stats summary --range week
    ///   ccr stats summary --by-model --by-project
    Summary(SummaryArgs),

    /// 📥 导入 CSV 格式成本数据
    ///
    /// 示例:
    ///   ccr stats import costs.csv
    ///   ccr stats import costs.csv --format claude-hub
    Import(ImportArgs),

    /// 📤 导出统计数据
    ///
    /// 示例:
    ///   ccr stats export
    ///   ccr stats export --format csv --output stats.csv
    ///   ccr stats export --format json --output stats.json
    Export(ExportArgs),

    /// 🗑️  清理历史数据
    ///
    /// 示例:
    ///   ccr stats clear --before 2025-01-01
    ///   ccr stats clear --before 2025-01-01 --force
    Clear(ClearArgs),

    /// 💰 成本统计 (已废弃,请使用 summary)
    #[deprecated(since = "3.10.3", note = "请使用 `ccr stats summary` 代替")]
    Cost(SummaryArgs),
}

/// 📊 统计摘要参数
#[derive(Args)]
pub struct SummaryArgs {
    /// 📅 时间范围: today, week, month, custom
    #[arg(long, default_value = "today")]
    pub range: String,

    /// 📅 自定义开始日期 (YYYY-MM-DD)
    #[arg(long)]
    pub start: Option<String>,

    /// 📅 自定义结束日期 (YYYY-MM-DD)
    #[arg(long)]
    pub end: Option<String>,

    /// 🤖 按模型分组
    #[arg(long)]
    pub by_model: bool,

    /// 📁 按项目分组
    #[arg(long)]
    pub by_project: bool,

    /// 🏷️  按平台分组
    #[arg(long)]
    pub by_platform: bool,

    /// 🏆 显示 Top N 会话
    #[arg(long)]
    pub top: Option<usize>,

    /// 📊 显示详细信息
    #[arg(long)]
    pub details: bool,
}

/// 📥 导入参数
#[derive(Args)]
pub struct ImportArgs {
    /// 📄 CSV 文件路径
    pub csv_file: PathBuf,

    /// 📋 导入格式: auto, claude-hub, custom
    #[arg(long, default_value = "auto")]
    pub format: String,

    /// ✅ 跳过验证
    #[arg(long)]
    pub skip_validation: bool,
}

/// 📤 导出参数
#[derive(Args)]
pub struct ExportArgs {
    /// 📋 导出格式: csv, json
    #[arg(long, default_value = "json")]
    pub format: String,

    /// 📄 输出文件路径
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// 📅 时间范围: today, week, month, custom
    #[arg(long)]
    pub range: Option<String>,

    /// 📅 自定义开始日期 (YYYY-MM-DD)
    #[arg(long)]
    pub start: Option<String>,

    /// 📅 自定义结束日期 (YYYY-MM-DD)
    #[arg(long)]
    pub end: Option<String>,
}

/// 🗑️  清理参数
#[derive(Args)]
pub struct ClearArgs {
    /// 📅 清理此日期之前的数据 (YYYY-MM-DD)
    #[arg(long)]
    pub before: Option<String>,

    /// ✅ 强制删除，不询问确认
    #[arg(long)]
    pub force: bool,

    /// 🔍 模拟运行，不实际删除
    #[arg(long)]
    pub dry_run: bool,
}

/// 执行统计命令
pub async fn stats_command(args: StatsArgs, _color_output: &mut ColorOutput) -> Result<()> {
    match args.command {
        #[allow(deprecated)]
        StatsSubcommand::Cost(summary_args) | StatsSubcommand::Summary(summary_args) => {
            summary_command(summary_args).await
        }
        StatsSubcommand::Import(import_args) => import_command(import_args).await,
        StatsSubcommand::Export(export_args) => export_command(export_args).await,
        StatsSubcommand::Clear(clear_args) => clear_command(clear_args).await,
    }
}

/// 📊 执行统计摘要命令
async fn summary_command(args: SummaryArgs) -> Result<()> {
    let storage_dir = CostTracker::default_storage_dir()?;
    let tracker = CostTracker::new(storage_dir)?;

    // 解析时间范围
    let (start, end) = parse_time_range(&args.range, args.start.as_deref(), args.end.as_deref())?;

    // 生成统计
    let stats = tracker.generate_stats(start, end)?;

    // 显示基本统计
    ColorOutput::title(&format!("📊 成本统计 - {}", args.range));
    println!();

    ColorOutput::info(&format!("💰 总成本: ${:.4}", stats.total_cost));
    ColorOutput::info(&format!("📊 记录数: {}", stats.record_count));
    ColorOutput::info(&format!(
        "📅 时间范围: {} 至 {}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
    ));
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

    // 按平台分组
    if args.by_platform || args.details {
        ColorOutput::success("🏷️  按平台分组:");
        if stats.by_provider.is_empty() {
            println!("  (无数据)");
        } else {
            let mut providers: Vec<_> = stats.by_provider.iter().collect();
            providers.sort_by(|a, b| b.1.cmp(a.1));

            for (provider, count) in providers {
                println!("  • {}: {} 次调用", provider, count);
            }
        }
        println!();
    }

    // 按模型分组
    if args.by_model || args.details {
        ColorOutput::success("🤖 按模型分组:");
        if stats.by_model.is_empty() {
            println!("  (无数据)");
        } else {
            let mut models: Vec<_> = stats.by_model.iter().collect();
            models.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (model, cost) in models {
                let short_model = shorten_model_name(model);
                println!("  • {}: ${:.4}", short_model, cost);
            }
        }
        println!();
    }

    // 按项目分组
    if args.by_project || args.details {
        ColorOutput::success("📁 按项目分组:");
        if stats.by_project.is_empty() {
            println!("  (无数据)");
        } else {
            let mut projects: Vec<_> = stats.by_project.iter().collect();
            projects.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (project, cost) in projects.iter().take(10) {
                let short_project = shorten_path(project);
                println!("  • {}: ${:.4}", short_project, cost);
            }
        }
        println!();
    }

    // Top 会话
    if let Some(limit) = args.top {
        ColorOutput::success(&format!("🏆 成本最高的 {} 个会话:", limit));
        let top_sessions = tracker.get_top_sessions(limit)?;

        if top_sessions.is_empty() {
            println!("  (无数据)");
        } else {
            for (i, (session_id, cost)) in top_sessions.iter().enumerate() {
                let short_id = shorten_id(session_id);
                println!("  {}. {}: ${:.4}", i + 1, short_id, cost);
            }
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

    Ok(())
}

/// 📥 执行导入命令
async fn import_command(args: ImportArgs) -> Result<()> {
    ColorOutput::title("📥 导入成本数据");
    println!();

    // 检查文件是否存在
    if !args.csv_file.exists() {
        return Err(CcrError::ValidationError(format!(
            "文件不存在: {}",
            args.csv_file.display()
        )));
    }

    ColorOutput::info(&format!("📄 文件: {}", args.csv_file.display()));
    ColorOutput::info(&format!("📋 格式: {}", args.format));
    println!();

    // 读取 CSV 文件
    let content = fs::read_to_string(&args.csv_file)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err(CcrError::ValidationError("文件为空".to_string()));
    }

    ColorOutput::info(&format!("📊 行数: {}", lines.len()));

    // 解析 CSV 数据
    let records = parse_csv_import(&lines, &args.format, args.skip_validation)?;

    ColorOutput::success(&format!("✅ 解析成功: {} 条记录", records.len()));
    println!();

    // 导入到 CostTracker
    let storage_dir = CostTracker::default_storage_dir()?;
    let tracker = CostTracker::new(storage_dir)?;

    for (i, record) in records.iter().enumerate() {
        tracker.record(
            record.session_id.clone(),
            record.project.clone(),
            record.model.clone(),
            record.token_usage.clone(),
            record.duration_ms,
            record.platform.clone(),
            record.description.clone(),
        )?;

        if (i + 1) % 100 == 0 {
            println!("  进度: {}/{}", i + 1, records.len());
        }
    }

    ColorOutput::success(&format!("✅ 导入完成: {} 条记录", records.len()));

    Ok(())
}

/// 📤 执行导出命令
async fn export_command(args: ExportArgs) -> Result<()> {
    ColorOutput::title("📤 导出统计数据");
    println!();

    let storage_dir = CostTracker::default_storage_dir()?;
    let tracker = CostTracker::new(storage_dir)?;

    // 解析时间范围
    let (start, end) = if let Some(range) = &args.range {
        parse_time_range(range, args.start.as_deref(), args.end.as_deref())?
    } else {
        // 默认导出全部
        let now = Utc::now();
        let start = now - Duration::days(365); // 最近一年
        (start, now)
    };

    // 生成统计
    let stats = tracker.generate_stats(start, end)?;

    ColorOutput::info(&format!("📋 格式: {}", args.format));
    ColorOutput::info(&format!(
        "📅 时间范围: {} 至 {}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
    ));
    ColorOutput::info(&format!("📊 记录数: {}", stats.record_count));
    println!();

    // 生成输出内容
    let content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&stats)?,
        "csv" => export_stats_to_csv(&stats)?,
        _ => {
            return Err(CcrError::ValidationError(format!(
                "不支持的格式: {}",
                args.format
            )));
        }
    };

    // 输出到文件或标准输出
    if let Some(output_path) = &args.output {
        fs::write(output_path, content)?;
        ColorOutput::success(&format!("✅ 已导出到: {}", output_path.display()));
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// 🗑️  执行清理命令
async fn clear_command(args: ClearArgs) -> Result<()> {
    ColorOutput::title("🗑️  清理历史数据");
    println!();

    let storage_dir = CostTracker::default_storage_dir()?;

    // 解析日期
    let before_date = if let Some(before) = &args.before {
        parse_date(before)?
    } else {
        // 默认清理 30 天前的数据
        Utc::now() - Duration::days(30)
    };

    ColorOutput::info(&format!(
        "📅 清理日期: {} 之前",
        before_date.format("%Y-%m-%d")
    ));

    // 扫描 stats 目录
    if !storage_dir.exists() {
        ColorOutput::warning("目录不存在，无需清理");
        return Ok(());
    }

    let entries = fs::read_dir(&storage_dir)?;
    let mut files_to_delete = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            let name = filename.to_string_lossy();
            if name.starts_with("costs_") && name.ends_with(".csv") {
                // 提取日期 costs_YYYYMM.csv
                if let Some(date_part) = name
                    .strip_prefix("costs_")
                    .and_then(|s| s.strip_suffix(".csv"))
                    && let Ok(year) = date_part[0..4].parse::<i32>()
                    && let Ok(month) = date_part[4..6].parse::<u32>()
                {
                    // 比较日期
                    let file_date = before_date
                        .with_year(year)
                        .and_then(|d| d.with_month(month))
                        .ok_or_else(|| CcrError::ValidationError("无效的日期".to_string()))?;

                    if file_date < before_date {
                        files_to_delete.push(path);
                    }
                }
            }
        }
    }

    if files_to_delete.is_empty() {
        ColorOutput::warning("没有找到需要清理的文件");
        return Ok(());
    }

    ColorOutput::info(&format!("📊 找到 {} 个文件", files_to_delete.len()));
    for path in &files_to_delete {
        println!("  • {}", path.display());
    }
    println!();

    if args.dry_run {
        ColorOutput::warning("🔍 模拟运行模式，未实际删除");
        return Ok(());
    }

    // 确认删除
    if !args.force {
        use std::io::{self, Write};
        print!("确认删除这些文件吗? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            ColorOutput::warning("已取消");
            return Ok(());
        }
    }

    // 执行删除
    for path in &files_to_delete {
        fs::remove_file(path)?;
    }

    ColorOutput::success(&format!("✅ 已删除 {} 个文件", files_to_delete.len()));

    Ok(())
}

// ============================================================
// 辅助函数
// ============================================================

/// 解析时间范围
fn parse_time_range(
    range: &str,
    start_str: Option<&str>,
    end_str: Option<&str>,
) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    match range {
        "today" => {
            let now = Utc::now();
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .expect("无效的日期时间")
                .and_utc();
            Ok((start, now))
        }
        "week" => {
            let now = Utc::now();
            let start = now - Duration::days(7);
            Ok((start, now))
        }
        "month" => {
            let now = Utc::now();
            let start = now
                .date_naive()
                .with_day(1)
                .expect("无法设置日期为每月第一天")
                .and_hms_opt(0, 0, 0)
                .expect("无效的日期时间")
                .and_utc();
            Ok((start, now))
        }
        "custom" => {
            let start = start_str
                .ok_or_else(|| CcrError::ValidationError("需要提供 --start 参数".to_string()))
                .and_then(parse_date)?;
            let end = end_str
                .ok_or_else(|| CcrError::ValidationError("需要提供 --end 参数".to_string()))
                .and_then(parse_date)?;
            Ok((start, end))
        }
        _ => Err(CcrError::ValidationError(
            "无效的时间范围，请使用: today, week, month, custom".to_string(),
        )),
    }
}

/// 解析日期字符串 (YYYY-MM-DD)
fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err(CcrError::ValidationError(format!(
            "无效的日期格式: {}，应为 YYYY-MM-DD",
            date_str
        )));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| CcrError::ValidationError(format!("无效的年份: {}", parts[0])))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| CcrError::ValidationError(format!("无效的月份: {}", parts[1])))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| CcrError::ValidationError(format!("无效的日期: {}", parts[2])))?;

    let date = Utc::now()
        .with_year(year)
        .and_then(|d| d.with_month(month))
        .and_then(|d| d.with_day(day))
        .and_then(|d| d.with_hour(0))
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .ok_or_else(|| CcrError::ValidationError(format!("无效的日期: {}", date_str)))?;

    Ok(date)
}

/// 解析导入的 CSV 数据
fn parse_csv_import(
    lines: &[&str],
    format: &str,
    skip_validation: bool,
) -> Result<Vec<CostRecord>> {
    let mut records = Vec::new();

    // 跳过表头
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let record = match format {
            "auto" | "custom" => parse_csv_line_standard(line, i + 1, skip_validation)?,
            "claude-hub" => parse_csv_line_claude_hub(line, i + 1, skip_validation)?,
            _ => {
                return Err(CcrError::ValidationError(format!(
                    "不支持的格式: {}",
                    format
                )));
            }
        };

        records.push(record);
    }

    Ok(records)
}

/// 解析标准 CSV 行
fn parse_csv_line_standard(
    line: &str,
    line_num: usize,
    _skip_validation: bool,
) -> Result<CostRecord> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() < 16 {
        return Err(CcrError::ValidationError(format!(
            "第 {} 行格式不正确: 列数不足",
            line_num
        )));
    }

    let timestamp = DateTime::parse_from_rfc3339(parts[0])
        .map_err(|_| CcrError::ValidationError(format!("第 {} 行时间戳格式不正确", line_num)))?
        .with_timezone(&Utc);

    Ok(CostRecord {
        id: parts[1].to_string(),
        timestamp,
        session_id: if parts[2].is_empty() {
            None
        } else {
            Some(parts[2].to_string())
        },
        project: parts[3].to_string(),
        platform: if parts[4].is_empty() {
            None
        } else {
            Some(parts[4].to_string())
        },
        model: parts[5].to_string(),
        token_usage: TokenUsage {
            input_tokens: parts[6].parse().unwrap_or(0),
            output_tokens: parts[7].parse().unwrap_or(0),
            cache_read_tokens: parts[8].parse().ok(),
            cache_creation_tokens: parts[9].parse().ok(),
        },
        cost: crate::models::stats::Cost {
            input_cost: parts[10].parse().unwrap_or(0.0),
            output_cost: parts[11].parse().unwrap_or(0.0),
            cache_cost: parts[12].parse().unwrap_or(0.0),
            total_cost: parts[13].parse().unwrap_or(0.0),
        },
        duration_ms: parts[14].parse().unwrap_or(0),
        description: if parts.len() > 15 && !parts[15].is_empty() {
            Some(parts[15].to_string())
        } else {
            None
        },
    })
}

/// 解析 Claude Hub 格式的 CSV 行
fn parse_csv_line_claude_hub(
    line: &str,
    line_num: usize,
    skip_validation: bool,
) -> Result<CostRecord> {
    // TODO: 实现 Claude Hub 特定的解析逻辑
    // 目前先使用标准格式
    parse_csv_line_standard(line, line_num, skip_validation)
}

/// 导出统计数据为 CSV 格式
fn export_stats_to_csv(stats: &crate::models::stats::CostStats) -> Result<String> {
    let mut csv = String::new();

    // 基本统计
    csv.push_str("指标,值\n");
    csv.push_str(&format!("总成本,{:.4}\n", stats.total_cost));
    csv.push_str(&format!("记录数,{}\n", stats.record_count));
    csv.push_str(&format!(
        "输入 Tokens,{}\n",
        stats.token_stats.total_input_tokens
    ));
    csv.push_str(&format!(
        "输出 Tokens,{}\n",
        stats.token_stats.total_output_tokens
    ));
    csv.push_str(&format!(
        "Cache Tokens,{}\n",
        stats.token_stats.total_cache_tokens
    ));
    csv.push_str(&format!(
        "Cache 效率,{:.2}%\n",
        stats.token_stats.cache_efficiency
    ));
    csv.push('\n');

    // 按模型分组
    csv.push_str("模型,成本\n");
    for (model, cost) in &stats.by_model {
        csv.push_str(&format!("{},{:.4}\n", model, cost));
    }
    csv.push('\n');

    // 按项目分组
    csv.push_str("项目,成本\n");
    for (project, cost) in &stats.by_project {
        csv.push_str(&format!("{},{:.4}\n", project, cost));
    }

    Ok(csv)
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn test_parse_date() {
        let date = parse_date("2025-01-15").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }
}
