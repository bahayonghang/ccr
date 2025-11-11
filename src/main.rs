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
mod help;
mod managers;
mod models;
mod platforms;
mod services;
mod utils;

#[cfg(feature = "tui")]
mod tui;

#[cfg(feature = "web")]
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
    disable_version_flag = true,
    disable_help_subcommand = true
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

    /// 显示帮助信息（支持顶层与子命令）
    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue)]
    help: Option<bool>,

    /// 显示版本信息
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

/// 📋 命令枚举 - 定义所有可用的 CLI 子命令
#[derive(Subcommand)]
enum Commands {
    /// 帮助子命令（美化版）
    ///
    /// 示例: ccr help            # 顶层帮助
    ///       ccr help switch     # 指定子命令帮助
    #[command(name = "help")]
    Help {
        /// 可选：指定要查看帮助的子命令名称
        subcmd: Option<String>,
    },
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
    ///       ccr web --no-browser
    #[cfg(feature = "web")]
    Web {
        /// 指定 Web 服务器监听端口(默认: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// 不自动打开浏览器
        #[arg(long)]
        no_browser: bool,
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
    #[cfg(feature = "tui")]
    Tui {
        /// 启动时启用自动确认模式
        #[arg(short = 'y', long = "yes")]
        auto_yes: bool,
    },

    /// WebDAV 配置同步
    ///
    /// 支持将配置文件同步到 WebDAV 服务器（默认支持坚果云）
    /// 示例: ccr sync config  # 配置同步
    ///       ccr sync status  # 查看状态
    ///       ccr sync push    # 上传配置
    ///       ccr sync pull    # 下载配置
    #[cfg(feature = "web")]
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },

    /// 启动 CCR UI (完整 Web 应用)
    ///
    /// 启动功能完整的 CCR UI 图形界面,支持多 CLI 工具管理
    /// 开发环境：自动检测并启动源码版本
    /// 生产环境：启动预构建版本(未来支持)
    /// 示例: ccr ui -p 3000
    Ui {
        /// 指定前端端口(默认: 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// 指定后端端口(默认: 8081)
        #[arg(long, default_value_t = 8081)]
        backend_port: u16,
    },

    /// 临时Token管理
    ///
    /// 管理临时配置覆盖,不修改永久配置文件
    /// 示例: ccr temp-token set sk-xxx
    ///       ccr temp-token show
    ///       ccr temp-token clear
    #[command(name = "temp-token")]
    TempToken {
        #[command(subcommand)]
        action: TempTokenAction,
    },

    /// 多平台管理
    ///
    /// 管理和切换不同的 AI CLI 平台 (Claude, Codex, Gemini 等)
    /// 示例: ccr platform list
    ///       ccr platform switch codex
    ///       ccr platform current
    Platform {
        #[command(subcommand)]
        action: PlatformAction,
    },

    /// 配置迁移
    ///
    /// 将 Legacy 模式配置迁移到 Unified 模式
    /// 示例: ccr migrate --check      # 检查迁移状态
    ///       ccr migrate              # 执行迁移
    Migrate {
        /// 仅检查迁移状态，不实际执行迁移 (dry-run 模式)
        #[arg(short, long)]
        check: bool,

        /// 只迁移指定平台的配置
        #[arg(short, long)]
        platform: Option<String>,
    },

    /// 统计与分析
    ///
    /// 查看使用统计、成本分析等信息
    /// 示例: ccr stats cost --today
    ///       ccr stats cost --by-model
    ///       ccr stats cost --top 10
    #[cfg(feature = "web")]
    Stats(commands::StatsArgs),
}

/// 🎯 临时Token操作子命令
#[derive(Subcommand)]
enum TempTokenAction {
    /// 设置临时Token
    ///
    /// 临时覆盖当前配置的token,不修改toml配置文件
    /// 示例: ccr temp-token set sk-test-xxx
    ///       ccr temp-token set sk-xxx --base-url https://api.test.com
    ///       ccr temp-token set sk-xxx --model claude-opus-4
    Set {
        /// 临时使用的token
        token: String,

        /// 临时base_url(可选)
        #[arg(long)]
        base_url: Option<String>,

        /// 临时model(可选)
        #[arg(long)]
        model: Option<String>,
    },

