// 💰 CCR 预算命令实现
// 提供预算配置和监控功能

use crate::core::ColorOutput;
use crate::core::error::{CcrError, Result};
use crate::managers::{BudgetManager, CostTracker};
use crate::models::budget::BudgetPeriod;
use clap::{Args, Subcommand};
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};

/// 💰 预算命令
#[derive(Args, Clone)]
pub struct BudgetArgs {
    #[command(subcommand)]
    pub command: BudgetSubcommand,
}

/// 📋 预算子命令
#[derive(Subcommand, Clone)]
pub enum BudgetSubcommand {
    /// 📊 显示当前预算状态
    ///
    /// 示例:
    ///   ccr budget status
    Status,

    /// ⚙️ 配置预算限制
    ///
    /// 示例:
    ///   ccr budget set --daily 10.0
    ///   ccr budget set --weekly 50.0 --monthly 200.0
    ///   ccr budget set --warn-at 90
    ///   ccr budget set --enable
    Set(SetArgs),

    /// 🗑️ 重置所有预算限制
    ///
    /// 示例:
    ///   ccr budget reset
    ///   ccr budget reset --force
    Reset(ResetArgs),
}

/// ⚙️ 预算配置参数
#[derive(Args, Clone)]
pub struct SetArgs {
    /// 📅 每日预算限制（美元）
    #[arg(long)]
    pub daily: Option<f64>,

    /// 📅 每周预算限制（美元）
    #[arg(long)]
    pub weekly: Option<f64>,

    /// 📅 每月预算限制（美元）
    #[arg(long)]
    pub monthly: Option<f64>,

    /// ⚠️ 警告阈值百分比 (0-100)
    #[arg(long)]
    pub warn_at: Option<u8>,

    /// ✅ 启用预算控制
    #[arg(long)]
    pub enable: bool,

    /// ❌ 禁用预算控制
    #[arg(long)]
    pub disable: bool,
}

/// 🗑️ 重置参数
#[derive(Args, Clone)]
pub struct ResetArgs {
    /// 强制重置，无需确认
    #[arg(long)]
    pub force: bool,
}

/// 执行预算命令
pub async fn budget_command(args: BudgetArgs) -> Result<()> {
    match args.command {
        BudgetSubcommand::Status => status_command().await,
        BudgetSubcommand::Set(set_args) => set_command(set_args).await,
        BudgetSubcommand::Reset(reset_args) => reset_command(reset_args).await,
    }
}

/// 📊 显示预算状态
async fn status_command() -> Result<()> {
    // 加载预算管理器和成本追踪器
    let budget_manager = BudgetManager::with_default()?;
    let storage_dir = CostTracker::default_storage_dir()?;
    let tracker = CostTracker::new(storage_dir)?;

    // 获取预算状态
    let status = budget_manager.check_status(&tracker)?;

    ColorOutput::title("💰 预算状态");

    // 显示启用状态
    if status.enabled {
        ColorOutput::success("✅ 预算控制已启用");
    } else {
        ColorOutput::warning("⚠️  预算控制已禁用");
        ColorOutput::info("使用 `ccr budget set --enable` 启用预算控制");
        return Ok(());
    }

    println!();

    // 创建成本表格
    let mut cost_table = Table::new();
    cost_table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("周期").fg(Color::Cyan),
            Cell::new("当前成本").fg(Color::Cyan),
            Cell::new("预算限制").fg(Color::Cyan),
            Cell::new("使用率").fg(Color::Cyan),
            Cell::new("状态").fg(Color::Cyan),
        ]);

    // 添加每日数据
    add_period_row(
        &mut cost_table,
        "📅 每日",
        status.current_costs.today,
        status.limits.daily,
    );

    // 添加每周数据
    add_period_row(
        &mut cost_table,
        "📅 每周",
        status.current_costs.this_week,
        status.limits.weekly,
    );

    // 添加每月数据
    add_period_row(
        &mut cost_table,
        "📅 每月",
        status.current_costs.this_month,
        status.limits.monthly,
    );

    println!("{cost_table}");

    // 显示警告
    if !status.warnings.is_empty() {
        println!();
        ColorOutput::title("⚠️  预算警告");
        for warning in &status.warnings {
            let period_str = match warning.period {
                BudgetPeriod::Daily => "每日",
                BudgetPeriod::Weekly => "每周",
                BudgetPeriod::Monthly => "每月",
            };

            if warning.usage_percent >= 100.0 {
                ColorOutput::error(&format!(
                    "❌ {} 预算已超出限制！当前: ${:.2}, 限制: ${:.2} ({:.1}%)",
                    period_str, warning.current_cost, warning.limit, warning.usage_percent
                ));
            } else {
                ColorOutput::warning(&format!(
                    "⚠️  {} 预算使用已达 {:.1}%！当前: ${:.2}, 限制: ${:.2}",
                    period_str, warning.usage_percent, warning.current_cost, warning.limit
                ));
            }
        }
    }

    println!();
    ColorOutput::info(&format!(
        "最后更新: {}",
        status.last_updated.format("%Y-%m-%d %H:%M:%S")
    ));

    Ok(())
}

