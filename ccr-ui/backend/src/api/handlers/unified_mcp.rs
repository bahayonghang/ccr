// 统一 MCP 管理 API 处理器
//
// 适配器模式：将各平台 MCP Manager 的差异化接口统一为标准 CRUD。
// 旧平台路由保持不变，此模块仅提供 /api/unified/mcp 系列端点。

use axum::{Json, extract::Path, extract::Query};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

use crate::api::handlers::response::ApiSuccess;
use crate::core::error::{ApiError, ApiResult};
use crate::models::api::{
    PlatformMcpCapability, UnifiedMcpListResponse, UnifiedMcpRequest, UnifiedMcpServer,
};

// 平台 Managers
use crate::managers::config::claude_manager::ClaudeConfigManager;
use crate::managers::config::codex_manager::CodexConfigManager;
use crate::managers::config::droid_manager::DroidConfigManager;
use crate::managers::config::gemini_manager::GeminiConfigManager;
use crate::managers::config::qwen_manager::QwenConfigManager;

// 平台 MCP 模型
use crate::managers::config::claude_manager::McpServerConfig;
use crate::models::platforms::codex::CodexMcpServer;
use crate::models::platforms::droid::DroidMcpServer;
use crate::models::platforms::gemini::GeminiMcpServer;
use crate::models::platforms::qwen::QwenMcpServer;

// ============ 能力矩阵 ============

fn capabilities() -> Vec<PlatformMcpCapability> {
    vec![
        PlatformMcpCapability {
            platform: "claude".into(),
            supports_toggle: true,
            supports_url: true,
            supports_headers: false,
            supports_timeout: false,
            supports_cwd: false,
            supports_trust: false,
            supports_include_tools: false,
        },
        PlatformMcpCapability {
            platform: "codex".into(),
            supports_toggle: false,
            supports_url: true,
            supports_headers: false,
            supports_timeout: false,
            supports_cwd: true,
            supports_trust: false,
            supports_include_tools: false,
        },
        PlatformMcpCapability {
            platform: "gemini".into(),
            supports_toggle: false,
            supports_url: false,
            supports_headers: false,
            supports_timeout: true,
            supports_cwd: true,
            supports_trust: true,
            supports_include_tools: true,
        },
        PlatformMcpCapability {
            platform: "qwen".into(),
            supports_toggle: false,
            supports_url: true,
            supports_headers: true,
            supports_timeout: true,
            supports_cwd: false,
            supports_trust: false,
            supports_include_tools: false,
        },
        PlatformMcpCapability {
            platform: "droid".into(),
            supports_toggle: false,
            supports_url: true,
            supports_headers: true,
            supports_timeout: true,
            supports_cwd: false,
            supports_trust: false,
            supports_include_tools: false,
        },
    ]
}

// ============ 平台适配器 - 列表 ============

fn list_claude() -> Result<Vec<UnifiedMcpServer>, String> {
    let manager =
        ClaudeConfigManager::default().map_err(|e| format!("初始化 Claude 管理器失败: {}", e))?;
    let servers = manager
        .get_mcp_servers()
        .map_err(|e| format!("读取 Claude MCP 服务器失败: {}", e))?;

    Ok(servers
        .into_iter()
        .map(|(name, cfg)| UnifiedMcpServer {
            platform: "claude".into(),
            name,
            command: cfg.command,
            url: cfg.url,
            args: cfg.args.unwrap_or_default(),
            env: cfg.env.unwrap_or_default(),
            headers: None,
            timeout: None,
            disabled: cfg.disabled.unwrap_or(false),
            cwd: None,
            trust: None,
            include_tools: None,
        })
        .collect())
}

fn list_codex() -> Result<Vec<UnifiedMcpServer>, String> {
    let manager =
        CodexConfigManager::default().map_err(|e| format!("初始化 Codex 管理器失败: {}", e))?;
    let servers = manager.list_mcp_servers()?;

    Ok(servers
        .into_iter()
        .map(|s| UnifiedMcpServer {
            platform: "codex".into(),
            name: s.name,
            command: s.server.command,
            url: s.server.url,
            args: s.server.args.unwrap_or_default(),
            env: s.server.env.unwrap_or_default(),
            headers: None,
            timeout: None,
            disabled: false,
            cwd: s.server.cwd,
            trust: None,
            include_tools: None,
        })
        .collect())
}

