// 命令分发器
//
// 将 CLI 命令路由到对应的处理函数

use crate::cli::subcommands::{AllSyncAction, FolderAction};
use crate::cli::{CleanAction, Cli, Commands, DEFAULT_CLEAN_BACKUP_DAYS};
use crate::help;
use ccr_core::core::error::CcrError;
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
            Some(Commands::Current(args)) => {
                crate::commands::current_command(args.verbose, args.json).await
            }
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
            Some(Commands::Clean(args)) => match &args.action {
                Some(CleanAction::Planfiles(planfiles_args)) => {
                    crate::commands::clean_planfiles_command(
                        planfiles_args.dry_run,
                        auto_yes || planfiles_args.force,
                    )
                    .await
                }
                Some(CleanAction::Backups(backups_args)) => {
                    crate::commands::clean_backups_command(
                        backups_args.days,
                        backups_args.dry_run,
                        auto_yes || backups_args.force,
                    )
                    .await
                }
                None => {
                    if args.has_legacy_backup_flags() {
                        crate::commands::clean_backups_command(
                            args.days.unwrap_or(DEFAULT_CLEAN_BACKUP_DAYS),
                            args.dry_run,
                            auto_yes || args.force,
                        )
                        .await
                    } else {
                        crate::commands::clean_menu_command(auto_yes).await
                    }
                }
            },
            Some(Commands::Clear { force }) => {
                crate::commands::clear_command(auto_yes || *force).await
            }

            Some(Commands::Ui {
                action,
                port,
                backend_port,
            }) => Self::dispatch_ui(action, *port, *backend_port, auto_yes).await,

            Some(Commands::Sync { action }) => Self::dispatch_sync(action, auto_yes).await,

            Some(Commands::TempToken { action }) => Self::dispatch_temp_token(action).await,

            Some(Commands::Platform { action }) => Self::dispatch_platform(action).await,

            Some(Commands::Stats(args)) => Self::dispatch_stats(args.clone()).await,

            Some(Commands::Budget(args)) => Self::dispatch_budget(args.clone()).await,

            Some(Commands::Pricing(args)) => Self::dispatch_pricing(args.clone()).await,

            Some(Commands::Skills(args)) => {
                crate::commands::skills_cmd::skills_command(args.clone()).await
            }
            Some(Commands::Prompts(args)) => {
                crate::commands::prompts_cmd::prompts_command(args.clone()).await
            }

            Some(Commands::Check { action }) => Self::dispatch_check(action).await,
            Some(Commands::Doctor(args)) => crate::commands::doctor_command(args.clone()).await,

            Some(Commands::Codex { action }) => Self::dispatch_codex(action).await,
            Some(Commands::OpenCode { action }) => Self::dispatch_opencode(action).await,

            Some(Commands::Claude { action }) => Self::dispatch_claude(action).await,

            Some(Commands::Sessions(args)) => {
                crate::commands::sessions_cmd::execute(args.clone()).await
            }
            Some(Commands::Provider(args)) => {
                crate::commands::provider_cmd::execute(args.clone()).await
            }

            // 无子命令时的处理
            None => Self::handle_no_subcommand(cli).await,

            // 帮助命令
            Some(Commands::Help { path }) => {
                help::print_command_help(path);
                Ok(())
            }
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
                crate::commands::current_command(false, false).await
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
        use crate::cli::subcommands::platform::PlatformProfileAction;

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
            PlatformAction::Profile { action } => match action.as_ref() {
                PlatformProfileAction::Create {
                    platform_name,
                    name,
                    description,
                    base_url,
                    auth_token,
                    model,
                    small_fast_model,
                    provider,
                    provider_type,
                    account,
                    tags,
                    auth_mode,
                    disabled,
                    json,
                } => {
                    crate::commands::platform::platform_profile_create_command(
                        crate::commands::platform::PlatformProfileCreateArgs {
                            platform_name: platform_name.clone(),
                            name: name.clone(),
                            description: description.clone(),
                            base_url: base_url.clone(),
                            auth_token: auth_token.clone(),
                            model: model.clone(),
                            small_fast_model: small_fast_model.clone(),
                            provider: provider.clone(),
                            provider_type: provider_type.clone(),
                            account: account.clone(),
                            tags: tags.clone(),
                            auth_mode: auth_mode.clone(),
                            disabled: *disabled,
                            json: *json,
                        },
                    )
                    .await
                }
                PlatformProfileAction::SetField {
                    platform_name,
                    name,
                    field,
                    value,
                    value_json,
                    clear,
                    json,
                } => {
                    crate::commands::platform::platform_profile_set_field_command(
                        platform_name,
                        name,
                        field,
                        value.clone(),
                        value_json.clone(),
                        *clear,
                        *json,
                    )
                    .await
                }
                PlatformProfileAction::Enable {
                    platform_name,
                    name,
                    json,
                } => {
                    crate::commands::platform::platform_profile_enable_command(
                        platform_name,
                        name,
                        *json,
                    )
                    .await
                }
                PlatformProfileAction::Disable {
                    platform_name,
                    name,
                    force,
                    json,
                } => {
                    crate::commands::platform::platform_profile_disable_command(
                        platform_name,
                        name,
                        *force,
                        *json,
                    )
                    .await
                }
                PlatformProfileAction::Delete {
                    platform_name,
                    name,
                    force,
                    json,
                } => {
                    crate::commands::platform::platform_profile_delete_command(
                        platform_name,
                        name,
                        *force,
                        *json,
                    )
                    .await
                }
            },
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
        use crate::cli::subcommands::codex::CodexSyncHistoryAction;
        use crate::cli::subcommands::{CodexAction, CodexAuthAction};

        match action {
            // 无子命令时启动主 TUI 的 Codex Auth 视图
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
            Some(CodexAction::Env { name }) => {
                crate::commands::codex::env::env_command(name.as_deref()).await
            }
            Some(CodexAction::Quota {
                account,
                json,
                refresh,
            }) => {
                crate::commands::codex::quota::quota_command(account.as_deref(), *json, *refresh)
                    .await
            }
            Some(CodexAction::SyncHistory {
                provider,
                keep,
                max_age_days,
                dry_run,
                codex_home,
                action,
            }) => match action {
                None => {
                    crate::commands::codex::sync_history::sync_command(
                        provider.clone(),
                        *keep,
                        *max_age_days,
                        *dry_run,
                        codex_home.clone(),
                    )
                    .await
                }
                Some(CodexSyncHistoryAction::Status { codex_home }) => {
                    crate::commands::codex::sync_history::status_command(codex_home.clone()).await
                }
                Some(CodexSyncHistoryAction::Restore {
                    backup_dir,
                    codex_home,
                    restore_state,
                }) => {
                    crate::commands::codex::sync_history::restore_command(
                        backup_dir,
                        codex_home.clone(),
                        *restore_state,
                    )
                    .await
                }
                Some(CodexSyncHistoryAction::PruneBackups { keep, codex_home }) => {
                    crate::commands::codex::sync_history::prune_backups_command(
                        *keep,
                        codex_home.clone(),
                    )
                    .await
                }
            },
            // auth 子命令
            Some(CodexAction::Auth { action }) => match action {
                CodexAuthAction::Help => {
                    help::print_nested_subcommand_help(&["codex", "auth"]);
                    Ok(())
                }
                CodexAuthAction::Save {
                    name,
                    description,
                    force,
                } => {
                    crate::commands::codex::auth::save_command(name, description.clone(), *force)
                        .await
                }
                CodexAuthAction::Update {
                    name,
                    description,
                    clear_description,
                    json,
                } => {
                    crate::commands::codex::auth::update_command(
                        name,
                        description.clone(),
                        *clear_description,
                        *json,
                    )
                    .await
                }
                CodexAuthAction::List => crate::commands::codex::auth::list_command().await,
                CodexAuthAction::Sync => crate::commands::codex::auth::sync_command().await,
                CodexAuthAction::Repair { name } => {
                    crate::commands::codex::auth::repair_command(name).await
                }
                CodexAuthAction::Switch { name } => {
                    crate::commands::codex::auth::switch_command(name).await
                }
                CodexAuthAction::Delete { name, force } => {
                    crate::commands::codex::auth::delete_command(name, *force).await
                }
                CodexAuthAction::Rename {
                    old_name,
                    new_name,
                    force,
                    json,
                } => {
                    crate::commands::codex::auth::rename_command(old_name, new_name, *force, *json)
                        .await
                }
                CodexAuthAction::Current { json } => {
                    crate::commands::codex::auth::current_command(*json).await
                }
                CodexAuthAction::Export { no_secrets } => {
                    crate::commands::codex::auth::export_command(*no_secrets).await
                }
                CodexAuthAction::Import { replace, force } => {
                    crate::commands::codex::auth::import_command(*replace, *force).await
                }
            },
        }
    }

    /// OpenCode 命令分发
    async fn dispatch_opencode(
        action: &Option<crate::cli::subcommands::OpenCodeAction>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::{OpenCodeAction, OpenCodeAuthAction};

        match action {
            None => {
                #[cfg(feature = "tui")]
                {
                    crate::tui::opencode_auth::run_opencode_auth_tui()
                }
                #[cfg(not(feature = "tui"))]
                {
                    help::print_subcommand_help("opencode");
                    Ok(())
                }
            }
            Some(OpenCodeAction::Help) => {
                help::print_subcommand_help("opencode");
                Ok(())
            }
            Some(OpenCodeAction::Auth { action }) => match action {
                OpenCodeAuthAction::Help => {
                    help::print_nested_subcommand_help(&["opencode", "auth"]);
                    Ok(())
                }
                OpenCodeAuthAction::ImportCodex { dry_run, json } => {
                    crate::commands::opencode::auth::import_codex_command(*dry_run, *json).await
                }
            },
        }
    }

    /// Claude 命令分发
    async fn dispatch_claude(
        action: &Option<crate::cli::subcommands::ClaudeAction>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::{ClaudeAction, ClaudeAuthAction};

        match action {
            None => {
                #[cfg(feature = "tui")]
                {
                    crate::tui::claude_auth::run_claude_auth_tui()
                }
                #[cfg(not(feature = "tui"))]
                {
                    crate::commands::claude::auth::list::list_command().await
                }
            }
            Some(ClaudeAction::Help) => {
                help::print_subcommand_help("claude");
                Ok(())
            }
            Some(ClaudeAction::Auth { action }) => match action {
                ClaudeAuthAction::Help => {
                    help::print_nested_subcommand_help(&["claude", "auth"]);
                    Ok(())
                }
                ClaudeAuthAction::Save {
                    name,
                    description,
                    force,
                } => {
                    crate::commands::claude::auth::save::save_command(
                        name,
                        description.clone(),
                        *force,
                    )
                    .await
                }
                ClaudeAuthAction::List => crate::commands::claude::auth::list::list_command().await,
                ClaudeAuthAction::Switch { name } => {
                    crate::commands::claude::auth::switch::switch_command(name).await
                }
                ClaudeAuthAction::Delete { name, force } => {
                    crate::commands::claude::auth::delete::delete_command(name, *force).await
                }
                ClaudeAuthAction::Current { json } => {
                    crate::commands::claude::auth::current::current_command(*json).await
                }
            },
        }
    }

    /// Stats 命令分发
    async fn dispatch_stats(args: crate::commands::StatsArgs) -> Result<(), CcrError> {
        use ccr_core::core::ColorOutput;
        let mut color_output = ColorOutput;
        crate::commands::stats_command(args, &mut color_output).await
    }

    /// Budget 命令分发
    async fn dispatch_budget(args: crate::commands::BudgetArgs) -> Result<(), CcrError> {
        crate::commands::budget_command(args).await
    }

    /// Pricing 命令分发
    async fn dispatch_pricing(args: crate::commands::PricingArgs) -> Result<(), CcrError> {
        crate::commands::pricing_command(args).await
    }

    /// 显示版本信息
    fn show_version() {
        use ccr_core::core::ColorOutput;

        let version = env!("CARGO_PKG_VERSION");
        ColorOutput::banner(version);

        println!();
        ColorOutput::key_value("版本", version, 2);
        ColorOutput::key_value("作者", env!("CARGO_PKG_AUTHORS"), 2);
        ColorOutput::key_value("描述", env!("CARGO_PKG_DESCRIPTION"), 2);
        println!();

        ColorOutput::info("常用入口:");
        println!("  ccr --version         输出简短版本号（适合脚本和 CI）");
        println!("  ccr --help            查看任务导向总帮助");
        println!("  ccr help codex auth   查看 Codex Auth 帮助");
        println!("  ccr help opencode auth 查看 OpenCode 导入帮助");
        println!();

        ColorOutput::info("核心任务:");
        println!("  平台切换: ccr platform list -> ccr platform switch <platform>");
        println!("  Codex 账号: ccr codex auth current -> ccr codex auth switch <name>");
        println!(
            "  OpenCode 导入: ccr opencode auth import-codex --dry-run -> ccr opencode auth import-codex"
        );
        println!();

        ColorOutput::info("详细版本说明: ccr version --help");
    }
}

/// 处理错误的辅助函数
pub fn handle_error(e: CcrError) {
    use ccr_core::core::ColorOutput;

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
