//! 📚 Sessions 命令
//!
//! 提供 Session 管理相关的 CLI 命令。

#![allow(clippy::unused_async)]

use crate::models::Platform;
use ccr_core::core::ColorOutput;
use ccr_core::core::error::{CcrError, Result};
use ccr_store::sessions::models::SessionFilter;
use ccr_store::sessions::{SessionIndexer, SessionSummary};
use clap::{Args, Subcommand};
use comfy_table::{Cell, Color, Table, presets::UTF8_FULL};

/// Sessions 命令参数
#[derive(Args, Debug, Clone)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommand,
}

/// Sessions 子命令
#[derive(Subcommand, Debug, Clone)]
pub enum SessionsCommand {
    /// 列出 sessions
    #[command(alias = "ls")]
    List {
        /// 平台过滤 (claude, codex, gemini)
        #[arg(short, long)]
        platform: Option<String>,

        /// 限制数量
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// 仅显示今天的 sessions
        #[arg(long)]
        today: bool,
    },

    /// 搜索 sessions
    Search {
        /// 搜索关键词
        query: String,

        /// 平台过滤
        #[arg(short, long)]
        platform: Option<String>,

        /// 限制数量
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 查看 session 详情
    Show {
        /// Session ID
        session_id: String,
    },

    /// 生成恢复 session 的命令
    Resume {
        /// Session ID
        session_id: String,

        /// 仅打印命令，不执行
        #[arg(long)]
        dry_run: bool,
    },

    /// 重建索引
    Reindex {
        /// 强制重建（清空后重新索引）
        #[arg(long)]
        force: bool,

        /// 指定平台
        #[arg(short, long)]
        platform: Option<String>,
    },

    /// 显示索引统计
    Stats,

    /// 清理过期 sessions（文件已删除）
    Prune {
        /// 跳过确认
        #[arg(long)]
        confirm: bool,
    },
}

/// 执行 sessions 命令
pub async fn execute(args: SessionsArgs) -> Result<()> {
    match args.command {
        SessionsCommand::List {
            platform,
            limit,
            today,
        } => cmd_list(platform, limit, today),
        SessionsCommand::Search {
            query,
            platform,
            limit,
        } => cmd_search(&query, platform, limit),
        SessionsCommand::Show { session_id } => cmd_show(&session_id),
        SessionsCommand::Resume {
            session_id,
            dry_run,
        } => cmd_resume(&session_id, dry_run),
        SessionsCommand::Reindex { force, platform } => cmd_reindex(force, platform),
        SessionsCommand::Stats => cmd_stats(),
        SessionsCommand::Prune { confirm } => cmd_prune(confirm).await,
    }
}

/// 列出 sessions
fn cmd_list(platform: Option<String>, limit: usize, today: bool) -> Result<()> {
    let indexer = SessionIndexer::new()?;

    // 先确保索引是最新的
    let _ = indexer.index_all();

    let mut filter = if today {
        SessionFilter::today()
    } else {
        SessionFilter::default()
    };

    filter.limit = Some(limit);

    if let Some(ref p) = platform {
        filter.platform = parse_platform(p);
    }

    let sessions = indexer.list(filter)?;

    if sessions.is_empty() {
        ColorOutput::warning("未找到任何 session");
        ColorOutput::info("提示: 运行 'ccr sessions reindex' 重建索引");
        return Ok(());
    }

    print_sessions_table(&sessions);

    ColorOutput::info(&format!("共 {} 个 session", sessions.len()));

    Ok(())
}

/// 搜索 sessions
fn cmd_search(query: &str, platform: Option<String>, limit: usize) -> Result<()> {
    let indexer = SessionIndexer::new()?;

    let mut sessions = indexer.search(query, limit)?;

    // 平台过滤
    if let Some(ref p) = platform
        && let Some(platform_filter) = parse_platform(p)
    {
        sessions.retain(|s| s.platform == platform_filter);
    }

    if sessions.is_empty() {
        ColorOutput::warning(&format!("未找到匹配 '{}' 的 session", query));
        return Ok(());
    }

    print_sessions_table(&sessions);

    ColorOutput::info(&format!("找到 {} 个匹配的 session", sessions.len()));

    Ok(())
}

/// 查看 session 详情
fn cmd_show(session_id: &str) -> Result<()> {
    let indexer = SessionIndexer::new()?;

    let session = indexer.get(session_id)?;

    match session {
        Some(s) => {
            println!();
            ColorOutput::title("Session 详情");
            println!();

            let mut table = Table::new();
            table.load_preset(UTF8_FULL);

            table.add_row(vec![Cell::new("ID").fg(Color::Cyan), Cell::new(&s.id)]);
            table.add_row(vec![
                Cell::new("平台").fg(Color::Cyan),
                Cell::new(format!("{:?}", s.platform)),
            ]);
            table.add_row(vec![
                Cell::new("标题").fg(Color::Cyan),
                Cell::new(s.title.as_deref().unwrap_or("-")),
            ]);
            table.add_row(vec![
                Cell::new("工作目录").fg(Color::Cyan),
                Cell::new(s.cwd.display().to_string()),
            ]);
            table.add_row(vec![
                Cell::new("文件路径").fg(Color::Cyan),
                Cell::new(s.file_path.display().to_string()),
            ]);
            table.add_row(vec![
                Cell::new("创建时间").fg(Color::Cyan),
                Cell::new(s.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]);
            table.add_row(vec![
                Cell::new("更新时间").fg(Color::Cyan),
                Cell::new(s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]);
            table.add_row(vec![
                Cell::new("消息数").fg(Color::Cyan),
                Cell::new(format!(
                    "总计 {} (用户: {}, 助手: {})",
                    s.message_count, s.user_message_count, s.assistant_message_count
                )),
            ]);
            table.add_row(vec![
                Cell::new("工具调用").fg(Color::Cyan),
                Cell::new(s.tool_use_count.to_string()),
            ]);

            println!("{}", table);
            println!();

            // 显示恢复命令
            ColorOutput::info(&format!("恢复命令: {}", s.resume_command()));
        }
        None => {
            ColorOutput::error(&format!("未找到 session: {}", session_id));
        }
    }

    Ok(())
}

/// 生成恢复命令
fn cmd_resume(session_id: &str, dry_run: bool) -> Result<()> {
    let indexer = SessionIndexer::new()?;

    let session = indexer.get(session_id)?;

    match session {
        Some(s) => {
            let cmd = s.resume_command();

            if dry_run {
                println!("{}", cmd);
            } else {
                ColorOutput::info(&format!("执行: {}", cmd));
                ColorOutput::warning("注意: 自动执行功能尚未实现，请手动运行上述命令");
                println!();
                println!("  {}", cmd);
            }
        }
        None => {
            ColorOutput::error(&format!("未找到 session: {}", session_id));
        }
    }

    Ok(())
}

/// 重建索引
fn cmd_reindex(force: bool, platform: Option<String>) -> Result<()> {
    let indexer = SessionIndexer::new()?;

    ColorOutput::info("开始索引 sessions...");

    let stats = if force {
        ColorOutput::warning("强制重建模式：清空现有索引");
        indexer.rebuild()?
    } else if let Some(ref p) = platform {
        if let Some(platform_filter) = parse_platform(p) {
            indexer.index_platform(platform_filter)?
        } else {
            ColorOutput::error(&format!("未知平台: {}", p));
            return Ok(());
        }
    } else {
        indexer.index_all()?
    };

    println!();
    ColorOutput::success("索引完成");
    println!();
    println!("  扫描文件: {}", stats.files_scanned);
    println!("  新增: {}", stats.sessions_added);
    println!("  更新: {}", stats.sessions_updated);
    println!("  跳过: {}", stats.files_skipped);
    println!("  错误: {}", stats.errors);
    println!("  耗时: {} ms", stats.duration_ms);

    Ok(())
}

/// 显示统计
fn cmd_stats() -> Result<()> {
    let indexer = SessionIndexer::new()?;
    let stats = indexer.stats()?;

    println!();
    ColorOutput::title("Session 索引统计");
    println!();
    println!("  总数: {}", stats.total);

    if !stats.by_platform.is_empty() {
        println!();
        println!("  按平台:");
        for (platform, count) in &stats.by_platform {
            println!("    {}: {}", platform, count);
        }
    }

    Ok(())
}

/// 清理过期 sessions
async fn cmd_prune(confirm: bool) -> Result<()> {
    if !confirm {
        ColorOutput::warning("将删除文件已不存在的 session 记录");
        ColorOutput::info("使用 --confirm 跳过确认");

        // 简单确认
        println!();
        let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
            print!("是否继续? (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

        if !confirmed {
            ColorOutput::info("已取消");
            return Ok(());
        }
    }

    let indexer = SessionIndexer::new()?;
    let count = indexer.prune_stale()?;

    ColorOutput::success(&format!("已清理 {} 个过期 session", count));

    Ok(())
}

/// 打印 sessions 表格
fn print_sessions_table(sessions: &[SessionSummary]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("ID").fg(Color::Cyan),
        Cell::new("平台").fg(Color::Cyan),
        Cell::new("标题").fg(Color::Cyan),
        Cell::new("消息").fg(Color::Cyan),
        Cell::new("时间").fg(Color::Cyan),
    ]);

    for session in sessions {
        let platform_color = match session.platform {
            Platform::Claude => Color::Magenta,
            Platform::Codex => Color::Green,
            Platform::Gemini => Color::Blue,
            _ => Color::White,
        };

        // 截断 ID
        let short_id = if session.id.len() > 12 {
            format!("{}...", &session.id[..12])
        } else {
            session.id.clone()
        };

        // 截断标题
        let title = session.display_title();
        let short_title = if title.len() > 40 {
            format!("{}...", &title[..37])
        } else {
            title.to_string()
        };

        table.add_row(vec![
            Cell::new(short_id),
            Cell::new(format!("{:?}", session.platform)).fg(platform_color),
            Cell::new(short_title),
            Cell::new(session.message_count.to_string()),
            Cell::new(session.relative_time()),
        ]);
    }

    println!();
    println!("{}", table);
    println!();
}

/// 解析平台字符串
fn parse_platform(s: &str) -> Option<Platform> {
    match s.to_lowercase().as_str() {
        "claude" => Some(Platform::Claude),
        "codex" => Some(Platform::Codex),
        "gemini" => Some(Platform::Gemini),
        "qwen" => Some(Platform::Qwen),
        _ => None,
    }
}