fn list_gemini() -> Result<Vec<UnifiedMcpServer>, String> {
    let manager =
        GeminiConfigManager::default().map_err(|e| format!("初始化 Gemini 管理器失败: {}", e))?;
    let servers = manager.list_mcp_servers()?;

    Ok(servers
        .into_iter()
        .map(|(name, s)| UnifiedMcpServer {
            platform: "gemini".into(),
            name,
            command: s.command,
            url: None,
            args: s.args.unwrap_or_default(),
            env: s.env.unwrap_or_default(),
            headers: None,
            timeout: s.timeout,
            disabled: false,
            cwd: s.cwd,
            trust: s.trust,
            include_tools: s.include_tools,
        })
        .collect())
}

fn list_qwen() -> Result<Vec<UnifiedMcpServer>, String> {
    let manager =
        QwenConfigManager::default().map_err(|e| format!("初始化 Qwen 管理器失败: {}", e))?;
    let servers = manager.list_mcp_servers()?;

    Ok(servers
        .into_iter()
        .map(|(name, s)| UnifiedMcpServer {
            platform: "qwen".into(),
            name,
            command: s.command,
            url: s.url,
            args: s.args.unwrap_or_default(),
            env: s.env.unwrap_or_default(),
            headers: s.headers,
            timeout: s.timeout,
            disabled: false,
            cwd: None,
            trust: None,
            include_tools: None,
        })
        .collect())
}

fn list_droid() -> Result<Vec<UnifiedMcpServer>, String> {
    let manager =
        DroidConfigManager::default().map_err(|e| format!("初始化 Droid 管理器失败: {}", e))?;
    let servers = manager.list_mcp_servers()?;

    Ok(servers
        .into_iter()
        .map(|(name, s)| UnifiedMcpServer {
            platform: "droid".into(),
            name,
            command: s.command,
            url: s.url,
            args: s.args.unwrap_or_default(),
            env: s.env.unwrap_or_default(),
            headers: s.headers,
            timeout: s.timeout,
            disabled: false,
            cwd: None,
            trust: None,
            include_tools: None,
        })
        .collect())
}

// ============ 平台适配器 - 添加 ============

fn add_to_claude(req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        ClaudeConfigManager::default().map_err(|e| format!("初始化 Claude 管理器失败: {}", e))?;
    let server = McpServerConfig {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        server_type: None,
        url: req.url.clone(),
        disabled: req.disabled,
    };
    manager
        .add_mcp_server(req.name.clone(), server)
        .map_err(|e| format!("添加 Claude MCP 服务器失败: {}", e))
}

fn add_to_codex(req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        CodexConfigManager::default().map_err(|e| format!("初始化 Codex 管理器失败: {}", e))?;
    let server = CodexMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        cwd: req.cwd.clone(),
        startup_timeout_ms: None,
        url: req.url.clone(),
        bearer_token: None,
        other: HashMap::new(),
    };
    manager.add_mcp_server(req.name.clone(), server)
}

fn add_to_gemini(req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        GeminiConfigManager::default().map_err(|e| format!("初始化 Gemini 管理器失败: {}", e))?;
    let server = GeminiMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        cwd: req.cwd.clone(),
        timeout: req.timeout,
        trust: req.trust,
        include_tools: req.include_tools.clone(),
        other: HashMap::new(),
    };
    manager.add_mcp_server(req.name.clone(), server)
}

fn add_to_qwen(req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        QwenConfigManager::default().map_err(|e| format!("初始化 Qwen 管理器失败: {}", e))?;
    let server = QwenMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        url: req.url.clone(),
        http_url: None,
        headers: req.headers.clone(),
        timeout: req.timeout,
        other: HashMap::new(),
    };
    manager.add_mcp_server(req.name.clone(), server)
}

fn add_to_droid(req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        DroidConfigManager::default().map_err(|e| format!("初始化 Droid 管理器失败: {}", e))?;
    let server = DroidMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        url: req.url.clone(),
        http_url: None,
        headers: req.headers.clone(),
        timeout: req.timeout,
        other: HashMap::new(),
    };
    manager.add_mcp_server(req.name.clone(), server)
}

// ============ 平台适配器 - 更新 ============

fn update_on_claude(name: &str, req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        ClaudeConfigManager::default().map_err(|e| format!("初始化 Claude 管理器失败: {}", e))?;
    let server = McpServerConfig {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        server_type: None,
        url: req.url.clone(),
        disabled: req.disabled,
    };
    manager
        .update_mcp_server(name, server)
        .map_err(|e| format!("更新 Claude MCP 服务器失败: {}", e))
}