    /// 显示当前临时配置
    ///
    /// 查看当前设置的临时配置状态
    /// 示例: ccr temp-token show
    Show,

    /// 清除临时配置
    ///
    /// 删除所有临时配置覆盖
    /// 示例: ccr temp-token clear
    Clear,
}

/// 🎯 平台管理操作子命令
#[derive(Subcommand)]
enum PlatformAction {
    /// 列出所有可用平台
    ///
    /// 显示所有支持的 AI CLI 平台及其状态
    /// 示例: ccr platform list
    /// 示例: ccr platform list --json
    List {
        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 切换到指定平台
    ///
    /// 切换当前激活的平台
    /// 示例: ccr platform switch codex
    Switch {
        /// 平台名称 (claude, codex, gemini, qwen, iflow)
        platform_name: String,
    },

    /// 显示当前平台信息
    ///
    /// 查看当前激活平台的详细信息
    /// 示例: ccr platform current
    /// 示例: ccr platform current --json
    Current {
        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 显示平台详细信息
    ///
    /// 查看指定平台的配置和状态
    /// 示例: ccr platform info claude
    /// 示例: ccr platform info claude --json
    Info {
        /// 平台名称
        platform_name: String,

        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 初始化平台配置
    ///
    /// 为指定平台创建配置目录结构
    /// 示例: ccr platform init codex
    Init {
        /// 平台名称
        platform_name: String,
    },
}

/// ☁️ 同步操作子命令
#[derive(Subcommand)]
enum SyncAction {
    /// 管理同步文件夹注册
    ///
    /// 管理可同步的文件夹列表
    /// 示例: ccr sync folder list
    /// 示例: ccr sync folder add claude ~/.claude
    Folder {
        #[command(subcommand)]
        action: FolderAction,
    },

    /// 批量操作所有启用的文件夹
    ///
    /// 对所有已启用的文件夹执行同步操作
    /// 示例: ccr sync all push
    /// 示例: ccr sync all status
    All {
        #[command(subcommand)]
        action: AllSyncAction,
    },

    /// 配置 WebDAV 同步
    ///
    /// 交互式配置 WebDAV 服务器连接信息
    /// 示例: ccr sync config
    Config,

    /// 显示同步状态
    ///
    /// 查看当前同步配置和所有文件夹状态
    /// 示例: ccr sync status
    Status,

    /// 上传配置到云端 (兼容旧命令)
    ///
    /// 将本地配置文件上传到 WebDAV 服务器
    /// 示例: ccr sync push --force
    /// 示例: ccr sync push --interactive  # 交互式选择内容
    Push {
        /// 强制覆盖远程配置，不提示确认
        #[arg(short, long)]
        force: bool,

        /// 交互式选择要同步的内容类型
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// 从云端下载配置 (兼容旧命令)
    ///
    /// 从 WebDAV 服务器下载配置文件到本地
    /// 示例: ccr sync pull --force
    Pull {
        /// 强制覆盖本地配置，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 同步特定文件夹 (动态子命令)
    ///
    /// 对指定文件夹执行同步操作
    /// 示例: ccr sync claude push
    /// 示例: ccr sync gemini pull
    /// 示例: ccr sync conf status
    #[command(external_subcommand)]
    #[allow(dead_code)]
    FolderSync(Vec<String>),
}

/// 📁 文件夹管理操作
#[derive(Subcommand)]
enum FolderAction {
    /// 列出所有注册的同步文件夹
    ///
    /// 显示文件夹名称、状态、路径等信息
    /// 示例: ccr sync folder list
    List,

    /// 添加新的同步文件夹
    ///
    /// 注册一个新文件夹用于同步
    /// 示例: ccr sync folder add claude ~/.claude
    Add {
        /// 文件夹名称（唯一标识）
        name: String,

        /// 本地路径（支持 ~ 扩展）
        local_path: String,

        /// 远程路径（可选，默认为 /ccr-sync/<name>）
        #[arg(short = 'r', long)]
        remote_path: Option<String>,

        /// 描述信息
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// 删除同步文件夹注册
    ///
    /// 从注册列表中移除文件夹（不删除本地文件）
    /// 示例: ccr sync folder remove claude
    Remove {
        /// 文件夹名称
        name: String,
    },

    /// 显示文件夹详细信息
    ///
    /// 查看文件夹的完整配置
    /// 示例: ccr sync folder info claude
    Info {
        /// 文件夹名称
        name: String,
    },

    /// 启用文件夹同步
    ///
    /// 启用文件夹的同步功能
    /// 示例: ccr sync folder enable claude
    Enable {
        /// 文件夹名称
        name: String,
    },

    /// 禁用文件夹同步
    ///
    /// 禁用文件夹的同步功能（保留配置）
    /// 示例: ccr sync folder disable codex
    Disable {
        /// 文件夹名称
        name: String,
    },
}

/// 🔄 批量同步操作
#[derive(Subcommand)]
enum AllSyncAction {
    /// 上传所有启用的文件夹
    ///
    /// 示例: ccr sync all push
    Push {
        /// 强制覆盖，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 下载所有启用的文件夹
    ///
    /// 示例: ccr sync all pull
    Pull {
        /// 强制覆盖，不提示确认
        #[arg(short, long)]
        force: bool,
    },

    /// 显示所有文件夹的状态
    ///
    /// 示例: ccr sync all status
    Status,
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

    // 🎨 优先处理自定义帮助输出
    if cli.help.unwrap_or(false) {
        match &cli.command {
            Some(cmd) => {
                let name = command_name(cmd);
                help::print_subcommand_help(name);
            }
            None => help::print_top_help(),
        }
        return;
    }

    // 🚀 执行命令并处理错误
    let result = match cli.command {
        Some(Commands::Help { subcmd }) => {
            match subcmd.as_deref() {
                Some(name) => help::print_subcommand_help(name),
                None => help::print_top_help(),
            }
            Ok(())
        }
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
        #[cfg(feature = "web")]
        Some(Commands::Web { port, no_browser }) => web::web_command(Some(port), no_browser),
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
        #[cfg(feature = "tui")]
        Some(Commands::Tui { auto_yes }) => tui::run_tui(cli.auto_yes || auto_yes),
        #[cfg(feature = "web")]
        Some(Commands::Sync { action }) => match action {
            // 🗂️ 文件夹管理命令
            SyncAction::Folder { action } => match action {
                FolderAction::List => commands::sync_cmd::sync_folder_list_command(),
                FolderAction::Add {
                    name,
                    local_path,
                    remote_path,
                    description,
                } => commands::sync_cmd::sync_folder_add_command(
                    &name,
                    &local_path,
                    remote_path.as_ref(),
                    description.as_ref(),
                ),
                FolderAction::Remove { name } => {
                    commands::sync_cmd::sync_folder_remove_command(&name)
                }
                FolderAction::Info { name } => commands::sync_cmd::sync_folder_info_command(&name),
                FolderAction::Enable { name } => {
                    commands::sync_cmd::sync_folder_enable_command(&name)
                }
                FolderAction::Disable { name } => {
                    commands::sync_cmd::sync_folder_disable_command(&name)
                }
            },
            // 🔄 批量同步命令
            SyncAction::All { action } => match action {
                AllSyncAction::Push { force } => commands::sync_cmd::sync_all_push_command(force),
                AllSyncAction::Pull { force } => commands::sync_cmd::sync_all_pull_command(force),
                AllSyncAction::Status => commands::sync_cmd::sync_all_status_command(),
            },
            // 📁 文件夹特定同步命令（动态分发）
            SyncAction::FolderSync(args) => commands::sync_cmd::sync_folder_specific_command(&args),
            // ☁️ 原有命令（向后兼容）
            SyncAction::Config => commands::sync_cmd::sync_config_command(),
            SyncAction::Status => commands::sync_cmd::sync_status_command(),
            SyncAction::Push { force, interactive } => {
                if interactive {
                    // 交互式模式：显示内容选择面板
                    let mut selector = commands::SyncContentSelector::new();
                    match selector.select_content() {
                        Ok(selection) => commands::sync_cmd::sync_push_command_with_selection(
                            force,
                            Some(selection),
                        ),
                        Err(e) => Err(e),
                    }
                } else {
                    // 默认模式：仅同步config
                    commands::sync_cmd::sync_push_command(force)
                }
            }
            SyncAction::Pull { force } => commands::sync_cmd::sync_pull_command(force),
        },
        Some(Commands::Ui { port, backend_port }) => commands::ui_command(port, backend_port),
        Some(Commands::TempToken { action }) => match action {
            TempTokenAction::Set {
                token,
                base_url,
                model,
            } => commands::temp_token_set(&token, base_url, model),
            TempTokenAction::Show => commands::temp_token_show(),
            TempTokenAction::Clear => commands::temp_token_clear(),
        },
        Some(Commands::Platform { action }) => match action {
            PlatformAction::List { json } => commands::platform_list_command(json),
            PlatformAction::Switch { platform_name } => {
                commands::platform_switch_command(&platform_name)
            }
            PlatformAction::Current { json } => commands::platform_current_command(json),
            PlatformAction::Info {
                platform_name,
                json,
            } => commands::platform_info_command(&platform_name, json),
            PlatformAction::Init { platform_name } => {
                commands::platform_init_command(&platform_name)
            }
        },
        Some(Commands::Migrate { check, platform }) => {
            if check {
                commands::migrate_check_command()
            } else {
                commands::migrate_command(false, platform.as_deref())
            }
        }
        #[cfg(feature = "web")]
        Some(Commands::Stats(args)) => match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(async {
                let mut color_output = ColorOutput;
                commands::stats_command(args, &mut color_output).await
            }),
            Err(e) => {
                eprintln!("❌ 创建异步运行时失败: {}", e);
                std::process::exit(1);
            }
        },
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
    println!("  • WebDAV 云端同步（支持坚果云）");
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
    println!("  ccr sync config       配置云端同步");
    println!("  ccr sync push         上传配置到云端");
    println!("  ccr sync push -i      交互式选择上传内容");
    println!("  ccr sync pull         从云端下载配置");
    println!("  ccr update            更新到最新版本");
    println!();

    ColorOutput::info("更多帮助: ccr --help");
}

/// 返回子命令名称（用于帮助渲染）
fn command_name(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Help { .. } => "help",
        Commands::List => "list",
        Commands::Current => "current",
        Commands::Switch { .. } => "switch",
        Commands::Add => "add",
        Commands::Delete { .. } => "delete",
        Commands::Validate => "validate",
        Commands::History { .. } => "history",
        #[cfg(feature = "web")]
        Commands::Web { .. } => "web",
        Commands::Update { .. } => "update",
        Commands::Init { .. } => "init",
        Commands::Export { .. } => "export",
        Commands::Import { .. } => "import",
        Commands::Clean { .. } => "clean",
        Commands::Optimize => "optimize",
        Commands::Version => "version",
        #[cfg(feature = "tui")]
        Commands::Tui { .. } => "tui",
        #[cfg(feature = "web")]
        Commands::Sync { .. } => "sync",
        Commands::Ui { .. } => "ui",
        Commands::TempToken { .. } => "temp-token",
        Commands::Platform { .. } => "platform",
        Commands::Migrate { .. } => "migrate",
        #[cfg(feature = "web")]
        Commands::Stats(_) => "stats",
    }
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
