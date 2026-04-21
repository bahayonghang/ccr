// 💰 CCR 价格表命令实现
// 提供模型定价配置和管理功能

use crate::managers::PricingManager;
use crate::models::stats::ModelPricing;
use ccr_core::core::ColorOutput;
use ccr_core::core::error::{CcrError, Result};
use clap::{Args, Subcommand};
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};

/// 💰 价格表命令
#[derive(Args, Clone)]
pub struct PricingArgs {
    #[command(subcommand)]
    pub command: PricingSubcommand,
}

/// 📋 价格表子命令
#[derive(Subcommand, Clone)]
pub enum PricingSubcommand {
    /// 📊 列出所有模型定价
    ///
    /// 示例:
    ///   ccr pricing list
    ///   ccr pricing list --verbose
    List(ListArgs),

    /// ⚙️ 设置模型定价
    ///
    /// 示例:
    ///   ccr pricing set my-model --input 3.0 --output 15.0
    ///   ccr pricing set my-model --input 3.0 --output 15.0 --cache-read 0.3 --cache-write 3.75
    Set(SetArgs),

    /// 🗑️ 移除模型定价
    ///
    /// 示例:
    ///   ccr pricing remove my-model
    ///   ccr pricing remove my-model --force
    Remove(RemoveArgs),

    /// 🔄 重置为默认定价
    ///
    /// 示例:
    ///   ccr pricing reset
    ///   ccr pricing reset --force
    Reset(ResetArgs),
}

/// 📊 列表参数
#[derive(Args, Clone)]
pub struct ListArgs {
    /// 显示详细信息（包括缓存定价）
    #[arg(short, long)]
    pub verbose: bool,
}

/// ⚙️ 设置参数
#[derive(Args, Clone)]
pub struct SetArgs {
    /// 模型名称
    pub model: String,

    /// 输入 Token 价格（每百万 Token，美元）
    #[arg(long)]
    pub input: f64,

    /// 输出 Token 价格（每百万 Token，美元）
    #[arg(long)]
    pub output: f64,

    /// 缓存读取价格（每百万 Token，美元）
    #[arg(long)]
    pub cache_read: Option<f64>,

    /// 缓存写入价格（每百万 Token，美元）
    #[arg(long)]
    pub cache_write: Option<f64>,
}

/// 🗑️ 移除参数
#[derive(Args, Clone)]
pub struct RemoveArgs {
    /// 模型名称
    pub model: String,

    /// 强制移除，无需确认
    #[arg(long)]
    pub force: bool,
}

/// 🔄 重置参数
#[derive(Args, Clone)]
pub struct ResetArgs {
    /// 强制重置，无需确认
    #[arg(long)]
    pub force: bool,
}

/// 执行价格表命令
pub async fn pricing_command(args: PricingArgs) -> Result<()> {
    match args.command {
        PricingSubcommand::List(list_args) => list_command(list_args).await,
        PricingSubcommand::Set(set_args) => set_command(set_args).await,
        PricingSubcommand::Remove(remove_args) => remove_command(remove_args).await,
        PricingSubcommand::Reset(reset_args) => reset_command(reset_args).await,
    }
}

