// CCR (Claude Code Configuration Switcher) 主程序
// 配置管理工具，支持完整审计追踪

mod commands;
mod config;
mod error;
mod history;
mod lock;
mod logging;
mod settings;
mod web;

use clap::{Parser, Subcommand};
use logging::{init_logger, ColorOutput};

/// Claude Code Configuration Switcher - 配置管理工具
#[derive(Parser)]
#[command(name = "ccr")]
#[command(about = "Claude Code Configuration Router - 配置管理工具", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 直接切换到指定配置（简写形式）
    config_name: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用配置
    #[command(alias = "ls")]
    List,

    /// 显示当前配置状态
    #[command(alias = "show")]
    #[command(alias = "status")]
    Current,

    /// 切换到指定配置
    Switch {
        /// 要切换到的配置名称
        config_name: String,
    },

    /// 验证配置和设置的完整性
    #[command(alias = "check")]
    Validate,

    /// 显示操作历史
    History {
        /// 限制显示的记录数量
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// 按操作类型筛选 (switch, backup, restore, validate, update)
        #[arg(short = 't', long)]
        filter_type: Option<String>,
    },

    /// 启动 Web 配置界面
    Web {
        /// 指定端口（默认 8080）
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// 显示版本信息
    #[command(alias = "ver")]
    Version,
}

fn main() {
    // 初始化日志系统
    init_logger();

    // 解析命令行参数
    let cli = Cli::parse();

    // 执行命令并处理错误
    let result = match cli.command {
        Some(Commands::List) => commands::list_command(),
        Some(Commands::Current) => commands::current_command(),
        Some(Commands::Switch { config_name }) => commands::switch_command(&config_name),
        Some(Commands::Validate) => commands::validate_command(),
        Some(Commands::History { limit, filter_type }) => {
            commands::history_command(Some(limit), filter_type)
        }
        Some(Commands::Web { port }) => web::web_command(Some(port)),
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
        None => {
            // 如果提供了配置名称，则执行切换
            if let Some(config_name) = cli.config_name {
                commands::switch_command(&config_name)
            } else {
                // 否则显示当前配置
                commands::current_command()
            }
        }
    };

    // 处理错误
    if let Err(e) = result {
        eprintln!();
        ColorOutput::error(&e.user_message());
        eprintln!();

        // 如果是致命错误，提供额外的帮助信息
        if e.is_fatal() {
            ColorOutput::error("这是一个致命错误，程序无法继续");
            ColorOutput::info("请检查错误信息并修复后重试");
            ColorOutput::info("运行 'ccr --help' 查看帮助信息");
        }

        // 使用错误码退出
        std::process::exit(e.exit_code());
    }
}

/// 显示版本信息
fn show_version() {
    let version = env!("CARGO_PKG_VERSION");
    ColorOutput::banner(version);

    println!();
    ColorOutput::key_value("版本", version, 2);
    ColorOutput::key_value("作者", env!("CARGO_PKG_AUTHORS"), 2);
    ColorOutput::key_value("描述", env!("CARGO_PKG_DESCRIPTION"), 2);
    println!();

    ColorOutput::info("CCR 特性:");
    println!("  • 直接写入 Claude Code 设置文件 (~/.claude/settings.json)");
    println!("  • 文件锁机制确保并发安全");
    println!("  • 完整的操作历史和审计追踪");
    println!("  • 配置备份和恢复功能");
    println!("  • 自动配置验证");
    println!("  • 与 CCS 完全兼容");
    println!();

    ColorOutput::info("常用命令:");
    println!("  ccr list              列出所有配置");
    println!("  ccr current           显示当前状态");
    println!("  ccr switch <name>     切换配置");
    println!("  ccr validate          验证配置");
    println!("  ccr history           查看历史");
    println!();

    ColorOutput::info("更多帮助: ccr --help");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // 测试基本的 CLI 解析
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
