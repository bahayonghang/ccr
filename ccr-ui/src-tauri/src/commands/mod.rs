//! Tauri 命令聚合模块 —— 按功能分组组织各子模块。
//!
//! 每个子模块对应一个平台或功能域，并提供 `#[tauri::command]` 命令实现。

pub mod builtin_prompts;
pub mod checkin;
pub mod claude;
pub mod claude_mcp_config;
pub mod codex;
pub mod command_exec;
pub mod config;
pub mod converter;
pub mod environment;
pub mod gemini;
pub mod install;
pub mod mcp_presets;
pub mod opencode;
pub mod pricing;
pub mod profile_lifecycle;
pub mod shell;
pub mod ssh;
pub mod stats;
pub mod sync;
pub mod system;
pub mod ui_state;
pub mod unified_mcp;
pub mod usage;
pub mod waf;
#[cfg(target_os = "windows")]
pub mod wsl;

/// 生成包含所有命令的 `invoke_handler`。
///
/// 用法：`tauri::Builder::default().invoke_handler(commands::generate_handler())`
macro_rules! generate_handler_common {
    ($($extra:path),* $(,)?) => {
        tauri::generate_handler![
        // —— 配置管理 ——
        config::list_configs,
        config::switch_config,
        config::add_config,
        config::delete_config,
        config::rename_config,
        config::duplicate_config,
        config::validate_configs,
        config::import_config,
        config::export_config,
        config::get_history,
        config::clear_history,
        // —— 同步 ——
        sync::sync_push,
        sync::sync_pull,
        sync::sync_status,
        sync::list_sync_folders,
        sync::add_sync_folder,
        sync::update_sync_folder,
        sync::delete_sync_folder,
        sync::set_webdav_config,
        sync::test_webdav_config,
        sync::clear_webdav_config,
        // —— Claude Code ——
        claude::claude_get_settings,
        claude::claude_update_settings,
        claude::claude_list_mcp_servers,
        claude::claude_add_mcp_server,
        claude::claude_update_mcp_server,
        claude::claude_delete_mcp_server,
        claude::claude_list_agents,
        claude::claude_add_agent,
        claude::claude_update_agent,
        claude::claude_delete_agent,
        claude::claude_list_slash_commands,
        claude::claude_add_slash_command,
        claude::claude_update_slash_command,
        claude::claude_delete_slash_command,
        claude::claude_list_plugins,
        claude::claude_add_plugin,
        claude::claude_update_plugin,
        claude::claude_delete_plugin,
        claude::claude_get_output_styles,
        claude::claude_update_output_styles,
        claude::claude_get_statusline,
        claude::claude_update_statusline,
        claude::claude_list_hooks,
        claude::claude_update_hooks,
        claude::claude_get_budgets,
        claude::claude_update_budgets,
        claude::claude_list_prompts,
        claude::claude_update_prompts,
        // —— Claude Code Profiles ——
        claude::claude_list_profiles,
        claude::claude_get_profile,
        claude::claude_add_profile,
        claude::claude_update_profile,
        claude::claude_delete_profile,
        claude::claude_apply_profile,
        claude::claude_export_profiles,
        claude::claude_list_auth_accounts,
        claude::claude_get_auth_current,
        claude::claude_save_auth,
        claude::claude_switch_auth,
        claude::claude_delete_auth,
        // —— Codex ——
        codex::codex_list_profiles,
        codex::codex_list_models,
        codex::codex_add_custom_model,
        codex::codex_add_profile,
        codex::codex_update_profile,
        codex::codex_delete_profile,
        codex::codex_get_profile_env,
        codex::codex_apply_profile,
        codex::codex_export_profiles,
        codex::codex_get_settings,
        codex::codex_update_settings,
        codex::codex_list_mcp_servers,
        codex::codex_add_mcp_server,
        codex::codex_update_mcp_server,
        codex::codex_delete_mcp_server,
        codex::codex_list_agents,
        codex::codex_add_agent,
        codex::codex_update_agent,
        codex::codex_delete_agent,
        codex::codex_rename_agent,
        codex::codex_copy_agent,
        codex::codex_validate_agent_toml,
        codex::codex_list_agent_sources,
        codex::codex_add_agent_source,
        codex::codex_remove_agent_source,
        codex::codex_sync_agent_source,
        codex::codex_get_agent_source_catalog,
        codex::codex_install_source_agent,
        codex::codex_sync_source_install,
        codex::codex_accept_local_source_install,
        codex::codex_untrack_source_install,
        codex::codex_list_sessions,
        codex::codex_get_session_detail,
        codex::codex_export_session,
        codex::codex_clone_session,
        codex::codex_delete_session,
        codex::codex_get_usage,
        codex::codex_get_dashboard_overview,
        codex::codex_get_dashboard_usage_summary,
        codex::codex_list_auth_accounts,
        codex::codex_get_auth_current,
        codex::codex_save_auth,
        codex::codex_switch_auth,
        codex::codex_delete_auth,
        codex::codex_rename_auth,
        codex::codex_detect_process,
        codex::codex_oauth_login_start,
        codex::codex_oauth_login_completed,
        codex::codex_oauth_login_cancel,
        codex::codex_oauth_submit_callback_url,
        codex::codex_is_oauth_port_in_use,
        codex::codex_release_oauth_port,
        codex::codex_open_external_url,
        codex::codex_import_auth_payload,
        codex::codex_import_auth_from_local,
        codex::codex_add_auth_with_api_key,
        codex::codex_list_model_providers,
        codex::codex_save_model_provider,
        codex::codex_delete_model_provider,
        codex::codex_get_tray_snapshot,
        codex::codex_get_all_quotas,
        codex::codex_get_quota,
        // —— Gemini ——
        gemini::gemini_get_settings,
        gemini::gemini_update_settings,
        gemini::gemini_list_mcp_servers,
        gemini::gemini_add_mcp_server,
        gemini::gemini_update_mcp_server,
        gemini::gemini_delete_mcp_server,
        gemini::gemini_list_slash_commands,
        gemini::gemini_add_slash_command,
        gemini::gemini_update_slash_command,
        gemini::gemini_delete_slash_command,
        gemini::gemini_list_extensions,
        // —— OpenCode ——
        opencode::opencode_get_settings,
        opencode::opencode_update_settings,
        opencode::opencode_get_tui_settings,
        opencode::opencode_update_tui_settings,
        opencode::opencode_get_keybindings,
        opencode::opencode_update_keybindings,
        opencode::opencode_list_themes,
        opencode::opencode_list_agents,
        opencode::opencode_add_agent,
        opencode::opencode_update_agent,
        opencode::opencode_delete_agent,
        opencode::opencode_list_commands,
        opencode::opencode_add_command,
        opencode::opencode_update_command,
        opencode::opencode_delete_command,
        opencode::opencode_list_local_plugins,
        // —— CheckIn ——
        checkin::list_providers,
        checkin::add_provider,
        checkin::update_provider,
        checkin::delete_provider,
        checkin::test_provider_connection,
        checkin::list_accounts,
        checkin::add_account,
        checkin::update_account,
        checkin::delete_account,
        checkin::batch_delete_accounts,
        checkin::execute_checkin,
        checkin::batch_checkin,
        checkin::start_checkin_job,
        checkin::get_checkin_job_status,
        checkin::get_checkin_records,
        checkin::get_balance,
        checkin::get_balance_history,
        checkin::get_balance_stats,
        checkin::export_checkin_data,
        checkin::export_checkin_stats,
        checkin::execute_cdk_recharge,
        checkin::get_cdk_history,
        checkin::list_waf_cookies,
        checkin::add_waf_cookie,
        checkin::delete_waf_cookie,
        // —— 统计 ——
        stats::get_cost_overview,
        stats::get_heatmap_data,
        stats::get_session_stats,
        // —— 系统 ——
        system::get_system_info,
        system::check_version,
        system::health_check,
        // —— 转换器 ——
        converter::convert_config,
        // —— UI 状态 ——
        ui_state::get_favorites,
        ui_state::add_favorite,
        ui_state::remove_favorite,
        ui_state::get_recent_items,
        ui_state::add_recent_item,
        ui_state::clear_recent_items,
        // —— WAF ——
        waf::open_waf_login,
        waf::get_waf_cookie_status,
        waf::waf_deliver_cookie,
        // —— 统一 MCP ——
        unified_mcp::unified_list_mcp_servers,
        unified_mcp::unified_add_mcp_server,
        unified_mcp::unified_update_mcp_server,
        unified_mcp::unified_delete_mcp_server,
        // —— 事件查询 ——
        system::get_recent_events,
        system::get_monitoring_feed,
        system::append_frontend_logs,
        system::get_runtime_metrics,
        // —— 环境管理 ——
        environment::list_environments,
        environment::get_current_environment,
        environment::switch_environment,
        environment::refresh_environments,
        environment::env_list_platforms,
        environment::env_detect_cli,
        // —— SSH ——
        ssh::ssh_list_hosts,
        ssh::ssh_add_host,
        ssh::ssh_connect,
        ssh::ssh_reconnect,
        ssh::ssh_disconnect,
        ssh::ssh_get_connection_state,
        ssh::ssh_probe_host_fingerprint,
        ssh::ssh_confirm_host_fingerprint,
        ssh::ssh_read_config,
        ssh::ssh_write_config,
        ssh::ssh_detect_cli,
        ssh::ssh_test_connection,
        ssh::ssh_list_keys,
        // —— 内置提示词 ——
        builtin_prompts::list_builtin_prompts,
        builtin_prompts::get_builtin_prompt,
        builtin_prompts::get_builtin_prompts_by_category,
        // —— 定价管理 ——
        pricing::set_pricing,
        pricing::get_pricing_list,
        pricing::remove_pricing,
        pricing::reset_pricing,
        // —— MCP 预设 ——
        mcp_presets::list_mcp_presets,
        mcp_presets::get_mcp_preset,
        mcp_presets::install_mcp_preset,
        mcp_presets::install_mcp_preset_single,
        mcp_presets::list_source_mcp_servers,
        mcp_presets::sync_mcp_server,
        mcp_presets::sync_all_mcp_servers,
        // —— Usage V2 ——
        usage::get_usage_summary_v2,
        usage::get_usage_capabilities_v2,
        usage::get_usage_trends_v2,
        usage::get_usage_by_model_v2,
        usage::get_usage_by_project_v2,
        usage::get_usage_heatmap_v2,
        usage::get_usage_logs_v2,
        usage::get_usage_dashboard_v2,
        usage::get_home_usage_overview_v2,
        usage::ensure_session_index_v2,
        usage::get_session_index_job_status_v2,
        usage::start_usage_import_job_v2,
        usage::get_usage_import_job_status_v2,
        usage::cancel_usage_import_job_v2,
        usage::import_usage_v2,
        usage::import_all_usage_v2,
        // —— 命令执行 ——
        command_exec::execute_ccr_command,
        command_exec::list_ccr_commands,
        command_exec::get_ccr_command_help,
        // —— 统计扩展 ——
        stats::get_cost_trend,
        stats::get_cost_by_model,
        stats::get_cost_by_project,
        stats::get_provider_usage,
        stats::get_top_sessions,
        stats::get_stats_summary,
        stats::get_daily_stats,
        // —— 签到扩展 ——
        checkin::list_builtin_providers,
        checkin::add_builtin_provider,
        checkin::get_checkin_account_cookies,
        checkin::export_checkin_config,
        checkin::preview_checkin_import,
        checkin::import_checkin_config,
        checkin::get_account_dashboard,
        // —— 配置扩展 ——
        config::update_config,
        config::clean_backups,
        // —— 退出确认 ——
        config::get_skip_exit_confirm,
        config::set_skip_exit_confirm,
        // —— Desktop Shell ——
        shell::shell_get_preferences,
        shell::shell_set_preferences,
        shell::shell_show_main_window,
        shell::shell_request_quit,
        shell::shell_begin_tray_panel_drag,
        shell::shell_complete_tray_panel_drag,
        shell::shell_detect_skillport_app,
        shell::shell_open_skillport_app,
        shell::shell_detect_skills_manage_app,
        shell::shell_open_skills_manage_app,
        // —— 系统扩展 ——
        system::update_ccr,
        system::get_cli_version,
        system::get_cli_versions,
        // —— llmusage 安装流程 ——
        install::llmusage_install_detect,
        install::llmusage_install_probe_capabilities,
        install::llmusage_install_plan,
        install::llmusage_install_execute,
        install::llmusage_install_cancel,
        install::llmusage_install_recent,
        install::llmusage_install_manual_catalog,
        install::llmusage_install_check,
        $($extra,)*
        ]
    };
}

#[cfg(not(target_os = "windows"))]
pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
    generate_handler_common!()
}

/// Windows 版本：包含 WSL 命令的 `invoke_handler`。
#[cfg(target_os = "windows")]
pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
    generate_handler_common!(
        wsl::wsl_list_distros,
        wsl::wsl_refresh_distros,
        wsl::wsl_clear_cache,
        wsl::wsl_cache_status,
        wsl::wsl_read_config,
        wsl::wsl_write_config,
        wsl::wsl_detect_cli,
        wsl::wsl_sync_config,
    )
}
