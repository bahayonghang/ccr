// CLI 结构定义
//
// 定义 CCR 命令行接口的结构，包括主结构和所有子命令

use clap::{Args, Parser, Subcommand};

pub const DEFAULT_CLEAN_BACKUP_DAYS: u64 = 7;

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
{name} {version}
{about-with-newline}

{usage-heading} {usage}

{all-args}{after-help}",
    override_usage = "ccr [选项] [配置名称] [命令]",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// ⚡ 自动确认模式（跳过所有确认提示）
    ///
    /// 等同于配置文件中的 skip_confirmation = true
    /// 所有需要确认的操作将自动执行，无需手动输入 'y'
    /// 示例：ccr --yes delete test  或  ccr -y delete test
    #[arg(short = 'y', long = "yes", global = true)]
    pub auto_yes: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 直接切换到指定配置(快捷方式,无需输入 switch 子命令)
    ///
    /// 示例：ccr anthropic  等同于  ccr switch anthropic
    pub config_name: Option<String>,
}

impl Cli {
    /// 🖥️ 检测是否为 TUI 模式
    ///
    /// 当没有指定子命令和配置名称时，会进入 TUI 模式
    /// `ccr codex` / `ccr opencode` 无参数也视为 TUI 路由模式
    #[cfg(feature = "tui")]
    pub fn is_tui_mode(&self) -> bool {
        if self.command.is_none() && self.config_name.is_none() {
            return true;
        }
        matches!(
            self.command,
            Some(Commands::Codex { action: None }) | Some(Commands::OpenCode { action: None })
        )
    }
}

