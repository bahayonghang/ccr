use crate::models::api::ApiResponse;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Use the enhanced SkillsManager from ccr crate
use ccr::managers::skills_manager::SkillsManager;
use ccr::models::Platform;
use ccr::models::skill::Skill;
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

fn find_project_root() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    let mut current = start.as_path();
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return Some(start),
        }
    }
}

fn list_project_claude_skills(project_root: &std::path::Path) -> Vec<Skill> {
    let base_path = project_root.join(".claude").join("skills");
    if !base_path.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();
    let entries = match std::fs::read_dir(&base_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let skill_file = path.join("SKILL.md");
        if !skill_file.exists() {
            continue;
        }
        let instruction = match std::fs::read_to_string(&skill_file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let description = instruction.lines().next().map(|s| s.to_string());
        let metadata = Skill::parse_metadata(&instruction);

        skills.push(Skill {
            name,
            description,
            path: path.to_string_lossy().to_string(),
            instruction,
            metadata,
            is_remote: false,
            repository: None,
        });
    }

    skills
}

pub async fn list_skills() -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let user_skills = match manager.list_skills() {
        Ok(skills) => skills,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let project_root = find_project_root();
    let project_skills = project_root
        .as_ref()
        .map(|p| list_project_claude_skills(p.as_path()))
        .unwrap_or_default();

    let mut merged: std::collections::BTreeMap<String, Skill> = std::collections::BTreeMap::new();
    for s in project_skills {
        merged.insert(s.name.clone(), s);
    }
    for s in user_skills {
        merged.entry(s.name.clone()).or_insert(s);
    }

    ApiResponse::success(merged.into_values().collect::<Vec<_>>())
}

pub async fn add_skill(Json(payload): Json<CreateSkillRequest>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.install_skill(&payload.name, &payload.instruction) {
        Ok(_) => ApiResponse::success("Skill created successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn update_skill(
    Path(name): Path<String>,
    Json(payload): Json<UpdateSkillRequest>,
) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    // For update, we need to uninstall and reinstall
    // Or we could add an update method to SkillsManager
    match manager.uninstall_skill(&name) {
        Ok(_) => {}
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.install_skill(&name, &payload.instruction) {
        Ok(_) => ApiResponse::success("Skill updated successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn get_skill(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.list_skills() {
        Ok(skills) => {
            if let Some(skill) = skills.into_iter().find(|s| s.name == name) {
                ApiResponse::success(skill)
            } else {
                ApiResponse::error(format!("Skill '{}' not found", name))
            }
        }
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn delete_skill(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.uninstall_skill(&name) {
        Ok(_) => ApiResponse::success("Skill deleted successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

// New endpoints for repository management

pub async fn list_repositories() -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.list_repositories() {
        Ok(repos) => ApiResponse::success(repos),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn add_repository(Json(payload): Json<AddRepositoryRequest>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let repo = SkillRepository {
        name: payload.name,
        url: payload.url,
        branch: payload.branch.unwrap_or_else(|| "main".to_string()),
        description: payload.description,
        skill_count: 0,
        last_synced: None,
        is_official: false,
    };

    match manager.add_repository(repo) {
        Ok(_) => ApiResponse::success("Repository added successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn remove_repository(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.remove_repository(&name) {
        Ok(_) => ApiResponse::success("Repository removed successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn scan_repository(Path(repo_name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.fetch_remote_skills(&repo_name).await {
        Ok(skills) => ApiResponse::success(skills),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}
