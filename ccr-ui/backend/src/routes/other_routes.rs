// Other routes collection
// 为了简化重构，将其他所有路由集中在一个文件中

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use crate::state::AppState;

/// Platform management routes
pub fn platform_routes() -> Router<AppState> {
    Router::new()
        .route("/platforms", get(crate::api::handlers::platform::list_platforms))
        .route("/platforms/current", get(crate::api::handlers::platform::get_current_platform))
        .route("/platforms/switch", post(crate::api::handlers::platform::switch_platform))
        .route("/platforms/{name}", get(crate::api::handlers::platform::get_platform))
        .route("/platforms/{name}", put(crate::api::handlers::platform::update_platform))
        .route("/platforms/{name}/enable", post(crate::api::handlers::platform::enable_platform))
        .route("/platforms/{name}/disable", post(crate::api::handlers::platform::disable_platform))
        .route("/platforms/{name}/profile", get(crate::api::handlers::platform::get_platform_profile))
        .route("/platforms/{name}/profile", post(crate::api::handlers::platform::set_platform_profile))
}

/// MCP server management routes
pub fn mcp_routes() -> Router<AppState> {
    Router::new()
        .merge(mcp_server_routes())
        .merge(mcp_presets_routes())
        .merge(mcp_sync_routes())
}

/// MCP server endpoints
fn mcp_server_routes() -> Router<AppState> {
    Router::new()
        .route("/mcp", get(crate::api::handlers::mcp::list_mcp_servers))
        .route("/mcp", post(crate::api::handlers::mcp::add_mcp_server))
        .route("/mcp/{name}", put(crate::api::handlers::mcp::update_mcp_server))
        .route("/mcp/{name}", delete(crate::api::handlers::mcp::delete_mcp_server))
        .route("/mcp/{name}/toggle", put(crate::api::handlers::mcp::toggle_mcp_server))
}

/// MCP presets routes
pub fn mcp_presets_routes() -> Router<AppState> {
    Router::new()
        .route("/mcp/presets", get(crate::api::handlers::mcp_presets::list_presets))
        .route("/mcp/presets/{id}", get(crate::api::handlers::mcp_presets::get_preset))
        .route("/mcp/presets/{id}/install", post(crate::api::handlers::mcp_presets::install_preset))
        .route("/mcp/presets/install", post(crate::api::handlers::mcp_presets::install_preset_single))
}

/// MCP sync routes
pub fn mcp_sync_routes() -> Router<AppState> {
    Router::new()
        .route("/mcp/sync/source", get(crate::api::handlers::mcp_presets::list_source_mcp_servers))
        .route("/mcp/sync/all", post(crate::api::handlers::mcp_presets::sync_all_mcp_servers))
        .route("/mcp/sync/{name}", post(crate::api::handlers::mcp_presets::sync_mcp_server))
}

/// Builtin prompts routes
pub fn builtin_prompts_routes() -> Router<AppState> {
    Router::new()
        .route("/prompts/builtin", get(crate::api::handlers::builtin_prompts::list_builtin_prompts))
        .route("/prompts/builtin/{id}", get(crate::api::handlers::builtin_prompts::get_builtin_prompt))
        .route(
            "/prompts/builtin/category/{category}",
            get(crate::api::handlers::builtin_prompts::get_builtin_prompts_by_category),
        )
}

/// Slash command management routes
pub fn slash_commands_routes() -> Router<AppState> {
    Router::new()
        .route("/slash-commands", get(crate::api::handlers::slash_commands::list_slash_commands))
        .route("/slash-commands", post(crate::api::handlers::slash_commands::add_slash_command))
        .route(
            "/slash-commands/{name}",
            put(crate::api::handlers::slash_commands::update_slash_command),
        )
        .route(
            "/slash-commands/{name}",
            delete(crate::api::handlers::slash_commands::delete_slash_command),
        )
        .route(
            "/slash-commands/{name}/toggle",
            put(crate::api::handlers::slash_commands::toggle_slash_command),
        )
}

/// Agent management routes
pub fn agents_routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(crate::api::handlers::agents::list_agents))
        .route("/agents", post(crate::api::handlers::agents::add_agent))
        .route("/agents/{name}", put(crate::api::handlers::agents::update_agent))
        .route("/agents/{name}", delete(crate::api::handlers::agents::delete_agent))
        .route("/agents/{name}/toggle", put(crate::api::handlers::agents::toggle_agent))
}

/// Plugin management routes
pub fn plugins_routes() -> Router<AppState> {
    Router::new()
        .route("/plugins", get(crate::api::handlers::plugins::list_plugins))
        .route("/plugins", post(crate::api::handlers::plugins::add_plugin))
        .route("/plugins/{id}", put(crate::api::handlers::plugins::update_plugin))
        .route("/plugins/{id}", delete(crate::api::handlers::plugins::delete_plugin))
        .route("/plugins/{id}/toggle", put(crate::api::handlers::plugins::toggle_plugin))
}

