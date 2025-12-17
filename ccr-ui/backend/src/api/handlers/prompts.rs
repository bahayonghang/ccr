use crate::models::api::ApiResponse;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

// Use the enhanced PromptsManager from ccr crate
use ccr::managers::prompts_manager::PromptsManager;
use ccr::models::Platform;
use ccr::models::prompt::{PromptPreset, PromptTarget};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPromptRequest {
    pub name: String,
    pub target: String,
    pub content: String,
    pub description: Option<String>,
}

pub async fn list_prompts() -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.list_presets() {
        Ok(presets) => ApiResponse::success(presets),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn add_prompt(Json(payload): Json<AddPromptRequest>) -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let target = match payload.target.parse::<PromptTarget>() {
        Ok(t) => t,
        Err(e) => return ApiResponse::error(e),
    };

    let preset = PromptPreset {
        name: payload.name,
        description: payload.description,
        target_file: target,
        content: payload.content,
        tags: vec![],
    };

    match manager.add_preset(preset) {
        Ok(_) => ApiResponse::success("Prompt preset added successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn get_prompt(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.get_preset(&name) {
        Ok(preset) => ApiResponse::success(preset),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn delete_prompt(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.remove_preset(&name) {
        Ok(_) => ApiResponse::success("Prompt preset removed successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn apply_prompt(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.apply_preset(&name) {
        Ok(_) => ApiResponse::success("Prompt preset applied successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn get_current_prompt(Path(target): Path<String>) -> impl IntoResponse {
    let manager = match PromptsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let target_enum = match target.parse::<PromptTarget>() {
        Ok(t) => t,
        Err(e) => return ApiResponse::error(e),
    };

    match manager.get_current_content(target_enum) {
        Ok(content) => ApiResponse::success(content),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}
