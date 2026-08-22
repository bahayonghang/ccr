// 命令分发器
//
// 将 CLI 命令路由到对应的处理函数

use crate::cli::definitions::{CleanAction, Cli, Commands, DEFAULT_CLEAN_BACKUP_DAYS};
use crate::cli::help;
use crate::cli::subcommands::{AllSyncAction, FolderAction, ProjectAction};
use ccr_core::core::error::CcrError;
use std::result::Result;

/// Injectable TUI entry points for the command dispatcher.
///
/// `ccr-cli` cannot depend on `ccr-tui` (which itself depends on `ccr-cli`),
/// so the binary crate constructs this struct and injects the TUI launchers
/// at runtime. Passing `None` to [`CommandDispatcher::dispatch`] selects the
/// non-TUI fallback branches instead.
pub struct TuiLaunchers {
    /// Launcher for the main profile-switching TUI (`ccr` with no subcommand).
    pub main: fn() -> Result<(), CcrError>,
    /// Launcher for the Codex auth TUI (`ccr codex` with no action).
    pub codex_auth: fn() -> Result<(), CcrError>,
    /// Launcher for the Grok auth TUI (`ccr grok auth` with no nested action).
    pub grok_auth: fn() -> Result<(), CcrError>,
    /// Launcher for the Claude auth TUI (`ccr claude` with no action).
    pub claude_auth: fn() -> Result<(), CcrError>,
}

