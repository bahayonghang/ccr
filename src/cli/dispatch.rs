// 命令分发器
//
// 将 CLI 命令路由到对应的处理函数

#[cfg(feature = "web")]
use crate::cli::subcommands::{AllSyncAction, FolderAction};
use crate::cli::{Cli, Commands};
use crate::core::error::CcrError;
use crate::help;
use std::result::Result;

/// 命令分发器
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// 分发并执行命令
    pub async fn dispatch(cli: &Cli) -> Result<(), CcrError> {
        Self::dispatch_async(cli).await
    }

    /// 异步分发并执行命令
    async fn dispatch_async(cli: &Cli) -> Result<(), CcrError> {
        let auto_yes = cli.auto_yes;

        match &cli.command {
            // 简单命令（无参数）
            Some(Commands::List) => crate::commands::list_command().await,
            Some(Commands::Current) => crate::commands::current_command().await,
            Some(Commands::Add) => crate::commands::add_command().await,
            Some(Commands::Validate) => crate::commands::validate_command().await,
            Some(Commands::Optimize) => crate::commands::optimize_command().await,
            Some(Commands::Version) => {
                Self::show_version();
                Ok(())
            }
            Some(Commands::Temp) => crate::commands::temp_command().await,

            // 带参数命令
            Some(Commands::Switch { config_name }) => {
                crate::commands::switch_command(config_name).await
            }
            Some(Commands::Delete { config_name, force }) => {
                crate::commands::delete_command(config_name, auto_yes || *force).await
            }
            Some(Commands::Enable { config_name }) => {
                crate::commands::enable_command(config_name).await
            }
            Some(Commands::Disable { config_name, force }) => {
                crate::commands::disable_command(config_name, auto_yes || *force).await
            }
            Some(Commands::History { limit, filter_type }) => {
                crate::commands::history_command(Some(*limit), filter_type.clone()).await
            }
            Some(Commands::Update { check, branch }) => {
                crate::commands::update_command(*check, branch).await
            }
            Some(Commands::Init { force }) => {
                crate::commands::init_command(auto_yes || *force).await
            }
            Some(Commands::Export { output, no_secrets }) => {
                crate::commands::export_command(output.clone(), !*no_secrets).await
            }
            Some(Commands::Import {
                input,
                merge,
                backup,
                force,
            }) => {
                let mode = if *merge {
                    crate::commands::ImportMode::Merge
                } else {
                    crate::commands::ImportMode::Replace
                };
                crate::commands::import_command(input.clone(), mode, *backup, auto_yes || *force)
                    .await
            }
            Some(Commands::Clean {
                days,
                dry_run,
                force,
            }) => crate::commands::clean_command(*days, *dry_run, auto_yes || *force).await,
            Some(Commands::Clear { force }) => {
                crate::commands::clear_command(auto_yes || *force).await
            }

            // 带特性的命令
            #[cfg(feature = "web")]
            Some(Commands::Web { port, no_browser }) => {
                crate::web::web_command(Some(*port), *no_browser).await
            }

            Some(Commands::Ui {
                action,
                port,
                backend_port,
            }) => Self::dispatch_ui(action, *port, *backend_port, auto_yes).await,

            #[cfg(feature = "web")]
            Some(Commands::Sync { action }) => Self::dispatch_sync(action, auto_yes).await,

            Some(Commands::TempToken { action }) => Self::dispatch_temp_token(action).await,

            Some(Commands::Platform { action }) => Self::dispatch_platform(action).await,

            #[cfg(feature = "web")]
            Some(Commands::Stats(args)) => Self::dispatch_stats(args.clone()).await,

            #[cfg(feature = "web")]
            Some(Commands::Budget(args)) => Self::dispatch_budget(args.clone()).await,

            #[cfg(feature = "web")]
            Some(Commands::Pricing(args)) => Self::dispatch_pricing(args.clone()).await,

            Some(Commands::Skills(args)) => {
                crate::commands::skills_cmd::skills_command(args.clone()).await
            }
            Some(Commands::Prompts(args)) => {
                crate::commands::prompts_cmd::prompts_command(args.clone()).await
            }

            Some(Commands::Check { action }) => Self::dispatch_check(action).await,

            Some(Commands::Codex { action }) => Self::dispatch_codex(action).await,

            Some(Commands::Sessions(args)) => {
                crate::commands::sessions_cmd::execute(args.clone()).await
            }
            Some(Commands::Provider(args)) => {
                crate::commands::provider_cmd::execute(args.clone()).await
            }

            // 无子命令时的处理
            None => Self::handle_no_subcommand(cli).await,

            // 帮助命令
            Some(Commands::Help { subcmd }) => match subcmd.as_deref() {
                Some(name) => {
                    help::print_subcommand_help(name);
                    Ok(())
                }
                None => {
                    help::print_top_help();
                    Ok(())
                }
            },
        }
    }

    /// 处理无子命令的情况（快捷切换或打开TUI）
    async fn handle_no_subcommand(cli: &Cli) -> Result<(), CcrError> {
        if let Some(config_name) = &cli.config_name {
            // 快捷切换配置
            crate::commands::switch_command(config_name).await
        } else {
            // 打开TUI配置选择器
            #[cfg(feature = "tui")]
            {
                crate::tui::run_tui()
            }
            #[cfg(not(feature = "tui"))]
            {
                crate::commands::current_command().await
            }
        }
    }

    /// UI 命令分发
    async fn dispatch_ui(
        action: &Option<crate::cli::subcommands::UiAction>,
        port: u16,
        backend_port: u16,
        auto_yes: bool,
    ) -> Result<(), CcrError> {
        match action {
            Some(crate::cli::subcommands::UiAction::Help) => {
                help::print_subcommand_help("ui");
                Ok(())
            }
            Some(crate::cli::subcommands::UiAction::Update) => {
                let ui_service = crate::services::ui_service::UiService::new()?;
                ui_service.update(auto_yes).await
            }
            None => crate::commands::ui_command(port, backend_port, auto_yes).await,
        }
    }

    /// Sync 命令分发
    #[cfg(feature = "web")]
    async fn dispatch_sync(
        action: &crate::cli::subcommands::SyncAction,
        _auto_yes: bool,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::SyncAction;

        match action {
            SyncAction::Help => {
                help::print_subcommand_help("sync");
                Ok(())
            }
            SyncAction::Folder { action } => Self::dispatch_folder(action),
            SyncAction::All { action } => Self::dispatch_all_sync(action).await,
            SyncAction::FolderSync(args) => {
                crate::commands::sync_cmd::sync_folder_specific_command(args).await
            }
            SyncAction::Config => crate::commands::sync_cmd::sync_config_command().await,
            SyncAction::Status => crate::commands::sync_cmd::sync_status_command().await,
            SyncAction::Push { force, interactive } => {
                if *interactive {
                    let mut selector = crate::commands::SyncContentSelector::new();
                    match selector.select_content() {
                        Ok(selection) => {
                            crate::commands::sync_cmd::sync_push_command_with_selection(
                                *force,
                                Some(selection),
                            )
                            .await
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    crate::commands::sync_cmd::sync_push_command(*force).await
                }
            }
            SyncAction::Pull { force } => {
                crate::commands::sync_cmd::sync_pull_command(*force).await
            }
        }
    }

    /// Folder 命令分发
    #[cfg(feature = "web")]
    fn dispatch_folder(action: &FolderAction) -> Result<(), CcrError> {
        match action {
            FolderAction::Help => {
                help::print_nested_subcommand_help(&["sync", "folder"]);
                Ok(())
            }
            FolderAction::List => crate::commands::sync_cmd::sync_folder_list_command(),
            FolderAction::Add {
                name,
                local_path,
                remote_path,
                description,
            } => crate::commands::sync_cmd::sync_folder_add_command(
                name,
                local_path,
                remote_path.as_ref(),
                description.as_ref(),
            ),
            FolderAction::Remove { name } => {
                crate::commands::sync_cmd::sync_folder_remove_command(name)
            }
            FolderAction::Info { name } => {
                crate::commands::sync_cmd::sync_folder_info_command(name)
            }
            FolderAction::Enable { name } => {
                crate::commands::sync_cmd::sync_folder_enable_command(name)
            }
            FolderAction::Disable { name } => {
                crate::commands::sync_cmd::sync_folder_disable_command(name)
            }
        }
    }

    /// AllSync 命令分发
    #[cfg(feature = "web")]
    async fn dispatch_all_sync(action: &AllSyncAction) -> Result<(), CcrError> {
        match action {
            AllSyncAction::Help => {
                help::print_nested_subcommand_help(&["sync", "all"]);
                Ok(())
            }
            AllSyncAction::Push { force } => {
                crate::commands::sync_cmd::sync_all_push_command(*force).await
            }
            AllSyncAction::Pull { force } => {
                crate::commands::sync_cmd::sync_all_pull_command(*force).await
            }
            AllSyncAction::Status => crate::commands::sync_cmd::sync_all_status_command().await,
        }
    }

    /// TempToken 命令分发
    async fn dispatch_temp_token(
        action: &crate::cli::subcommands::TempTokenAction,
    ) -> Result<(), CcrError> {
        match action {
            crate::cli::subcommands::TempTokenAction::Help => {
                help::print_subcommand_help("temp-token");
                Ok(())
            }
            crate::cli::subcommands::TempTokenAction::Set {
                token,
                base_url,
                model,
            } => crate::commands::temp_token_set(token, base_url.clone(), model.clone()).await,
            crate::cli::subcommands::TempTokenAction::Show => {
                crate::commands::temp_token_show().await
            }
            crate::cli::subcommands::TempTokenAction::Clear => {
                crate::commands::temp_token_clear().await
            }
        }
    }

    /// Platform 命令分发
    async fn dispatch_platform(
        action: &crate::cli::subcommands::PlatformAction,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::PlatformAction;

        match action {
            PlatformAction::Help => {
                help::print_subcommand_help("platform");
                Ok(())
            }
            PlatformAction::List { json } => crate::commands::platform_list_command(*json).await,
            PlatformAction::Switch { platform_name } => {
                crate::commands::platform_switch_command(platform_name).await
            }
            PlatformAction::Current { json } => {
                crate::commands::platform_current_command(*json).await
            }
            PlatformAction::Info {
                platform_name,
                json,
            } => crate::commands::platform_info_command(platform_name, *json).await,
            PlatformAction::Init { platform_name } => {
                crate::commands::platform_init_command(platform_name).await
            }
        }
    }

    /// Check 命令分发
    async fn dispatch_check(action: &crate::cli::subcommands::CheckAction) -> Result<(), CcrError> {
        match action {
            crate::cli::subcommands::CheckAction::Help => {
                help::print_subcommand_help("check");
                Ok(())
            }
            crate::cli::subcommands::CheckAction::Conflicts => {
                crate::commands::check_conflicts_command().await
            }
        }
    }

    /// Codex 命令分发
    async fn dispatch_codex(
        action: &Option<crate::cli::subcommands::CodexAction>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::{CodexAction, CodexAuthAction};

        match action {
            // 无子命令时启动 TUI
            None => {
                #[cfg(feature = "tui")]
                {
                    crate::tui::codex_auth::run_codex_auth_tui()
                }
                #[cfg(not(feature = "tui"))]
                {
                    // 无 TUI 时显示账号列表
                    crate::commands::codex::auth::list_command().await
                }
            }
            // Codex help 子命令
            Some(CodexAction::Help) => {
                help::print_subcommand_help("codex");
                Ok(())
            }
            // auth 子命令
            Some(CodexAction::Auth { action }) => match action {
                CodexAuthAction::Help => {
                    help::print_nested_subcommand_help(&["codex", "auth"]);
                    Ok(())
                }
                CodexAuthAction::Save {
                    name,
                    description,
                    expires_at,
                    force,
                } => {
                    crate::commands::codex::auth::save_command(
                        name,
                        description.clone(),
                        expires_at.clone(),
                        *force,
                    )
                    .await
                }
                CodexAuthAction::List => crate::commands::codex::auth::list_command().await,
                CodexAuthAction::Switch { name } => {
                    crate::commands::codex::auth::switch_command(name).await
                }
                CodexAuthAction::Delete { name, force } => {
                    crate::commands::codex::auth::delete_command(name, *force).await
                }
                CodexAuthAction::Current => crate::commands::codex::auth::current_command().await,
                CodexAuthAction::Export { no_secrets } => {
                    crate::commands::codex::auth::export_command(*no_secrets).await
                }
                CodexAuthAction::Import { replace, force } => {
                    crate::commands::codex::auth::import_command(*replace, *force).await
                }
            },
        }
    }

    /// Stats 命令分发
    #[cfg(feature = "web")]
    async fn dispatch_stats(args: crate::commands::StatsArgs) -> Result<(), CcrError> {
        use crate::core::ColorOutput;
        let mut color_output = ColorOutput;
        crate::commands::stats_command(args, &mut color_output).await
    }

    /// Budget 命令分发
    #[cfg(feature = "web")]
    async fn dispatch_budget(args: crate::commands::BudgetArgs) -> Result<(), CcrError> {
        crate::commands::budget_command(args).await
    }

    /// Pricing 命令分发
    #[cfg(feature = "web")]
    async fn dispatch_pricing(args: crate::commands::PricingArgs) -> Result<(), CcrError> {
        crate::commands::pricing_command(args).await
    }

    /// 显示版本信息
    fn show_version() {
        use crate::core::ColorOutput;

        let version = env!("CARGO_PKG_VERSION");
        ColorOutput::banner(version);

        println!();
        ColorOutput::key_value("版本", version, 2);
        ColorOutput::key_value("作者", env!("CARGO_PKG_AUTHORS"), 2);
        ColorOutput::key_value("描述", env!("CARGO_PKG_DESCRIPTION"), 2);
        println!();

        ColorOutput::info("CCR 特性:");
        println!("  直接写入 Claude Code 设置文件 (~/.claude/settings.json)");
        println!("  文件锁机制确保并发安全");
        println!("  完整的操作历史和审计追踪");
        println!("  配置备份和恢复功能");
        println!("  自动配置验证");
        println!("  WebDAV 云端同步（支持坚果云）");
        println!("  与 CCS 完全兼容");
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
}

/// 处理错误的辅助函数
pub fn handle_error(e: CcrError) {
    use crate::core::ColorOutput;

    eprintln!();
    ColorOutput::error(&e.user_message());
    eprintln!();

    if e.is_fatal() {
        ColorOutput::error("这是一个致命错误,程序无法继续");
        ColorOutput::info("请检查错误信息并修复后重试");
        ColorOutput::info("运行 'ccr --help' 查看帮助信息");
    }

    std::process::exit(e.exit_code());
}
