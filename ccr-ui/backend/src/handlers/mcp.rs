use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use crate::claude_config_manager::{ClaudeConfigManager, McpServerConfig};
use crate::models::{McpServerRequest, McpServersResponse, McpServerWithName};

#[get("/api/mcp")]
async fn list_mcp_servers() -> impl Responder {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize config manager: {}", e)
            }))
        }
    };

    match manager.get_mcp_servers() {
        Ok(servers) => {
            let servers_list: Vec<McpServerWithName> = servers
                .into_iter()
                .map(|(name, config)| McpServerWithName {
                    name,
                    command: config.command.unwrap_or_default(),
                    args: config.args.unwrap_or_default(),
                    env: config.env.unwrap_or_default(),
                    server_type: config.server_type,
                    url: config.url,
                    disabled: false, // .claude.json doesn't store disabled field
                })
                .collect();

            HttpResponse::Ok().json(McpServersResponse {
                success: true,
                data: json!({ "servers": servers_list }),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to read MCP servers: {}", e)
        })),
    }
}

#[post("/api/mcp")]
async fn add_mcp_server(req: web::Json<McpServerRequest>) -> impl Responder {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize config manager: {}", e)
            }))
        }
    };

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
    };

    match manager.add_mcp_server(req.name.clone(), server_config) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "MCP server added successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to add MCP server: {}", e)
        })),
    }
}

#[put("/api/mcp/{name}")]
async fn update_mcp_server(
    path: web::Path<String>,
    req: web::Json<McpServerRequest>,
) -> impl Responder {
    let name = path.into_inner();
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize config manager: {}", e)
            }))
        }
    };

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
    };

    match manager.update_mcp_server(&name, server_config) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "MCP server updated successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to update MCP server: {}", e)
        })),
    }
}

#[delete("/api/mcp/{name}")]
async fn delete_mcp_server(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize config manager: {}", e)
            }))
        }
    };

    match manager.delete_mcp_server(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "MCP server deleted successfully",
            "message": null
        })),
        Err(e) => HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to delete MCP server: {}", e)
        })),
    }
}

#[put("/api/mcp/{name}/toggle")]
async fn toggle_mcp_server(path: web::Path<String>) -> impl Responder {
    // Note: .claude.json doesn't support a "disabled" field for MCP servers
    // This endpoint exists for API compatibility but doesn't actually modify anything
    let _name = path.into_inner();

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": "Toggle not supported for MCP servers in .claude.json",
        "message": "MCP servers in .claude.json don't have a disabled field"
    }))
}
