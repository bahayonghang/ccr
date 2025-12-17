// 内置 Prompt 模板 API 端点
// 提供常用的 AI 编程助手提示词模板

use axum::{Json, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize};

use ccr::managers::builtin_prompts::{PromptCategory, get_builtin_prompts, get_prompt_by_id};

use crate::models::api::ApiResponse;

/// Prompt 模板响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// GET /api/prompts/builtin - 获取所有内置模板
pub async fn list_builtin_prompts() -> impl IntoResponse {
    let prompts = get_builtin_prompts();

    let responses: Vec<PromptResponse> = prompts
        .into_iter()
        .map(|p| PromptResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            content: p.content,
            category: p.category.as_str().to_string(),
            tags: p.tags,
        })
        .collect();

    Json(ApiResponse::success(responses))
}

/// GET /api/prompts/builtin/:id - 获取单个模板详情
pub async fn get_builtin_prompt(Path(id): Path<String>) -> impl IntoResponse {
    match get_prompt_by_id(&id) {
        Some(p) => {
            let response = PromptResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                content: p.content,
                category: p.category.as_str().to_string(),
                tags: p.tags,
            };
            Json(ApiResponse::success(response))
        }
        None => Json(ApiResponse::error(format!(
            "Prompt template '{}' not found",
            id
        ))),
    }
}

/// GET /api/prompts/builtin/category/:category - 按分类获取模板
pub async fn get_builtin_prompts_by_category(Path(category): Path<String>) -> impl IntoResponse {
    let cat = match category.as_str() {
        "code_review" => PromptCategory::CodeReview,
        "debugging" => PromptCategory::Debugging,
        "refactoring" => PromptCategory::Refactoring,
        "testing" => PromptCategory::Testing,
        "documentation" => PromptCategory::Documentation,
        "security" => PromptCategory::Security,
        "general" => PromptCategory::General,
        _ => {
            return Json(ApiResponse::error(format!(
                "Unknown category: {}",
                category
            )));
        }
    };

    let prompts = get_builtin_prompts()
        .into_iter()
        .filter(|p| p.category.as_str() == cat.as_str())
        .map(|p| PromptResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            content: p.content,
            category: p.category.as_str().to_string(),
            tags: p.tags,
        })
        .collect::<Vec<_>>();

    Json(ApiResponse::success(prompts))
}