/// Statistics routes
pub fn stats_routes() -> Router<AppState> {
    Router::new()
        .route("/stats/cost", get(crate::api::handlers::stats::cost_overview))
        .route("/stats/cost/today", get(crate::api::handlers::stats::cost_today))
        .route("/stats/cost/week", get(crate::api::handlers::stats::cost_week))
        .route("/stats/cost/month", get(crate::api::handlers::stats::cost_month))
        .route("/stats/cost/trend", get(crate::api::handlers::stats::cost_trend))
        .route("/stats/cost/by-model", get(crate::api::handlers::stats::cost_by_model))
        .route("/stats/cost/by-project", get(crate::api::handlers::stats::cost_by_project))
        .route("/stats/provider-usage", get(crate::api::handlers::stats::provider_usage))
        .route("/stats/cost/top-sessions", get(crate::api::handlers::stats::cost_top_sessions))
        .route("/stats/summary", get(crate::api::handlers::stats::stats_summary))
}

/// Skills management routes
pub fn skills_routes() -> Router<AppState> {
    Router::new()
        .route("/skills", get(crate::api::handlers::skills::list_skills))
        .route("/skills", post(crate::api::handlers::skills::add_skill))
        .route("/skills/{name}", put(crate::api::handlers::skills::update_skill))
        .route("/skills/{name}", delete(crate::api::handlers::skills::delete_skill))
        .route("/skills/repositories", get(crate::api::handlers::skills::list_repositories))
        .route("/skills/repositories", post(crate::api::handlers::skills::add_repository))
        .route("/skills/repositories/{name}", delete(crate::api::handlers::skills::remove_repository))
        .route("/skills/repositories/{name}/scan", get(crate::api::handlers::skills::scan_repository))
}

/// Prompts management routes
pub fn prompts_routes() -> Router<AppState> {
    Router::new()
        .route("/prompts", get(crate::api::handlers::prompts::list_prompts))
        .route("/prompts", post(crate::api::handlers::prompts::add_prompt))
        .route("/prompts/{name}", get(crate::api::handlers::prompts::get_prompt))
        .route("/prompts/{name}", delete(crate::api::handlers::prompts::delete_prompt))
        .route("/prompts/{name}/apply", post(crate::api::handlers::prompts::apply_prompt))
        .route("/prompts/current/{target}", get(crate::api::handlers::prompts::get_current_prompt))
}

/// Budget management routes
pub fn budget_routes() -> Router<AppState> {
    Router::new()
        .route("/budget/status", get(crate::api::handlers::budget::get_budget_status))
        .route("/budget/set", post(crate::api::handlers::budget::set_budget))
        .route("/budget/reset", post(crate::api::handlers::budget::reset_budget))
}

/// Pricing management routes
pub fn pricing_routes() -> Router<AppState> {
    Router::new()
        .route("/pricing/list", get(crate::api::handlers::pricing::get_pricing_list))
        .route("/pricing/set", post(crate::api::handlers::pricing::set_pricing))
        .route("/pricing/remove/{model}", delete(crate::api::handlers::pricing::remove_pricing))
        .route("/pricing/reset", post(crate::api::handlers::pricing::reset_pricing))
}

/// Usage analytics routes
pub fn usage_routes() -> Router<AppState> {
    Router::new()
        .route("/usage/records", get(crate::api::handlers::usage::get_usage_records))
}

/// Sync management routes
pub fn sync_routes() -> Router<AppState> {
    Router::new()
        .merge(sync_basic_routes())
        .merge(sync_folder_routes())
        .merge(sync_batch_routes())
}

/// Basic sync routes
fn sync_basic_routes() -> Router<AppState> {
    Router::new()
        .route("/sync/status", get(crate::api::handlers::sync::get_sync_status))
        .route("/sync/push", post(crate::api::handlers::sync::push_config))
        .route("/sync/pull", post(crate::api::handlers::sync::pull_config))
        .route("/sync/info", get(crate::api::handlers::sync::get_sync_info))
        .route("/sync/config", post(crate::api::handlers::sync::configure_sync))
}

/// Folder sync routes
fn sync_folder_routes() -> Router<AppState> {
    Router::new()
        .route("/sync/folders", get(crate::api::handlers::sync::list_sync_folders))
        .route("/sync/folders", post(crate::api::handlers::sync::add_sync_folder))
        .route("/sync/folders/{name}", delete(crate::api::handlers::sync::remove_sync_folder))
        .route("/sync/folders/{name}", get(crate::api::handlers::sync::get_sync_folder_info))
        .route("/sync/folders/{name}/enable", put(crate::api::handlers::sync::enable_sync_folder))
        .route("/sync/folders/{name}/disable", put(crate::api::handlers::sync::disable_sync_folder))
        .route("/sync/folders/{name}/push", post(crate::api::handlers::sync::push_sync_folder))
        .route("/sync/folders/{name}/pull", post(crate::api::handlers::sync::pull_sync_folder))
        .route("/sync/folders/{name}/status", get(crate::api::handlers::sync::get_sync_folder_status))
}

