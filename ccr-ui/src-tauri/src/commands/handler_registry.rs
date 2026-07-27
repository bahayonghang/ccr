//! Domain command registry for the Tauri invoke handler.
// The registry is the single source for both command metadata and the
// `tauri::generate_handler!` command list. Keep new commands inside the
// smallest matching domain module instead of expanding `commands::mod` again.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandRisk {
    ReadOnly,
    LocalMutation,
    SecretMutation,
    NetworkMutation,
    ProcessExecution,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandAuthorization {
    LocalUser,
    SecretAccess,
    SystemCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandConfirmation {
    None,
    UserGesture,
    OpaqueCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandConcurrency {
    Parallel,
    ModuleExclusive,
    Singleton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandAudit {
    MetadataOnly,
    Redacted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandSchema {
    Generated,
    LegacyJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CommandPlatform {
    Base,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandModule {
    pub(crate) key: &'static str,
    pub(crate) title: &'static str,
    pub(crate) commands: &'static [&'static str],
    pub(crate) default_risk: CommandRisk,
    pub(crate) schema: CommandSchema,
    pub(crate) platform: CommandPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct CommandDescriptor {
    pub(crate) id: &'static str,
    pub(crate) handler_path: &'static str,
    pub(crate) module: &'static str,
    pub(crate) title: &'static str,
    pub(crate) platform: CommandPlatform,
    pub(crate) risk: CommandRisk,
    pub(crate) input_schema: CommandSchema,
    pub(crate) output_schema: CommandSchema,
    pub(crate) timeout_ms: u64,
    pub(crate) concurrency: CommandConcurrency,
    pub(crate) confirmation: CommandConfirmation,
    pub(crate) authorization: CommandAuthorization,
    pub(crate) audit: CommandAudit,
}

impl CommandDescriptor {
    fn from_module(module: &'static CommandModule, handler_path: &'static str) -> Self {
        let id = handler_path.rsplit("::").next().unwrap_or(handler_path);
        let risk = effective_risk(module.default_risk, id);
        let (timeout_ms, concurrency, confirmation, authorization, audit) =
            capability_policy(risk, module.default_risk, id);

        Self {
            id,
            handler_path,
            module: module.key,
            title: module.title,
            platform: module.platform,
            risk,
            input_schema: module.schema,
            output_schema: module.schema,
            timeout_ms,
            concurrency,
            confirmation,
            authorization,
            audit,
        }
    }

    pub(crate) fn is_typed(self) -> bool {
        self.input_schema == CommandSchema::Generated
            && self.output_schema == CommandSchema::Generated
    }
}

fn effective_risk(default_risk: CommandRisk, command: &str) -> CommandRisk {
    const DESTRUCTIVE_PREFIXES: &[&str] = &[
        "clear_", "clean_", "delete_", "remove_", "reset_", "restore_",
    ];
    const READ_PREFIXES: &[&str] = &[
        "check_",
        "detect_",
        "env_detect_",
        "export_",
        "get_",
        "health_",
        "is_",
        "list_",
        "preview_",
        "probe_",
        "read_",
        "validate_",
    ];

    if DESTRUCTIVE_PREFIXES
        .iter()
        .any(|prefix| command_has_action(command, prefix))
    {
        CommandRisk::Destructive
    } else if READ_PREFIXES
        .iter()
        .any(|prefix| command_has_action(command, prefix))
    {
        CommandRisk::ReadOnly
    } else {
        default_risk
    }
}

fn command_has_action(command: &str, prefix: &str) -> bool {
    let action = prefix.trim_end_matches('_');
    command == action
        || command.starts_with(prefix)
        || command.ends_with(&format!("_{action}"))
        || command.contains(&format!("_{action}_"))
}

fn capability_policy(
    risk: CommandRisk,
    default_risk: CommandRisk,
    command: &str,
) -> (
    u64,
    CommandConcurrency,
    CommandConfirmation,
    CommandAuthorization,
    CommandAudit,
) {
    let (timeout_ms, concurrency, confirmation, mut authorization, mut audit) = match risk {
        CommandRisk::ReadOnly => (
            15_000,
            CommandConcurrency::Parallel,
            CommandConfirmation::None,
            CommandAuthorization::LocalUser,
            CommandAudit::MetadataOnly,
        ),
        CommandRisk::LocalMutation => (
            30_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::LocalUser,
            CommandAudit::MetadataOnly,
        ),
        CommandRisk::SecretMutation => (
            30_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
        CommandRisk::NetworkMutation => (
            120_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
        CommandRisk::ProcessExecution => (
            120_000,
            CommandConcurrency::Singleton,
            if command == "llmusage_install_execute" {
                CommandConfirmation::OpaqueCapability
            } else {
                CommandConfirmation::UserGesture
            },
            CommandAuthorization::SystemCapability,
            CommandAudit::Redacted,
        ),
        CommandRisk::Destructive => (
            60_000,
            CommandConcurrency::ModuleExclusive,
            CommandConfirmation::UserGesture,
            CommandAuthorization::SecretAccess,
            CommandAudit::Redacted,
        ),
    };

    match default_risk {
        CommandRisk::SecretMutation | CommandRisk::NetworkMutation | CommandRisk::Destructive => {
            authorization = CommandAuthorization::SecretAccess;
            audit = CommandAudit::Redacted;
        }
        CommandRisk::ProcessExecution => {
            authorization = CommandAuthorization::SystemCapability;
            audit = CommandAudit::Redacted;
        }
        CommandRisk::ReadOnly | CommandRisk::LocalMutation => {}
    }

    (timeout_ms, concurrency, confirmation, authorization, audit)
}

pub(crate) fn command_descriptors() -> impl Iterator<Item = CommandDescriptor> {
    COMMAND_MODULES
        .iter()
        .chain(WINDOWS_COMMAND_MODULES)
        .flat_map(|module| {
            module
                .commands
                .iter()
                .map(|command| CommandDescriptor::from_module(module, command))
        })
}

pub(crate) fn command_descriptor(command: &str) -> Option<CommandDescriptor> {
    command_descriptors().find(|descriptor| descriptor.id == command)
}

macro_rules! define_command_registry {
    (
        $(
            $key:ident: $title:literal [$risk:ident, $schema:ident] => [$($command:path),* $(,)?]
        ),* $(,)?
    ) => {
        pub(crate) const COMMAND_MODULES: &[CommandModule] = &[
            $(
                CommandModule {
                    key: stringify!($key),
                    title: $title,
                    commands: &[$(stringify!($command)),*],
                    default_risk: CommandRisk::$risk,
                    schema: CommandSchema::$schema,
                    platform: CommandPlatform::Base,
                },
            )*
        ];

        #[cfg(not(target_os = "windows"))]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            let handler: Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> = Box::new(tauri::generate_handler![
                $(
                    $($command,)*
                )*
            ]);
            move |invoke: tauri::ipc::Invoke| {
                audit_invoke(&invoke);
                handler(invoke)
            }
        }

        #[cfg(target_os = "windows")]
        pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
            debug_assert!(command_registry_is_well_formed());
            let handler: Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> = Box::new(tauri::generate_handler![
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
            ]);
            move |invoke: tauri::ipc::Invoke| {
                audit_invoke(&invoke);
                handler(invoke)
            }
        }
    };
}

fn audit_invoke(invoke: &tauri::ipc::Invoke) {
    let command = invoke.message.command();
    if let Some(descriptor) = command_descriptor(command) {
        tracing::debug!(
            command = descriptor.id,
            module = descriptor.module,
            risk = ?descriptor.risk,
            audit = ?descriptor.audit,
            typed = descriptor.is_typed(),
            "tauri command accepted by capability manifest"
        );
    } else {
        tracing::warn!(command, "tauri command missing capability metadata");
    }
}

define_command_registry! {
    config: "配置管理" [LocalMutation, Generated] => [
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
    settings_raw: "配置源文件" [SecretMutation, LegacyJson] => [
        super::settings_raw::claude_get_settings_raw_text,
        super::settings_raw::claude_save_settings_raw_text,
        super::settings_raw::codex_get_config_raw_text,
        super::settings_raw::codex_save_config_raw_text,
        super::settings_raw::claude_list_settings_layers,
        super::settings_raw::codex_list_config_layers,
    ],
    system_prompts: "系统提示词" [LocalMutation, Generated] => [
        super::system_prompts::system_prompts_list,
        super::system_prompts::system_prompts_get,
        super::system_prompts::system_prompts_save,
        super::system_prompts::system_prompts_create,
    ],
    sync: "同步" [NetworkMutation, Generated] => [
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
    claude: "Claude Code" [SecretMutation, Generated] => [
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
    claude_profiles: "Claude Code Profiles" [SecretMutation, Generated] => [
        super::claude::claude_list_profiles,
        super::claude::claude_get_profile,
        super::claude::claude_add_profile,
        super::claude::claude_update_profile,
        super::claude::claude_delete_profile,
        super::claude::claude_apply_profile,
        super::claude::claude_export_profiles,
        super::claude::claude_get_profiles_raw,
        super::claude::claude_save_profiles_raw,
    ],
    claude_auth: "Claude Auth" [SecretMutation, Generated] => [
        super::claude::claude_list_auth_accounts,
        super::claude::claude_get_auth_current,
        super::claude::claude_save_auth,
        super::claude::claude_switch_auth,
        super::claude::claude_delete_auth,
    ],
    codex: "Codex" [SecretMutation, Generated] => [
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
        super::codex::codex_get_tray_snapshot,
        super::codex::codex_get_all_quotas,
        super::codex::codex_get_quota,
    ],
    codex_auth: "Codex Auth" [SecretMutation, Generated] => [
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
    ],
    codex_model_providers: "Codex Model Providers" [SecretMutation, Generated] => [
        super::codex::codex_list_model_providers,
        super::codex::codex_save_model_provider,
        super::codex::codex_delete_model_provider,
    ],
    gemini: "Gemini" [SecretMutation, Generated] => [
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
    opencode: "OpenCode" [SecretMutation, Generated] => [
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
    checkin: "CheckIn" [NetworkMutation, LegacyJson] => [
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
    system_info: "系统信息" [ReadOnly, Generated] => [
        super::system::get_system_info,
        super::system::check_version,
    ],
    system: "系统" [ReadOnly, LegacyJson] => [
        super::system::health_check,
    ],
    converter: "转换器" [ReadOnly, Generated] => [
        super::converter::convert_config,
    ],
    ui_state: "UI 状态" [LocalMutation, Generated] => [
        super::ui_state::get_favorites,
        super::ui_state::add_favorite,
        super::ui_state::remove_favorite,
        super::ui_state::get_recent_items,
        super::ui_state::add_recent_item,
        super::ui_state::clear_recent_items,
    ],
    waf: "WAF" [NetworkMutation, LegacyJson] => [
        super::waf::open_waf_login,
        super::waf::get_waf_cookie_status,
        super::waf::validate_waf_cookie_for_account,
        super::waf::waf_deliver_cookie,
    ],
    unified_mcp: "统一 MCP" [SecretMutation, LegacyJson] => [
        super::unified_mcp::unified_list_mcp_servers,
        super::unified_mcp::unified_add_mcp_server,
        super::unified_mcp::unified_update_mcp_server,
        super::unified_mcp::unified_delete_mcp_server,
    ],
    events: "事件查询" [ReadOnly, Generated] => [
        super::system::get_recent_events,
        super::system::get_monitoring_feed,
        super::system::append_frontend_logs,
        super::system::get_runtime_metrics,
    ],
    environment: "环境管理" [LocalMutation, Generated] => [
        super::environment::list_environments,
        super::environment::get_current_environment,
        super::environment::switch_environment,
        super::environment::refresh_environments,
    ],
    environment_legacy: "环境动态探测" [LocalMutation, LegacyJson] => [
        super::environment::env_list_platforms,
        super::environment::env_detect_cli,
    ],
    ssh: "SSH" [NetworkMutation, Generated] => [
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
    builtin_prompts: "内置提示词" [ReadOnly, Generated] => [
        super::builtin_prompts::list_builtin_prompts,
        super::builtin_prompts::get_builtin_prompt,
        super::builtin_prompts::get_builtin_prompts_by_category,
    ],
    pricing: "定价管理" [LocalMutation, LegacyJson] => [
        super::pricing::set_pricing,
        super::pricing::get_pricing_list,
        super::pricing::remove_pricing,
        super::pricing::reset_pricing,
    ],
    mcp_presets: "MCP 预设" [NetworkMutation, LegacyJson] => [
        super::mcp_presets::list_mcp_presets,
        super::mcp_presets::get_mcp_preset,
        super::mcp_presets::install_mcp_preset,
        super::mcp_presets::install_mcp_preset_single,
        super::mcp_presets::list_source_mcp_servers,
        super::mcp_presets::sync_mcp_server,
        super::mcp_presets::sync_all_mcp_servers,
    ],
    usage_v2: "Usage V2" [ReadOnly, Generated] => [
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
    command_exec: "命令执行" [ProcessExecution, Generated] => [
        super::command_exec::execute_ccr_command,
        super::command_exec::list_ccr_commands,
        super::command_exec::get_ccr_command_help,
        super::command_exec::start_ccr_command_job,
        super::command_exec::get_ccr_command_job_status,
        super::command_exec::cancel_ccr_command_job,
    ],
    checkin_extended: "签到扩展" [NetworkMutation, LegacyJson] => [
        super::checkin::list_builtin_providers,
        super::checkin::add_builtin_provider,
        super::checkin::get_checkin_account_cookies,
        super::checkin::export_checkin_config,
        super::checkin::preview_checkin_import,
        super::checkin::import_checkin_config,
        super::checkin::get_account_dashboard,
    ],
    config_extended: "配置扩展" [LocalMutation, LegacyJson] => [
        super::config::update_config,
        super::config::clean_backups,
    ],
    exit_confirm: "退出确认" [LocalMutation, Generated] => [
        super::config::get_skip_exit_confirm,
        super::config::set_skip_exit_confirm,
    ],
    shell: "Desktop Shell" [ProcessExecution, Generated] => [
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
    system_extended_legacy: "系统更新" [ProcessExecution, LegacyJson] => [
        super::system::update_ccr,
    ],
    system_extended: "CLI 版本探测" [ProcessExecution, Generated] => [
        super::system::get_cli_version,
        super::system::get_cli_versions,
    ],
    install: "llmusage 安装流程" [ProcessExecution, Generated] => [
        super::install::llmusage_install_detect,
        super::install::llmusage_install_probe_capabilities,
        super::install::llmusage_install_plan,
        super::install::llmusage_install_execute,
        super::install::llmusage_install_cancel,
        super::install::llmusage_install_recent,
        super::install::llmusage_install_manual_catalog,
        super::install::llmusage_install_check,
    ],
    claude_observer: "Claude Observer" [ReadOnly, Generated] => [
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
    default_risk: CommandRisk::ProcessExecution,
    schema: CommandSchema::LegacyJson,
    platform: CommandPlatform::Windows,
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
        COMMAND_MODULES, CommandAudit, CommandAuthorization, CommandConcurrency,
        CommandConfirmation, CommandDescriptor, CommandPlatform, CommandRisk, CommandSchema,
        WINDOWS_COMMAND_MODULES, command_descriptor, command_descriptors,
        command_registry_is_well_formed, registered_command_count,
    };

    #[derive(serde::Serialize)]
    struct CommandManifest {
        schema_version: u32,
        base_command_count: usize,
        windows_command_count: usize,
        typed_command_count: usize,
        commands: Vec<CommandDescriptor>,
    }

    fn command_manifest() -> CommandManifest {
        let commands = command_descriptors().collect::<Vec<_>>();
        let base_command_count = commands
            .iter()
            .filter(|descriptor| descriptor.platform == CommandPlatform::Base)
            .count();
        let windows_command_count = commands.len();
        let typed_command_count = commands
            .iter()
            .filter(|descriptor| {
                descriptor.platform == CommandPlatform::Base && descriptor.is_typed()
            })
            .count();

        CommandManifest {
            schema_version: 1,
            base_command_count,
            windows_command_count,
            typed_command_count,
            commands,
        }
    }

    fn serialized_enum<T: serde::Serialize>(value: T) -> String {
        serde_json::to_string(&value)
            .expect("serialize command capability enum")
            .trim_matches('"')
            .to_string()
    }

    fn command_inventory_markdown() -> String {
        let manifest = command_manifest();
        let mut output = String::from(
            "# Tauri Command Inventory\n\n> Generated from `commands/handler_registry.rs`; do not edit manually.\n\n",
        );
        writeln!(output, "- Base commands: {}", manifest.base_command_count)
            .expect("write inventory count");
        writeln!(
            output,
            "- Windows commands: {}",
            manifest.windows_command_count
        )
        .expect("write inventory count");
        writeln!(output, "- Base modules: {}\n", COMMAND_MODULES.len())
            .expect("write inventory count");
        writeln!(
            output,
            "- Capability metadata: {}/{}",
            manifest.base_command_count, manifest.base_command_count
        )
        .expect("write metadata coverage");
        writeln!(
            output,
            "- Generated typed commands: {}/{} ({:.2}%)\n",
            manifest.typed_command_count,
            manifest.base_command_count,
            manifest.typed_command_count as f64 * 100.0 / manifest.base_command_count as f64
        )
        .expect("write typed coverage");
        output.push_str(
            "| Module | Title | Platform | Commands | Default risk | Schema |\n| --- | --- | --- | ---: | --- | --- |\n",
        );
        for module in COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` | {} | base | {} | `{}` | `{}` |",
                module.key,
                module.title,
                module.commands.len(),
                serialized_enum(module.default_risk),
                serialized_enum(module.schema)
            )
            .expect("write inventory row");
        }
        for module in WINDOWS_COMMAND_MODULES {
            writeln!(
                output,
                "| `{}` | {} | windows | {} | `{}` | `{}` |",
                module.key,
                module.title,
                module.commands.len(),
                serialized_enum(module.default_risk),
                serialized_enum(module.schema)
            )
            .expect("write inventory row");
        }
        output
    }

    fn command_manifest_json() -> String {
        let mut output =
            serde_json::to_string_pretty(&command_manifest()).expect("serialize command manifest");
        output.push('\n');
        output
    }

    fn command_capabilities_typescript() -> String {
        let manifest = command_manifest_json();
        let mut output = [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "export type CommandRisk = 'read_only' | 'local_mutation' | 'secret_mutation' | 'network_mutation' | 'process_execution' | 'destructive'\n",
            "export type CommandSchema = 'generated' | 'legacy_json'\n",
            "export type CommandPlatform = 'base' | 'windows'\n\n",
            "export interface CommandCapability {\n",
            "  id: string\n",
            "  handler_path: string\n",
            "  module: string\n",
            "  title: string\n",
            "  platform: CommandPlatform\n",
            "  risk: CommandRisk\n",
            "  input_schema: CommandSchema\n",
            "  output_schema: CommandSchema\n",
            "  timeout_ms: number\n",
            "  concurrency: 'parallel' | 'module_exclusive' | 'singleton'\n",
            "  confirmation: 'none' | 'user_gesture' | 'opaque_capability'\n",
            "  authorization: 'local_user' | 'secret_access' | 'system_capability'\n",
            "  audit: 'metadata_only' | 'redacted'\n",
            "}\n\n",
            "export const COMMAND_MANIFEST = ",
        ]
        .concat();
        output.push_str(manifest.trim_end());
        output.push_str(
            &[
                " as const satisfies {\n",
                "  schema_version: number\n",
                "  base_command_count: number\n",
                "  windows_command_count: number\n",
                "  typed_command_count: number\n",
                "  commands: readonly CommandCapability[]\n",
                "}\n\n",
                "export type TauriCommandName = (typeof COMMAND_MANIFEST.commands)[number]['id']\n",
            ]
            .concat(),
        );
        output
    }

    fn command_exec_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CommandCatalog } from '@/types/generated/command_exec/CommandCatalog'\n",
            "import type { CommandExecutionResult } from '@/types/generated/command_exec/CommandExecutionResult'\n",
            "import type { CommandHelpResponse } from '@/types/generated/command_exec/CommandHelpResponse'\n",
            "import type { CommandJobSnapshot } from '@/types/generated/command_exec/CommandJobSnapshot'\n",
            "import type { StartCommandJobResponse } from '@/types/generated/command_exec/StartCommandJobResponse'\n\n",
            "export type ExecuteCcrCommandInput = {\n",
            "  command: string\n",
            "  args?: string[]\n",
            "  confirmationToken?: string | null\n",
            "}\n\n",
            "export const executeCcrCommand = (input: ExecuteCcrCommandInput): Promise<CommandExecutionResult> =>\n",
            "  invoke('execute_ccr_command', input)\n\n",
            "export const listCcrCommands = (): Promise<CommandCatalog> =>\n",
            "  invoke('list_ccr_commands')\n\n",
            "export const getCcrCommandHelp = (command: string): Promise<CommandHelpResponse> =>\n",
            "  invoke('get_ccr_command_help', { command })\n\n",
            "export const startCcrCommandJob = (input: ExecuteCcrCommandInput): Promise<StartCommandJobResponse> =>\n",
            "  invoke('start_ccr_command_job', input)\n\n",
            "export const getCcrCommandJobStatus = (jobId: string): Promise<CommandJobSnapshot> =>\n",
            "  invoke('get_ccr_command_job_status', { jobId })\n\n",
            "export const cancelCcrCommandJob = (jobId: string): Promise<CommandJobSnapshot> =>\n",
            "  invoke('cancel_ccr_command_job', { jobId })\n",
        ]
        .concat()
    }

    fn sync_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { SyncAllAssetsInput } from '@/types/generated/sync/SyncAllAssetsInput'\n",
            "import type { SyncAssetInfo } from '@/types/generated/sync/SyncAssetInfo'\n",
            "import type { SyncAssetOperationInput } from '@/types/generated/sync/SyncAssetOperationInput'\n",
            "import type { SyncFolderInfo } from '@/types/generated/sync/SyncFolderInfo'\n",
            "import type { SyncOperationResult } from '@/types/generated/sync/SyncOperationResult'\n",
            "import type { SyncStatusInfo } from '@/types/generated/sync/SyncStatusInfo'\n",
            "import type { WebDavConfigDetails } from '@/types/generated/sync/WebDavConfigDetails'\n",
            "import type { WebDavConfigInput } from '@/types/generated/sync/WebDavConfigInput'\n",
            "import type { WebDavTestResult } from '@/types/generated/sync/WebDavTestResult'\n\n",
            "export type AddSyncFolderInput = { name: string; localPath: string; remotePath: string; description?: string }\n",
            "export type UpdateSyncFolderInput = { id: string; name?: string; enabled?: boolean; localPath?: string; remotePath?: string; description?: string }\n\n",
            "export const syncPush = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_push', { force })\n",
            "export const syncPull = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull', { force })\n",
            "export const listSyncAssets = (): Promise<SyncAssetInfo[]> => invoke('list_sync_assets')\n",
            "export const syncPushAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_push_asset', { payload })\n",
            "export const syncPullAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_pull_asset', { payload })\n",
            "export const syncAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_asset', { payload })\n",
            "export const syncAllAssets = (payload: SyncAllAssetsInput = {}): Promise<SyncOperationResult> => invoke('sync_all_assets', { payload })\n",
            "export const syncPushFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_push_folder', { id, force })\n",
            "export const syncPullFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull_folder', { id, force })\n",
            "export const syncStatus = (): Promise<SyncStatusInfo> => invoke('sync_status')\n",
            "export const listSyncFolders = (): Promise<SyncFolderInfo[]> => invoke('list_sync_folders')\n",
            "export const addSyncFolder = (input: AddSyncFolderInput): Promise<SyncFolderInfo> => invoke('add_sync_folder', input)\n",
            "export const updateSyncFolder = (input: UpdateSyncFolderInput): Promise<SyncFolderInfo> => invoke('update_sync_folder', input)\n",
            "export const deleteSyncFolder = (id: string): Promise<SyncOperationResult> => invoke('delete_sync_folder', { id })\n",
            "export const setWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavConfigDetails> => invoke('set_webdav_config', { payload })\n",
            "export const testWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavTestResult> => invoke('test_webdav_config', { payload })\n",
            "export const clearWebdavConfig = (): Promise<void> => invoke('clear_webdav_config')\n",
        ]
        .concat()
    }

    fn ssh_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { AddSshHostRequest } from '@/types/generated/ssh/AddSshHostRequest'\n",
            "import type { SshCliStatusDto } from '@/types/generated/ssh/SshCliStatusDto'\n",
            "import type { SshConnectionState } from '@/types/generated/ssh/SshConnectionState'\n",
            "import type { SshConnectionStateResponse } from '@/types/generated/ssh/SshConnectionStateResponse'\n",
            "import type { SshConnectResultDto } from '@/types/generated/ssh/SshConnectResultDto'\n",
            "import type { SshFingerprintProbeResult } from '@/types/generated/ssh/SshFingerprintProbeResult'\n",
            "import type { SshHostConfigDto } from '@/types/generated/ssh/SshHostConfigDto'\n",
            "import type { SshKeyInfoDto } from '@/types/generated/ssh/SshKeyInfoDto'\n",
            "import type { SshProbeFingerprintRequest } from '@/types/generated/ssh/SshProbeFingerprintRequest'\n\n",
            "export type SshConnectInput = { envId: string; password?: string }\n",
            "export type SshReadConfigInput = { envId: string; platform: string; path: string }\n",
            "export type SshWriteConfigInput = SshReadConfigInput & { content: string; enableBackup?: boolean }\n\n",
            "export const sshListHosts = (): Promise<SshHostConfigDto[]> => invoke('ssh_list_hosts')\n",
            "export const sshAddHost = (host: AddSshHostRequest): Promise<SshHostConfigDto> => invoke('ssh_add_host', { host })\n",
            "export const sshConnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_connect', input)\n",
            "export const sshReconnect = (input: SshConnectInput): Promise<SshConnectionState> => invoke('ssh_reconnect', input)\n",
            "export const sshDisconnect = (): Promise<SshConnectionState> => invoke('ssh_disconnect')\n",
            "export const sshGetConnectionState = (envId?: string): Promise<SshConnectionStateResponse> => invoke('ssh_get_connection_state', { envId })\n",
            "export const sshProbeHostFingerprint = (request: SshProbeFingerprintRequest): Promise<SshFingerprintProbeResult> => invoke('ssh_probe_host_fingerprint', { request })\n",
            "export const sshConfirmHostFingerprint = (challengeId: string): Promise<void> => invoke('ssh_confirm_host_fingerprint', { request: { challenge_id: challengeId } })\n",
            "export const sshReadConfig = (input: SshReadConfigInput): Promise<string> => invoke('ssh_read_config', input)\n",
            "export const sshWriteConfig = (input: SshWriteConfigInput): Promise<void> => invoke('ssh_write_config', input)\n",
            "export const sshDetectCli = (envId: string): Promise<SshCliStatusDto[]> => invoke('ssh_detect_cli', { envId })\n",
            "export const sshTestConnection = (envId: string): Promise<SshConnectResultDto> => invoke('ssh_test_connection', { envId })\n",
            "export const sshListKeys = (): Promise<SshKeyInfoDto[]> => invoke('ssh_list_keys')\n",
        ]
        .concat()
    }

    fn claude_auth_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ClaudeAuthActionResponse } from '@/types/generated/claude_auth/ClaudeAuthActionResponse'\n",
            "import type { ClaudeAuthCurrentResponse } from '@/types/generated/claude_auth/ClaudeAuthCurrentResponse'\n",
            "import type { ClaudeAuthListResponse } from '@/types/generated/claude_auth/ClaudeAuthListResponse'\n\n",
            "export type ClaudeAuthSaveRequest = {\n",
            "  name: string\n",
            "  description?: string | null\n",
            "  force?: boolean\n",
            "}\n\n",
            "export const listClaudeAuthAccounts = (): Promise<ClaudeAuthListResponse> =>\n",
            "  invoke('claude_list_auth_accounts')\n\n",
            "export const getClaudeAuthCurrent = (): Promise<ClaudeAuthCurrentResponse> =>\n",
            "  invoke('claude_get_auth_current')\n\n",
            "export const saveClaudeAuth = (request: ClaudeAuthSaveRequest): Promise<ClaudeAuthActionResponse> =>\n",
            "  invoke('claude_save_auth', {\n",
            "    name: request.name,\n",
            "    description: request.description ?? null,\n",
            "    force: request.force ?? false,\n",
            "  })\n\n",
            "export const switchClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>\n",
            "  invoke('claude_switch_auth', { name })\n\n",
            "export const deleteClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>\n",
            "  invoke('claude_delete_auth', { name })\n",
        ]
        .concat()
    }

    fn codex_auth_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CodexAuthActionResponse } from '@/types/generated/codex_auth/CodexAuthActionResponse'\n",
            "import type { CodexAuthCurrentResponse } from '@/types/generated/codex_auth/CodexAuthCurrentResponse'\n",
            "import type { CodexAuthListResponse } from '@/types/generated/codex_auth/CodexAuthListResponse'\n",
            "import type { CodexAuthImportPayload } from '@/types/generated/codex_auth/CodexAuthImportPayload'\n",
            "import type { CodexAuthMutationResponse } from '@/types/generated/codex_auth/CodexAuthMutationResponse'\n",
            "import type { CodexAuthProcessResponse } from '@/types/generated/codex_auth/CodexAuthProcessResponse'\n",
            "import type { CodexAuthRenameResponse } from '@/types/generated/codex_auth/CodexAuthRenameResponse'\n",
            "import type { CodexApiKeyAddPayload } from '@/types/generated/codex_auth/CodexApiKeyAddPayload'\n",
            "import type { CodexModelProviderDeleteResponse } from '@/types/generated/codex_auth/CodexModelProviderDeleteResponse'\n",
            "import type { CodexModelProvidersResponse } from '@/types/generated/codex_auth/CodexModelProvidersResponse'\n",
            "import type { CodexModelProviderSaveResponse } from '@/types/generated/codex_auth/CodexModelProviderSaveResponse'\n",
            "import type { CodexModelProviderUpsertPayload } from '@/types/generated/codex_auth/CodexModelProviderUpsertPayload'\n\n",
            "import type { CodexOAuthStartResponse } from '@/types/generated/codex_auth/CodexOAuthStartResponse'\n",
            "import type { OAuthPortReleaseReport } from '@/types/generated/codex_auth/OAuthPortReleaseReport'\n\n",
            "export type CodexAuthSaveRequest = { name: string; description?: string; force?: boolean }\n\n",
            "export const listCodexAuthAccounts = (): Promise<CodexAuthListResponse> => invoke('codex_list_auth_accounts')\n",
            "export const getCodexAuthCurrent = (): Promise<CodexAuthCurrentResponse> => invoke('codex_get_auth_current')\n",
            "export const saveCodexAuth = (request: CodexAuthSaveRequest): Promise<CodexAuthActionResponse> =>\n",
            "  invoke('codex_save_auth', { name: request.name, description: request.description ?? null, force: request.force ?? false })\n",
            "export const switchCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_switch_auth', { name })\n",
            "export const deleteCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_delete_auth', { name })\n",
            "export const renameCodexAuth = (oldName: string, newName: string, force = false): Promise<CodexAuthRenameResponse> =>\n",
            "  invoke('codex_rename_auth', { oldName, newName, force })\n",
            "export const detectCodexProcess = (): Promise<CodexAuthProcessResponse> => invoke('codex_detect_process')\n",
            "export const codexOAuthLoginStart = (): Promise<CodexOAuthStartResponse> => invoke('codex_oauth_login_start')\n",
            "export const codexOAuthLoginCompleted = (loginId: string, preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>\n",
            "  invoke('codex_oauth_login_completed', { loginId, preferredAccountName: preferredAccountName ?? null })\n",
            "export const codexOAuthLoginCancel = (loginId?: string | null): Promise<void> =>\n",
            "  invoke('codex_oauth_login_cancel', { loginId: loginId ?? null })\n",
            "export const codexOAuthSubmitCallbackUrl = (loginId: string, callbackUrl: string): Promise<void> =>\n",
            "  invoke('codex_oauth_submit_callback_url', { loginId, callbackUrl })\n",
            "export const codexIsOAuthPortInUse = (): Promise<boolean> => invoke('codex_is_oauth_port_in_use')\n",
            "export const codexReleaseOAuthPort = (): Promise<OAuthPortReleaseReport> => invoke('codex_release_oauth_port')\n",
            "export const codexOpenExternalUrl = (url: string): Promise<void> => invoke('codex_open_external_url', { url })\n",
            "export const codexImportAuthPayload = (payload: CodexAuthImportPayload): Promise<CodexAuthMutationResponse> =>\n",
            "  invoke('codex_import_auth_payload', { payload })\n",
            "export const codexImportAuthFromLocal = (preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>\n",
            "  invoke('codex_import_auth_from_local', { preferredAccountName: preferredAccountName ?? null })\n",
            "export const codexAddAuthWithApiKey = (payload: CodexApiKeyAddPayload): Promise<CodexAuthMutationResponse> =>\n",
            "  invoke('codex_add_auth_with_api_key', { payload })\n",
            "export const codexListModelProviders = (): Promise<CodexModelProvidersResponse> => invoke('codex_list_model_providers')\n",
            "export const codexSaveModelProvider = (payload: CodexModelProviderUpsertPayload): Promise<CodexModelProviderSaveResponse> =>\n",
            "  invoke('codex_save_model_provider', { payload })\n",
            "export const codexDeleteModelProvider = (providerId: string): Promise<CodexModelProviderDeleteResponse> =>\n",
            "  invoke('codex_delete_model_provider', { providerId })\n",
        ]
        .concat()
    }

    fn config_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ConfigInfo } from '@/types/generated/config/ConfigInfo'\n",
            "import type { ExportResult } from '@/types/generated/config/ExportResult'\n",
            "import type { HistoryEntry } from '@/types/generated/config/HistoryEntry'\n",
            "import type { ImportResult } from '@/types/generated/config/ImportResult'\n\n",
            "export type AddConfigInput = {\n",
            "  name: string\n",
            "  description?: string | null\n",
            "  baseUrl: string\n",
            "  authToken: string\n",
            "  model?: string | null\n",
            "  smallFastModel?: string | null\n",
            "  provider?: string | null\n",
            "  providerType?: string | null\n",
            "  account?: string | null\n",
            "  tags?: string[] | null\n",
            "}\n",
            "export type ImportConfigInput = { content: string; mode?: string; backup?: boolean }\n\n",
            "const confirmationTokenFor = (action: 'delete_config' | 'import_config' | 'restore_config') => `desktop-confirm:${action}`\n\n",
            "export const listConfigsTyped = (): Promise<ConfigInfo[]> => invoke('list_configs')\n",
            "export const switchConfigTyped = (name: string): Promise<string> => invoke('switch_config', { name })\n",
            "export const addConfigTyped = (input: AddConfigInput): Promise<string> => invoke('add_config', input)\n",
            "export const deleteConfigTyped = (name: string): Promise<string> => invoke('delete_config', { name, confirmationToken: confirmationTokenFor('delete_config') })\n",
            "export const renameConfigTyped = (oldName: string, newName: string): Promise<string> => invoke('rename_config', { oldName, newName })\n",
            "export const duplicateConfigTyped = (source: string, target: string): Promise<string> => invoke('duplicate_config', { source, target })\n",
            "export const validateConfigsTyped = (): Promise<string> => invoke('validate_configs')\n",
            "export const importConfigTyped = (input: ImportConfigInput): Promise<ImportResult> => invoke('import_config', { content: input.content, mode: input.mode ?? 'merge', backup: input.backup ?? true, confirmationToken: confirmationTokenFor('import_config') })\n",
            "export const restoreConfigTyped = (backupPath: string): Promise<string> => invoke('restore_config', { backupPath, confirmationToken: confirmationTokenFor('restore_config') })\n",
            "export const exportConfigTyped = (includeSecrets = false): Promise<ExportResult> => invoke('export_config', { includeSecrets })\n",
            "export const getHistoryTyped = (limit = 100): Promise<HistoryEntry[]> => invoke('get_history', { limit })\n",
            "export const clearHistoryTyped = (): Promise<string> => invoke('clear_history')\n",
        ]
        .concat()
    }

    fn ui_state_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CommandHistoryDto } from '@/types/generated/ui_state/CommandHistoryDto'\n",
            "import type { FavoriteCommandDto } from '@/types/generated/ui_state/FavoriteCommandDto'\n\n",
            "export const getFavorites = (): Promise<FavoriteCommandDto[]> => invoke('get_favorites')\n",
            "export const addFavorite = (command: string, args: string[], displayName: string | null | undefined, module: string): Promise<FavoriteCommandDto> =>\n",
            "  invoke('add_favorite', { command, args, displayName: displayName ?? null, module })\n",
            "export const removeFavorite = (id: string): Promise<boolean> => invoke('remove_favorite', { id })\n",
            "export const getRecentItems = (limit?: number): Promise<CommandHistoryDto[]> => invoke('get_recent_items', { limit })\n",
            "export const addRecentItem = (command: string, args: string[], success: boolean, durationMs: number): Promise<CommandHistoryDto> =>\n",
            "  invoke('add_recent_item', { command, args, success, durationMs })\n",
            "export const clearRecentItems = (): Promise<string> => invoke('clear_recent_items')\n",
        ]
        .concat()
    }

    fn system_info_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { SystemInfo } from '@/types/generated/system/SystemInfo'\n",
            "import type { VersionInfo } from '@/types/generated/system/VersionInfo'\n\n",
            "export const getSystemInfo = (): Promise<SystemInfo> => invoke('get_system_info')\n",
            "export const checkVersion = (): Promise<VersionInfo> => invoke('check_version')\n",
        ]
        .concat()
    }

    fn converter_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { ConverterRequestDto } from '@/types/generated/converter/ConverterRequestDto'\n",
            "import type { ConvertResult } from '@/types/generated/converter/ConvertResult'\n\n",
            "export const convertConfig = (request: ConverterRequestDto): Promise<ConvertResult> =>\n",
            "  invoke('convert_config', { request })\n",
        ]
        .concat()
    }

    fn exit_confirm_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n\n",
            "export const getSkipExitConfirm = (): Promise<boolean> => invoke('get_skip_exit_confirm')\n",
            "export const setSkipExitConfirm = (skip: boolean): Promise<void> => invoke('set_skip_exit_confirm', { skip })\n",
        ]
        .concat()
    }

    fn environment_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'\n\n",
            "export const listEnvironments = (): Promise<EnvironmentInfo[]> => invoke('list_environments')\n",
            "export const getCurrentEnvironment = (): Promise<EnvironmentInfo> => invoke('get_current_environment')\n",
            "export const switchEnvironment = (envId: string): Promise<EnvironmentInfo> => invoke('switch_environment', { envId })\n",
            "export const refreshEnvironments = (forceRefresh?: boolean): Promise<EnvironmentInfo[]> => invoke('refresh_environments', { forceRefresh })\n",
        ]
        .concat()
    }

    fn events_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { EventLogEntryDto } from '@/types/generated/events/EventLogEntryDto'\n",
            "import type { FrontendLogInputDto } from '@/types/generated/events/FrontendLogInputDto'\n",
            "import type { MonitoringEntryDto } from '@/types/generated/events/MonitoringEntryDto'\n",
            "import type { MonitoringFeedQueryDto } from '@/types/generated/events/MonitoringFeedQueryDto'\n",
            "import type { RuntimeMetricsResponse } from '@/types/generated/events/RuntimeMetricsResponse'\n\n",
            "export const getRecentEvents = (count?: number): Promise<EventLogEntryDto[]> => invoke('get_recent_events', { count })\n",
            "export const getMonitoringFeed = (query: MonitoringFeedQueryDto = {}): Promise<MonitoringEntryDto[]> => invoke('get_monitoring_feed', { query })\n",
            "export const appendFrontendLogs = (entries: FrontendLogInputDto[]): Promise<void> => invoke('append_frontend_logs', { entries })\n",
            "export const getRuntimeMetrics = (): Promise<RuntimeMetricsResponse> => invoke('get_runtime_metrics')\n",
        ]
        .concat()
    }

    fn shell_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { DesktopShellPreferences } from '@/types/generated/shell/DesktopShellPreferences'\n",
            "import type { SkillportAppStatus } from '@/types/generated/shell/SkillportAppStatus'\n",
            "import type { TrayPanelManualPosition } from '@/types/generated/shell/TrayPanelManualPosition'\n\n",
            "export const shellGetPreferences = (): Promise<DesktopShellPreferences> => invoke('shell_get_preferences')\n",
            "export const shellSetPreferences = (preferences: DesktopShellPreferences): Promise<DesktopShellPreferences> => invoke('shell_set_preferences', { preferences })\n",
            "export const shellShowMainWindow = (targetRoute?: string): Promise<void> => invoke('shell_show_main_window', { targetRoute })\n",
            "export const shellRequestQuit = (): Promise<void> => invoke('shell_request_quit')\n",
            "export const shellBeginTrayPanelDrag = (): Promise<void> => invoke('shell_begin_tray_panel_drag')\n",
            "export const shellCompleteTrayPanelDrag = (position?: TrayPanelManualPosition | null): Promise<void> => invoke('shell_complete_tray_panel_drag', { position: position ?? null })\n",
            "export const shellDetectSkillportApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skillport_app')\n",
            "export const shellOpenSkillportApp = (): Promise<void> => invoke('shell_open_skillport_app')\n",
            "export const shellDetectSkillsManageApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skills_manage_app')\n",
            "export const shellOpenSkillsManageApp = (): Promise<void> => invoke('shell_open_skills_manage_app')\n",
        ]
        .concat()
    }

    fn system_extended_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { CliVersionEntry } from '@/types/generated/system/CliVersionEntry'\n",
            "import type { CliVersionOptions } from '@/types/generated/system/CliVersionOptions'\n",
            "import type { CliVersionsOptions } from '@/types/generated/system/CliVersionsOptions'\n",
            "import type { CliVersionsResponse } from '@/types/generated/system/CliVersionsResponse'\n\n",
            "export const getCliVersions = (options?: CliVersionsOptions): Promise<CliVersionsResponse> => invoke('get_cli_versions', { options })\n",
            "export const getCliVersion = (options: CliVersionOptions): Promise<CliVersionEntry> => invoke('get_cli_version', { options })\n",
        ]
        .concat()
    }

    fn builtin_prompts_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { BuiltinPromptDto } from '@/types/generated/builtin_prompts/BuiltinPromptDto'\n\n",
            "export const listBuiltinPrompts = (): Promise<BuiltinPromptDto[]> => invoke('list_builtin_prompts')\n",
            "export const getBuiltinPrompt = (id: string): Promise<BuiltinPromptDto | null> => invoke('get_builtin_prompt', { id })\n",
            "export const getBuiltinPromptsByCategory = (category: string): Promise<BuiltinPromptDto[]> => invoke('get_builtin_prompts_by_category', { category })\n",
        ]
        .concat()
    }

    fn gemini_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
            "export const getGeminiSettings = (): Promise<OpenJsonValueDto> => invoke('gemini_get_settings')\n",
            "export const updateGeminiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_settings', { settings })\n",
            "export const listGeminiMcpServers = (): Promise<OpenJsonValueDto> => invoke('gemini_list_mcp_servers')\n",
            "export const addGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_mcp_server', { name, config })\n",
            "export const updateGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_mcp_server', { name, config })\n",
            "export const deleteGeminiMcpServer = (name: string): Promise<string> => invoke('gemini_delete_mcp_server', { name })\n",
            "export const listGeminiSlashCommands = (): Promise<OpenJsonValueDto> => invoke('gemini_list_slash_commands')\n",
            "export const addGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_slash_command', { name, config })\n",
            "export const updateGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_slash_command', { name, config })\n",
            "export const deleteGeminiSlashCommand = (name: string): Promise<string> => invoke('gemini_delete_slash_command', { name })\n",
            "export const listGeminiExtensions = (): Promise<OpenJsonValueDto> => invoke('gemini_list_extensions')\n",
        ]
        .concat()
    }

    fn opencode_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n",
            "import type { OpenCodePluginFileRecord } from '@/types/generated/opencode/OpenCodePluginFileRecord'\n",
            "import type { OpenCodeThemeRecord } from '@/types/generated/opencode/OpenCodeThemeRecord'\n\n",
            "export const getOpenCodeSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_settings')\n",
            "export const updateOpenCodeSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_settings', { settings })\n",
            "export const getOpenCodeTuiSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_tui_settings')\n",
            "export const updateOpenCodeTuiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_tui_settings', { settings })\n",
            "export const getOpenCodeKeybindings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_keybindings')\n",
            "export const updateOpenCodeKeybindings = (keybindings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_keybindings', { keybindings })\n",
            "export const listOpenCodeThemes = (): Promise<OpenCodeThemeRecord[]> => invoke('opencode_list_themes')\n",
            "export const listOpenCodeAgents = (): Promise<OpenJsonValueDto> => invoke('opencode_list_agents')\n",
            "export const addOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_agent', { config })\n",
            "export const updateOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_agent', { config })\n",
            "export const deleteOpenCodeAgent = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_agent', { name, context })\n",
            "export const listOpenCodeCommands = (): Promise<OpenJsonValueDto> => invoke('opencode_list_commands')\n",
            "export const addOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_command', { config })\n",
            "export const updateOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_command', { config })\n",
            "export const deleteOpenCodeCommand = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_command', { name, context })\n",
            "export const listOpenCodeLocalPlugins = (): Promise<OpenCodePluginFileRecord[]> => invoke('opencode_list_local_plugins')\n",
        ]
        .concat()
    }

    fn claude_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
            "export const getClaudeSettings = (): Promise<OpenJsonValueDto> => invoke('claude_get_settings')\n",
            "export const updateClaudeSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_settings', { settings })\n",
            "export const listClaudeMcpServers = (): Promise<OpenJsonValueDto> => invoke('claude_list_mcp_servers')\n",
            "export const addClaudeMcpServer = (name: string, config: OpenJsonValueDto, scope?: string): Promise<OpenJsonValueDto> => invoke('claude_add_mcp_server', { name, config, scope })\n",
            "export const updateClaudeMcpServer = (name: string, config: OpenJsonValueDto, scope?: string): Promise<OpenJsonValueDto> => invoke('claude_update_mcp_server', { name, config, scope })\n",
            "export const deleteClaudeMcpServer = (name: string, scope?: string): Promise<string> => invoke('claude_delete_mcp_server', { name, scope })\n",
            "export const listClaudeAgents = (): Promise<OpenJsonValueDto> => invoke('claude_list_agents')\n",
            "export const addClaudeAgent = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_agent', { name, config })\n",
            "export const updateClaudeAgent = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_agent', { name, config })\n",
            "export const deleteClaudeAgent = (name: string): Promise<string> => invoke('claude_delete_agent', { name })\n",
            "export const listClaudeSlashCommands = (): Promise<OpenJsonValueDto> => invoke('claude_list_slash_commands')\n",
            "export const addClaudeSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_slash_command', { name, config })\n",
            "export const updateClaudeSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_slash_command', { name, config })\n",
            "export const deleteClaudeSlashCommand = (name: string): Promise<string> => invoke('claude_delete_slash_command', { name })\n",
            "export const listClaudePlugins = (): Promise<OpenJsonValueDto> => invoke('claude_list_plugins')\n",
            "export const addClaudePlugin = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_plugin', { name, config })\n",
            "export const updateClaudePlugin = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_plugin', { name, config })\n",
            "export const deleteClaudePlugin = (name: string): Promise<string> => invoke('claude_delete_plugin', { name })\n",
            "export const getClaudeOutputStyles = (): Promise<OpenJsonValueDto> => invoke('claude_get_output_styles')\n",
            "export const updateClaudeOutputStyles = (styles: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_output_styles', { styles })\n",
            "export const getClaudeStatusline = (): Promise<OpenJsonValueDto> => invoke('claude_get_statusline')\n",
            "export const updateClaudeStatusline = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_statusline', { config })\n",
            "export const listClaudeHooks = (): Promise<OpenJsonValueDto> => invoke('claude_list_hooks')\n",
            "export const updateClaudeHooks = (hooks: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_hooks', { hooks })\n",
            "export const getClaudeBudgets = (): Promise<OpenJsonValueDto> => invoke('claude_get_budgets')\n",
            "export const updateClaudeBudgets = (budgets: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_budgets', { budgets })\n",
            "export const listClaudePrompts = (): Promise<OpenJsonValueDto> => invoke('claude_list_prompts')\n",
            "export const updateClaudePrompts = (prompts: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_prompts', { prompts })\n",
            "export const listClaudeProfiles = (): Promise<OpenJsonValueDto> => invoke('claude_list_profiles')\n",
            "export const getClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_get_profile', { name })\n",
            "export const addClaudeProfile = (request: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_add_profile', { request })\n",
            "export const updateClaudeProfile = (name: string, request: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('claude_update_profile', { name, request })\n",
            "export const deleteClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_delete_profile', { name })\n",
            "export const applyClaudeProfile = (name: string): Promise<OpenJsonValueDto> => invoke('claude_apply_profile', { name })\n",
            "export const exportClaudeProfiles = (includeSecrets: boolean): Promise<OpenJsonValueDto> => invoke('claude_export_profiles', { includeSecrets })\n",
            "export const getClaudeProfilesRaw = (): Promise<OpenJsonValueDto> => invoke('claude_get_profiles_raw')\n",
            "export const saveClaudeProfilesRaw = (content: string, token: string, force: boolean): Promise<OpenJsonValueDto> => invoke('claude_save_profiles_raw', { content, token, force })\n",
        ]
        .concat()
    }

    fn codex_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
            "export interface CodexAgentContextRequest { mode?: string; projectRoot?: string }\n",
            "export interface CodexAgentSourceInstallRequest { sourceId: string; agentId: string; targetName?: string | null; conflictMode?: string | null }\n",
            "export interface CodexAgentSourceSyncRequest { installId: string; force?: boolean }\n\n",
            "export const listCodexProfiles = (): Promise<OpenJsonValueDto> => invoke('codex_list_profiles')\n",
            "export const listCodexModels = (): Promise<OpenJsonValueDto> => invoke('codex_list_models')\n",
            "export const addCodexProfile = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_add_profile', { name, config })\n",
            "export const updateCodexProfile = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_profile', { name, config })\n",
            "export const deleteCodexProfile = (name: string): Promise<OpenJsonValueDto> => invoke('codex_delete_profile', { name })\n",
            "export const getCodexProfileEnv = (name: string): Promise<OpenJsonValueDto> => invoke('codex_get_profile_env', { name })\n",
            "export const applyCodexProfile = (name: string): Promise<OpenJsonValueDto> => invoke('codex_apply_profile', { name })\n",
            "export const exportCodexProfiles = (includeSecrets: boolean): Promise<OpenJsonValueDto> => invoke('codex_export_profiles', { includeSecrets })\n",
            "export const getCodexProfilesRaw = (): Promise<OpenJsonValueDto> => invoke('codex_get_profiles_raw')\n",
            "export const saveCodexProfilesRaw = (content: string, token: string, force: boolean): Promise<OpenJsonValueDto> => invoke('codex_save_profiles_raw', { content, token, force })\n",
            "export const getCodexSettings = (): Promise<OpenJsonValueDto> => invoke('codex_get_settings')\n",
            "export const updateCodexSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_settings', { settings })\n",
            "export const listCodexMcpServers = (): Promise<OpenJsonValueDto> => invoke('codex_list_mcp_servers')\n",
            "export const addCodexMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_add_mcp_server', { name, config })\n",
            "export const updateCodexMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('codex_update_mcp_server', { name, config })\n",
            "export const deleteCodexMcpServer = (name: string): Promise<string> => invoke('codex_delete_mcp_server', { name })\n",
            "export const listCodexAgents = (context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_list_agents', { context })\n",
            "export const addCodexAgent = (name: string, config: OpenJsonValueDto, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_add_agent', { name, config, context })\n",
            "export const updateCodexAgent = (name: string, config: OpenJsonValueDto, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_update_agent', { name, config, context })\n",
            "export const deleteCodexAgent = (name: string, context?: CodexAgentContextRequest): Promise<string> => invoke('codex_delete_agent', { name, context })\n",
            "export const renameCodexAgent = (name: string, newName: string, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_rename_agent', { name, newName, context })\n",
            "export const copyCodexAgent = (name: string, sourceContext?: CodexAgentContextRequest, targetContext?: CodexAgentContextRequest, targetName?: string): Promise<OpenJsonValueDto> => invoke('codex_copy_agent', { name, sourceContext, targetContext, targetName })\n",
            "export const validateCodexAgentToml = (name: string, context?: CodexAgentContextRequest): Promise<OpenJsonValueDto> => invoke('codex_validate_agent_toml', { name, context })\n",
            "export const listCodexAgentSources = (): Promise<OpenJsonValueDto> => invoke('codex_list_agent_sources')\n",
            "export const addCodexAgentSource = (url: string): Promise<OpenJsonValueDto> => invoke('codex_add_agent_source', { request: { url } })\n",
            "export const removeCodexAgentSource = (sourceId: string): Promise<void> => invoke('codex_remove_agent_source', { sourceId })\n",
            "export const syncCodexAgentSource = (sourceId: string): Promise<OpenJsonValueDto> => invoke('codex_sync_agent_source', { sourceId })\n",
            "export const getCodexAgentSourceCatalog = (sourceId: string): Promise<OpenJsonValueDto> => invoke('codex_get_agent_source_catalog', { sourceId })\n",
            "export const installCodexSourceAgent = (request: CodexAgentSourceInstallRequest): Promise<OpenJsonValueDto> => invoke('codex_install_source_agent', { request })\n",
            "export const syncCodexSourceInstall = (request: CodexAgentSourceSyncRequest): Promise<OpenJsonValueDto> => invoke('codex_sync_source_install', { request })\n",
            "export const acceptLocalCodexSourceInstall = (installId: string): Promise<OpenJsonValueDto> => invoke('codex_accept_local_source_install', { request: { installId } })\n",
            "export const untrackCodexSourceInstall = (installId: string): Promise<OpenJsonValueDto> => invoke('codex_untrack_source_install', { request: { installId } })\n",
            "export const listCodexSessions = (limit?: number, query?: string): Promise<OpenJsonValueDto> => invoke('codex_list_sessions', { limit, query })\n",
            "export const getCodexSessionDetail = (filePath: string, messageLimit?: number): Promise<OpenJsonValueDto> => invoke('codex_get_session_detail', { filePath, messageLimit })\n",
            "export const exportCodexSession = (filePath: string, maxMessages?: number): Promise<OpenJsonValueDto> => invoke('codex_export_session', { filePath, maxMessages })\n",
            "export const cloneCodexSession = (filePath: string): Promise<OpenJsonValueDto> => invoke('codex_clone_session', { filePath })\n",
            "export const deleteCodexSession = (filePath: string): Promise<OpenJsonValueDto> => invoke('codex_delete_session', { filePath })\n",
            "export const getCodexUsage = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_usage', { force })\n",
            "export const getCodexDashboardOverview = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_dashboard_overview', { force })\n",
            "export const getCodexDashboardUsageSummary = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_dashboard_usage_summary', { force })\n",
            "export const getCodexTraySnapshot = (force?: boolean): Promise<OpenJsonValueDto> => invoke('codex_get_tray_snapshot', { force })\n",
            "export const getCodexAllQuotas = (): Promise<OpenJsonValueDto> => invoke('codex_get_all_quotas')\n",
            "export const getCodexQuota = (account: string): Promise<OpenJsonValueDto> => invoke('codex_get_quota', { account })\n",
        ]
        .concat()
    }

    fn system_prompts_client_typescript() -> String {
        [
            "/* Generated from commands/handler_registry.rs; do not edit. */\n\n",
            "import { invoke } from '@tauri-apps/api/core'\n",
            "import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'\n\n",
            "export const listSystemPrompts = (platform: string): Promise<OpenJsonValueDto> => invoke('system_prompts_list', { platform })\n",
            "export const getSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_get', { platform, id })\n",
            "export const saveSystemPrompt = (platform: string, id: string, content: string, token: string): Promise<OpenJsonValueDto> => invoke('system_prompts_save', { platform, id, content, token })\n",
            "export const createSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_create', { platform, id })\n",
        ]
        .concat()
    }

    fn generated_artifacts() -> Vec<(PathBuf, String)> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        vec![
            (
                root.join("docs/reference/tauri-command-inventory.md"),
                command_inventory_markdown(),
            ),
            (
                root.join("docs/en/reference/tauri-command-inventory.md"),
                command_inventory_markdown(),
            ),
            (
                root.join("ccr-ui/src/api/generated/command-manifest.json"),
                command_manifest_json(),
            ),
            (
                root.join("ccr-ui/src/api/generated/commandCapabilities.ts"),
                command_capabilities_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/commandExec.ts"),
                command_exec_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/sync.ts"),
                sync_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/ssh.ts"),
                ssh_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/claudeAuth.ts"),
                claude_auth_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/codexAuth.ts"),
                codex_auth_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/config.ts"),
                config_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/uiState.ts"),
                ui_state_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemInfo.ts"),
                system_info_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/converter.ts"),
                converter_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/exitConfirm.ts"),
                exit_confirm_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/environment.ts"),
                environment_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/events.ts"),
                events_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/shell.ts"),
                shell_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemExtended.ts"),
                system_extended_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/builtinPrompts.ts"),
                builtin_prompts_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/gemini.ts"),
                gemini_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/openCode.ts"),
                opencode_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/claude.ts"),
                claude_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/codex.ts"),
                codex_client_typescript(),
            ),
            (
                root.join("ccr-ui/src/api/generated/systemPrompts.ts"),
                system_prompts_client_typescript(),
            ),
        ]
    }

    #[test]
    fn command_registry_shape_matches_current_handler_surface() {
        assert_eq!(COMMAND_MODULES.len(), 36);

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
    fn command_capability_descriptors_are_complete_and_unique() {
        let descriptors = command_descriptors().collect::<Vec<_>>();
        assert_eq!(descriptors.len(), 323);

        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for descriptor in descriptors {
            assert!(
                ids.insert(descriptor.id),
                "duplicate command id: {}",
                descriptor.id
            );
            assert!(
                paths.insert(descriptor.handler_path),
                "duplicate handler path: {}",
                descriptor.handler_path
            );
            assert!(!descriptor.id.is_empty());
            assert!(!descriptor.module.is_empty());
            assert!(!descriptor.title.is_empty());
            assert!(descriptor.timeout_ms > 0);
            assert_eq!(descriptor.input_schema, descriptor.output_schema);
        }

        let manifest = command_manifest();
        assert_eq!(manifest.base_command_count, 315);
        assert_eq!(manifest.windows_command_count, 323);
        assert_eq!(manifest.typed_command_count, 252);
    }

    #[test]
    fn command_capability_policy_covers_security_boundaries() {
        let delete_config = command_descriptor("delete_config").expect("delete config descriptor");
        assert_eq!(delete_config.risk, CommandRisk::Destructive);
        assert_eq!(delete_config.confirmation, CommandConfirmation::UserGesture);
        assert_eq!(
            delete_config.authorization,
            CommandAuthorization::SecretAccess
        );
        assert_eq!(delete_config.audit, CommandAudit::Redacted);

        let sync_push = command_descriptor("sync_push").expect("sync push descriptor");
        assert_eq!(sync_push.risk, CommandRisk::NetworkMutation);
        assert_eq!(sync_push.concurrency, CommandConcurrency::ModuleExclusive);

        let sync_list =
            command_descriptor("list_sync_assets").expect("list sync assets descriptor");
        assert_eq!(sync_list.risk, CommandRisk::ReadOnly);
        assert_eq!(sync_list.authorization, CommandAuthorization::SecretAccess);
        assert_eq!(sync_list.audit, CommandAudit::Redacted);

        let command_list =
            command_descriptor("list_ccr_commands").expect("list commands descriptor");
        assert_eq!(command_list.risk, CommandRisk::ReadOnly);
        assert_eq!(
            command_list.authorization,
            CommandAuthorization::SystemCapability
        );
        assert_eq!(command_list.audit, CommandAudit::Redacted);

        let install = command_descriptor("llmusage_install_execute")
            .expect("llmusage install execute descriptor");
        assert_eq!(install.risk, CommandRisk::ProcessExecution);
        assert_eq!(install.confirmation, CommandConfirmation::OpaqueCapability);
        assert_eq!(install.input_schema, CommandSchema::Generated);
        assert_eq!(install.output_schema, CommandSchema::Generated);

        let codex_delete =
            command_descriptor("codex_delete_session").expect("Codex delete descriptor");
        assert_eq!(codex_delete.risk, CommandRisk::Destructive);
        assert_eq!(codex_delete.audit, CommandAudit::Redacted);

        let claude_get = command_descriptor("claude_get_settings").expect("Claude read descriptor");
        assert_eq!(claude_get.risk, CommandRisk::ReadOnly);
        assert_eq!(claude_get.authorization, CommandAuthorization::SecretAccess);

        let prompts_list =
            command_descriptor("system_prompts_list").expect("system prompts list descriptor");
        assert_eq!(prompts_list.risk, CommandRisk::ReadOnly);
    }

    #[test]
    fn generated_command_exec_client_covers_the_typed_registry_module() {
        let client = command_exec_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "command_exec")
            .expect("command_exec module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_sync_client_covers_the_typed_registry_module() {
        let client = sync_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "sync")
            .expect("sync module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_ssh_client_covers_the_typed_registry_module() {
        let client = ssh_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "ssh")
            .expect("ssh module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_claude_auth_client_covers_the_typed_registry_module() {
        let client = claude_auth_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "claude_auth")
            .expect("claude_auth module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
        assert!(!include_str!("claude_auth.rs").contains("Result<Value"));
    }

    #[test]
    fn generated_codex_auth_client_covers_typed_auth_and_provider_modules() {
        let client = codex_auth_client_typescript();
        let modules = COMMAND_MODULES
            .iter()
            .filter(|module| matches!(module.key, "codex_auth" | "codex_model_providers"))
            .collect::<Vec<_>>();
        let command_count = modules
            .iter()
            .map(|module| module.commands.len())
            .sum::<usize>();

        assert_eq!(modules.len(), 2);
        assert!(
            modules
                .iter()
                .all(|module| module.schema == CommandSchema::Generated)
        );
        assert_eq!(client.matches("invoke('").count(), command_count);
        for handler_path in modules.iter().flat_map(|module| module.commands) {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_config_client_covers_the_typed_registry_module() {
        let client = config_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "config")
            .expect("config module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_claude_client_covers_core_and_profile_modules() {
        let client = claude_client_typescript();
        let modules = COMMAND_MODULES
            .iter()
            .filter(|module| matches!(module.key, "claude" | "claude_profiles"))
            .collect::<Vec<_>>();

        assert_eq!(modules.len(), 2);
        assert!(
            modules
                .iter()
                .all(|module| module.schema == CommandSchema::Generated)
        );
        assert_eq!(
            client.matches("invoke('").count(),
            modules
                .iter()
                .map(|module| module.commands.len())
                .sum::<usize>()
        );
        for handler_path in modules.iter().flat_map(|module| module.commands) {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_codex_client_covers_the_typed_registry_module() {
        let client = codex_client_typescript();
        let module = COMMAND_MODULES
            .iter()
            .find(|module| module.key == "codex")
            .expect("codex module");

        assert_eq!(module.schema, CommandSchema::Generated);
        assert_eq!(client.matches("invoke('").count(), module.commands.len());
        for handler_path in module.commands {
            let command = handler_path
                .rsplit("::")
                .next()
                .expect("command handler name");
            assert!(
                client.contains(&format!("invoke('{command}'")),
                "generated client missing typed command: {command}"
            );
        }
        assert!(!client.contains("unknown"));
        assert!(!client.contains("any"));
        assert!(!client.contains("<T"));
    }

    #[test]
    fn generated_small_domain_clients_cover_their_typed_registry_modules() {
        for (key, client) in [
            ("ui_state", ui_state_client_typescript()),
            ("system_info", system_info_client_typescript()),
            ("converter", converter_client_typescript()),
            ("exit_confirm", exit_confirm_client_typescript()),
            ("environment", environment_client_typescript()),
            ("events", events_client_typescript()),
            ("shell", shell_client_typescript()),
            ("system_extended", system_extended_client_typescript()),
            ("builtin_prompts", builtin_prompts_client_typescript()),
            ("gemini", gemini_client_typescript()),
            ("opencode", opencode_client_typescript()),
            ("system_prompts", system_prompts_client_typescript()),
        ] {
            let module = COMMAND_MODULES
                .iter()
                .find(|module| module.key == key)
                .expect("typed module");

            assert_eq!(module.schema, CommandSchema::Generated);
            assert_eq!(client.matches("invoke('").count(), module.commands.len());
            for handler_path in module.commands {
                let command = handler_path
                    .rsplit("::")
                    .next()
                    .expect("command handler name");
                assert!(
                    client.contains(&format!("invoke('{command}'")),
                    "generated client missing typed command: {command}"
                );
            }
            assert!(!client.contains("unknown"));
            assert!(!client.contains("<T"));
        }
    }

    #[test]
    fn command_inventory_document_matches_registry() {
        for (path, expected) in generated_artifacts() {
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
