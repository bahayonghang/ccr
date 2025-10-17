// 🚀 CCR (Claude Code Configuration Switcher) 主程序
// 📦 配置管理工具,支持完整审计追踪
//
// 核心功能：
// - ⚙️  配置切换与管理
// - 📝 操作历史追踪
// - 🔒 文件锁保证并发安全
// - 🌐 Web 管理界面

mod commands;
mod core;
mod managers;
mod services;
mod tui;
mod utils;
mod web;

use clap::{Parser, Subcommand};
use core::{ColorOutput, init_logger};

/// 🎯 Claude Code Configuration Switcher - 配置管理工具
#[derive(Parser)]
#[command(name = "ccr")]
#[command(
    about = "Claude Code 配置管理工具 - 快速切换和管理多套配置",
    long_about = "\
🎯 Claude Code Configuration Switcher (Rust Version)

一个强大的 Claude Code 配置管理工具,支持：
    • 多套配置快速切换
    • 完整的操作审计追踪
    • 自动备份和恢复
    • 配置导入导出
    • Web 可视化界面

🚀 快速开始:
    ccr init              # 初始化配置文件
    ccr list              # 查看所有配置
    ccr switch <名称>      # 切换配置
    ccr anthropic         # 快捷切换(省略 switch)

📖 获取帮助:
    ccr --help            # 显示此帮助
    ccr <命令> --help      # 显示特定命令的帮助"
)]
#[command(version)]
#[command(
    help_template = "\
{before-help}{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
",
    override_usage = "ccr [选项] [配置名称] [命令]",
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// ⚡ 自动确认模式（跳过所有确认提示）
    ///
    /// 等同于配置文件中的 skip_confirmation = true
    /// 所有需要确认的操作将自动执行，无需手动输入 'y'
    /// 示例：ccr --yes delete test  或  ccr -y delete test
    #[arg(short = 'y', long = "yes", global = true)]
    auto_yes: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// 直接切换到指定配置(快捷方式,无需输入 switch 子命令)
    ///
    /// 示例：ccr anthropic  等同于  ccr switch anthropic
    config_name: Option<String>,

    /// 显示帮助信息（使用 '-h' 查看简短摘要）
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// 显示版本信息
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