/// 添加周期行到表格
fn add_period_row(table: &mut Table, period: &str, current: f64, limit: Option<f64>) {
    let current_str = format!("${:.2}", current);

    let (limit_str, usage_str, status_str) = if let Some(limit_val) = limit {
        let usage_percent = (current / limit_val) * 100.0;
        let limit_str = format!("${:.2}", limit_val);
        let usage_str = format!("{:.1}%", usage_percent);

        let status_str = if usage_percent >= 100.0 {
            "❌ 超出"
        } else if usage_percent >= 90.0 {
            "⚠️  警告"
        } else if usage_percent >= 75.0 {
            "⚡ 接近"
        } else {
            "✅ 正常"
        };

        (limit_str, usage_str, status_str.to_string())
    } else {
        ("无限制".to_string(), "-".to_string(), "✅ 正常".to_string())
    };

    table.add_row(vec![
        Cell::new(period),
        Cell::new(current_str).set_alignment(CellAlignment::Right),
        Cell::new(limit_str).set_alignment(CellAlignment::Right),
        Cell::new(usage_str).set_alignment(CellAlignment::Right),
        Cell::new(status_str),
    ]);
}

/// ⚙️ 配置预算限制
async fn set_command(args: SetArgs) -> Result<()> {
    // 检查冲突参数
    if args.enable && args.disable {
        return Err(CcrError::ValidationError(
            "不能同时使用 --enable 和 --disable 选项".to_string(),
        ));
    }

    let mut manager = BudgetManager::with_default()?;
    let mut changed = false;

    // 启用/禁用预算控制
    if args.enable {
        manager.enable()?;
        ColorOutput::success("✅ 预算控制已启用");
        changed = true;
    }

    if args.disable {
        manager.disable()?;
        ColorOutput::warning("⚠️  预算控制已禁用");
        changed = true;
    }

    // 设置预算限制
    if let Some(daily) = args.daily {
        manager.set_daily_limit(Some(daily))?;
        ColorOutput::success(&format!("✅ 每日预算限制已设置为: ${:.2}", daily));
        changed = true;
    }

    if let Some(weekly) = args.weekly {
        manager.set_weekly_limit(Some(weekly))?;
        ColorOutput::success(&format!("✅ 每周预算限制已设置为: ${:.2}", weekly));
        changed = true;
    }

    if let Some(monthly) = args.monthly {
        manager.set_monthly_limit(Some(monthly))?;
        ColorOutput::success(&format!("✅ 每月预算限制已设置为: ${:.2}", monthly));
        changed = true;
    }

    if let Some(warn_at) = args.warn_at {
        manager.set_warn_threshold(warn_at)?;
        ColorOutput::success(&format!("✅ 警告阈值已设置为: {}%", warn_at));
        changed = true;
    }

    if !changed {
        ColorOutput::warning("⚠️  未指定任何配置项");
        ColorOutput::info("使用 `ccr budget set --help` 查看可用选项");
        return Ok(());
    }

    println!();
    ColorOutput::info("💡 提示: 使用 `ccr budget status` 查看当前预算状态");

    Ok(())
}

/// 🗑️ 重置预算限制
async fn reset_command(args: ResetArgs) -> Result<()> {
    // 如果没有 --force，询问确认
    if !args.force {
        ColorOutput::warning("⚠️  这将重置所有预算限制配置！");
        ColorOutput::info("当前配置将被清除，预算控制保持当前启用状态");

        let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
            print!("\n确认重置？(y/N): ");
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim().to_lowercase();
            Ok(input == "y" || input == "yes")
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

        if !confirmed {
            ColorOutput::info("✅ 已取消重置");
            return Ok(());
        }
    }

    let mut manager = BudgetManager::with_default()?;
    manager.reset_limits()?;

    ColorOutput::success("✅ 预算限制已重置");
    ColorOutput::info("💡 提示: 预算控制状态未改变，使用 `ccr budget set --enable/--disable` 修改");

    Ok(())
}