/// 📋 命令枚举 - 定义所有可用的 CLI 子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 帮助子命令（美化版）
    ///
    /// 示例: ccr help            # 顶层帮助
    ///       ccr help platform   # 指定子命令帮助
    ///       ccr help codex auth # 指定嵌套子命令帮助
    #[command(name = "help")]
    Help {
        /// 可选：指定要查看帮助的命令路径
        #[arg(value_name = "COMMAND_PATH")]
        path: Vec<String>,
    },

    /// 列出所有可用的配置方案
    ///
    /// 显示配置文件中定义的所有配置方案,包括配置名称、环境变量设置等信息
    /// 别名: ls
    #[command(alias = "ls")]
    List,

    /// 显示当前 Claude/Codex 运行状态总览
    ///
    /// 默认显示 Claude 与 Codex 双平台摘要；使用 --verbose 查看诊断详情
    /// 别名: status, show (推荐使用 ccr status)
    #[command(alias = "status")]
    #[command(alias = "show")]
    Current(CurrentArgs),

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

    /// 启用指定的配置方案
    ///
    /// 将配置标记为启用状态，使其可以正常使用
    /// 示例: ccr enable my_config
    Enable {
        /// 要启用的配置方案名称
        config_name: String,
    },

    /// 禁用指定的配置方案
    ///
    /// 将配置标记为禁用状态，暂时不可使用（不会删除）
    /// 示例: ccr disable old_config
    Disable {
        /// 要禁用的配置方案名称
        config_name: String,

        /// 强制禁用（即使是当前正在使用的配置）
        #[arg(short, long)]
        force: bool,
    },

    /// 验证配置文件和设置的完整性
    ///
    /// 检查配置文件格式是否正确,以及 Claude Code 设置文件是否有效
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

    /// 从 GitHub 更新到最新版本
    ///
    /// 检查并安装 CCR 的最新版本
    /// 示例: ccr update --check  # 仅检查不安装
    ///       ccr update dev  # 从 dev 分支更新
    Update {
        /// 仅检查是否有新版本,不执行安装
        #[arg(short, long)]
        check: bool,

        /// 指定更新的分支(默认: main)
        #[arg(default_value = "main")]
        branch: String,
    },

    /// 初始化当前项目的 Git、Trellis 工作流和 Agent 忽略规则
    ///
    /// 示例: ccr project init
    ///       ccr -y project init
    Project {
        #[command(subcommand)]
        action: super::subcommands::ProjectAction,
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

    /// 交互式清理备份文件或规划文件
    ///
    /// 裸 `ccr clean` 会进入交互式菜单。
    /// 也支持显式目标 `ccr clean planfiles` 和 `ccr clean backups`。
    /// 示例: ccr clean
    ///       ccr clean planfiles --dry-run
    ///       ccr clean backups -d 30 --dry-run
    Clean(CleanArgs),

    /// 清理 CCR 写入的配置
    ///
    /// 清空 settings.json 中的 ANTHROPIC_* 环境变量,使其恢复默认状态
    /// 执行后 Claude Code 将无法正常工作,直到重新执行 switch 切换配置
    /// 示例: ccr clear
    ///       ccr clear --force  # 跳过确认
    Clear {
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

    /// WebDAV 配置同步
    ///
    /// 支持将配置文件同步到 WebDAV 服务器（默认支持坚果云）
    /// 示例: ccr sync config  # 配置同步
    ///       ccr sync status  # 查看状态
    ///       ccr sync push    # 上传配置
    ///       ccr sync pull    # 下载配置
    Sync {
        #[command(subcommand)]
        action: super::subcommands::sync::SyncAction,
    },

    /// 启动 CCR UI (完整 Web 应用)
    ///
    /// 推荐作为主要 Web 界面使用，提供功能完整的 CCR UI 图形界面并支持多 CLI 工具管理
    /// 开发环境：自动检测并启动源码版本
    /// 生产环境：启动预构建版本(未来支持)
    /// 示例: ccr ui -p 15173
    Ui {
        /// UI 子命令
        ///
        /// - 不传子命令：启动 UI
        /// - help：显示帮助
        /// - update：更新/安装 UI 到最新
        #[command(subcommand)]
        action: Option<super::subcommands::ui::UiAction>,

        /// 指定前端端口(默认: 15173)
        #[arg(short, long, default_value_t = 15173)]
        port: u16,

        /// 指定后端端口(默认: 38081)
        #[arg(long, default_value_t = 38081)]
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
        action: super::subcommands::ui::TempTokenAction,
    },

    /// 临时配置快速设置（交互式）
    ///
    /// 无需依赖现有 TOML 配置，直接交互式输入 base_url、token、model
    /// 并立即写入 settings.json。支持模型名称智能解析。
    /// 示例: ccr temp
    Temp,

    /// 多平台管理
    ///
    /// 管理和切换不同的 AI CLI 平台 (Claude, Codex, Gemini 等)
    /// 示例: ccr platform list
    ///       ccr platform switch codex
    ///       ccr platform current
    Platform {
        #[command(subcommand)]
        action: super::subcommands::platform::PlatformAction,
    },

    /// 统计与分析

    ///
    /// 查看使用统计、成本分析等信息
    /// 示例: ccr stats cost --today
    ///       ccr stats cost --by-model
    ///       ccr stats cost --top 10
    Stats(crate::commands::StatsArgs),

    /// 💰 预算管理
    ///
    /// 管理和监控 API 使用成本预算
    /// 示例: ccr budget status
    ///       ccr budget set --daily 10.0 --monthly 200.0
    ///       ccr budget reset
    Budget(crate::commands::BudgetArgs),

    /// 💲 价格表管理
    ///
    /// 管理模型定价配置
    /// 示例: ccr pricing list
    ///       ccr pricing set my-model --input 3.0 --output 15.0
    ///       ccr pricing remove my-model
    Pricing(crate::commands::PricingArgs),

    /// 🛠️ 技能管理
    ///
    /// 管理 AI 助手的技能 (Skills)
    /// 示例: ccr skills list
    ///       ccr skills scan official
    ///       ccr skills install computer-use
    Skills(crate::commands::skills_cmd::SkillsArgs),

    /// 📝 提示词预设管理
    ///
    /// 管理系统提示词预设 (Prompts)
    /// 示例: ccr prompts list
    ///       ccr prompts add my-preset --target claude --content @prompt.md
    ///       ccr prompts apply my-preset
    Prompts(crate::commands::prompts_cmd::PromptsArgs),

    /// 🔍 检测配置冲突
    ///
    /// 检测不同 AI CLI 平台之间的环境变量冲突
    /// 示例: ccr check conflicts
    Check {
        #[command(subcommand)]
        action: super::subcommands::check::CheckAction,
    },

    /// 🩺 统一体检
    ///
    /// 聚合 CCR 本地环境、平台配置、当前 profile、认证状态和可选在线探活
    /// 示例: ccr doctor
    ///       ccr doctor --online
    ///       ccr doctor --all-platforms --verbose
    Doctor(crate::commands::doctor_cmd::DoctorArgs),

    /// 🔐 Codex 多账号管理
    ///
    /// 管理 Codex CLI 的多账号登录状态
    /// 示例: ccr codex auth list
    ///       ccr codex auth save my-account
    ///       ccr codex auth switch work
    /// 提示: 直接运行 `ccr codex` 可启动 TUI 界面
    Codex {
        #[command(subcommand)]
        action: Option<super::subcommands::codex::CodexAction>,
    },

    /// 🔐 OpenCode 多账号管理
    ///
    /// 管理 OpenCode openai provider 的多账号快照
    /// 示例: ccr opencode auth import-codex
    /// 提示: 直接运行 `ccr opencode` 可启动 TUI 界面
    #[command(name = "opencode")]
    OpenCode {
        #[command(subcommand)]
        action: Option<super::subcommands::opencode::OpenCodeAction>,
    },

    /// 🔐 Claude 官方订阅账号管理
    ///
    /// 管理 Claude Code 官方订阅凭据快照
    /// 示例: ccr claude auth list
    ///       ccr claude auth save work
    ///       ccr claude auth switch personal
    Claude {
        #[command(subcommand)]
        action: Option<super::subcommands::claude::ClaudeAction>,
    },

    /// 📚 Session 管理
    ///
    /// 管理 AI CLI 的会话记录
    /// 示例: ccr sessions list
    ///       ccr sessions search "refactoring"
    ///       ccr sessions reindex
    Sessions(crate::commands::sessions_cmd::SessionsArgs),

    /// 🏥 Provider 健康检查
    ///
    /// 测试 Provider 端点连通性和 API Key 有效性
    /// 示例: ccr provider test --all
    ///       ccr provider test my-config --verbose
    ///       ccr provider verify my-config
    Provider(crate::commands::provider_cmd::ProviderArgs),
}

/// 🧹 clean 命令参数
#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct CleanArgs {
    /// clean 子命令
    #[command(subcommand)]
    pub action: Option<CleanAction>,

    /// 兼容入口：清理 N 天前的备份文件(未指定时由交互菜单处理)
    #[arg(short, long)]
    pub days: Option<u64>,

    /// 兼容入口：模拟运行备份清理,不实际删除
    #[arg(long)]
    pub dry_run: bool,

    /// 兼容入口：跳过备份清理确认提示，直接清理（危险操作）
    #[arg(short, long)]
    pub force: bool,

    /// 默认规划文件清理入口：递归检索当前目录下所有相关规划文件
    #[arg(long)]
    pub all: bool,
}

