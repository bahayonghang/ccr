//! Tauri 鍛戒护鑱氬悎妯″潡 鈥?鎸夊煙鍒嗙粍鐨?18 涓懡浠ゅ瓙妯″潡銆?
//!
//! 姣忎釜瀛愭ā鍧楀搴斾竴涓钩鍙版垨鍔熻兘鍩燂紝鍖呭惈 `#[tauri::command]` 鍑芥暟銆?

pub mod builtin_prompts;
pub mod checkin;
pub mod claude;
pub mod codex;
pub mod command_exec;
pub mod config;
pub mod converter;
pub mod droid;
pub mod environment;
pub mod gemini;
pub mod iflow;
pub mod mcp_presets;
pub mod opencode;
pub mod pricing;
pub mod qwen;
pub mod skill_hub;
pub mod skills;
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

/// 鐢熸垚鍖呭惈鎵€鏈夊懡浠ょ殑 `invoke_handler`銆?
///
/// 鐢ㄦ硶锛歚tauri::Builder::default().invoke_handler(commands::generate_handler())`
macro_rules! generate_handler_common {
    ($($extra:path),* $(,)?) => {
        tauri::generate_handler![
        // 鈹€鈹€ 閰嶇疆绠＄悊 鈹€鈹€
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
        // 鈹€鈹€ 鍚屾 鈹€鈹€
        sync::sync_push,
        sync::sync_pull,
        sync::sync_status,
        sync::list_sync_folders,
        sync::add_sync_folder,
        sync::update_sync_folder,
        sync::delete_sync_folder,
        // 鈹€鈹€ Claude Code 鈹€鈹€
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
        // 鈹€鈹€ Codex 鈹€鈹€
        codex::codex_list_profiles,
        codex::codex_add_profile,
        codex::codex_update_profile,
        codex::codex_delete_profile,
        codex::codex_apply_profile,
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
        codex::codex_get_usage,
        codex::codex_list_auth_accounts,
        codex::codex_get_auth_current,
        codex::codex_save_auth,
        codex::codex_switch_auth,
        codex::codex_delete_auth,
        codex::codex_detect_process,
        // 鈹€鈹€ Gemini 鈹€鈹€
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
        // 鈹€鈹€ Qwen 鈹€鈹€
        qwen::qwen_get_settings,
        qwen::qwen_update_settings,
        qwen::qwen_list_mcp_servers,
        qwen::qwen_add_mcp_server,
        qwen::qwen_update_mcp_server,
        qwen::qwen_delete_mcp_server,
        qwen::qwen_list_slash_commands,
        qwen::qwen_add_slash_command,
        qwen::qwen_update_slash_command,
        qwen::qwen_delete_slash_command,
        // 鈹€鈹€ iFlow 鈹€鈹€
        iflow::iflow_get_settings,
        iflow::iflow_update_settings,
        iflow::iflow_list_mcp_servers,
        iflow::iflow_add_mcp_server,
        iflow::iflow_update_mcp_server,
        iflow::iflow_delete_mcp_server,
        iflow::iflow_list_slash_commands,
        // 鈹€鈹€ Droid 鈹€鈹€
        droid::droid_get_settings,
        droid::droid_update_settings,
        droid::droid_list_mcp_servers,
        droid::droid_add_mcp_server,
        droid::droid_update_mcp_server,
        droid::droid_delete_mcp_server,
        droid::droid_list_agents,
        droid::droid_add_agent,
        droid::droid_update_agent,
        droid::droid_delete_agent,
        droid::droid_list_plugins,
        droid::droid_add_plugin,
        droid::droid_update_plugin,
        droid::droid_delete_plugin,
        droid::droid_list_slash_commands,
        droid::droid_add_slash_command,
        droid::droid_update_slash_command,
        droid::droid_delete_slash_command,
        droid::droid_list_models,
        // 鈹€鈹€ OpenCode 鈹€鈹€
        opencode::opencode_get_settings,
        opencode::opencode_update_settings,
        opencode::opencode_get_keybindings,
        opencode::opencode_update_keybindings,
        opencode::opencode_list_themes,
        // 鈹€鈹€ CheckIn 鈹€鈹€
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
        // 鈹€鈹€ 缁熻 鈹€鈹€
        stats::get_cost_overview,
        stats::get_heatmap_data,
        stats::get_session_stats,
        // 鈹€鈹€ 绯荤粺 鈹€鈹€
        system::get_system_info,
        system::check_version,
        system::health_check,
        // 鈹€鈹€ 杞崲鍣?鈹€鈹€
        converter::convert_config,
        // 鈹€鈹€ UI 鐘舵€?鈹€鈹€
        ui_state::get_favorites,
        ui_state::add_favorite,
        ui_state::remove_favorite,
        ui_state::get_recent_items,
        ui_state::add_recent_item,
        ui_state::clear_recent_items,
        // 鈹€鈹€ WAF 鈹€鈹€
        waf::open_waf_login,
        waf::get_waf_cookie_status,
        waf::waf_deliver_cookie,
        // 鈹€鈹€ 缁熶竴 MCP 鈹€鈹€
        unified_mcp::unified_list_mcp_servers,
        unified_mcp::unified_add_mcp_server,
        unified_mcp::unified_delete_mcp_server,
        // 鈹€鈹€ 浜嬩欢鏌ヨ 鈹€鈹€
        system::get_recent_events,
        system::get_monitoring_feed,
        system::append_frontend_logs,
        system::get_runtime_metrics,
        // 鈹€鈹€ 鐜绠＄悊 鈹€鈹€
        environment::list_environments,
        environment::get_current_environment,
        environment::switch_environment,
        environment::refresh_environments,
        environment::env_list_platforms,
        environment::env_detect_cli,
        // 鈹€鈹€ SSH 鈹€鈹€
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
        // 鈹€鈹€ 鍐呯疆鎻愮ず璇?鈹€鈹€
        builtin_prompts::list_builtin_prompts,
        builtin_prompts::get_builtin_prompt,
        builtin_prompts::get_builtin_prompts_by_category,
        // 鈹€鈹€ 瀹氫环绠＄悊 鈹€鈹€
        pricing::set_pricing,
        pricing::get_pricing_list,
        pricing::remove_pricing,
        pricing::reset_pricing,
        // 鈹€鈹€ MCP 棰勮 鈹€鈹€
        mcp_presets::list_mcp_presets,
        mcp_presets::get_mcp_preset,
        mcp_presets::install_mcp_preset,
        mcp_presets::install_mcp_preset_single,
        mcp_presets::list_source_mcp_servers,
        mcp_presets::sync_mcp_server,
        mcp_presets::sync_all_mcp_servers,
        // 鈹€鈹€ 鎶€鑳界鐞?鈹€鈹€
        skills::list_skills,
        skills::add_skill,
        skills::delete_skill,
        skills::list_skill_repositories,
        skills::add_skill_repository,
        skills::remove_skill_repository,
        skills::scan_skill_repository,
        // 鈹€鈹€ Usage V2 鈹€鈹€
        usage::get_usage_summary_v2,
        usage::get_usage_trends_v2,
        usage::get_usage_by_model_v2,
        usage::get_usage_by_project_v2,
        usage::get_usage_logs_v2,
        usage::get_usage_dashboard_v2,
        usage::import_usage_v2,
        usage::import_all_usage_v2,
        // 鈹€鈹€ 鍛戒护鎵ц 鈹€鈹€
        command_exec::execute_ccr_command,
        command_exec::list_ccr_commands,
        command_exec::get_ccr_command_help,
        // 鈹€鈹€ 鎶€鑳界鐞嗭紙鎵╁睍锛?鈹€鈹€
        skills::get_skill,
        skills::update_skill,
        // 鈹€鈹€ SkillHub 甯傚満 鈹€鈹€
        skill_hub::skill_hub_trending,
        skill_hub::skill_hub_search,
        skill_hub::skill_hub_agents,
        skill_hub::skill_hub_agent_skills,
        skill_hub::skill_hub_install,
        skill_hub::skill_hub_remove,
        skill_hub::skill_hub_unified,
        skill_hub::skill_hub_skill_content,
        skill_hub::skill_hub_save_skill_content,
        skill_hub::skill_hub_import_github,
        skill_hub::skill_hub_import_local,
        skill_hub::skill_hub_import_npx,
        skill_hub::skill_hub_batch_install,
        skill_hub::skill_hub_check_npx,
        skill_hub::skill_hub_browse_folder,
        // 鈹€鈹€ 缁熻鎵╁睍 鈹€鈹€
        stats::get_cost_trend,
        stats::get_cost_by_model,
        stats::get_cost_by_project,
        stats::get_provider_usage,
        stats::get_top_sessions,
        stats::get_stats_summary,
        stats::get_daily_stats,
        // 鈹€鈹€ 绛惧埌鎵╁睍 鈹€鈹€
        checkin::list_builtin_providers,
        checkin::add_builtin_provider,
        checkin::get_checkin_account_cookies,
        checkin::export_checkin_config,
        checkin::preview_checkin_import,
        checkin::import_checkin_config,
        checkin::get_account_dashboard,
        // 鈹€鈹€ 閰嶇疆鎵╁睍 鈹€鈹€
        config::update_config,
        config::clean_backups,
        // 鈹€鈹€ 閫€鍑虹‘璁?鈹€鈹€
        config::get_skip_exit_confirm,
        config::set_skip_exit_confirm,
        // 鈹€鈹€ 绯荤粺鎵╁睍 鈹€鈹€
        system::update_ccr,
        system::get_cli_versions,
        $($extra,)*
        ]
    };
}

#[cfg(not(target_os = "windows"))]
pub fn generate_handler() -> impl Fn(tauri::ipc::Invoke) -> bool {
    generate_handler_common!()
}

/// Windows 鐗堟湰锛氬寘鍚?WSL 鍛戒护鐨?`invoke_handler`銆?
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

