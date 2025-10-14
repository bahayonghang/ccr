use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use std::fs;
use std::path::Path;
use crate::markdown_manager::{AgentFrontmatter, MarkdownFile, MarkdownManager};
use crate::models::{Agent, AgentRequest, AgentsResponse};

/// 从 Markdown 内容中提取描述（用于没有 frontmatter 的文件）
fn extract_agent_description(content: &str, name: &str) -> String {
    // 查找第一个 ## 标题后的内容
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            // 找到标题后的第一行非空内容
            if let Some(desc_line) = content.lines()
                .skip_while(|l| !l.trim().starts_with("## "))
                .skip(1)
                .find(|l| !l.trim().is_empty() && !l.trim().starts_with('-') && !l.trim().starts_with('#')) {
                let desc = desc_line.trim();
                if desc.len() > 10 && desc.len() < 200 {
                    return desc.to_string();
                }
            }
        }
    }
    format!("Agent: {}", name)
}

#[get("/api/agents")]
async fn list_agents() -> impl Responder {
    let manager = match MarkdownManager::from_home_subdir("agents") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize agents manager: {}", e)
            }))
        }
    };

    // Get all files with their folder information
    match manager.list_files_with_folders() {
        Ok(files_with_folders) => {
            let mut agents = Vec::new();

            for (name, folder) in files_with_folders {
                // 尝试读取带 frontmatter 的文件
                let full_path = if folder.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", folder, name)
                };

                match manager.read_file::<AgentFrontmatter>(&full_path) {
                    Ok(file) => {
                        // 有 frontmatter 的情况
                        let tools: Vec<String> = file
                            .frontmatter
                            .tools
                            .unwrap_or_default()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        agents.push(Agent {
                            name: file.frontmatter.name.clone(),
                            model: "claude-sonnet-4-5-20250929".to_string(),
                            tools,
                            system_prompt: Some(file.content),
                            disabled: false,
                            folder: folder.clone(),
                        });
                    }
                    Err(_) => {
                        // 无 frontmatter 的情况，直接读取文件内容
                        let home = std::env::var("HOME").unwrap_or_default();
                        let file_path = Path::new(&home)
                            .join(".claude")
                            .join("agents")
                            .join(format!("{}.md", &full_path));

                        if let Ok(content) = fs::read_to_string(&file_path) {
                            let _description = extract_agent_description(&content, &name);
                            
                            agents.push(Agent {
                                name: name.clone(),
                                model: "claude-sonnet-4-5-20250929".to_string(),
                                tools: Vec::new(),
                                system_prompt: Some(content),
                                disabled: false,
                                folder: folder.clone(),
                            });
                            
                            tracing::info!("Agent '{}' in folder '{}' has no frontmatter", name, folder);
                        } else {
                            tracing::warn!("Failed to read agent file '{}'", full_path);
                        }
                    }
                }
            }

            // Also return list of subdirectories
            let subdirs = manager.list_subdirs().unwrap_or_default();

            HttpResponse::Ok().json(AgentsResponse {
                success: true,
                data: json!({ 
                    "agents": agents,
                    "folders": subdirs
                }),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to list agents: {}", e)
        })),
    }
}

#[post("/api/agents")]
async fn add_agent(req: web::Json<AgentRequest>) -> impl Responder {
    let manager = match MarkdownManager::from_home_subdir("agents") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize agents manager: {}", e)
            }))
        }
    };

    let tools_str = req.tools.clone().unwrap_or_default().join(", ");
    let frontmatter = AgentFrontmatter {
        name: req.name.clone(),
        description: Some(format!("Agent for {}", req.name)),
        tools: if tools_str.is_empty() {
            None
        } else {
            Some(tools_str)
        },
    };

    let content = req
        .system_prompt
        .clone()
        .unwrap_or_else(|| format!("# {} Agent\n\nYou are an AI agent named {}.", req.name, req.name));

    let md_file = MarkdownFile {
        frontmatter,
        content,
    };

    match manager.write_file(&req.name, &md_file) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Agent added successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to add agent: {}", e)
        })),
    }
}

#[put("/api/agents/{name}")]
async fn update_agent(
    path: web::Path<String>,
    req: web::Json<AgentRequest>,
) -> impl Responder {
    let name = path.into_inner();
    let manager = match MarkdownManager::from_home_subdir("agents") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize agents manager: {}", e)
            }))
        }
    };

    let tools_str = req.tools.clone().unwrap_or_default().join(", ");
    let frontmatter = AgentFrontmatter {
        name: req.name.clone(),
        description: Some(format!("Agent for {}", req.name)),
        tools: if tools_str.is_empty() {
            None
        } else {
            Some(tools_str)
        },
    };

    let content = req
        .system_prompt
        .clone()
        .unwrap_or_else(|| format!("# {} Agent\n\nYou are an AI agent named {}.", req.name, req.name));

    let md_file = MarkdownFile {
        frontmatter,
        content,
    };

    // Delete old file if name changed
    if name != req.name {
        let _ = manager.delete_file(&name);
    }

    match manager.write_file(&req.name, &md_file) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Agent updated successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to update agent: {}", e)
        })),
    }
}

#[delete("/api/agents/{name}")]
async fn delete_agent(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let manager = match MarkdownManager::from_home_subdir("agents") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize agents manager: {}", e)
            }))
        }
    };

    match manager.delete_file(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Agent deleted successfully",
            "message": null
        })),
        Err(e) => HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to delete agent: {}", e)
        })),
    }
}

#[put("/api/agents/{name}/toggle")]
async fn toggle_agent(path: web::Path<String>) -> impl Responder {
    // Note: Agent markdown files don't support a "disabled" field
    // This endpoint exists for API compatibility
    let _name = path.into_inner();

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": "Toggle not supported for agents",
        "message": "Agent files don't have a disabled field"
    }))
}
