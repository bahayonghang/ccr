//! Domain command registry for the Tauri invoke handler.
// The registry is the single source for both command metadata and the
// `tauri::generate_handler!` command list. Keep new commands inside the
// smallest matching domain module instead of expanding `commands::mod` again.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandModule {
    pub(crate) key: &'static str,
    pub(crate) title: &'static str,
    pub(crate) commands: &'static [&'static str],
}

macro_rules! define_command_registry {
    (
        $(
            $key:ident: $title:literal => [$($command:path),* $(,)?]
        ),* $(,)?
    ) => {
        pub(crate) const COMMAND_MODULES: &[CommandModule] = &[
            $(
                CommandModule {
                    key: stringify!($key),
                    title: $title,
                    commands: &[$(stringify!($command)),*],
                },
            )*
        ];

        #[cfg(not(target_os = "windows"))]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            tauri::generate_handler![
                $(
                    $($command,)*
                )*
            ]
        }

        #[cfg(target_os = "windows")]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            tauri::generate_handler![
                $(
                    $($command,)*
                )*
                super::wsl::wsl_list_distros,
                super::wsl::wsl_refresh_distros,
                super::wsl::wsl_clear_cache,
                super::wsl::wsl_cache_status,
                super::wsl::wsl_read_config,
                super::wsl::wsl_write_config,
                super::wsl::wsl_detect_cli,
                super::wsl::wsl_sync_config,
            ]
        }
    };
}