fn update_on_codex(name: &str, req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        CodexConfigManager::default().map_err(|e| format!("初始化 Codex 管理器失败: {}", e))?;
    let server = CodexMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        cwd: req.cwd.clone(),
        startup_timeout_ms: None,
        url: req.url.clone(),
        bearer_token: None,
        other: HashMap::new(),
    };
    manager.update_mcp_server(name, server)
}

fn update_on_gemini(name: &str, req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        GeminiConfigManager::default().map_err(|e| format!("初始化 Gemini 管理器失败: {}", e))?;
    let server = GeminiMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        cwd: req.cwd.clone(),
        timeout: req.timeout,
        trust: req.trust,
        include_tools: req.include_tools.clone(),
        other: HashMap::new(),
    };
    manager.update_mcp_server(name, server)
}

fn update_on_qwen(name: &str, req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        QwenConfigManager::default().map_err(|e| format!("初始化 Qwen 管理器失败: {}", e))?;
    let server = QwenMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        url: req.url.clone(),
        http_url: None,
        headers: req.headers.clone(),
        timeout: req.timeout,
        other: HashMap::new(),
    };
    manager.update_mcp_server(name, server)
}

fn update_on_droid(name: &str, req: &UnifiedMcpRequest) -> Result<(), String> {
    let manager =
        DroidConfigManager::default().map_err(|e| format!("初始化 Droid 管理器失败: {}", e))?;
    let server = DroidMcpServer {
        command: req.command.clone(),
        args: req.args.clone(),
        env: req.env.clone(),
        url: req.url.clone(),
        http_url: None,
        headers: req.headers.clone(),
        timeout: req.timeout,
        other: HashMap::new(),
    };
    manager.update_mcp_server(name, server)
}

// ============ 平台适配器 - 删除 ============

fn delete_from_platform(platform: &str, name: &str) -> Result<(), String> {
    match platform {
        "claude" => {
            let manager = ClaudeConfigManager::default()
                .map_err(|e| format!("初始化 Claude 管理器失败: {}", e))?;
            manager
                .delete_mcp_server(name)
                .map_err(|e| format!("删除 Claude MCP 服务器失败: {}", e))
        }
        "codex" => {
            let manager = CodexConfigManager::default()
                .map_err(|e| format!("初始化 Codex 管理器失败: {}", e))?;
            manager.delete_mcp_server(name)
        }
        "gemini" => {
            let manager = GeminiConfigManager::default()
                .map_err(|e| format!("初始化 Gemini 管理器失败: {}", e))?;
            manager.delete_mcp_server(name)
        }
        "qwen" => {
            let manager = QwenConfigManager::default()
                .map_err(|e| format!("初始化 Qwen 管理器失败: {}", e))?;
            manager.delete_mcp_server(name)
        }
        "droid" => {
            let manager = DroidConfigManager::default()
                .map_err(|e| format!("初始化 Droid 管理器失败: {}", e))?;
            manager.delete_mcp_server(name)
        }
        _ => Err(format!("不支持的平台: {}", platform)),
    }
}

// ============ Query 参数 ============

#[derive(Debug, Deserialize, Default)]
pub struct McpQueryParams {
    /// 筛选平台（可选，多个以逗号分隔）
    pub platform: Option<String>,
    /// 指定 profile（预留，暂使用 current）
    #[allow(dead_code)]
    pub profile: Option<String>,
}

// ============ API Handlers ============

/// GET /api/unified/mcp - 列出所有平台的 MCP 服务器
pub async fn list_unified_mcp(
    Query(params): Query<McpQueryParams>,
) -> ApiResult<ApiSuccess<UnifiedMcpListResponse>> {
    let requested_platforms: Vec<String> = params
        .platform
        .map(|p| p.split(',').map(|s| s.trim().to_lowercase()).collect())
        .unwrap_or_default();

    let all_platforms = ["claude", "codex", "gemini", "qwen", "droid"];
    let target_platforms: Vec<&str> = if requested_platforms.is_empty() {
        all_platforms.to_vec()
    } else {
        all_platforms
            .iter()
            .filter(|p| requested_platforms.contains(&p.to_string()))
            .copied()
            .collect()
    };

    let mut all_servers = Vec::new();

    for platform in &target_platforms {
        let result = match *platform {
            "claude" => list_claude(),
            "codex" => list_codex(),
            "gemini" => list_gemini(),
            "qwen" => list_qwen(),
            "droid" => list_droid(),
            _ => continue,
        };

        match result {
            Ok(servers) => all_servers.extend(servers),
            Err(e) => {
                // 单个平台失败不阻塞其他平台，记录警告
                tracing::warn!(platform = *platform, error = %e, "读取平台 MCP 服务器失败，跳过");
            }
        }
    }

    let caps = capabilities()
        .into_iter()
        .filter(|c| target_platforms.contains(&c.platform.as_str()))
        .collect();

    Ok(ApiSuccess(UnifiedMcpListResponse {
        servers: all_servers,
        capabilities: caps,
    }))
}