/// 📋 命令枚举 - 定义所有可用的 CLI 子命令
#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用的配置方案
    ///
    /// 显示配置文件中定义的所有配置方案,包括配置名称、环境变量设置等信息
    /// 别名: ls
    #[command(alias = "ls")]
    List,

    /// 显示当前激活的配置状态
    ///
    /// 查看当前正在使用的配置方案详情,包括所有环境变量设置
    /// 别名: show, status
    #[command(alias = "show")]
    #[command(alias = "status")]
    Current,

    /// 切换到指定的配置方案
    ///
    /// 将 Claude Code 的配置切换到指定方案,自动备份当前配置并应用新配置
    /// 示例: ccr switch anthropic
    Switch {
        /// 要切换到的配置方案名称(必须在配置文件中已定义)
        config_name: String,
    },

    /// 添加新的配置方案
    ///
    /// 交互式地添加新配置,按照提示输入配置信息
    /// 示例: ccr add
    Add,

    /// 删除指定的配置方案
    ///
    /// 删除配置文件中的指定配置节
    /// 示例: ccr delete my_config
    Delete {
        /// 要删除的配置方案名称
        config_name: String,

        /// 跳过确认提示，直接删除（危险操作）
        #[arg(short, long)]
        force: bool,
    },

    /// 验证配置文件和设置的完整性
    ///
    /// 检查配置文件格式是否正确,以及 Claude Code 设置文件是否有效
    /// 别名: check
    #[command(alias = "check")]
    Validate,

    /// 查看配置操作的历史记录
    ///
    /// 显示所有配置切换、导入导出等操作的审计日志,支持按类型筛选
    /// 示例: ccr history -l 50 -t switch
    History {
        /// 显示最近 N 条记录(默认显示 20 条)
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// 按操作类型筛选记录
        ///
        /// 可选值: switch(切换)、backup(备份)、restore(恢复)、
        ///         validate(验证)、update(更新)
        #[arg(short = 't', long)]
        filter_type: Option<String>,
    },

    /// 启动 Web 管理界面
    ///
    /// 在浏览器中打开可视化的配置管理界面,支持所有配置操作
    /// 示例: ccr web -p 3000
    Web {
        /// 指定 Web 服务器监听端口(默认: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// 从 GitHub 更新到最新版本
    ///
    /// 检查并安装 CCR 的最新版本
    /// 示例: ccr update --check  # 仅检查不安装
    Update {
        /// 仅检查是否有新版本,不执行安装
        #[arg(short, long)]
        check: bool,
    },

    /// 初始化配置文件
    ///
    /// 在 ~/.ccs_config.toml 创建配置文件模板,包含示例配置方案
    /// 示例: ccr init --force  # 强制覆盖现有配置
    Init {
        /// 强制覆盖已存在的配置文件(危险操作,会丢失当前配置)
        #[arg(short, long)]
        force: bool,
    },

    /// 导出配置到文件
    ///
    /// 将当前配置导出为 TOML 文件,方便备份或分享
    /// 示例: ccr export -o my_config.toml --no-secrets
    Export {
        /// 指定导出文件路径
        ///
        /// 不指定时自动生成文件名: ccs_config_export_<时间戳>.toml
        #[arg(short, long)]
        output: Option<String>,

        /// 导出时排除敏感信息(如 API 密钥),仅保留配置结构
        #[arg(long)]
        no_secrets: bool,
    },

    /// 从文件导入配置
    ///
    /// 从 TOML 文件导入配置方案,支持替换或合并模式
    /// 示例: ccr import config.toml --merge
    Import {
        /// 要导入的配置文件路径
        input: String,

        /// 使用合并模式(保留现有配置,仅添加新配置方案)
        ///
        /// 不指定此选项时,将完全替换现有配置文件
        #[arg(short, long)]
        merge: bool,

        /// 导入前自动备份当前配置文件(强烈建议保持开启)
        #[arg(short, long, default_value_t = true)]
        backup: bool,

        /// 跳过确认提示，直接导入（危险操作，在 Replace 模式下会完全覆盖现有配置）
        #[arg(short, long)]
        force: bool,
    },

    /// 清理过期的备份文件
    ///
    /// 删除 ~/.claude/backups/ 目录中的旧备份文件,释放磁盘空间
    /// 示例: ccr clean -d 30 --dry-run
    Clean {
        /// 清理 N 天前的备份文件(默认: 7 天)
        #[arg(short, long, default_value_t = 7)]
        days: u64,

        /// 模拟运行(dry-run)：仅显示将要删除的文件,不实际删除
        #[arg(long)]
        dry_run: bool,

        /// 跳过确认提示，直接清理（危险操作）
        #[arg(short, long)]
        force: bool,
    },

    /// 优化配置文件结构
    ///
    /// 按照配置节名称的字母顺序重新排列配置文件,提升可读性
    /// 示例: ccr optimize
    Optimize,

    /// 显示详细的版本信息
    ///
    /// 查看 CCR 版本号、特性列表和常用命令
    /// 别名: ver
    #[command(alias = "ver")]
    Version,

    /// 启动 TUI (Terminal User Interface) 交互式界面
    ///
    /// 提供可视化的配置管理界面，支持实时操作和自动确认模式切换
    /// 示例: ccr tui
    Tui {
        /// 启动时启用自动确认模式
        #[arg(short = 'y', long = "yes")]
        auto_yes: bool,
    },
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
        Some(Commands::Add) => commands::add_command(),
        Some(Commands::Delete { config_name, force }) => {
            commands::delete_command(&config_name, cli.auto_yes || force)
        }
        Some(Commands::Validate) => commands::validate_command(),
        Some(Commands::History { limit, filter_type }) => {
            commands::history_command(Some(limit), filter_type)
        }
        Some(Commands::Web { port }) => web::web_command(Some(port)),
        Some(Commands::Update { check }) => commands::update_command(check),
        Some(Commands::Init { force }) => commands::init_command(cli.auto_yes || force),
        Some(Commands::Export { output, no_secrets }) => {
            commands::export_command(output, !no_secrets)
        }
        Some(Commands::Import {
            input,
            merge,
            backup,
            force,
        }) => {
            let mode = if merge {
                commands::ImportMode::Merge
            } else {
                commands::ImportMode::Replace
            };
            commands::import_command(input, mode, backup, cli.auto_yes || force)
        }
        Some(Commands::Clean {
            days,
            dry_run,
            force,
        }) => commands::clean_command(days, dry_run, cli.auto_yes || force),
        Some(Commands::Optimize) => commands::optimize_command(),
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
        Some(Commands::Tui { auto_yes }) => tui::run_tui(cli.auto_yes || auto_yes),
        None => {
            // 💡 智能处理：有配置名称则切换,否则显示当前状态
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
            ColorOutput::error("这是一个致命错误,程序无法继续");
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
    println!("  ccr add               添加新配置");
    println!("  ccr delete <name>     删除配置");
    println!("  ccr validate          验证配置");
    println!("  ccr optimize          优化配置文件结构");
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