/// 📊 列出所有模型定价
async fn list_command(args: ListArgs) -> Result<()> {
    let manager = PricingManager::with_default()?;
    let config = manager.get_config();

    ColorOutput::title("💰 模型定价配置");

    // 检查是否有配置的模型
    if manager.is_empty() {
        ColorOutput::warning("⚠️  未配置任何模型定价");
        ColorOutput::info(
            "使用 `ccr pricing set <模型名> --input <价格> --output <价格>` 添加定价",
        );
        return Ok(());
    }

    println!();

    // 创建定价表格
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // 根据 verbose 参数决定表头
    if args.verbose {
        table.set_header(vec![
            Cell::new("模型名称").fg(Color::Cyan),
            Cell::new("输入价格").fg(Color::Cyan),
            Cell::new("输出价格").fg(Color::Cyan),
            Cell::new("缓存读取").fg(Color::Cyan),
            Cell::new("缓存写入").fg(Color::Cyan),
        ]);
    } else {
        table.set_header(vec![
            Cell::new("模型名称").fg(Color::Cyan),
            Cell::new("输入价格").fg(Color::Cyan),
            Cell::new("输出价格").fg(Color::Cyan),
        ]);
    }

    // 添加模型定价行
    let model_names = manager.model_names();
    for model_name in model_names {
        if let Some(pricing) = manager.get_pricing(&model_name) {
            if args.verbose {
                table.add_row(vec![
                    Cell::new(&pricing.model),
                    Cell::new(format!("${:.2}/M", pricing.input_price))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format!("${:.2}/M", pricing.output_price))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(
                        pricing
                            .cache_read_price
                            .map(|p| format!("${:.2}/M", p))
                            .unwrap_or_else(|| "-".to_string()),
                    )
                    .set_alignment(CellAlignment::Right),
                    Cell::new(
                        pricing
                            .cache_write_price
                            .map(|p| format!("${:.2}/M", p))
                            .unwrap_or_else(|| "-".to_string()),
                    )
                    .set_alignment(CellAlignment::Right),
                ]);
            } else {
                table.add_row(vec![
                    Cell::new(&pricing.model),
                    Cell::new(format!("${:.2}/M", pricing.input_price))
                        .set_alignment(CellAlignment::Right),
                    Cell::new(format!("${:.2}/M", pricing.output_price))
                        .set_alignment(CellAlignment::Right),
                ]);
            }
        }
    }

    println!("{table}");

    // 显示默认定价
    if let Some(default_pricing) = config.default_pricing.as_ref() {
        println!();
        ColorOutput::title("🔧 默认定价（用于未配置的模型）");
        println!();
        println!("  输入价格: ${:.2}/M", default_pricing.input_price);
        println!("  输出价格: ${:.2}/M", default_pricing.output_price);
        if let Some(cache_read) = default_pricing.cache_read_price {
            println!("  缓存读取: ${:.2}/M", cache_read);
        }
        if let Some(cache_write) = default_pricing.cache_write_price {
            println!("  缓存写入: ${:.2}/M", cache_write);
        }
    }

    println!();
    ColorOutput::info(&format!("共 {} 个模型定价配置", manager.model_count()));
    if !args.verbose {
        ColorOutput::info("💡 提示: 使用 --verbose 查看缓存定价详情");
    }

    Ok(())
}

/// ⚙️ 设置模型定价
async fn set_command(args: SetArgs) -> Result<()> {
    // 验证价格为正数
    if args.input < 0.0 || args.output < 0.0 {
        return Err(CcrError::ValidationError("定价不能为负数".to_string()));
    }

    if let Some(cache_read) = args.cache_read
        && cache_read < 0.0
    {
        return Err(CcrError::ValidationError(
            "缓存读取价格不能为负数".to_string(),
        ));
    }

    if let Some(cache_write) = args.cache_write
        && cache_write < 0.0
    {
        return Err(CcrError::ValidationError(
            "缓存写入价格不能为负数".to_string(),
        ));
    }

    let mut manager = PricingManager::with_default()?;

    let pricing = ModelPricing {
        model: args.model.clone(),
        input_price: args.input,
        output_price: args.output,
        cache_read_price: args.cache_read,
        cache_write_price: args.cache_write,
    };

    manager.set_pricing(args.model.clone(), pricing)?;

    ColorOutput::success(&format!("✅ 模型 {} 的定价已设置", args.model));
    println!();
    println!("  输入价格: ${:.2}/M", args.input);
    println!("  输出价格: ${:.2}/M", args.output);
    if let Some(cache_read) = args.cache_read {
        println!("  缓存读取: ${:.2}/M", cache_read);
    }
    if let Some(cache_write) = args.cache_write {
        println!("  缓存写入: ${:.2}/M", cache_write);
    }

    println!();
    ColorOutput::info("💡 提示: 使用 `ccr pricing list` 查看所有定价配置");

    Ok(())
}

/// 🗑️ 移除模型定价
async fn remove_command(args: RemoveArgs) -> Result<()> {
    // 如果没有 --force，询问确认
    if !args.force {
        ColorOutput::warning(&format!("⚠️  这将移除模型 {} 的定价配置！", args.model));
        ColorOutput::info("移除后，该模型将使用默认定价（如果已配置）");

        let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
            print!("\n确认移除？(y/N): ");
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
            ColorOutput::info("✅ 已取消移除");
            return Ok(());
        }
    }

    let mut manager = PricingManager::with_default()?;
    let removed = manager.remove_pricing(&args.model)?;

    if removed.is_some() {
        ColorOutput::success(&format!("✅ 模型 {} 的定价已移除", args.model));
    } else {
        ColorOutput::warning(&format!("⚠️  模型 {} 没有配置定价", args.model));
    }

    Ok(())
}

/// 🔄 重置为默认定价
async fn reset_command(args: ResetArgs) -> Result<()> {
    // 如果没有 --force，询问确认
    if !args.force {
        ColorOutput::warning("⚠️  这将重置所有定价配置为 Claude 默认值！");
        ColorOutput::info("所有自定义模型定价将被删除");

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

    let mut manager = PricingManager::with_default()?;
    manager.reset_to_defaults()?;

    ColorOutput::success("✅ 价格表已重置为 Claude 默认配置");
    ColorOutput::info("💡 提示: 使用 `ccr pricing list` 查看默认配置");

    Ok(())
}