/// POST /api/unified/mcp - 添加 MCP 服务器到指定平台
pub async fn add_unified_mcp(
    Json(req): Json<UnifiedMcpRequest>,
) -> ApiResult<ApiSuccess<serde_json::Value>> {
    // 验证必填字段
    if req.name.is_empty() {
        return Err(ApiError::bad_request("name 字段不能为空"));
    }
    if req.command.is_none() && req.url.is_none() {
        return Err(ApiError::bad_request(
            "必须提供 command（STDIO）或 url（HTTP）",
        ));
    }

    let result = match req.platform.as_str() {
        "claude" => add_to_claude(&req),
        "codex" => add_to_codex(&req),
        "gemini" => add_to_gemini(&req),
        "qwen" => add_to_qwen(&req),
        "droid" => add_to_droid(&req),
        other => return Err(ApiError::bad_request(format!("不支持的平台: {}", other))),
    };

    result.map_err(ApiError::internal)?;

    Ok(ApiSuccess(json!({
        "message": format!("{} 平台 MCP 服务器 '{}' 添加成功", req.platform, req.name)
    })))
}

/// PUT /api/unified/mcp/:platform/:name - 更新指定平台的 MCP 服务器
pub async fn update_unified_mcp(
    Path((platform, name)): Path<(String, String)>,
    Json(req): Json<UnifiedMcpRequest>,
) -> ApiResult<ApiSuccess<serde_json::Value>> {
    let result = match platform.as_str() {
        "claude" => update_on_claude(&name, &req),
        "codex" => update_on_codex(&name, &req),
        "gemini" => update_on_gemini(&name, &req),
        "qwen" => update_on_qwen(&name, &req),
        "droid" => update_on_droid(&name, &req),
        other => return Err(ApiError::bad_request(format!("不支持的平台: {}", other))),
    };

    result.map_err(ApiError::internal)?;

    Ok(ApiSuccess(json!({
        "message": format!("{} 平台 MCP 服务器 '{}' 更新成功", platform, name)
    })))
}

/// DELETE /api/unified/mcp/:platform/:name - 删除指定平台的 MCP 服务器
pub async fn delete_unified_mcp(
    Path((platform, name)): Path<(String, String)>,
) -> ApiResult<ApiSuccess<serde_json::Value>> {
    delete_from_platform(&platform, &name).map_err(ApiError::internal)?;

    Ok(ApiSuccess(json!({
        "message": format!("{} 平台 MCP 服务器 '{}' 删除成功", platform, name)
    })))
}

/// PUT /api/unified/mcp/:platform/:name/toggle - 切换 MCP 服务器启用/禁用状态
pub async fn toggle_unified_mcp(
    Path((platform, name)): Path<(String, String)>,
) -> ApiResult<ApiSuccess<serde_json::Value>> {
    match platform.as_str() {
        "claude" => {
            let manager = ClaudeConfigManager::default()
                .map_err(|e| ApiError::internal(format!("初始化 Claude 管理器失败: {}", e)))?;

            let mut servers = manager
                .get_mcp_servers()
                .map_err(|e| ApiError::internal(format!("读取 MCP 服务器失败: {}", e)))?;

            let server = servers
                .get_mut(&name)
                .ok_or_else(|| ApiError::not_found(format!("MCP 服务器 '{}' 不存在", name)))?;

            let new_state = !server.disabled.unwrap_or(false);
            server.disabled = Some(new_state);

            manager
                .update_mcp_server(&name, server.clone())
                .map_err(|e| ApiError::internal(format!("更新 MCP 服务器失败: {}", e)))?;

            let state_str = if new_state { "disabled" } else { "enabled" };
            Ok(ApiSuccess(json!({
                "message": format!("Claude MCP 服务器 '{}' 已 {}", name, state_str),
                "disabled": new_state
            })))
        }
        other => Err(ApiError::bad_request(format!(
            "{} 平台不支持 toggle 操作",
            other
        ))),
    }
}
