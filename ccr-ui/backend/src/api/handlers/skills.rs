use crate::models::api::ApiResponse;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

// Use the enhanced SkillsManager from ccr crate
use ccr::managers::skills_manager::SkillsManager;
use ccr::models::Platform;
use ccr::models::skill::SkillRepository;

// Request/Response models
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub instruction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSkillRequest {
    pub instruction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRepositoryRequest {
    pub name: String,
    pub url: String,
    pub branch: Option<String>,
    pub description: Option<String>,
}

pub async fn list_skills() -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.list_skills() {
        Ok(skills) => Json(ApiResponse::success(skills)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn add_skill(Json(payload): Json<CreateSkillRequest>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.install_skill(&payload.name, &payload.instruction) {
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
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    // For update, we need to uninstall and reinstall
    // Or we could add an update method to SkillsManager
    match manager.uninstall_skill(&name) {
        Ok(_) => {}
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.install_skill(&name, &payload.instruction) {
        Ok(_) => Json(ApiResponse::success(
            "Skill updated successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn delete_skill(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.uninstall_skill(&name) {
        Ok(_) => Json(ApiResponse::success(
            "Skill deleted successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

// New endpoints for repository management

pub async fn list_repositories() -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.list_repositories() {
        Ok(repos) => Json(ApiResponse::success(repos)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn add_repository(Json(payload): Json<AddRepositoryRequest>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let repo = SkillRepository {
        name: payload.name,
        url: payload.url,
        branch: payload.branch.unwrap_or_else(|| "main".to_string()),
        description: payload.description,
    };

    match manager.add_repository(repo) {
        Ok(_) => Json(ApiResponse::success(
            "Repository added successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn remove_repository(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.remove_repository(&name) {
        Ok(_) => Json(ApiResponse::success(
            "Repository removed successfully".to_string(),
        )),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

pub async fn scan_repository(Path(repo_name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    match manager.fetch_remote_skills(&repo_name).await {
        Ok(skills) => Json(ApiResponse::success(skills)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}
