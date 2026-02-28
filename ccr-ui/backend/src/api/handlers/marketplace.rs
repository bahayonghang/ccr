// Marketplace API Handler
// 统一的资源市场 API，整合 Skills、MCP Presets、Plugins、Commands

use axum::{Json, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::managers::config::claude_manager::ClaudeConfigManager;
use crate::models::api::ApiResponse;

use ccr::managers::mcp_preset_manager::get_builtin_presets;
use ccr::managers::skills_manager::SkillsManager;
use ccr::models::Platform;

// ===== Market Item Models =====

/// 市场项目的分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MarketItemCategory {
    Skill,
    Mcp,
    Plugin,
    Command,
}

/// 市场项目的来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketItemSource {
    Builtin,
    Remote,
    Local,
}

/// 市场项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: MarketItemCategory,
    pub author: Option<String>,
    pub version: Option<String>,
    pub downloads: Option<u64>,
    pub rating: Option<f32>,
    pub installed: bool,
    pub source: MarketItemSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default)]
    pub requires_api_key: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

/// 市场项目列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceResponse {
    pub items: Vec<MarketItem>,
    pub total: usize,
}

/// 安装请求
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallRequest {
    pub item_id: String,
    pub category: MarketItemCategory,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

// ===== Helpers =====

/// 获取已安装的 MCP server 名称集合
fn get_installed_mcp_names() -> std::collections::HashSet<String> {
    ClaudeConfigManager::default()
        .and_then(|mgr| mgr.get_mcp_servers())
        .map(|servers| servers.keys().cloned().collect())
        .unwrap_or_default()
}

/// 构建 Skill MarketItem
fn skill_to_market_item(skill: &ccr::models::skill::Skill) -> MarketItem {
    MarketItem {
        id: format!("skill-{}", skill.name),
        name: skill.name.clone(),
        description: skill
            .description
            .clone()
            .unwrap_or_else(|| "Local skill".to_string()),
        category: MarketItemCategory::Skill,
        author: Some("Local".to_string()),
        version: None,
        downloads: None,
        rating: None,
        installed: true,
        source: MarketItemSource::Local,
        tags: None,
        homepage: None,
        requires_api_key: false,
        api_key_env: None,
    }
}

/// 构建 MCP Preset MarketItem
fn preset_to_market_item(
    preset: &ccr::models::mcp_preset::McpPreset,
    installed_names: &std::collections::HashSet<String>,
) -> MarketItem {
    MarketItem {
        id: format!("mcp-{}", preset.id),
        name: preset.name.clone(),
        description: preset.description.clone(),
        category: MarketItemCategory::Mcp,
        author: Some("MCP".to_string()),
        version: None,
        downloads: None,
        rating: Some(5.0),
        installed: installed_names.contains(&preset.id),
        source: MarketItemSource::Builtin,
        tags: Some(preset.tags.clone()),
        homepage: preset.homepage.clone(),
        requires_api_key: preset.requires_api_key,
        api_key_env: preset.api_key_env.clone(),
    }
}

/// 构建已安装 MCP server MarketItem（非预设的自定义 server）
fn mcp_server_to_market_item(name: &str) -> MarketItem {
    MarketItem {
        id: format!("mcp-{}", name),
        name: name.to_string(),
        description: "Installed MCP server".to_string(),
        category: MarketItemCategory::Mcp,
        author: None,
        version: None,
        downloads: None,
        rating: None,
        installed: true,
        source: MarketItemSource::Local,
        tags: None,
        homepage: None,
        requires_api_key: false,
        api_key_env: None,
    }
}

/// 收集 Skills 列表
fn collect_skills() -> Vec<MarketItem> {
    SkillsManager::new(Platform::Claude)
        .and_then(|mgr| mgr.list_skills())
        .map(|skills| skills.iter().map(skill_to_market_item).collect())
        .unwrap_or_default()
}

/// 收集 MCP Presets 列表（带安装状态检测）
fn collect_mcp_presets(installed_names: &std::collections::HashSet<String>) -> Vec<MarketItem> {
    get_builtin_presets()
        .iter()
        .map(|p| preset_to_market_item(p, installed_names))
        .collect()
}

// ===== API Handlers =====

/// GET /api/marketplace - 获取所有市场项目
pub async fn list_marketplace_items() -> impl IntoResponse {
    let installed_names = get_installed_mcp_names();
    let mut items: Vec<MarketItem> = Vec::new();

    items.extend(collect_skills());
    items.extend(collect_mcp_presets(&installed_names));

    let total = items.len();
    ApiResponse::success(MarketplaceResponse { items, total })
}

/// GET /api/marketplace/category/:category - 按分类获取市场项目
pub async fn list_items_by_category(Path(category): Path<String>) -> impl IntoResponse {
    let target_category = match category.to_lowercase().as_str() {
        "skill" | "skills" => MarketItemCategory::Skill,
        "mcp" => MarketItemCategory::Mcp,
        "plugin" | "plugins" => MarketItemCategory::Plugin,
        "command" | "commands" => MarketItemCategory::Command,
        _ => return ApiResponse::error(format!("Unknown category: {}", category)),
    };

    let items: Vec<MarketItem> = match target_category {
        MarketItemCategory::Skill => collect_skills(),
        MarketItemCategory::Mcp => {
            let installed_names = get_installed_mcp_names();
            collect_mcp_presets(&installed_names)
        }
        MarketItemCategory::Plugin | MarketItemCategory::Command => Vec::new(),
    };

    let total = items.len();
    ApiResponse::success(MarketplaceResponse { items, total })
}

/// POST /api/marketplace/install - 安装市场项目
pub async fn install_item(Json(req): Json<InstallRequest>) -> impl IntoResponse {
    match req.category {
        MarketItemCategory::Mcp => {
            let preset_id = req.item_id.strip_prefix("mcp-").unwrap_or(&req.item_id);
            let sync_manager = ccr::managers::mcp_preset_manager::McpSyncManager::new();

            let mut target_platforms: Vec<Platform> = req
                .platforms
                .iter()
                .filter_map(|s| s.parse::<Platform>().ok())
                .collect();
            if target_platforms.is_empty() {
                target_platforms.push(Platform::Claude);
            }

            let custom_env = if req.env.is_empty() {
                None
            } else {
                Some(req.env)
            };

            match sync_manager.sync_preset_to_all(preset_id, custom_env, &target_platforms) {
                Ok(results) => {
                    if results.iter().all(|(_, r)| r.is_ok()) {
                        ApiResponse::success(format!(
                            "MCP preset '{}' installed successfully",
                            preset_id
                        ))
                    } else {
                        ApiResponse::error("Some platforms failed to install".to_string())
                    }
                }
                Err(e) => ApiResponse::error(format!("Installation failed: {}", e)),
            }
        }
        MarketItemCategory::Skill => {
            let skill_name = req.item_id.strip_prefix("skill-").unwrap_or(&req.item_id);
            ApiResponse::error(format!(
                "Skill installation for '{}' not yet implemented",
                skill_name
            ))
        }
        MarketItemCategory::Plugin => {
            ApiResponse::error("Plugin installation not yet implemented".to_string())
        }
        MarketItemCategory::Command => {
            ApiResponse::error("Command template installation not yet implemented".to_string())
        }
    }
}

/// GET /api/marketplace/installed - 获取已安装的项目
pub async fn list_installed_items() -> impl IntoResponse {
    let mut items: Vec<MarketItem> = Vec::new();

    // 已安装的 Skills
    items.extend(collect_skills());

    // 已安装的 MCP servers
    let installed_names = get_installed_mcp_names();
    let preset_ids: std::collections::HashSet<String> =
        get_builtin_presets().iter().map(|p| p.id.clone()).collect();

    for name in &installed_names {
        if preset_ids.contains(name) {
            // 是内置预设，用预设信息
            if let Some(preset) = get_builtin_presets().into_iter().find(|p| &p.id == name) {
                items.push(preset_to_market_item(&preset, &installed_names));
            }
        } else {
            // 自定义 MCP server
            items.push(mcp_server_to_market_item(name));
        }
    }

    let total = items.len();
    ApiResponse::success(MarketplaceResponse { items, total })
}

/// DELETE /api/marketplace/uninstall/:item_id - 卸载市场项目
pub async fn uninstall_item(Path(item_id): Path<String>) -> impl IntoResponse {
    if let Some(skill_name) = item_id.strip_prefix("skill-")
        && let Ok(manager) = SkillsManager::new(Platform::Claude)
    {
        match manager.uninstall_skill(skill_name) {
            Ok(_) => {
                return ApiResponse::success(format!(
                    "Skill '{}' uninstalled successfully",
                    skill_name
                ));
            }
            Err(e) => return ApiResponse::error(format!("Failed to uninstall skill: {}", e)),
        }
    }

    if let Some(mcp_name) = item_id.strip_prefix("mcp-") {
        if let Ok(manager) = ClaudeConfigManager::default() {
            match manager.delete_mcp_server(mcp_name) {
                Ok(_) => {
                    return ApiResponse::success(format!(
                        "MCP server '{}' uninstalled successfully",
                        mcp_name
                    ));
                }
                Err(e) => {
                    return ApiResponse::error(format!("Failed to uninstall MCP server: {}", e));
                }
            }
        }
        return ApiResponse::error("Failed to initialize config manager".to_string());
    }

    ApiResponse::error(format!("Unknown item type for: {}", item_id))
}