define_command_registry! {
    config: "配置管理" => [
        super::config::list_configs,
        super::config::switch_config,
        super::config::add_config,
        super::config::delete_config,
        super::config::rename_config,
        super::config::duplicate_config,
        super::config::validate_configs,
        super::config::import_config,
        super::config::restore_config,
        super::config::export_config,
        super::config::get_history,
        super::config::clear_history,
    ],
    settings_raw: "配置源文件" => [
        super::settings_raw::claude_get_settings_raw_text,
        super::settings_raw::claude_save_settings_raw_text,
        super::settings_raw::codex_get_config_raw_text,
        super::settings_raw::codex_save_config_raw_text,
        super::settings_raw::claude_list_settings_layers,
        super::settings_raw::codex_list_config_layers,
    ],
    system_prompts: "系统提示词" => [
        super::system_prompts::system_prompts_list,
        super::system_prompts::system_prompts_get,
        super::system_prompts::system_prompts_save,
        super::system_prompts::system_prompts_create,
    ],
    sync: "同步" => [
        super::sync::sync_push,
        super::sync::sync_pull,
        super::sync::list_sync_assets,
        super::sync::sync_push_asset,
        super::sync::sync_pull_asset,
        super::sync::sync_asset,
        super::sync::sync_all_assets,
        super::sync::sync_push_folder,
        super::sync::sync_pull_folder,
        super::sync::sync_status,
        super::sync::list_sync_folders,
        super::sync::add_sync_folder,
        super::sync::update_sync_folder,
        super::sync::delete_sync_folder,
        super::sync::set_webdav_config,
        super::sync::test_webdav_config,
        super::sync::clear_webdav_config,
    ],
    claude: "Claude Code" => [
        super::claude::claude_get_settings,
        super::claude::claude_update_settings,
        super::claude::claude_list_mcp_servers,
        super::claude::claude_add_mcp_server,
        super::claude::claude_update_mcp_server,
        super::claude::claude_delete_mcp_server,
        super::claude::claude_list_agents,
        super::claude::claude_add_agent,
        super::claude::claude_update_agent,
        super::claude::claude_delete_agent,
        super::claude::claude_list_slash_commands,
        super::claude::claude_add_slash_command,
        super::claude::claude_update_slash_command,
        super::claude::claude_delete_slash_command,
        super::claude::claude_list_plugins,
        super::claude::claude_add_plugin,
        super::claude::claude_update_plugin,
        super::claude::claude_delete_plugin,
        super::claude::claude_get_output_styles,
        super::claude::claude_update_output_styles,
        super::claude::claude_get_statusline,
        super::claude::claude_update_statusline,
        super::claude::claude_list_hooks,
        super::claude::claude_update_hooks,
        super::claude::claude_get_budgets,
        super::claude::claude_update_budgets,
        super::claude::claude_list_prompts,
        super::claude::claude_update_prompts,
    ],
    claude_profiles: "Claude Code Profiles" => [
        super::claude::claude_list_profiles,
        super::claude::claude_get_profile,
        super::claude::claude_add_profile,
        super::claude::claude_update_profile,
        super::claude::claude_delete_profile,
        super::claude::claude_apply_profile,
        super::claude::claude_export_profiles,
        super::claude::claude_get_profiles_raw,
        super::claude::claude_save_profiles_raw,
        super::claude::claude_list_auth_accounts,
        super::claude::claude_get_auth_current,
        super::claude::claude_save_auth,
        super::claude::claude_switch_auth,
        super::claude::claude_delete_auth,
    ],
    codex: "Codex" => [
        super::codex::codex_list_profiles,
        super::codex::codex_list_models,
        super::codex::codex_add_profile,
        super::codex::codex_update_profile,
        super::codex::codex_delete_profile,
        super::codex::codex_get_profile_env,
        super::codex::codex_apply_profile,
        super::codex::codex_export_profiles,
        super::codex::codex_get_profiles_raw,
        super::codex::codex_save_profiles_raw,
        super::codex::codex_get_settings,
        super::codex::codex_update_settings,
        super::codex::codex_list_mcp_servers,
        super::codex::codex_add_mcp_server,
        super::codex::codex_update_mcp_server,
        super::codex::codex_delete_mcp_server,
        super::codex::codex_list_agents,
        super::codex::codex_add_agent,
        super::codex::codex_update_agent,
        super::codex::codex_delete_agent,
        super::codex::codex_rename_agent,
        super::codex::codex_copy_agent,
        super::codex::codex_validate_agent_toml,
        super::codex::codex_list_agent_sources,
        super::codex::codex_add_agent_source,
        super::codex::codex_remove_agent_source,
        super::codex::codex_sync_agent_source,
        super::codex::codex_get_agent_source_catalog,
        super::codex::codex_install_source_agent,
        super::codex::codex_sync_source_install,
        super::codex::codex_accept_local_source_install,
        super::codex::codex_untrack_source_install,
        super::codex::codex_list_sessions,
        super::codex::codex_get_session_detail,
        super::codex::codex_export_session,
        super::codex::codex_clone_session,
        super::codex::codex_delete_session,
        super::codex::codex_get_usage,
        super::codex::codex_get_dashboard_overview,
        super::codex::codex_get_dashboard_usage_summary,
        super::codex::codex_list_auth_accounts,
        super::codex::codex_get_auth_current,
        super::codex::codex_save_auth,
        super::codex::codex_switch_auth,
        super::codex::codex_delete_auth,
        super::codex::codex_rename_auth,
        super::codex::codex_detect_process,
        super::codex::codex_oauth_login_start,
        super::codex::codex_oauth_login_completed,
        super::codex::codex_oauth_login_cancel,
        super::codex::codex_oauth_submit_callback_url,
        super::codex::codex_is_oauth_port_in_use,
        super::codex::codex_release_oauth_port,
        super::codex::codex_open_external_url,
        super::codex::codex_import_auth_payload,
        super::codex::codex_import_auth_from_local,
        super::codex::codex_add_auth_with_api_key,
        super::codex::codex_list_model_providers,
        super::codex::codex_save_model_provider,
        super::codex::codex_delete_model_provider,
        super::codex::codex_get_tray_snapshot,
        super::codex::codex_get_all_quotas,
        super::codex::codex_get_quota,
    ],
    gemini: "Gemini" => [
        super::gemini::gemini_get_settings,
        super::gemini::gemini_update_settings,
        super::gemini::gemini_list_mcp_servers,
        super::gemini::gemini_add_mcp_server,
        super::gemini::gemini_update_mcp_server,
        super::gemini::gemini_delete_mcp_server,
        super::gemini::gemini_list_slash_commands,
        super::gemini::gemini_add_slash_command,
        super::gemini::gemini_update_slash_command,
        super::gemini::gemini_delete_slash_command,
        super::gemini::gemini_list_extensions,
    ],
    opencode: "OpenCode" => [
        super::opencode::opencode_get_settings,
        super::opencode::opencode_update_settings,
        super::opencode::opencode_get_tui_settings,
        super::opencode::opencode_update_tui_settings,
        super::opencode::opencode_get_keybindings,
        super::opencode::opencode_update_keybindings,
        super::opencode::opencode_list_themes,
        super::opencode::opencode_list_agents,
        super::opencode::opencode_add_agent,
        super::opencode::opencode_update_agent,
        super::opencode::opencode_delete_agent,
        super::opencode::opencode_list_commands,
        super::opencode::opencode_add_command,
        super::opencode::opencode_update_command,
        super::opencode::opencode_delete_command,
        super::opencode::opencode_list_local_plugins,
    ],
    checkin: "CheckIn" => [
        super::checkin::list_providers,
        super::checkin::add_provider,
        super::checkin::update_provider,
        super::checkin::delete_provider,
        super::checkin::test_provider_connection,
        super::checkin::list_accounts,
        super::checkin::add_account,
        super::checkin::update_account,
        super::checkin::delete_account,
        super::checkin::batch_delete_accounts,
        super::checkin::execute_checkin,
        super::checkin::batch_checkin,
        super::checkin::start_checkin_job,
        super::checkin::get_checkin_job_status,
        super::checkin::get_checkin_records,
        super::checkin::get_balance,
        super::checkin::get_balance_history,
        super::checkin::get_balance_stats,
        super::checkin::export_checkin_data,
        super::checkin::export_checkin_stats,
        super::checkin::execute_cdk_recharge,
        super::checkin::get_cdk_history,
        super::checkin::list_waf_cookies,
        super::checkin::add_waf_cookie,
        super::checkin::delete_waf_cookie,
    ],
    system: "系统" => [
        super::system::get_system_info,
        super::system::check_version,
        super::system::health_check,
    ],
    converter: "转换器" => [
        super::converter::convert_config,
    ],
    ui_state: "UI 状态" => [
        super::ui_state::get_favorites,
        super::ui_state::add_favorite,
        super::ui_state::remove_favorite,
        super::ui_state::get_recent_items,
        super::ui_state::add_recent_item,
        super::ui_state::clear_recent_items,
    ],
    waf: "WAF" => [
        super::waf::open_waf_login,
        super::waf::get_waf_cookie_status,
        super::waf::validate_waf_cookie_for_account,
        super::waf::waf_deliver_cookie,
    ],
    unified_mcp: "统一 MCP" => [
        super::unified_mcp::unified_list_mcp_servers,
        super::unified_mcp::unified_add_mcp_server,
        super::unified_mcp::unified_update_mcp_server,
        super::unified_mcp::unified_delete_mcp_server,
    ],
    events: "事件查询" => [
        super::system::get_recent_events,
        super::system::get_monitoring_feed,
        super::system::append_frontend_logs,
        super::system::get_runtime_metrics,
    ],
    environment: "环境管理" => [
        super::environment::list_environments,
        super::environment::get_current_environment,
        super::environment::switch_environment,
        super::environment::refresh_environments,
        super::environment::env_list_platforms,
        super::environment::env_detect_cli,
    ],
    ssh: "SSH" => [
        super::ssh::ssh_list_hosts,
        super::ssh::ssh_add_host,
        super::ssh::ssh_connect,
        super::ssh::ssh_reconnect,
        super::ssh::ssh_disconnect,
        super::ssh::ssh_get_connection_state,
        super::ssh::ssh_probe_host_fingerprint,
        super::ssh::ssh_confirm_host_fingerprint,
        super::ssh::ssh_read_config,
        super::ssh::ssh_write_config,
        super::ssh::ssh_detect_cli,
        super::ssh::ssh_test_connection,
        super::ssh::ssh_list_keys,
    ],
    builtin_prompts: "内置提示词" => [
        super::builtin_prompts::list_builtin_prompts,
        super::builtin_prompts::get_builtin_prompt,
        super::builtin_prompts::get_builtin_prompts_by_category,
    ],
    pricing: "定价管理" => [
        super::pricing::set_pricing,
        super::pricing::get_pricing_list,
        super::pricing::remove_pricing,
        super::pricing::reset_pricing,
    ],
    mcp_presets: "MCP 预设" => [
        super::mcp_presets::list_mcp_presets,
        super::mcp_presets::get_mcp_preset,
        super::mcp_presets::install_mcp_preset,
        super::mcp_presets::install_mcp_preset_single,
        super::mcp_presets::list_source_mcp_servers,
        super::mcp_presets::sync_mcp_server,
        super::mcp_presets::sync_all_mcp_servers,
    ],
    usage_v2: "Usage V2" => [
        super::usage::get_usage_summary_v2,
        super::usage::get_usage_capabilities_v2,
        super::usage::get_usage_trends_v2,
        super::usage::get_usage_by_model_v2,
        super::usage::get_usage_by_provider_v2,
        super::usage::get_usage_by_project_v2,
        super::usage::get_usage_heatmap_v2,
        super::usage::get_usage_logs_v2,
        super::usage::get_usage_dashboard_v2,
        super::usage::get_home_usage_overview_v2,
        super::usage::ensure_session_index_v2,
        super::usage::get_session_index_job_status_v2,
        super::usage::start_usage_import_job_v2,
        super::usage::get_usage_import_job_status_v2,
        super::usage::cancel_usage_import_job_v2,
        super::usage::import_usage_v2,
        super::usage::import_all_usage_v2,
    ],
    command_exec: "命令执行" => [
        super::command_exec::execute_ccr_command,
        super::command_exec::list_ccr_commands,
        super::command_exec::get_ccr_command_help,
        super::command_exec::start_ccr_command_job,
        super::command_exec::get_ccr_command_job_status,
        super::command_exec::cancel_ccr_command_job,
    ],
    checkin_extended: "签到扩展" => [
        super::checkin::list_builtin_providers,
        super::checkin::add_builtin_provider,
        super::checkin::get_checkin_account_cookies,
        super::checkin::export_checkin_config,
        super::checkin::preview_checkin_import,
        super::checkin::import_checkin_config,
        super::checkin::get_account_dashboard,
    ],
    config_extended: "配置扩展" => [
        super::config::update_config,
        super::config::clean_backups,
    ],
    exit_confirm: "退出确认" => [
        super::config::get_skip_exit_confirm,
        super::config::set_skip_exit_confirm,
    ],
    shell: "Desktop Shell" => [
        super::shell::shell_get_preferences,
        super::shell::shell_set_preferences,
        super::shell::shell_show_main_window,
        super::shell::shell_request_quit,
        super::shell::shell_begin_tray_panel_drag,
        super::shell::shell_complete_tray_panel_drag,
        super::shell::shell_detect_skillport_app,
        super::shell::shell_open_skillport_app,
        super::shell::shell_detect_skills_manage_app,
        super::shell::shell_open_skills_manage_app,
    ],
    system_extended: "系统扩展" => [
        super::system::update_ccr,
        super::system::get_cli_version,
        super::system::get_cli_versions,
    ],
    install: "llmusage 安装流程" => [
        super::install::llmusage_install_detect,
        super::install::llmusage_install_probe_capabilities,
        super::install::llmusage_install_plan,
        super::install::llmusage_install_execute,
        super::install::llmusage_install_cancel,
        super::install::llmusage_install_recent,
        super::install::llmusage_install_manual_catalog,
        super::install::llmusage_install_check,
    ],
    claude_observer: "Claude Observer" => [
        super::claude_observer::claude_observer_get_insight,
        super::claude_observer::claude_observer_daily_trend,
        super::claude_observer::claude_observer_cost_breakdown,
        super::claude_observer::claude_observer_cache_stats,
        super::claude_observer::claude_observer_top_sessions,
        super::claude_observer::claude_observer_tool_heatmap,
        super::claude_observer::claude_observer_top_tools,
        super::claude_observer::claude_observer_subscription_get,
        super::claude_observer::claude_observer_subscription_set,
    ],
}