/// 当前状态命令参数
#[derive(Args, Debug, Clone, Default)]
pub struct CurrentArgs {
    /// 显示旧版诊断详情，包括路径、完整 profile 字段和环境变量状态
    #[arg(long)]
    pub verbose: bool,

    /// 以 JSON 输出 Claude/Codex 双平台结构化摘要
    #[arg(long, conflicts_with = "verbose")]
    pub json: bool,
}

impl CleanArgs {
    /// 判断裸 `ccr clean` 是否使用旧备份清理兼容参数。
    pub fn has_legacy_backup_flags(&self) -> bool {
        self.days.is_some() || self.dry_run || self.force
    }
}

/// 🧹 clean 子命令枚举
#[derive(Subcommand, Debug, Clone)]
pub enum CleanAction {
    /// 清理当前目录中的规划文件，使用 --all 递归扫描子目录
    Planfiles(CleanPlanfilesArgs),

    /// 显式清理旧备份文件
    Backups(CleanBackupsArgs),
}

/// 🧹 clean planfiles 命令参数
#[derive(Args, Debug, Clone, Default)]
pub struct CleanPlanfilesArgs {
    /// 模拟运行(dry-run)：仅显示将要删除的文件,不实际删除
    #[arg(long)]
    pub dry_run: bool,

    /// 跳过确认提示，直接清理（危险操作）
    #[arg(short, long)]
    pub force: bool,

