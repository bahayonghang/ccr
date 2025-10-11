// 🚀 CCR (Claude Code Configuration Switcher) 主程序
// 📦 配置管理工具，支持完整审计追踪
//
// 核心功能：
// - ⚙️  配置切换与管理
// - 📝 操作历史追踪
// - 🔒 文件锁保证并发安全
// - 🌐 Web 管理界面

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

/// 🎯 Claude Code Configuration Switcher - 配置管理工具
#[derive(Parser)]
#[command(name = "ccr")]
#[command(about = "Claude Code Configuration Router - 配置管理工具", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 💡 直接切换到指定配置（简写形式）
    /// 例如: ccr anthropic
    config_name: Option<String>,
}

/// 📋 命令枚举 - 定义所有可用的 CLI 子命令
#[derive(Subcommand)]
enum Commands {
    /// 📜 列出所有可用配置
    #[command(alias = "ls")]
    List,

    /// 🔍 显示当前配置状态
    #[command(alias = "show")]
    #[command(alias = "status")]
    Current,

    /// 🔄 切换到指定配置
    Switch {
        /// 目标配置名称
        config_name: String,
    },

    /// ✅ 验证配置和设置的完整性
    #[command(alias = "check")]
    Validate,

    /// 📚 显示操作历史
    History {
        /// 限制显示的记录数量
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// 🏷️ 按操作类型筛选 (switch, backup, restore, validate, update)
        #[arg(short = 't', long)]
        filter_type: Option<String>,
    },

    /// 🌐 启动 Web 配置界面
    Web {
        /// 🔌 指定端口（默认 8080）
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// 🔄 检查并更新到最新版本
    Update {
        /// 仅检查更新，不执行安装
        #[arg(short, long)]
        check: bool,
    },

    /// 🎬 初始化配置文件
    Init {
        /// ⚠️ 强制覆盖现有配置
        #[arg(short, long)]
        force: bool,
    },

    /// 📤 导出配置到文件
    Export {
        /// 📁 输出文件路径（默认: ccs_config_export_<timestamp>.toml）
        #[arg(short, long)]
        output: Option<String>,

        /// 🔒 排除敏感信息（API密钥等）
        #[arg(long)]
        no_secrets: bool,
    },

    /// 📥 从文件导入配置
    Import {
        /// 📁 输入文件路径
        input: String,

        /// 🔗 合并模式（保留现有配置，添加新的）
        #[arg(short, long)]
        merge: bool,

        /// 💾 导入前备份当前配置
        #[arg(short, long, default_value_t = true)]
        backup: bool,
    },

    /// 🧹 清理旧备份文件
    Clean {
        /// 📅 清理多少天前的备份（默认 7 天）
        #[arg(short, long, default_value_t = 7)]
        days: u64,

        /// 🔬 模拟运行，不实际删除文件
        #[arg(long)]
        dry_run: bool,
    },

    /// ℹ️ 显示版本信息
    #[command(alias = "ver")]
    Version,
}

/// 🎯 主函数入口
/// 
/// 执行流程:
/// 1. 🔧 初始化日志系统
/// 2. 📝 解析命令行参数
/// 3. 🚀 路由并执行对应命令
/// 4. ❌ 处理错误并返回退出码
fn main() {
    // 🔧 初始化日志系统
    init_logger();

    // 📝 解析命令行参数
    let cli = Cli::parse();

    // 🚀 执行命令并处理错误
    let result = match cli.command {
        Some(Commands::List) => commands::list_command(),
        Some(Commands::Current) => commands::current_command(),
        Some(Commands::Switch { config_name }) => commands::switch_command(&config_name),
        Some(Commands::Validate) => commands::validate_command(),
        Some(Commands::History { limit, filter_type }) => {
            commands::history_command(Some(limit), filter_type)
        }
        Some(Commands::Web { port }) => web::web_command(Some(port)),
        Some(Commands::Update { check }) => commands::update_command(check),
        Some(Commands::Init { force }) => commands::init_command(force),
        Some(Commands::Export { output, no_secrets }) => {
            commands::export_command(output, !no_secrets)
        }
        Some(Commands::Import { input, merge, backup }) => {
            let mode = if merge {
                commands::ImportMode::Merge
            } else {
                commands::ImportMode::Replace
            };
            commands::import_command(input, mode, backup)
        }
        Some(Commands::Clean { days, dry_run }) => {
            commands::clean_command(days, dry_run)
        }
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
        None => {
            // 💡 智能处理：有配置名称则切换，否则显示当前状态
            if let Some(config_name) = cli.config_name {
                commands::switch_command(&config_name)
            } else {
                commands::current_command()
            }
        }
    };

    // ❌ 错误处理与退出
    if let Err(e) = result {
        eprintln!();
        ColorOutput::error(&e.user_message());
        eprintln!();

        // 🚨 致命错误额外提示
        if e.is_fatal() {
            ColorOutput::error("这是一个致命错误，程序无法继续");
            ColorOutput::info("请检查错误信息并修复后重试");
            ColorOutput::info("运行 'ccr --help' 查看帮助信息");
        }

        // 🔢 使用错误码退出
        std::process::exit(e.exit_code());
    }
}

/// 📋 显示版本信息和帮助
/// 
/// 包含内容:
/// - ℹ️ 版本号、作者、描述
/// - ⭐ 核心特性列表
/// - 📖 常用命令示例
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
    println!("  ccr init              初始化配置文件");
    println!("  ccr list              列出所有配置");
    println!("  ccr current           显示当前状态");
    println!("  ccr switch <name>     切换配置");
    println!("  ccr validate          验证配置");
    println!("  ccr history           查看历史");
    println!("  ccr export            导出配置");
    println!("  ccr import <file>     导入配置");
    println!("  ccr clean             清理旧备份");
    println!("  ccr update            更新到最新版本");
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