pub(crate) const WINDOWS_COMMAND_MODULES: &[CommandModule] = &[CommandModule {
    key: "wsl",
    title: "WSL",
    commands: &[
        stringify!(super::wsl::wsl_list_distros),
        stringify!(super::wsl::wsl_refresh_distros),
        stringify!(super::wsl::wsl_clear_cache),
        stringify!(super::wsl::wsl_cache_status),
        stringify!(super::wsl::wsl_read_config),
        stringify!(super::wsl::wsl_write_config),
        stringify!(super::wsl::wsl_detect_cli),
        stringify!(super::wsl::wsl_sync_config),
    ],
}];

fn registered_command_count() -> usize {
    let base_count = COMMAND_MODULES
        .iter()
        .map(|module| module.commands.len())
        .sum::<usize>();

    #[cfg(target_os = "windows")]
    {
        base_count
            + WINDOWS_COMMAND_MODULES
                .iter()
                .map(|module| module.commands.len())
                .sum::<usize>()
    }

    #[cfg(not(target_os = "windows"))]
    {
        base_count
    }
}

fn command_registry_is_well_formed() -> bool {
    let base_modules_are_well_formed = COMMAND_MODULES.iter().all(|module| {
        !module.key.is_empty() && !module.title.is_empty() && !module.commands.is_empty()
    });

    #[cfg(target_os = "windows")]
    let platform_modules_are_well_formed = WINDOWS_COMMAND_MODULES.iter().all(|module| {
        !module.key.is_empty() && !module.title.is_empty() && !module.commands.is_empty()
    });
    #[cfg(not(target_os = "windows"))]
    let platform_modules_are_well_formed = true;

    base_modules_are_well_formed
        && platform_modules_are_well_formed
        && registered_command_count() > 0
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fmt::Write as _;
    use std::path::PathBuf;

    use super::{
        COMMAND_MODULES, WINDOWS_COMMAND_MODULES, command_registry_is_well_formed,
        registered_command_count,
    };

    fn command_inventory_markdown() -> String {
        let base_count = COMMAND_MODULES
            .iter()
            .map(|module| module.commands.len())
            .sum::<usize>();
        let windows_count = base_count
            + WINDOWS_COMMAND_MODULES
                .iter()
                .map(|module| module.commands.len())
                .sum::<usize>();
        let mut output = String::from(
            "# Tauri Command Inventory\n\n> Generated from `commands/handler_registry.rs`; do not edit manually.\n\n",
        );
        writeln!(output, "- Base commands: {base_count}").expect("write inventory count");
        writeln!(output, "- Windows commands: {windows_count}").expect("write inventory count");
        writeln!(output, "- Base modules: {}\n", COMMAND_MODULES.len())
            .expect("write inventory count");
        output.push_str("| Module | Title | Commands |\n| --- | --- | ---: |\n");
        for module in COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                module.key,
                module.title,
                module.commands.len()
            )
            .expect("write inventory row");
        }
        for module in WINDOWS_COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` (Windows) | {} | {} |",
                module.key,
                module.title,
                module.commands.len()
            )
            .expect("write inventory row");
        }
        output
    }

    fn command_inventory_paths() -> [PathBuf; 2] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        [
            root.join("docs/reference/tauri-command-inventory.md"),
            root.join("docs/en/reference/tauri-command-inventory.md"),
        ]
    }

    #[test]
    fn command_registry_shape_matches_current_handler_surface() {
        assert_eq!(COMMAND_MODULES.len(), 30);

        #[cfg(target_os = "windows")]
        assert_eq!(registered_command_count(), 323);

        #[cfg(not(target_os = "windows"))]
        assert_eq!(registered_command_count(), 315);
    }

    #[test]
    fn command_registry_modules_are_well_formed() {
        assert!(command_registry_is_well_formed());
    }

    #[test]
    fn command_registry_paths_are_unique() {
        let mut seen = HashSet::new();

        for module in COMMAND_MODULES {
            for command in module.commands {
                assert!(
                    seen.insert(*command),
                    "duplicate command path in Tauri command registry: {command}"
                );
            }
        }

        #[cfg(target_os = "windows")]
        for module in super::WINDOWS_COMMAND_MODULES {
            for command in module.commands {
                assert!(
                    seen.insert(*command),
                    "duplicate Windows command path in Tauri command registry: {command}"
                );
            }
        }
    }

    #[test]
    fn command_inventory_document_matches_registry() {
        let expected = command_inventory_markdown();
        for path in command_inventory_paths() {
            if std::env::var_os("CCR_UPDATE_COMMAND_INVENTORY").is_some() {
                std::fs::create_dir_all(path.parent().expect("inventory parent"))
                    .expect("create inventory directory");
                std::fs::write(&path, &expected).expect("write command inventory");
            }
            let actual = std::fs::read_to_string(&path).expect("read command inventory");
            assert_eq!(actual, expected, "run `just tauri-command-inventory`");
        }
    }
}
