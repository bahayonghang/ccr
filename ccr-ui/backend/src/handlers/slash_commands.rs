use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use std::fs;
use std::path::Path;
use crate::markdown_manager::{CommandFrontmatter, MarkdownFile, MarkdownManager};
use crate::models::{SlashCommand, SlashCommandRequest, SlashCommandsResponse};

/// 从 Markdown 内容中提取描述
fn extract_description_from_content(content: &str, name: &str) -> String {
    // 尝试查找 ## Context 或 ## Usage 后的内容
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## Context") || trimmed.starts_with("## Your Role") {
            // 找到下一行非空内容
            if let Some(desc_line) = content.lines()
                .skip_while(|l| !l.trim().starts_with("## Context") && !l.trim().starts_with("## Your Role"))
                .skip(1)
                .find(|l| !l.trim().is_empty() && !l.trim().starts_with('-') && !l.trim().starts_with('#')) {
                let desc = desc_line.trim();
                if desc.len() > 10 && desc.len() < 200 {
                    return desc.to_string();
                }
            }
        }
    }
    
    // 如果找不到，使用默认描述
    format!("Slash command: {}", name)
}

#[get("/api/slash-commands")]
async fn list_slash_commands() -> impl Responder {
    let manager = match MarkdownManager::from_home_subdir("commands") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize commands manager: {}", e)
            }))
        }
    };

    // Get all files with their folder information
    match manager.list_files_with_folders() {
        Ok(files_with_folders) => {
            let mut commands = Vec::new();

            for (name, folder) in files_with_folders {
                // 构建完整路径
                let full_path = if folder.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", folder, name)
                };

                // 尝试读取带 frontmatter 的文件
                match manager.read_file::<CommandFrontmatter>(&full_path) {
                    Ok(file) => {
                        // 有 frontmatter 的情况
                        commands.push(SlashCommand {
                            name: file.frontmatter.name.unwrap_or(name.clone()),
                            description: file.frontmatter.description.unwrap_or_default(),
                            command: name,
                            args: None,
                            disabled: false,
                            folder: folder.clone(),
                        });
                    }
                    Err(_) => {
                        // 无 frontmatter 的情况，直接读取文件内容
                        let home = std::env::var("HOME").unwrap_or_default();
                        let file_path = Path::new(&home)
                            .join(".claude")
                            .join("commands")
                            .join(format!("{}.md", &full_path));
                        
                        let description = if let Ok(content) = fs::read_to_string(&file_path) {
                            extract_description_from_content(&content, &name)
                        } else {
                            format!("Slash command: {}", &name)
                        };

                        tracing::info!("Command '{}' in folder '{}' has no frontmatter", name, folder);
                        
                        commands.push(SlashCommand {
                            name: name.clone(),
                            description,
                            command: name,
                            args: None,
                            disabled: false,
                            folder: folder.clone(),
                        });
                    }
                }
            }

            // Also return list of subdirectories
            let subdirs = manager.list_subdirs().unwrap_or_default();

            HttpResponse::Ok().json(SlashCommandsResponse {
                success: true,
                data: json!({ 
                    "commands": commands,
                    "folders": subdirs
                }),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to list slash commands: {}", e)
        })),
    }
}

#[post("/api/slash-commands")]
async fn add_slash_command(req: web::Json<SlashCommandRequest>) -> impl Responder {
    let manager = match MarkdownManager::from_home_subdir("commands") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize commands manager: {}", e)
            }))
        }
    };

    let frontmatter = CommandFrontmatter {
        name: Some(req.name.clone()),
        description: Some(req.description.clone()),
        argument_hint: None,
    };

    // Generate a simple default content
    let content = format!("## Usage\n\n`/{} <ARGUMENTS>`\n\n## Description\n\n{}\n\n## Content\n\n{}", 
        req.name, 
        req.description,
        req.command
    );

    let md_file = MarkdownFile {
        frontmatter,
        content,
    };

    match manager.write_file(&req.name, &md_file) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Slash command added successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to add slash command: {}", e)
        })),
    }
}

#[put("/api/slash-commands/{name}")]
async fn update_slash_command(
    path: web::Path<String>,
    req: web::Json<SlashCommandRequest>,
) -> impl Responder {
    let name = path.into_inner();
    let manager = match MarkdownManager::from_home_subdir("commands") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize commands manager: {}", e)
            }))
        }
    };

    // Try to read existing file to preserve content
    let existing_content = match manager.read_file::<CommandFrontmatter>(&name) {
        Ok(file) => file.content,
        Err(_) => format!("## Usage\n\n`/{} <ARGUMENTS>`\n\n## Description\n\n{}\n\n## Content\n\n{}", 
            req.name, 
            req.description,
            req.command
        ),
    };

    let frontmatter = CommandFrontmatter {
        name: Some(req.name.clone()),
        description: Some(req.description.clone()),
        argument_hint: None,
    };

    let md_file = MarkdownFile {
        frontmatter,
        content: existing_content,
    };

    // Delete old file if name changed
    if name != req.name {
        let _ = manager.delete_file(&name);
    }

    match manager.write_file(&req.name, &md_file) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Slash command updated successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to update slash command: {}", e)
        })),
    }
}

#[delete("/api/slash-commands/{name}")]
async fn delete_slash_command(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let manager = match MarkdownManager::from_home_subdir("commands") {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize commands manager: {}", e)
            }))
        }
    };

    match manager.delete_file(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Slash command deleted successfully",
            "message": null
        })),
        Err(e) => HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to delete slash command: {}", e)
        })),
    }
}

#[put("/api/slash-commands/{name}/toggle")]
async fn toggle_slash_command(path: web::Path<String>) -> impl Responder {
    // Note: Slash command markdown files don't support a "disabled" field
    // This endpoint exists for API compatibility
    let _name = path.into_inner();

    HttpResponse::Ok().json(json!({
        "success": true,
        "data": "Toggle not supported for slash commands",
        "message": "Slash command files don't have a disabled field"
    }))
}