/// Batch sync operations routes
fn sync_batch_routes() -> Router<AppState> {
    Router::new()
        .route("/sync/all/push", post(crate::api::handlers::sync::push_all_folders))
        .route("/sync/all/pull", post(crate::api::handlers::sync::pull_all_folders))
        .route("/sync/all/status", get(crate::api::handlers::sync::get_all_folders_status))
}

/// Codex platform routes
pub fn codex_routes() -> Router<AppState> {
    Router::new()
        .merge(codex_mcp_routes())
        .merge(codex_profiles_routes())
        .merge(codex_config_routes())
}

/// Codex MCP routes
fn codex_mcp_routes() -> Router<AppState> {
    Router::new()
        .route("/codex/mcp", get(crate::api::handlers::platforms::codex::list_codex_mcp_servers))
        .route("/codex/mcp", post(crate::api::handlers::platforms::codex::add_codex_mcp_server))
        .route("/codex/mcp/{name}", put(crate::api::handlers::platforms::codex::update_codex_mcp_server))
        .route("/codex/mcp/{name}", delete(crate::api::handlers::platforms::codex::delete_codex_mcp_server))
}

/// Codex profiles routes
fn codex_profiles_routes() -> Router<AppState> {
    Router::new()
        .route("/codex/profiles", get(crate::api::handlers::platforms::codex::list_codex_profiles))
        .route("/codex/profiles", post(crate::api::handlers::platforms::codex::add_codex_profile))
        .route("/codex/profiles/{name}", get(crate::api::handlers::platforms::codex::get_codex_profile))
        .route("/codex/profiles/{name}", put(crate::api::handlers::platforms::codex::update_codex_profile))
        .route("/codex/profiles/{name}", delete(crate::api::handlers::platforms::codex::delete_codex_profile))
        .route("/codex/profiles/{name}/apply", post(crate::api::handlers::platforms::codex::apply_codex_profile))
}

/// Codex config routes
fn codex_config_routes() -> Router<AppState> {
    Router::new()
        .route("/codex/config", get(crate::api::handlers::platforms::codex::get_codex_config))
        .route("/codex/config", put(crate::api::handlers::platforms::codex::update_codex_base_config))
}

/// Gemini platform routes
pub fn gemini_routes() -> Router<AppState> {
    Router::new()
        .merge(gemini_mcp_routes())
        .merge(gemini_config_routes())
}

/// Gemini MCP routes
fn gemini_mcp_routes() -> Router<AppState> {
    Router::new()
        .route("/gemini/mcp", get(crate::api::handlers::platforms::gemini::list_gemini_mcp_servers))
        .route("/gemini/mcp", post(crate::api::handlers::platforms::gemini::add_gemini_mcp_server))
        .route("/gemini/mcp/{name}", put(crate::api::handlers::platforms::gemini::update_gemini_mcp_server))
        .route("/gemini/mcp/{name}", delete(crate::api::handlers::platforms::gemini::delete_gemini_mcp_server))
}

/// Gemini config routes
fn gemini_config_routes() -> Router<AppState> {
    Router::new()
        .route("/gemini/config", get(crate::api::handlers::platforms::gemini::get_gemini_config))
        .route("/gemini/config", put(crate::api::handlers::platforms::gemini::update_gemini_config))
}

/// Qwen platform routes
pub fn qwen_routes() -> Router<AppState> {
    Router::new()
        .merge(qwen_mcp_routes())
        .merge(qwen_config_routes())
}

/// Qwen MCP routes
fn qwen_mcp_routes() -> Router<AppState> {
    Router::new()
        .route("/qwen/mcp", get(crate::api::handlers::platforms::qwen::list_qwen_mcp_servers))
        .route("/qwen/mcp", post(crate::api::handlers::platforms::qwen::add_qwen_mcp_server))
        .route("/qwen/mcp/{name}", put(crate::api::handlers::platforms::qwen::update_qwen_mcp_server))
        .route("/qwen/mcp/{name}", delete(crate::api::handlers::platforms::qwen::delete_qwen_mcp_server))
}

/// Qwen config routes
fn qwen_config_routes() -> Router<AppState> {
    Router::new()
        .route("/qwen/config", get(crate::api::handlers::platforms::qwen::get_qwen_config))
        .route("/qwen/config", put(crate::api::handlers::platforms::qwen::update_qwen_config))
}

/// Config converter routes
pub fn converter_routes() -> Router<AppState> {
    Router::new()
        .route("/converter/convert", post(crate::api::handlers::converter::convert_config))
}

/// UI state routes
pub fn ui_state_routes() -> Router<AppState> {
    Router::new()
        .route("/ui-state/favorites", get(crate::api::handlers::ui_state::get_favorites))
        .route("/ui-state/favorites", post(crate::api::handlers::ui_state::add_favorite))
        .route("/ui-state/favorites/{id}", delete(crate::api::handlers::ui_state::remove_favorite))
        .route("/ui-state/history", get(crate::api::handlers::ui_state::get_history))
        .route("/ui-state/history", post(crate::api::handlers::ui_state::add_history))
        .route("/ui-state/history", delete(crate::api::handlers::ui_state::clear_history))
}