    /// 递归检索当前目录及子目录中的规划文件
    #[arg(long)]
    pub all: bool,
}

/// 🧹 clean backups 命令参数
#[derive(Args, Debug, Clone)]
pub struct CleanBackupsArgs {
    /// 清理 N 天前的备份文件(默认: 7 天)
    #[arg(short, long, default_value_t = DEFAULT_CLEAN_BACKUP_DAYS)]
    pub days: u64,

    /// 模拟运行(dry-run)：仅显示将要删除的文件,不实际删除
    #[arg(long)]
    pub dry_run: bool,

    /// 跳过确认提示，直接清理（危险操作）
    #[arg(short, long)]
    pub force: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cli::subcommands::{CodexAction, ProjectAction};
    use clap::Parser;

    #[test]
    fn project_init_and_legacy_init_parse_as_distinct_commands() {
        let project = Cli::try_parse_from(["ccr", "project", "init"]).unwrap();
        assert!(matches!(
            project.command,
            Some(Commands::Project {
                action: ProjectAction::Init
            })
        ));

        let legacy = Cli::try_parse_from(["ccr", "init", "--force"]).unwrap();
        assert!(matches!(
            legacy.command,
            Some(Commands::Init { force: true })
        ));
    }

    #[test]
    fn clean_without_flags_parses_as_interactive_menu() {
        let cli = Cli::try_parse_from(["ccr", "clean"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => {
                assert!(args.action.is_none());
                assert_eq!(args.days, None);
                assert!(!args.dry_run);
                assert!(!args.force);
                assert!(!args.all);
                assert!(!args.has_legacy_backup_flags());
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_backup_flags_still_parse() {
        let cli = Cli::try_parse_from(["ccr", "clean", "--days", "30", "--dry-run"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => {
                assert!(args.action.is_none());
                assert_eq!(args.days, Some(30));
                assert!(args.dry_run);
                assert!(!args.force);
                assert!(!args.all);
                assert!(args.has_legacy_backup_flags());
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_all_parses_as_planfiles_all_not_legacy_backups() {
        let cli = Cli::try_parse_from(["ccr", "clean", "--all"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => {
                assert!(args.action.is_none());
                assert!(args.all);
                assert!(!args.has_legacy_backup_flags());
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_planfiles_subcommand_parses() {
        let cli = Cli::try_parse_from(["ccr", "clean", "planfiles", "--dry-run"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => match args.action {
                Some(CleanAction::Planfiles(planfiles)) => {
                    assert!(planfiles.dry_run);
                    assert!(!planfiles.force);
                    assert!(!planfiles.all);
                }
                other => panic!("unexpected clean action: {:?}", other.map(|_| "other")),
            },
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_planfiles_all_subcommand_parses() {
        let cli = Cli::try_parse_from(["ccr", "clean", "planfiles", "--all", "--dry-run"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => match args.action {
                Some(CleanAction::Planfiles(planfiles)) => {
                    assert!(planfiles.all);
                    assert!(planfiles.dry_run);
                    assert!(!planfiles.force);
                }
                other => panic!("unexpected clean action: {:?}", other.map(|_| "other")),
            },
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_backups_subcommand_parses() {
        let cli =
            Cli::try_parse_from(["ccr", "clean", "backups", "--days", "30", "--dry-run"]).unwrap();

        match cli.command {
            Some(Commands::Clean(args)) => match args.action {
                Some(CleanAction::Backups(backups)) => {
                    assert_eq!(backups.days, 30);
                    assert!(backups.dry_run);
                    assert!(!backups.force);
                }
                other => panic!("unexpected clean action: {:?}", other.map(|_| "other")),
            },
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn clean_planfiles_inherits_global_yes() {
        let cli = Cli::try_parse_from(["ccr", "-y", "clean", "planfiles"]).unwrap();

        assert!(cli.auto_yes);
        match cli.command {
            Some(Commands::Clean(args)) => {
                assert!(matches!(args.action, Some(CleanAction::Planfiles(_))));
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn current_status_verbose_alias_parses() {
        let cli = Cli::try_parse_from(["ccr", "status", "--verbose"]).unwrap();

        match cli.command {
            Some(Commands::Current(args)) => {
                assert!(args.verbose);
                assert!(!args.json);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn current_json_parses() {
        let cli = Cli::try_parse_from(["ccr", "current", "--json"]).unwrap();

        match cli.command {
            Some(Commands::Current(args)) => {
                assert!(!args.verbose);
                assert!(args.json);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn show_verbose_alias_parses() {
        let cli = Cli::try_parse_from(["ccr", "show", "--verbose"]).unwrap();

        match cli.command {
            Some(Commands::Current(args)) => {
                assert!(args.verbose);
                assert!(!args.json);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_sync_history_bridge_flags_parse() {
        let cli = Cli::try_parse_from([
            "ccr",
            "codex",
            "sync-history",
            "--bridge",
            "official-custom",
            "--all-history",
            "--include-provider",
            "duckcoding",
            "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::SyncHistory {
                        provider,
                        bridge,
                        all_history,
                        include_providers,
                        dry_run,
                        action,
                        ..
                    }),
            }) => {
                assert!(provider.is_none());
                assert_eq!(bridge.as_deref(), Some("official-custom"));
                assert!(all_history);
                assert_eq!(include_providers, vec!["duckcoding".to_string()]);
                assert!(dry_run);
                assert!(action.is_none());
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_sync_history_provider_conflicts_with_bridge() {
        let err = match Cli::try_parse_from([
            "ccr",
            "codex",
            "sync-history",
            "--provider",
            "custom",
            "--bridge",
            "official-custom",
        ]) {
            Ok(_) => panic!("expected provider/bridge conflict"),
            Err(err) => err,
        };

        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn codex_sessions_trash_restore_flags_parse() {
        let cli = Cli::try_parse_from([
            "ccr",
            "codex",
            "sessions",
            "trash",
            "thread-a",
            "thread-b",
            "--codex-home",
            "D:/tmp/.codex",
        ])
        .unwrap();

        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Sessions {
                        action:
                            crate::cli::subcommands::codex::CodexSessionsAction::Trash {
                                session_ids,
                                codex_home,
                            },
                    }),
            }) => {
                assert_eq!(session_ids, vec!["thread-a", "thread-b"]);
                assert_eq!(codex_home.as_deref(), Some("D:/tmp/.codex"));
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }

        let cli = Cli::try_parse_from([
            "ccr",
            "codex",
            "sessions",
            "restore",
            "thread-a",
            "--codex-home",
            "D:/tmp/.codex",
        ])
        .unwrap();

        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Sessions {
                        action:
                            crate::cli::subcommands::codex::CodexSessionsAction::Restore {
                                session_ids,
                                codex_home,
                            },
                    }),
            }) => {
                assert_eq!(session_ids, vec!["thread-a"]);
                assert_eq!(codex_home.as_deref(), Some("D:/tmp/.codex"));
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_sessions_trash_list_parses() {
        let cli = Cli::try_parse_from(["ccr", "codex", "sessions", "trash-list"]).unwrap();

        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Sessions {
                        action:
                            crate::cli::subcommands::codex::CodexSessionsAction::TrashList {
                                codex_home,
                            },
                    }),
            }) => {
                assert!(codex_home.is_none());
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_fix_parses() {
        let cli = Cli::try_parse_from(["ccr", "codex", "fix"]).unwrap();
        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Fix {
                        dry_run,
                        repair_runtime,
                    }),
            }) => {
                assert!(!dry_run);
                assert!(!repair_runtime);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_fix_dry_run_parses() {
        let cli = Cli::try_parse_from(["ccr", "codex", "fix", "--dry-run"]).unwrap();
        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Fix {
                        dry_run,
                        repair_runtime,
                    }),
            }) => {
                assert!(dry_run);
                assert!(!repair_runtime);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }

    #[test]
    fn codex_fix_repair_runtime_parses_with_dry_run() {
        let cli =
            Cli::try_parse_from(["ccr", "codex", "fix", "--dry-run", "--repair-runtime"]).unwrap();
        match cli.command {
            Some(Commands::Codex {
                action:
                    Some(CodexAction::Fix {
                        dry_run,
                        repair_runtime,
                    }),
            }) => {
                assert!(dry_run);
                assert!(repair_runtime);
            }
            other => panic!("unexpected command: {:?}", other.map(|_| "other")),
        }
    }
}