/// 命令分发器
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// 分发并执行命令
    pub async fn dispatch(cli: &Cli, tui: Option<&TuiLaunchers>) -> Result<(), CcrError> {
        Self::dispatch_async(cli, tui).await
    }

    /// 异步分发并执行命令
    async fn dispatch_async(cli: &Cli, tui: Option<&TuiLaunchers>) -> Result<(), CcrError> {
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
            Some(Commands::Project { action }) => match action {
                ProjectAction::Init => crate::commands::project_init_command(auto_yes),
            },
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
                        planfiles_args.all,
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
                    } else if args.all {
                        crate::commands::clean_planfiles_command(false, auto_yes, true).await
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

            Some(Commands::Codex { action }) => Self::dispatch_codex(action, tui).await,

            Some(Commands::Claude { action }) => Self::dispatch_claude(action, tui).await,
            Some(Commands::Grok { action }) => Self::dispatch_grok(action, tui).await,

            Some(Commands::Sessions(args)) => {
                crate::commands::sessions_cmd::execute(args.clone()).await
            }
            Some(Commands::Provider(args)) => {
                crate::commands::provider_cmd::execute(args.clone()).await
            }

            // 无子命令时的处理
            None => Self::handle_no_subcommand(cli, tui).await,

            // 帮助命令
            Some(Commands::Help { path }) => {
                help::print_command_help(path);
                Ok(())
            }
        }
    }

    /// 处理无子命令的情况（快捷切换或打开TUI）
    async fn handle_no_subcommand(cli: &Cli, tui: Option<&TuiLaunchers>) -> Result<(), CcrError> {
        if let Some(config_name) = &cli.config_name {
            // 快捷切换配置
            Err(crate::commands::migration::legacy_shortcut_error(
                config_name,
            ))
        } else {
            // 打开TUI配置选择器（未注入启动器时降级为显示当前配置）
            match tui {
                Some(launchers) => (launchers.main)(),
                None => crate::commands::current_command(false, false).await,
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
        match action {
            PlatformAction::Help => {
                help::print_subcommand_help("platform");
                Ok(())
            }
            PlatformAction::List { json } => crate::commands::platform_list_command(*json).await,
            PlatformAction::Switch { .. } => Err(
                crate::commands::migration::legacy_platform_command_error("switch"),
            ),
            PlatformAction::Current { .. } => Err(
                crate::commands::migration::legacy_platform_command_error("current"),
            ),
            PlatformAction::Info { .. } => Err(
                crate::commands::migration::legacy_platform_command_error("info"),
            ),
            PlatformAction::Init { .. } => {
                Err(crate::commands::migration::legacy_platform_init_error())
            }
            PlatformAction::Profile { .. } => Err(
                crate::commands::migration::legacy_platform_command_error("profile"),
            ),
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
        tui: Option<&TuiLaunchers>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::codex::{CodexSessionsAction, CodexSyncHistoryAction};
        use crate::cli::subcommands::{CodexAction, CodexAuthAction, CodexProfileAction};

        match action {
            // 无子命令时启动主 TUI 的 Codex Auth 视图（未注入启动器时显示账号列表）
            None => match tui {
                Some(launchers) => (launchers.codex_auth)(),
                None => crate::commands::codex::auth::list_command().await,
            },
            // Codex help 子命令
            Some(CodexAction::Help) => {
                help::print_subcommand_help("codex");
                Ok(())
            }
            Some(CodexAction::Env { name }) => {
                crate::commands::codex::env::env_command(name.as_deref()).await
            }
            Some(CodexAction::Fix {
                dry_run,
                repair_runtime,
                doctor,
            }) => {
                crate::commands::codex::fix::fix_command(*dry_run, *repair_runtime, *doctor).await
            }
            Some(CodexAction::Profile { action }) => match action {
                CodexProfileAction::Help => {
                    help::print_nested_subcommand_help(&["codex", "profile"]);
                    Ok(())
                }
                CodexProfileAction::Init { json } => {
                    crate::commands::codex::profile::init_command(*json).await
                }
                CodexProfileAction::Open { json } => {
                    crate::commands::codex::profile::open_command(*json).await
                }
                CodexProfileAction::Current { json } => {
                    crate::commands::codex::profile::current_command(*json).await
                }
                CodexProfileAction::List { json } => {
                    crate::commands::codex::profile::list_command(*json).await
                }
                CodexProfileAction::Switch { name } => {
                    crate::commands::codex::profile::switch_command(name).await
                }
                CodexProfileAction::Create(args) => {
                    crate::commands::codex::profile::create_command(args.clone()).await
                }
                CodexProfileAction::SetField(args) => {
                    crate::commands::codex::profile::set_field_command(args.clone()).await
                }
                CodexProfileAction::Enable(args) => {
                    crate::commands::codex::profile::enable_command(args.clone()).await
                }
                CodexProfileAction::Disable(args) => {
                    crate::commands::codex::profile::disable_command(args.clone()).await
                }
                CodexProfileAction::Delete(args) => {
                    crate::commands::codex::profile::delete_command(args.clone()).await
                }
                CodexProfileAction::Off(args) => {
                    crate::commands::codex::profile::off_command(args.json).await
                }
            },
            Some(CodexAction::Quota {
                account,
                json,
                refresh,
            }) => {
                crate::commands::codex::quota::quota_command(account.as_deref(), *json, *refresh)
                    .await
            }
            Some(CodexAction::Sessions { action }) => match action {
                CodexSessionsAction::Trash {
                    session_ids,
                    codex_home,
                } => {
                    crate::commands::codex::sessions::trash_command(
                        session_ids.clone(),
                        codex_home.clone(),
                    )
                    .await
                }
                CodexSessionsAction::TrashList { codex_home } => {
                    crate::commands::codex::sessions::list_command(codex_home.clone()).await
                }
                CodexSessionsAction::Restore {
                    session_ids,
                    codex_home,
                } => {
                    crate::commands::codex::sessions::restore_command(
                        session_ids.clone(),
                        codex_home.clone(),
                    )
                    .await
                }
            },
            Some(CodexAction::SyncHistory {
                provider,
                bridge,
                keep,
                max_age_days,
                all_history,
                include_providers,
                dry_run,
                codex_home,
                action,
            }) => match action {
                None => {
                    let args = crate::commands::codex::sync_history::CodexSyncHistoryCommandArgs {
                        provider: provider.clone(),
                        bridge: bridge.clone(),
                        keep: *keep,
                        max_age_days: *max_age_days,
                        all_history: *all_history,
                        include_providers: include_providers.clone(),
                        dry_run: *dry_run,
                        codex_home: codex_home.clone(),
                    };
                    crate::commands::codex::sync_history::sync_command(args).await
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
                CodexAuthAction::Off { json } => {
                    crate::commands::codex::auth::off_command(*json).await
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

    /// Grok Build profile command dispatch.
    async fn dispatch_grok(
        action: &Option<crate::cli::subcommands::GrokAction>,
        tui: Option<&TuiLaunchers>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::{GrokAction, GrokAuthAction, GrokProfileAction};

        match action {
            None | Some(GrokAction::Help) => {
                help::print_subcommand_help("grok");
                Ok(())
            }
            Some(GrokAction::Auth { action }) => match action {
                None => match tui {
                    Some(launchers) => (launchers.grok_auth)(),
                    None => {
                        help::print_nested_subcommand_help(&["grok", "auth"]);
                        Ok(())
                    }
                },
                Some(GrokAuthAction::Help) => {
                    help::print_nested_subcommand_help(&["grok", "auth"]);
                    Ok(())
                }
                Some(GrokAuthAction::Current { json }) => {
                    crate::commands::grok::auth::current_command(*json).await
                }
                Some(GrokAuthAction::Off { json }) => {
                    crate::commands::grok::auth::off_command(*json).await
                }
            },
            Some(GrokAction::Profile { action }) => match action.as_ref() {
                GrokProfileAction::Help => {
                    help::print_nested_subcommand_help(&["grok", "profile"]);
                    Ok(())
                }
                GrokProfileAction::Init { json } => {
                    crate::commands::grok::profile::init_command(*json).await
                }
                GrokProfileAction::Open { json } => {
                    crate::commands::grok::profile::open_command(*json).await
                }
                GrokProfileAction::Current { json } => {
                    crate::commands::grok::profile::current_command(*json).await
                }
                GrokProfileAction::List { json } => {
                    crate::commands::grok::profile::list_command(*json).await
                }
                GrokProfileAction::Switch { name } => {
                    crate::commands::grok::profile::switch_command(name).await
                }
                GrokProfileAction::Create(args) => {
                    crate::commands::grok::profile::create_command(args.as_ref().clone()).await
                }
                GrokProfileAction::SetField(args) => {
                    crate::commands::grok::profile::set_field_command(args.clone()).await
                }
                GrokProfileAction::Enable(args) => {
                    crate::commands::grok::profile::enable_command(args.clone()).await
                }
                GrokProfileAction::Disable(args) => {
                    crate::commands::grok::profile::disable_command(args.clone()).await
                }
                GrokProfileAction::Delete(args) => {
                    crate::commands::grok::profile::delete_command(args.clone()).await
                }
                GrokProfileAction::Off(args) => {
                    crate::commands::grok::profile::off_command(args.json).await
                }
            },
        }
    }

    /// Claude 命令分发
    async fn dispatch_claude(
        action: &Option<crate::cli::subcommands::ClaudeAction>,
        tui: Option<&TuiLaunchers>,
    ) -> Result<(), CcrError> {
        use crate::cli::subcommands::{ClaudeAction, ClaudeAuthAction, ClaudeProfileAction};

        match action {
            // 无子命令时启动 Claude Auth TUI（未注入启动器时显示账号列表）
            None => match tui {
                Some(launchers) => (launchers.claude_auth)(),
                None => crate::commands::claude::auth::list::list_command().await,
            },
            Some(ClaudeAction::Help) => {
                help::print_subcommand_help("claude");
                Ok(())
            }
            Some(ClaudeAction::Profile { action }) => match action.as_ref() {
                ClaudeProfileAction::Help => {
                    help::print_nested_subcommand_help(&["claude", "profile"]);
                    Ok(())
                }
                ClaudeProfileAction::Init { json } => {
                    crate::commands::claude::profile::init_command(*json).await
                }
                ClaudeProfileAction::Open { json } => {
                    crate::commands::claude::profile::open_command(*json).await
                }
                ClaudeProfileAction::Current { json } => {
                    crate::commands::claude::profile::current_command(*json).await
                }
                ClaudeProfileAction::List { json } => {
                    crate::commands::claude::profile::list_command(*json).await
                }
                ClaudeProfileAction::Switch { name } => {
                    crate::commands::claude::profile::switch_command(name).await
                }
                ClaudeProfileAction::Create(args) => {
                    crate::commands::claude::profile::create_command(args.clone()).await
                }
                ClaudeProfileAction::SetField(args) => {
                    crate::commands::claude::profile::set_field_command(args.clone()).await
                }
                ClaudeProfileAction::Enable(args) => {
                    crate::commands::claude::profile::enable_command(args.clone()).await
                }
                ClaudeProfileAction::Disable(args) => {
                    crate::commands::claude::profile::disable_command(args.clone()).await
                }
                ClaudeProfileAction::Delete(args) => {
                    crate::commands::claude::profile::delete_command(args.clone()).await
                }
                ClaudeProfileAction::Off(args) => {
                    crate::commands::claude::profile::off_command(args.json).await
                }
            },
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
                ClaudeAuthAction::Off { json } => {
                    crate::commands::claude::auth::off::off_command(*json).await
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
        // 描述的是 ccr 二进制而非本 crate；dispatch 迁入 ccr-cli 后
        // env!("CARGO_PKG_DESCRIPTION") 会解析成 ccr-cli 的描述，故用常量保持原文案
        ColorOutput::key_value(
            "描述",
            "Claude Code Configuration Switcher (Rust implementation)",
            2,
        );
        println!();

        ColorOutput::info("常用入口:");
        println!("  ccr --version         输出简短版本号（适合脚本和 CI）");
        println!("  ccr --help            查看任务导向总帮助");
        println!("  ccr help codex auth   查看 Codex Auth 帮助");
        println!("  ccr help grok auth    查看 Grok Auth 帮助");
        println!();

        ColorOutput::info("核心任务:");
        println!("  平台切换: ccr platform list -> ccr platform switch <platform>");
        println!("  Codex 账号: ccr codex auth current -> ccr codex auth switch <name>");
        println!("  官方登出: ccr claude auth off / ccr codex auth off / ccr grok auth off");
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
