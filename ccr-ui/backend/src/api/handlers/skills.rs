use crate::managers::skills_manager::SkillsManager;
use crate::models::api::ApiResponse;
use crate::models::skill::{CreateSkillRequest, UpdateSkillRequest};
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

pub async fn list_skills() -> impl IntoResponse {
    let manager = SkillsManager::new();
    match manager.list_skills() {
        Ok(skills) => Json(ApiResponse::success(skills)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn add_skill(Json(payload): Json<CreateSkillRequest>) -> impl IntoResponse {
    let manager = SkillsManager::new();
    match manager.create_skill(&payload.name, &payload.instruction) {
        Ok(_) => Json(ApiResponse::success(
            "Skill created successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn update_skill(
    Path(name): Path<String>,
    Json(payload): Json<UpdateSkillRequest>,
) -> impl IntoResponse {
    let manager = SkillsManager::new();
    match manager.update_skill(&name, &payload.instruction) {
        Ok(_) => Json(ApiResponse::success(
            "Skill updated successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn delete_skill(Path(name): Path<String>) -> impl IntoResponse {
    let manager = SkillsManager::new();
    match manager.delete_skill(&name) {
        Ok(_) => Json(ApiResponse::success(
            "Skill deleted successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}
