// Marketplace API Handler
// 统一的资源市场 API，整合 Skills、MCP Presets、Plugins、Commands

use axum::{Json, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};

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

// ===== API Handlers =====

/// GET /api/marketplace - 获取所有市场项目
pub async fn list_marketplace_items() -> impl IntoResponse {
    let mut items: Vec<MarketItem> = Vec::new();

    // 1. 获取本地 Skills
    if let Ok(manager) = SkillsManager::new(Platform::Claude)
        && let Ok(skills) = manager.list_skills()
    {
        for skill in skills {
            items.push(MarketItem {
                id: format!("skill-{}", skill.name),
                name: skill.name.clone(),
                description: skill
                    .description
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
            });
        }
    }

    // 2. 获取内置 MCP Presets
    let presets = get_builtin_presets();
    for preset in presets {
        items.push(MarketItem {
            id: format!("mcp-{}", preset.id),
            name: preset.name,
            description: preset.description,
            category: MarketItemCategory::Mcp,
            author: Some("Anthropic".to_string()),
            version: None,
            downloads: None,
            rating: Some(5.0),
            installed: false, // TODO: 检查是否已安装
            source: MarketItemSource::Builtin,
            tags: Some(preset.tags),
            homepage: preset.homepage,
        });
    }

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

    let mut items: Vec<MarketItem> = Vec::new();

    match target_category {
        MarketItemCategory::Skill => {
            // 获取本地 Skills
            if let Ok(manager) = SkillsManager::new(Platform::Claude)
                && let Ok(skills) = manager.list_skills()
            {
                for skill in skills {
                    items.push(MarketItem {
                        id: format!("skill-{}", skill.name),
                        name: skill.name.clone(),
                        description: skill
                            .description
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
                    });
                }
            }
        }
        MarketItemCategory::Mcp => {
            let presets = get_builtin_presets();
            for preset in presets {
                items.push(MarketItem {
                    id: format!("mcp-{}", preset.id),
                    name: preset.name,
                    description: preset.description,
                    category: MarketItemCategory::Mcp,
                    author: Some("Anthropic".to_string()),
                    version: None,
                    downloads: None,
                    rating: Some(5.0),
                    installed: false,
                    source: MarketItemSource::Builtin,
                    tags: Some(preset.tags),
                    homepage: preset.homepage,
                });
            }
        }
        MarketItemCategory::Plugin => {
            // TODO: 获取插件列表
        }
        MarketItemCategory::Command => {
            // TODO: 获取命令模板列表
        }
    }

    let total = items.len();
    ApiResponse::success(MarketplaceResponse { items, total })
}

/// POST /api/marketplace/install - 安装市场项目
pub async fn install_item(Json(req): Json<InstallRequest>) -> impl IntoResponse {
    match req.category {
        MarketItemCategory::Mcp => {
            // 提取 preset_id
            let preset_id = req.item_id.strip_prefix("mcp-").unwrap_or(&req.item_id);

            // 使用 MCP Preset Manager 安装
            let sync_manager = ccr::managers::mcp_preset_manager::McpSyncManager::new();

            let mut target_platforms: Vec<Platform> = Vec::new();
            for platform_str in &req.platforms {
                if let Ok(p) = platform_str.parse::<Platform>() {
                    target_platforms.push(p);
                }
            }

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
                    let all_success = results.iter().all(|(_, r)| r.is_ok());
                    if all_success {
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
            // 提取 skill 名称
            let skill_name = req.item_id.strip_prefix("skill-").unwrap_or(&req.item_id);

            // TODO: 实现远程 Skill 安装
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

    // 获取本地 Skills
    if let Ok(manager) = SkillsManager::new(Platform::Claude)
        && let Ok(skills) = manager.list_skills()
    {
        for skill in skills {
            items.push(MarketItem {
                id: format!("skill-{}", skill.name),
                name: skill.name.clone(),
                description: skill
                    .description
                    .unwrap_or_else(|| "Installed skill".to_string()),
                category: MarketItemCategory::Skill,
                author: Some("Local".to_string()),
                version: None,
                downloads: None,
                rating: None,
                installed: true,
                source: MarketItemSource::Local,
                tags: None,
                homepage: None,
            });
        }
    }

    // TODO: 获取已安装的 MCP servers、Plugins 等

    let total = items.len();
    ApiResponse::success(MarketplaceResponse { items, total })
}

/// DELETE /api/marketplace/uninstall/:item_id - 卸载市场项目
pub async fn uninstall_item(Path(item_id): Path<String>) -> impl IntoResponse {
    // 解析 item_id 格式: category-name
    if let Some(skill_name) = item_id.strip_prefix("skill-") {
        // 卸载 Skill
        if let Ok(manager) = SkillsManager::new(Platform::Claude) {
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
    }

    if item_id.starts_with("mcp-") {
        // TODO: 实现 MCP server 移除
        return ApiResponse::error("MCP uninstallation not yet implemented".to_string());
    }

    ApiResponse::error(format!("Unknown item type for: {}", item_id))
}
