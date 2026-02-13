use crate::models::api::ApiResponse;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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
        // Use frontmatter parsing to extract description and metadata
        // Use frontmatter parsing to extract description and metadata
        let (metadata, description) = Skill::parse_with_fallback(&instruction);

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

/// Installed plugin info from installed_plugins.json
#[derive(Debug, Deserialize)]
struct InstalledPlugin {
    #[serde(rename = "installPath")]
    install_path: String,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InstalledPluginsFile {
    #[allow(dead_code)]
    version: Option<u32>,
    plugins: HashMap<String, Vec<InstalledPlugin>>,
}

#[derive(Clone)]
struct CacheEntry<T> {
    expires_at: Instant,
    value: T,
}

static LEGACY_SKILLS_CACHE: Lazy<RwLock<Option<CacheEntry<Vec<Skill>>>>> =
    Lazy::new(|| RwLock::new(None));
const LEGACY_SKILLS_TTL: Duration = Duration::from_secs(30);

async fn invalidate_legacy_skills_cache() {
    let mut cache = LEGACY_SKILLS_CACHE.write().await;
    *cache = None;
}

/// List all skills from installed plugins
/// Scans ~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/skills/*/SKILL.md
fn list_plugin_skills() -> Vec<Skill> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let plugins_file = home
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");
    if !plugins_file.exists() {
        return Vec::new();
    }

    // Parse installed_plugins.json
    let content = match std::fs::read_to_string(&plugins_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let installed: InstalledPluginsFile = match serde_json::from_str(&content) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    let mut skills = Vec::new();

    for (plugin_key, installations) in &installed.plugins {
        // plugin_key format: "plugin-name@marketplace-name"
        // rsplitn(2, '@') on "code-review@claude-plugins-official" => ["claude-plugins-official", "code-review"]
        let parts: Vec<&str> = plugin_key.rsplitn(2, '@').collect();
        let (plugin_name, _marketplace) = if parts.len() == 2 {
            (parts[1], Some(parts[0]))
        } else {
            (plugin_key.as_str(), None)
        };

        for install in installations {
            let install_path = PathBuf::from(&install.install_path);
            if !install_path.exists() {
                continue;
            }

            // Look for skills in multiple possible locations:
            // 1. <install_path>/skills/*/SKILL.md (standard plugin structure)
            // 2. <install_path>/*/SKILL.md (flat plugin structure like daymade-skills)
            // 3. <install_path>/<plugin-name>/*/SKILL.md (nested structure like scientific-skills)
            let skills_dirs = vec![
                install_path.join("skills"),
                install_path.clone(),
                install_path.join(plugin_name),
            ];

            for skills_dir in skills_dirs {
                if !skills_dir.exists() || !skills_dir.is_dir() {
                    continue;
                }

                let entries = match std::fs::read_dir(&skills_dir) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let skill_name = match path.file_name().and_then(|s| s.to_str()) {
                        Some(n) => n.to_string(),
                        None => continue,
                    };

                    // Skip non-skill directories
                    if skill_name.starts_with('.') || skill_name == "node_modules" {
                        continue;
                    }

                    let skill_file = path.join("SKILL.md");
                    if !skill_file.exists() {
                        continue;
                    }

                    let instruction = match std::fs::read_to_string(&skill_file) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    // Use frontmatter parsing to extract description and metadata
                    // Use frontmatter parsing to extract description and metadata
                    let (metadata, description) = Skill::parse_with_fallback(&instruction);

                    // Create qualified name: "plugin-name:skill-name"
                    let qualified_name = format!("{}:{}", plugin_name, skill_name);

                    skills.push(Skill {
                        name: qualified_name,
                        description,
                        path: path.to_string_lossy().to_string(),
                        instruction,
                        metadata,
                        is_remote: false,
                        repository: Some(plugin_key.clone()),
                    });
                }
            }
        }
    }

    skills
}

fn collect_all_skills_sync() -> Result<Vec<Skill>, String> {
    let manager = SkillsManager::new(Platform::Claude).map_err(|e| e.to_string())?;
    let user_skills = manager.list_skills().map_err(|e| e.to_string())?;
    let project_root = find_project_root();
    let project_skills = project_root
        .as_ref()
        .map(|p| list_project_claude_skills(p.as_path()))
        .unwrap_or_default();
    let plugin_skills = list_plugin_skills();

    // Merge all skill sources with priority: project > user > plugin
    let mut merged: std::collections::BTreeMap<String, Skill> = std::collections::BTreeMap::new();
    for s in plugin_skills {
        merged.insert(s.name.clone(), s);
    }
    for s in user_skills {
        merged.insert(s.name.clone(), s);
    }
    for s in project_skills {
        merged.insert(s.name.clone(), s);
    }

    Ok(merged.into_values().collect::<Vec<_>>())
}

async fn list_skills_cached() -> Result<Vec<Skill>, String> {
    {
        let cache = LEGACY_SKILLS_CACHE.read().await;
        if let Some(entry) = cache.as_ref()
            && Instant::now() < entry.expires_at
        {
            return Ok(entry.value.clone());
        }
    }

    let skills = tokio::task::spawn_blocking(collect_all_skills_sync)
        .await
        .map_err(|e| format!("Skill scan task failed: {}", e))??;

    {
        let mut cache = LEGACY_SKILLS_CACHE.write().await;
        *cache = Some(CacheEntry {
            expires_at: Instant::now() + LEGACY_SKILLS_TTL,
            value: skills.clone(),
        });
    }

    Ok(skills)
}

pub async fn list_skills() -> impl IntoResponse {
    match list_skills_cached().await {
        Ok(skills) => ApiResponse::success(skills),
        Err(e) => ApiResponse::error(e),
    }
}

pub async fn add_skill(Json(payload): Json<CreateSkillRequest>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.install_skill(&payload.name, &payload.instruction) {
        Ok(_) => {
            invalidate_legacy_skills_cache().await;
            ApiResponse::success("Skill created successfully".to_string())
        }
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
        Ok(_) => {
            invalidate_legacy_skills_cache().await;
            ApiResponse::success("Skill updated successfully".to_string())
        }
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn get_skill(Path(name): Path<String>) -> impl IntoResponse {
    let skills = match list_skills_cached().await {
        Ok(skills) => skills,
        Err(e) => return ApiResponse::error(e),
    };
    if let Some(skill) = skills.into_iter().find(|s| s.name == name) {
        return ApiResponse::success(skill);
    }

    ApiResponse::error(format!("Skill '{}' not found", name))
}

pub async fn delete_skill(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.uninstall_skill(&name) {
        Ok(_) => {
            invalidate_legacy_skills_cache().await;
            ApiResponse::success("Skill deleted successfully".to_string())
        }
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
        Ok(_) => {
            invalidate_legacy_skills_cache().await;
            ApiResponse::success("Repository added successfully".to_string())
        }
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn remove_repository(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match SkillsManager::new(Platform::Claude) {
        Ok(m) => m,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    match manager.remove_repository(&name) {
        Ok(_) => {
            invalidate_legacy_skills_cache().await;
            ApiResponse::success("Repository removed successfully".to_string())
        }
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
